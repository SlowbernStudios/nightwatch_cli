use crate::engine::{Action, Game, Initial, Input, Origin, Profile, Session};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const MAX_BYTES: usize = 1_000_000;
const MAX_EVENTS: usize = 20_000;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Save is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Save IO failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Invalid(&'static str),
}
pub struct Loaded {
    pub session: Session,
    pub legacy: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Native {
    format: String,
    version: u32,
    engine_revision: u32,
    origin: Origin,
    events: Vec<Input>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct V1 {
    version: u32,
    seed: u32,
    shift_number: u64,
    commands: Vec<Action>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OldInitial {
    seed: u32,
    shift_number: u64,
    practice: bool,
    profile: Profile,
    legacy: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct V2 {
    version: u32,
    initial: OldInitial,
    events: Vec<Input>,
}

fn valid_initial(initial: &Initial) -> Result<(), Error> {
    if initial.shift_number > 9_007_199_254_740_991 || !initial.profile.valid() {
        return Err(Error::Invalid("Invalid shift counter or learning profile."));
    }
    Ok(())
}
fn bounded<T>(items: &[T]) -> Result<(), Error> {
    if items.len() > MAX_EVENTS {
        Err(Error::Invalid(
            "A replay segment exceeds 20,000 inputs. Finish this shift for a fresh journal.",
        ))
    } else {
        Ok(())
    }
}
fn replay(origin: &Origin, events: &[Input]) -> Result<Session, Error> {
    valid_initial(&origin.initial)?;
    bounded(&origin.legacy_events)?;
    bounded(events)?;
    let mut s = Session::from_initial(origin.initial.clone());
    if let Some(commands) = &origin.legacy_commands {
        bounded(commands)?;
        if !origin.initial.practice {
            return Err(Error::Invalid("Legacy origins require mixed practice."));
        }
        let mut g = Game::new(origin.initial.seed);
        g.set_shift_number(origin.initial.shift_number);
        for &action in commands {
            g.apply(action, false);
        }
        s.restore_game(g);
    }
    for event in &origin.legacy_events {
        s.apply(event.clone());
    }
    s.origin = origin.clone();
    s.events.clear();
    for event in events {
        s.apply(event.clone());
    }
    Ok(s)
}
fn legacy_v1(bytes: &[u8]) -> Result<V1, Error> {
    let old: V1 = serde_json::from_slice(bytes)?;
    if old.version != 1 || old.shift_number > 9_007_199_254_740_991 {
        return Err(Error::Invalid("Invalid legacy v1 origin."));
    }
    bounded(&old.commands)?;
    Ok(old)
}
pub fn decode(bytes: &[u8]) -> Result<Loaded, Error> {
    if bytes.len() > MAX_BYTES {
        return Err(Error::Invalid("Save exceeds the 1 MB limit."));
    }
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    let version = value
        .get("version")
        .and_then(|v| v.as_u64())
        .ok_or(Error::Invalid("Missing save version."))?;
    let session = match version {
        1 => {
            let old = legacy_v1(bytes)?;
            let origin = Origin {
                initial: Initial {
                    seed: old.seed,
                    shift_number: old.shift_number,
                    practice: true,
                    profile: Profile::default(),
                },
                legacy_commands: Some(old.commands),
                legacy_events: vec![],
            };
            replay(&origin, &[])?
        }
        2 => {
            if value["initial"].get("legacy").is_some_and(|v| v.is_null()) {
                return Err(Error::Invalid("Invalid legacy origin."));
            }
            let old: V2 = serde_json::from_slice(bytes)?;
            if old.version != 2 {
                return Err(Error::Invalid("Invalid v2 origin."));
            }
            bounded(&old.events)?;
            let initial = Initial {
                seed: old.initial.seed,
                shift_number: old.initial.shift_number,
                practice: old.initial.practice,
                profile: old.initial.profile,
            };
            let commands = if let Some(legacy) = old.initial.legacy {
                let game = legacy_v1(legacy.as_bytes())?;
                if game.seed != initial.seed || game.shift_number != initial.shift_number {
                    return Err(Error::Invalid("Embedded legacy seed/shift mismatch."));
                }
                Some(game.commands)
            } else {
                None
            };
            let origin = Origin {
                initial,
                legacy_commands: commands,
                legacy_events: vec![],
            };
            let mut s = replay(&origin, &old.events)?;
            s.origin.legacy_events = std::mem::take(&mut s.events);
            s
        }
        3 => {
            let native: Native = serde_json::from_slice(bytes)?;
            if native.format != "nightwatch-cli" || native.engine_revision != 1 {
                return Err(Error::Invalid(
                    "Unsupported native format or engine revision.",
                ));
            }
            replay(&native.origin, &native.events)?
        }
        _ => {
            return Err(Error::Invalid(
                "Unsupported save version. Expected legacy 1/2 or native 3.",
            ));
        }
    };
    Ok(Loaded {
        session,
        legacy: version != 3,
    })
}
pub fn encode(session: &Session) -> Result<Vec<u8>, Error> {
    bounded(&session.events)?;
    bounded(&session.origin.legacy_events)?;
    valid_initial(&session.origin.initial)?;
    if let Some(commands) = &session.origin.legacy_commands {
        bounded(commands)?;
    }
    let bytes = serde_json::to_vec(&Native {
        format: "nightwatch-cli".into(),
        version: 3,
        engine_revision: 1,
        origin: session.origin.clone(),
        events: session.events.clone(),
    })?;
    if bytes.len() > MAX_BYTES {
        return Err(Error::Invalid("Save exceeds the 1 MB limit."));
    }
    Ok(bytes)
}
pub fn read(path: &Path) -> Result<Loaded, Error> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    decode(&bytes)
}
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() > MAX_BYTES {
        return Err(Error::Invalid("Save exceeds the 1 MB limit."));
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::Builder::new()
        .prefix(".nightwatch-")
        .suffix(".tmp")
        .tempfile_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|e| Error::Io(e.error))?;
    Ok(())
}
fn resolved(path: &Path) -> Result<PathBuf, Error> {
    if path.exists() {
        return Ok(path.canonicalize()?);
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    Ok(parent.canonicalize()?.join(
        path.file_name()
            .ok_or(Error::Invalid("A save filename is required."))?,
    ))
}
pub fn destination(
    loaded: Option<&Path>,
    legacy: bool,
    explicit: Option<&Path>,
) -> Result<PathBuf, Error> {
    let destination = explicit
        .or(if legacy { None } else { loaded })
        .unwrap_or_else(|| Path::new(".nightwatch-save.json"))
        .to_owned();
    if legacy
        && let Some(source) = loaded
        && resolved(source)? == resolved(&destination)?
    {
        return Err(Error::Invalid(
            "Refusing to overwrite the legacy input. Choose a separate --save destination.",
        ));
    }
    Ok(destination)
}

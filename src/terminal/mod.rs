use crate::{
    app::{App, Effect, Event},
    engine::Input,
    save, ui,
};
use crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event as TerminalEvent, KeyCode,
    KeyEventKind, KeyModifiers,
};
use std::{io, path::Path};

#[cfg(unix)]
pub struct Shutdown {
    flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    registrations: Vec<signal_hook::SigId>,
}
#[cfg(unix)]
impl Shutdown {
    pub fn new() -> io::Result<Self> {
        let mut result = Self {
            flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            registrations: vec![],
        };
        for signal in [
            signal_hook::consts::SIGTERM,
            signal_hook::consts::SIGHUP,
            signal_hook::consts::SIGINT,
        ] {
            result
                .registrations
                .push(signal_hook::flag::register(signal, result.flag.clone())?);
        }
        Ok(result)
    }
    pub fn requested(&self) -> bool {
        self.flag.load(std::sync::atomic::Ordering::SeqCst)
    }
}
#[cfg(unix)]
impl Drop for Shutdown {
    fn drop(&mut self) {
        for id in self.registrations.drain(..) {
            signal_hook::low_level::unregister(id);
        }
    }
}

pub fn normalize(event: TerminalEvent) -> Vec<Event> {
    match event {
        TerminalEvent::Resize(_, _) => vec![Event::Resize],
        TerminalEvent::Paste(text) => text
            .chars()
            .filter_map(|c| match c {
                '\n' | '\r' => Some(Event::Input(Input::Enter)),
                '\t' => Some(Event::Menu),
                c if c.is_ascii() && !c.is_ascii_control() => Some(Event::Input(Input::Key(c))),
                _ => None,
            })
            .collect(),
        TerminalEvent::Key(key) if key.kind == KeyEventKind::Press => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return if matches!(key.code, KeyCode::Char('c' | 'C' | 'd' | 'D')) {
                    vec![Event::Interrupt]
                } else {
                    vec![]
                };
            }
            if key
                .modifiers
                .intersects(KeyModifiers::ALT | KeyModifiers::SUPER)
                && key.code != KeyCode::Esc
            {
                return vec![];
            }
            vec![match key.code {
                KeyCode::Tab => Event::Menu,
                KeyCode::Esc => Event::Input(Input::Escape),
                KeyCode::Enter => Event::Input(Input::Enter),
                KeyCode::Backspace => Event::Input(Input::Backspace),
                KeyCode::Left => Event::Input(Input::Left),
                KeyCode::Right => Event::Input(Input::Right),
                KeyCode::Up => Event::Input(Input::Up),
                KeyCode::Down => Event::Input(Input::Down),
                KeyCode::Char(c) if c.is_ascii() && !c.is_ascii_control() => {
                    Event::Input(Input::Key(c))
                }
                _ => return vec![],
            }]
        }
        _ => vec![],
    }
}
pub fn save_requested(app: &mut App, path: &Path) {
    match save::encode(app.session()).and_then(|bytes| save::write_atomic(path, &bytes)) {
        Ok(()) => app.report(format!("Saved to {}. Paperwork secured.", path.display())),
        Err(e) => app.report(format!("Save failed: {e}")),
    }
}
pub fn dispatch(app: &mut App, event: Event, width: u16, height: u16) -> Option<Effect> {
    if (width < 60 || height < 22)
        && event != Event::Interrupt
        && !(app.overlay() == Some(crate::app::Overlay::Quit) && matches!(event, Event::Input(_)))
    {
        return None;
    }
    app.handle(event)
}
struct PasteGuard;
impl Drop for PasteGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), DisableBracketedPaste);
    }
}
/// Shared by the real frontend and the physical-terminal cleanup probe.
pub fn with_terminal<T>(
    body: impl FnOnce(&mut ratatui::DefaultTerminal) -> io::Result<T>,
) -> io::Result<T> {
    ratatui::run(|terminal| {
        let _guard = PasteGuard;
        crossterm::execute!(io::stdout(), EnableBracketedPaste)?;
        body(terminal)
    })
}
pub fn run(app: &mut App, path: &Path, no_color: bool) -> io::Result<()> {
    #[cfg(unix)]
    let shutdown = Shutdown::new()?;
    with_terminal(|terminal| {
        loop {
            terminal.draw(|frame| ui::draw(frame, app, no_color))?;
            #[cfg(unix)]
            loop {
                if shutdown.requested() {
                    return Ok(());
                }
                if event::poll(std::time::Duration::from_millis(250))? {
                    break;
                }
            }
            for event in normalize(event::read()?) {
                let size = terminal.size()?;
                match dispatch(app, event, size.width, size.height) {
                    Some(Effect::Quit) => return Ok(()),
                    Some(Effect::Save) => save_requested(app, path),
                    None => (),
                }
            }
        }
    })
}

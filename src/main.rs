use clap::Parser;
use nightwatch_cli::{
    app::{App, Effect},
    args::Args,
    engine::Session,
    plain, save, terminal,
};
use std::io::{self, BufRead, IsTerminal, Write};

fn main() -> std::process::ExitCode {
    match run(Args::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("nightwatch: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let seed = args.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u32
    });
    let loaded = args.load.as_deref().map(save::read).transpose()?;
    let legacy = loaded.as_ref().is_some_and(|s| s.legacy);
    let destination = save::destination(args.load.as_deref(), legacy, args.save.as_deref())?;
    let session = loaded
        .map(|s| s.session)
        .unwrap_or_else(|| Session::new(seed, args.practice));
    let use_plain = args.plain
        || !io::stdin().is_terminal()
        || !io::stdout().is_terminal()
        || std::env::var("TERM").is_ok_and(|s| s == "dumb");
    let mut app = App::new(session, !use_plain);
    if !use_plain {
        terminal::run(
            &mut app,
            &destination,
            args.no_color || std::env::var_os("NO_COLOR").is_some(),
        )?;
        return Ok(());
    }
    let mut stdout = io::stdout().lock();
    write!(stdout, "{}", plain::render(&app))?;
    stdout.flush()?;
    for line in io::stdin().lock().lines() {
        match plain::parse(&line?) {
            Ok(events) => {
                for event in events {
                    match app.handle(event) {
                        Some(Effect::Quit) => return Ok(()),
                        Some(Effect::Save) => {
                            match save::encode(app.session())
                                .and_then(|bytes| save::write_atomic(&destination, &bytes))
                            {
                                Ok(()) => app.report(format!(
                                    "Saved to {}. Paperwork secured.",
                                    destination.display()
                                )),
                                Err(e) => app.report(format!("Save failed: {e}")),
                            }
                        }
                        None => (),
                    }
                }
            }
            Err(e) => app.report(e),
        }
        write!(stdout, "{}", plain::render(&app))?;
        stdout.flush()?;
    }
    Ok(())
}

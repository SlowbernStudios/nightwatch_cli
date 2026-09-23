use crate::{
    app::{App, Command, Event, Overlay},
    engine::{
        Input,
        world::{Point, SupplyKind, Terrain},
    },
};

pub fn parse(line: &str) -> Result<Vec<Event>, String> {
    let line = line.trim_end_matches(['\r', '\n']);
    if let Some(keys) = line.strip_prefix("keys ") {
        return literal(keys);
    }
    let command = line.trim().to_ascii_lowercase();
    if command == "menu" || command == "tab" {
        return Ok(vec![Event::Menu]);
    }
    if command == "close" {
        return Ok(vec![Event::Action(Command::Resume)]);
    }
    if let Some(c) = Command::ALL.into_iter().find(|c| c.name() == command) {
        return Ok(vec![Event::Action(c)]);
    }
    let key = match command.as_str() {
        "" => "enter",
        "esc" => "escape",
        s => s.strip_prefix("go ").unwrap_or(s),
    };
    if let Ok(input) = key.parse::<Input>() {
        return Ok(vec![Event::Input(input)]);
    }
    if command.is_ascii()
        && command.len() > 1
        && command[..command.len() - 1]
            .bytes()
            .all(|c| c.is_ascii_digit())
        && matches!(command.as_bytes().last(), Some(b'h' | b'j' | b'k' | b'l'))
    {
        return literal(&command);
    }
    Err("Unknown command. Try help, menu, or keys hjkl3l.".into())
}
fn literal(mut keys: &str) -> Result<Vec<Event>, String> {
    let mut events = Vec::new();
    while !keys.is_empty() {
        if let Some((token, input)) = [
            ("<Esc>", Input::Escape),
            ("<Enter>", Input::Enter),
            ("<BS>", Input::Backspace),
        ]
        .into_iter()
        .find(|(t, _)| keys.starts_with(t))
        {
            events.push(Event::Input(input));
            keys = &keys[token.len()..];
        } else {
            let c = keys.chars().next().unwrap();
            if !c.is_ascii() || c.is_ascii_control() {
                return Err("Console input is printable ASCII only.".into());
            }
            events.push(Event::Input(Input::Key(c)));
            keys = &keys[c.len_utf8()..];
        }
    }
    Ok(events)
}
pub fn glyph(app: &App, p: Point) -> char {
    let g = app.session().game();
    let v = g.world().view();
    if p == g.player() {
        return '@';
    }
    if app.session().checkpoint() == Some(p) {
        return '+';
    }
    if !g.explored().contains(&p) {
        return ' ';
    }
    if g.visible().contains(&p) {
        if g.cat_position() == p {
            return 'f';
        }
        if let Some(s) = v
            .supplies
            .iter()
            .find(|s| s.position == p && !g.is_collected(&s.id))
        {
            return if s.kind == SupplyKind::Coffee {
                '!'
            } else {
                '%'
            };
        }
        if v.hazards.contains(&p) {
            return '~';
        }
    }
    if g.world().terrain(p) == Terrain::Door && g.is_open(p) {
        return '/';
    }
    if p.x < 0 || p.y < 0 || p.x >= v.width || p.y >= v.height {
        ' '
    } else {
        v.terrain[p.y as usize].as_bytes()[p.x as usize] as char
    }
}
pub fn render(app: &App) -> String {
    let s = app.session().view();
    let mut out = format!(
        "\nNIGHTWATCH | Shift {} | seed {} | turn {}\nTASK: {}\n{}\n{}\n",
        s.game.shift_number + 1,
        s.game.seed,
        s.game.turn,
        app.task(),
        app.destination(),
        app.hint()
    );
    match app.overlay() {
        Some(Overlay::Actions) => {
            out.push_str("Game actions\n");
            for (i, c) in Command::ALL.into_iter().enumerate() {
                out.push_str(&format!(
                    "{} {}: {}\n",
                    if i == app.selection() { ">" } else { " " },
                    c.name(),
                    c.label()
                ));
            }
        }
        Some(Overlay::Quit) => {
            out.push_str("Quit without saving? Enter confirms; any other key cancels.\n")
        }
        Some(Overlay::Page(_)) => out.push_str(&app.page()),
        None => {
            if let Some(c) = s.console {
                out.push_str(&format!(
                    "{:?} | pending {} | step {}\n",
                    c.editor.mode,
                    c.editor.pending,
                    c.step + 1
                ));
                for (i, line) in c.editor.lines.iter().enumerate() {
                    out.push_str(&format!("{:>2} {line}\n", i + 1));
                    if i == c.editor.row {
                        out.push_str(&format!("{}^\n", " ".repeat(c.editor.col + 3)));
                    }
                }
            } else {
                let p = s.game.player;
                for y in p.y - 5..=p.y + 5 {
                    for x in p.x - 30..=p.x + 30 {
                        out.push(glyph(app, Point { x, y }));
                    }
                    out.push('\n');
                }
                out.push_str(&format!("NORMAL | pending {}\n", s.pending));
            }
        }
    }
    out.push_str(&format!("\n{}\n> ", app.feedback()));
    out
}

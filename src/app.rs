use crate::engine::{
    Input, Session,
    repairs::{Repair, Stage},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Resume,
    Inventory,
    Map,
    Look,
    Log,
    Help,
    Progress,
    Pet,
    Feed,
    Pickup,
    Coffee,
    Wait,
    Reset,
    Example,
    Practice,
    Next,
    Save,
    Quit,
}
impl Command {
    pub const ALL: [Self; 18] = [
        Self::Resume,
        Self::Inventory,
        Self::Map,
        Self::Look,
        Self::Log,
        Self::Help,
        Self::Progress,
        Self::Pet,
        Self::Feed,
        Self::Pickup,
        Self::Coffee,
        Self::Wait,
        Self::Reset,
        Self::Example,
        Self::Practice,
        Self::Next,
        Self::Save,
        Self::Quit,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Resume => "resume",
            Self::Inventory => "inventory",
            Self::Map => "map",
            Self::Look => "look",
            Self::Log => "log",
            Self::Help => "help",
            Self::Progress => "progress",
            Self::Pet => "pet",
            Self::Feed => "feed",
            Self::Pickup => "pickup",
            Self::Coffee => "coffee",
            Self::Wait => "wait",
            Self::Reset => "reset",
            Self::Example => "example",
            Self::Practice => "practice",
            Self::Next => "next",
            Self::Save => "save",
            Self::Quit => "quit",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Resume => "Resume shift",
            Self::Inventory => "Pockets & paperwork",
            Self::Map => "Facility map",
            Self::Look => "Look nearby",
            Self::Log => "Dispatch log",
            Self::Help => "Controls & legend",
            Self::Progress => "Learning progress",
            Self::Pet => "Pet supervisor",
            Self::Feed => "Offer biscuit",
            Self::Pickup => "Collect supply",
            Self::Coffee => "Drink pocket coffee",
            Self::Wait => "Wait one turn",
            Self::Reset => "Reset console",
            Self::Example => "Show worked example",
            Self::Practice => "Skip to mixed practice",
            Self::Next => "Start next shift",
            Self::Save => "Save shift",
            Self::Quit => "Quit without saving",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Input(Input),
    Action(Command),
    Menu,
    Interrupt,
    Resize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Save,
    Quit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overlay {
    Actions,
    Page(Command),
    Quit,
}
pub struct App {
    session: Session,
    overlay: Option<Overlay>,
    selection: usize,
    scroll: usize,
    confirm_quit: bool,
    notice: String,
    log: Vec<String>,
}
impl App {
    pub fn new(session: Session, confirm_quit: bool) -> Self {
        Self {
            session,
            overlay: None,
            selection: 0,
            scroll: 0,
            confirm_quit,
            notice: String::new(),
            log: vec!["DISPATCH: Restore power. Benefits remain theoretical.".into()],
        }
    }
    pub fn session(&self) -> &Session {
        &self.session
    }
    pub fn overlay(&self) -> Option<Overlay> {
        self.overlay
    }
    pub fn selection(&self) -> usize {
        self.selection
    }
    pub fn scroll(&self) -> usize {
        self.scroll
    }
    pub fn feedback(&self) -> &str {
        if !self.notice.is_empty() {
            &self.notice
        } else if !self.session.notice().is_empty() {
            self.session.notice()
        } else {
            self.session.game().message()
        }
    }
    pub fn report(&mut self, message: String) {
        self.notice = message;
        self.record();
    }
    fn record(&mut self) {
        let message = self.feedback().to_owned();
        if self.log.last() != Some(&message) {
            self.log.push(message);
            if self.log.len() > 24 {
                self.log.remove(0);
            }
        }
    }
    pub fn handle(&mut self, event: Event) -> Option<Effect> {
        if event == Event::Resize {
            return None;
        }
        self.notice.clear();
        match event {
            Event::Interrupt => return self.action(Command::Quit),
            Event::Action(c) => return self.action(c),
            Event::Menu => {
                self.overlay = if self.overlay == Some(Overlay::Actions) {
                    None
                } else {
                    Some(Overlay::Actions)
                };
                return None;
            }
            Event::Input(key) => match self.overlay {
                Some(Overlay::Quit) => {
                    self.overlay = None;
                    if key == Input::Enter {
                        return Some(Effect::Quit);
                    }
                    return None;
                }
                Some(Overlay::Actions) => {
                    match key {
                        Input::Escape => self.overlay = None,
                        Input::Key('j') | Input::Down => {
                            self.selection = (self.selection + 1) % Command::ALL.len()
                        }
                        Input::Key('k') | Input::Up => {
                            self.selection =
                                (self.selection + Command::ALL.len() - 1) % Command::ALL.len()
                        }
                        Input::Enter => return self.action(Command::ALL[self.selection]),
                        _ => (),
                    };
                    return None;
                }
                Some(Overlay::Page(_)) => {
                    match key {
                        Input::Escape | Input::Enter => self.overlay = None,
                        Input::Key('j') | Input::Down => {
                            self.scroll =
                                (self.scroll + 1).min(self.page().lines().count().saturating_sub(1))
                        }
                        Input::Key('k') | Input::Up => self.scroll = self.scroll.saturating_sub(1),
                        _ => (),
                    };
                    return None;
                }
                None => {
                    self.session.apply(key);
                    self.record();
                }
            },
            Event::Resize => (),
        }
        None
    }
    fn action(&mut self, c: Command) -> Option<Effect> {
        self.overlay = None;
        self.scroll = 0;
        match c {
            Command::Resume => (),
            Command::Quit => {
                if self.confirm_quit {
                    self.overlay = Some(Overlay::Quit);
                } else {
                    return Some(Effect::Quit);
                }
            }
            Command::Save => return Some(Effect::Save),
            Command::Inventory
            | Command::Map
            | Command::Look
            | Command::Log
            | Command::Help
            | Command::Progress => self.overlay = Some(Overlay::Page(c)),
            _ => {
                self.session.apply(c.name().parse().unwrap());
                self.record();
            }
        }
        None
    }
    pub fn task(&self) -> String {
        let s = self.session.view();
        if let Some(c) = s.console {
            return if c.done {
                "Console verified. Enter submits.".into()
            } else {
                self.session.console_lesson().unwrap().steps[c.step]
                    .instruction
                    .clone()
            };
        }
        match s.lesson {
            0 => {
                return format!(
                    "Visit checkpoint {}/4: {}",
                    s.checkpoint_index + 1,
                    ["h left", "j down", "k up", "l right"][s.checkpoint_index]
                );
            }
            1 => return "Reach + three squares right. Try 3l.".into(),
            _ => (),
        }
        match s.game.stage {
            Stage::Start => "Try the START button".into(),
            Stage::Starter => "Pull the auxiliary handle".into(),
            Stage::Ready => "Press START. Surely this time.".into(),
            Stage::Ignition => format!("Stand by. Look impressed. {}/8", s.game.elapsed),
            Stage::Credits => "SHIFT COMPLETE - one status sign, fully powered.".into(),
            Stage::Repair => match s.game.repairs[s.game.repair_index] {
                Repair::Calibration => format!(
                    "Set tolerance to {} (now {})",
                    s.game.dial_target + 1,
                    s.game.dial + 1
                ),
                Repair::Ventilation => {
                    format!("Crank ventilation: {}/{}", s.game.cranks, s.game.crank_goal)
                }
                Repair::Reboot if s.game.wait > 0 => {
                    "Let the breaker discharge. Wait one turn.".into()
                }
                _ => self
                    .session
                    .game()
                    .adjacent_station()
                    .filter(|p| p.id == self.session.game().required())
                    .map(|p| p.label.to_string())
                    .unwrap_or_else(|| format!("Find {}", self.session.game().required())),
            },
        }
    }
    pub fn hint(&self) -> String {
        let s = self.session.view();
        if let Some(c) = s.console {
            if c.done {
                return "Enter: submit | Tab: actions".into();
            }
            if s.profile.score(c.kind) < 2 {
                return self.session.console_lesson().unwrap().steps[c.step]
                    .hint
                    .clone();
            }
            return "Vim console | Tab: example / reset".into();
        }
        if s.game.stage == Stage::Credits {
            "Tab: next shift / save / quit".into()
        } else if s.game.stage == Stage::Ignition {
            "Tab: wait (plain mode: wait)".into()
        } else {
            "hjkl/arrows: walk | Enter: interact | Tab: actions".into()
        }
    }
    pub fn destination(&self) -> String {
        let g = self.session.game();
        if self.session.checkpoint().is_some() {
            return "+ marks your next safe checkpoint".into();
        }
        g.world()
            .view()
            .stations
            .iter()
            .find(|s| s.id == g.required())
            .map(|s| {
                let room = g
                    .world()
                    .view()
                    .rooms
                    .iter()
                    .find(|r| r.id == s.room)
                    .unwrap();
                let dx = s.position.x - g.player().x;
                let dy = s.position.y - g.player().y;
                if s.position.distance(g.player()) <= 1 {
                    format!("[{}] {} - beside you", s.glyph, s.label)
                } else {
                    format!(
                        "[{}] {} - {}{}",
                        s.glyph,
                        room.title,
                        if dy < 0 {
                            "north"
                        } else if dy > 0 {
                            "south"
                        } else {
                            ""
                        },
                        if dx < 0 {
                            " west"
                        } else if dx > 0 {
                            " east"
                        } else {
                            ""
                        }
                    )
                }
            })
            .unwrap_or_default()
    }
    pub fn page(&self) -> String {
        let Some(Overlay::Page(command)) = self.overlay else {
            return String::new();
        };
        let s = self.session.view();
        match command {
            Command::Help=>"Controls & legend\n\nh left / j down / k up / l right; arrows also work.\nCounts: 3l. Escape cancels pending input.\nEnter interacts. Tab opens game actions.\n\nConsoles: w/e/b words, 0/$ line ends, i inserts.\nEscape returns to NORMAL. x deletes, dd removes lines, u undoes.\nNo full Vim: no registers, visual mode, search or redo.\nEditing, menus and resize pause the world.\n\n@ you   f supervisor   + closed door/checkpoint\n/ open door  # corridor  ~ spill  ! coffee  % biscuit\nG generator  H auxiliary  ? paperwork  ] storage\nD dial  B breaker  F fan  K kettle  u mug\n\nPlain: keys iON<Esc> preserves case and spaces.\nNamed actions: pet, feed, pickup, coffee, wait, reset, example.\nhelp / inventory / map / log / progress; close returns.\nSave is explicit. Quit and EOF do not save.\n\nEscape/Enter: close | j/k: scroll".into(),
            Command::Progress=>format!("Learning progress\n\nUnassisted completions (examples and arrows do not claim mastery):\nhjkl {}   counts {}\nwords {}   bounds {}   insert {}   undo {}\n\nHints fade after two unassisted console completions.\nShift {} | seed {}\nEscape/Enter: close",s.profile.movement,s.profile.counts,s.profile.words,s.profile.bounds,s.profile.insert,s.profile.undo,s.game.shift_number+1,s.game.seed),
            Command::Inventory=>format!("Pockets & paperwork\n\nCoffee: {}  Biscuits: {}\n{}\nPatience: {}  HR write-ups: {}\nSupervisor pets: {}\nEscape/Enter: close",s.game.pockets.coffee,s.game.pockets.biscuit,s.game.inventory.join(", "),s.game.patience,s.game.writeups,s.game.pets),
            Command::Log=>format!("Dispatch log\n\n{}",self.log.join("\n")),
            Command::Look=>format!("Look nearby\n\n{}",self.session.game().world().view().stations.iter().filter(|p|self.session.game().visible().contains(&p.position)).map(|p|format!("{} {} ({}, {})",p.glyph,p.label,p.position.x,p.position.y)).collect::<Vec<_>>().join("\n")),
            Command::Map=>self.facility_map(),_=>String::new(),
        }
    }
    fn facility_map(&self) -> String {
        let rooms = &self.session.game().world().view().rooms;
        let coordinates: Vec<_> = rooms
            .iter()
            .map(|r| {
                (
                    ((r.center.x - 12) / 22) as usize * 9,
                    ((r.center.y - 8) / 14) as usize * 3,
                )
            })
            .collect();
        let width = coordinates.iter().map(|(x, _)| x + 3).max().unwrap();
        let height = coordinates.iter().map(|(_, y)| y + 1).max().unwrap();
        let mut grid = vec![vec![' '; width]; height];
        for (i, &(x, y)) in coordinates.iter().enumerate() {
            for &(xx, yy) in &coordinates[i + 1..] {
                if y == yy && x.abs_diff(xx) == 9 {
                    for cell in &mut grid[y][x.min(xx) + 3..x.max(xx)] {
                        *cell = '-';
                    }
                }
                if x == xx && y.abs_diff(yy) == 3 {
                    for row in &mut grid[y.min(yy) + 1..y.max(yy)] {
                        row[x + 1] = '|';
                    }
                }
            }
            let label = match rooms[i].id {
                "generator" => "GEN",
                "workshop" => "WRK",
                "fuel" => "FUL",
                "office" => "OFF",
                "control" => "CTL",
                "break" => "BRK",
                "stores" => "STR",
                _ => "REC",
            };
            for (offset, c) in label.chars().enumerate() {
                grid[y][x + offset] = c;
            }
        }
        let room = self
            .session
            .game()
            .world()
            .room(self.session.game().player())
            .map_or("Corridor", |r| r.title);
        format!(
            "Facility map\nYou are in: {room}\n\n{}\n\nGEN generator | WRK workshop | FUL fuel\nOFF compliance | CTL control | BRK break\nSTR lost property | REC records\n\n{}\nEscape/Enter: close | j/k: scroll",
            grid.into_iter()
                .map(|row| row.into_iter().collect::<String>())
                .collect::<Vec<_>>()
                .join("\n"),
            self.destination()
        )
    }
}

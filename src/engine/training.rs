use super::{
    Action, Game, GameView,
    lessons::{Kind, Lesson},
    repairs::Stage,
    rng::Seeded,
    world::Point,
};
use crate::editor::{Editor, EditorView, Mode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Input {
    Key(char),
    Enter,
    Escape,
    Backspace,
    Left,
    Right,
    Up,
    Down,
    North,
    South,
    East,
    West,
    Apply,
    Wait,
    Pet,
    Feed,
    Coffee,
    Pickup,
    Reset,
    Example,
    Practice,
    Next,
}
impl std::str::FromStr for Input {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}
impl TryFrom<String> for Input {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(match s.as_str() {
            "enter" => Self::Enter,
            "escape" => Self::Escape,
            "backspace" => Self::Backspace,
            "left" => Self::Left,
            "right" => Self::Right,
            "up" => Self::Up,
            "down" => Self::Down,
            "north" => Self::North,
            "south" => Self::South,
            "east" => Self::East,
            "west" => Self::West,
            "apply" => Self::Apply,
            "wait" => Self::Wait,
            "pet" => Self::Pet,
            "feed" => Self::Feed,
            "coffee" => Self::Coffee,
            "pickup" => Self::Pickup,
            "reset" => Self::Reset,
            "example" => Self::Example,
            "practice" => Self::Practice,
            "next" => Self::Next,
            _ if s.len() == 1 && (32..=126).contains(&s.as_bytes()[0]) => {
                Self::Key(s.as_bytes()[0] as char)
            }
            _ => return Err("Invalid session input"),
        })
    }
}
impl From<Input> for String {
    fn from(i: Input) -> Self {
        match i {
            Input::Key(c) => return c.to_string(),
            Input::Enter => "enter",
            Input::Escape => "escape",
            Input::Backspace => "backspace",
            Input::Left => "left",
            Input::Right => "right",
            Input::Up => "up",
            Input::Down => "down",
            Input::North => "north",
            Input::South => "south",
            Input::East => "east",
            Input::West => "west",
            Input::Apply => "apply",
            Input::Wait => "wait",
            Input::Pet => "pet",
            Input::Feed => "feed",
            Input::Coffee => "coffee",
            Input::Pickup => "pickup",
            Input::Reset => "reset",
            Input::Example => "example",
            Input::Practice => "practice",
            Input::Next => "next",
        }
        .into()
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub movement: u16,
    pub counts: u16,
    pub words: u16,
    pub bounds: u16,
    pub insert: u16,
    pub undo: u16,
}
impl Profile {
    pub fn score(&self, k: Kind) -> u16 {
        match k {
            Kind::Words => self.words,
            Kind::Bounds => self.bounds,
            Kind::Insert => self.insert,
            Kind::Undo => self.undo,
        }
    }
    fn credit(&mut self, k: Kind) {
        let p = match k {
            Kind::Words => &mut self.words,
            Kind::Bounds => &mut self.bounds,
            Kind::Insert => &mut self.insert,
            Kind::Undo => &mut self.undo,
        };
        *p = (*p + 1).min(999);
    }
    pub fn valid(&self) -> bool {
        [
            self.movement,
            self.counts,
            self.words,
            self.bounds,
            self.insert,
            self.undo,
        ]
        .iter()
        .all(|n| *n <= 999)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Initial {
    pub seed: u32,
    pub shift_number: u64,
    pub practice: bool,
    pub profile: Profile,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    pub initial: Initial,
    pub legacy_commands: Option<Vec<Action>>,
    pub legacy_events: Vec<Input>,
}
#[derive(Clone, Debug)]
struct Console {
    lesson: Lesson,
    editor: Editor,
    step: usize,
    station: String,
    assisted: bool,
    done: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct ConsoleView {
    pub kind: Kind,
    pub station: String,
    pub step: usize,
    pub assisted: bool,
    pub done: bool,
    pub editor: EditorView,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub game: GameView,
    pub practice: bool,
    pub lesson: usize,
    pub checkpoint_index: usize,
    pub movement_assisted: bool,
    pub pending: String,
    pub profile: Profile,
    pub console: Option<ConsoleView>,
}
#[derive(Clone, Debug)]
pub struct Session {
    game: Game,
    practice: bool,
    lesson: usize,
    checkpoint_index: usize,
    movement_assisted: bool,
    pending: String,
    profile: Profile,
    console: Option<Console>,
    notice: String,
    pub(crate) origin: Origin,
    pub(crate) events: Vec<Input>,
}
impl Session {
    pub(crate) fn restore_game(&mut self, game: Game) {
        self.game = game;
    }
    pub fn new(seed: u32, practice: bool) -> Self {
        Self::from_initial(Initial {
            seed,
            shift_number: 0,
            practice,
            profile: Profile::default(),
        })
    }
    pub(crate) fn from_initial(initial: Initial) -> Self {
        let mut game = Game::new(initial.seed);
        game.set_shift_number(initial.shift_number);
        Self {
            game,
            practice: initial.practice,
            lesson: if initial.practice { 2 } else { 0 },
            checkpoint_index: 0,
            movement_assisted: false,
            pending: String::new(),
            profile: initial.profile.clone(),
            console: None,
            notice: if initial.practice {
                "Mixed practice. Walk to G, then Enter."
            } else {
                "Welcome. Visit +. No time pressure. Tab opens game actions."
            }
            .into(),
            origin: Origin {
                initial,
                legacy_commands: None,
                legacy_events: vec![],
            },
            events: vec![],
        }
    }
    pub fn origin(&self) -> &Origin {
        &self.origin
    }
    pub fn view(&self) -> SessionView {
        SessionView {
            game: self.game.view(),
            practice: self.practice,
            lesson: self.lesson,
            checkpoint_index: self.checkpoint_index,
            movement_assisted: self.movement_assisted,
            pending: self.pending.clone(),
            profile: self.profile.clone(),
            console: self.console.as_ref().map(|c| ConsoleView {
                kind: c.lesson.kind,
                station: c.station.clone(),
                step: c.step,
                assisted: c.assisted,
                done: c.done,
                editor: c.editor.view().clone(),
            }),
        }
    }
    pub fn game(&self) -> &Game {
        &self.game
    }
    pub fn notice(&self) -> &str {
        &self.notice
    }
    pub fn console_lesson(&self) -> Option<&Lesson> {
        self.console.as_ref().map(|c| &c.lesson)
    }
    pub fn checkpoint(&self) -> Option<Point> {
        let start = self.game.world().view().start;
        match self.lesson {
            0 => {
                let (x, y) = [(-1, 0), (-1, 1), (-1, 0), (0, 0)][self.checkpoint_index];
                Some(start.offset(x, y))
            }
            1 => Some(start.offset(3, 0)),
            _ => None,
        }
    }
    pub fn apply(&mut self, input: Input) -> bool {
        if matches!(input,Input::Key(c) if !c.is_ascii() || c.is_ascii_control()) {
            return false;
        }
        let before = self.game.turn();
        if self.events.len() <= 20_000 {
            self.events.push(input.clone());
        }
        self.notice.clear();
        self.dispatch(input);
        self.game.turn() != before
    }
    fn safe(&self) -> bool {
        !self.practice && self.lesson < 6
    }
    fn dispatch(&mut self, input: Input) {
        if input == Input::Practice {
            if self.console.is_some() {
                self.notice = "Finish or reset this console before changing tracks.".into();
            } else {
                self.practice = true;
                self.lesson = 2;
                self.pending.clear();
                self.notice = "Mixed practice ready. No mastery awarded.".into();
            }
            return;
        }
        if input == Input::Next {
            if self.game.stage() != Stage::Credits {
                self.notice = "Finish this shift first.".into();
                return;
            }
            let game = self.game.next_shift();
            *self = Self::from_initial(Initial {
                seed: game.world().view().seed,
                shift_number: game.view().shift_number,
                practice: true,
                profile: self.profile.clone(),
            });
            return;
        }
        if self.console.is_some() {
            self.console_input(input);
            return;
        }
        if input == Input::Escape {
            self.pending.clear();
            return;
        }
        if let Input::Key(c) = input
            && c.is_ascii_digit()
        {
            if c == '0' && self.pending.is_empty() {
                self.notice = "0 is a line motion inside consoles. Use hjkl here.".into();
                return;
            }
            let next = format!("{}{c}", self.pending);
            if next.parse::<u32>().unwrap_or(100) > 99 {
                self.pending.clear();
                self.notice = "Training counts are limited to 99.".into();
            } else {
                self.pending = next;
            }
            return;
        }
        let count = self.pending.parse::<usize>().unwrap_or(1);
        self.pending.clear();
        let motion = match input {
            Input::Key('h') | Input::Left | Input::West => Some(Action::West),
            Input::Key('j') | Input::Down | Input::South => Some(Action::South),
            Input::Key('k') | Input::Up | Input::North => Some(Action::North),
            Input::Key('l') | Input::Right | Input::East => Some(Action::East),
            _ => None,
        };
        if let Some(action) = motion {
            let initial_lesson = self.lesson;
            let literal = matches!(input, Input::Key(_));
            for _ in 0..count {
                let before = self.game.player();
                let seen = self.game.visible().clone();
                self.game.apply(action, self.safe());
                if before == self.game.player() {
                    break;
                }
                if self.checkpoint() == Some(self.game.player()) {
                    if !literal {
                        self.movement_assisted = true;
                    }
                    if self.lesson == 0 {
                        self.checkpoint_index += 1;
                        if self.checkpoint_index == 4 {
                            if !self.movement_assisted {
                                self.profile.movement = (self.profile.movement + 1).min(999);
                            }
                            self.lesson = 1;
                            self.notice = "Checkpoints complete. Try 3l.".into();
                        }
                    } else {
                        if count > 1 && literal {
                            self.profile.counts = (self.profile.counts + 1).min(999);
                        }
                        self.lesson = 2;
                        self.notice = "Route complete. Travel expenses unchanged.".into();
                    }
                }
                let world = self.game.world().view();
                if self.lesson != initial_lesson
                    || world.hazards.contains(&self.game.player())
                    || self.game.supply_underfoot()
                    || world.stations.iter().any(|p| {
                        self.game.visible().contains(&p.position) && !seen.contains(&p.position)
                    })
                {
                    break;
                }
            }
            if !literal {
                self.notice = "Arrows work. When ready: h left, j down, k up, l right.".into();
            }
            return;
        }
        if matches!(input, Input::Enter | Input::Apply) {
            if self.lesson < 2 {
                self.notice = "Visit + first, or choose mixed practice from Tab.".into();
                return;
            }
            if self.lesson < 6
                && self
                    .game
                    .adjacent_station()
                    .is_some_and(|s| s.id == self.game.required())
            {
                let index = self.lesson - 2;
                let mut order = Kind::ALL;
                if self.practice {
                    Seeded::new(self.game.world().view().seed ^ 0xd00d).shuffle(&mut order);
                }
                let lesson = Lesson::new(
                    order[index],
                    self.game.world().view().seed.wrapping_add(index as u32),
                );
                self.notice = lesson.intro.into();
                self.console = Some(Console {
                    editor: Editor::new(lesson.lines.clone()).unwrap(),
                    lesson,
                    step: 0,
                    station: self.game.required().into(),
                    assisted: false,
                    done: false,
                });
                return;
            }
            self.game.apply(Action::Apply, self.safe());
            return;
        }
        let action = match input {
            Input::Wait => Action::Wait,
            Input::Pet => Action::Pet,
            Input::Feed => Action::Feed,
            Input::Coffee => Action::Coffee,
            Input::Pickup => Action::Pickup,
            _ => {
                self.notice="Use hjkl/arrows, Enter to interact, Tab for actions. Editing keys belong in consoles.".into();
                return;
            }
        };
        self.game.apply(action, self.safe());
    }
    fn console_input(&mut self, input: Input) {
        let c = self.console.as_mut().unwrap();
        if input == Input::Reset {
            c.editor = Editor::new(c.lesson.lines.clone()).unwrap();
            c.step = 0;
            c.done = false;
            self.notice = "Console reset. No penalty. Your union negotiated this.".into();
            return;
        }
        if input == Input::Example {
            if c.done {
                self.notice = "Already solved. Enter submits.".into();
                return;
            }
            c.assisted = true;
            c.editor = Editor::new(c.lesson.lines.clone()).unwrap();
            for step in &c.lesson.steps[..=c.step] {
                for key in &step.solution {
                    c.editor.apply(key);
                }
            }
            self.notice = format!(
                "Example: {}. Assisted, not mastery.",
                c.lesson.steps[c.step].solution.join(" ")
            );
            c.step += 1;
            c.done = c.step == c.lesson.steps.len();
            return;
        }
        if c.done {
            if matches!(input, Input::Enter | Input::Apply) {
                if !c.assisted {
                    self.profile.credit(c.lesson.kind);
                }
                self.game
                    .apply(Action::Apply, !self.practice && self.lesson < 6);
                self.lesson += 1;
                self.console = None;
                self.pending.clear();
                self.notice = "Work submitted. Management remains astonished.".into();
            } else {
                self.notice = "Enter submits the solved console. Tab offers reset.".into();
            }
            return;
        }
        let mapped = match input {
            Input::Left => Some("h"),
            Input::Right => Some("l"),
            Input::Up => Some("k"),
            Input::Down => Some("j"),
            _ => None,
        };
        if mapped.is_some() {
            c.assisted = true;
        }
        if !matches!(
            input,
            Input::Key(_)
                | Input::Enter
                | Input::Escape
                | Input::Backspace
                | Input::Left
                | Input::Right
                | Input::Up
                | Input::Down
        ) {
            self.notice =
                "Finish the console or use reset/example. Editing pauses the world.".into();
            return;
        }
        let key = String::from(input);
        let key = if c.editor.view().mode == Mode::Normal {
            mapped.unwrap_or(&key)
        } else {
            &key
        };
        let event = c.editor.apply(key);
        if c.lesson.steps[c.step].achieved(c.editor.view(), event.as_deref()) {
            c.step += 1;
            c.done = c.step == c.lesson.steps.len();
            self.notice = if c.done {
                "SUPERVISOR: Prrrp. Competence detected. Enter submits."
            } else {
                "Verified. Next small task."
            }
            .into();
        } else {
            self.notice = c.editor.message().into();
        }
    }
}

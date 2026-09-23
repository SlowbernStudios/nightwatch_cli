use super::{
    repairs::{self, Shift, ShiftView, Stage},
    rng::Seeded,
    world::{Point, Station, SupplyKind, Terrain, World},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    North,
    South,
    West,
    East,
    Northwest,
    Northeast,
    Southwest,
    Southeast,
    Apply,
    Wait,
    Pickup,
    Pet,
    Coffee,
    Feed,
}

impl Action {
    fn delta(self) -> Option<(i32, i32)> {
        Some(match self {
            Self::North => (0, -1),
            Self::South => (0, 1),
            Self::West => (-1, 0),
            Self::East => (1, 0),
            Self::Northwest => (-1, -1),
            Self::Northeast => (1, -1),
            Self::Southwest => (-1, 1),
            Self::Southeast => (1, 1),
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Cat {
    pub position: Point,
    pub fed: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Pockets {
    pub coffee: u32,
    pub biscuit: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    #[serde(flatten)]
    pub shift: ShiftView,
    pub player: Point,
    pub cat: Cat,
    pub turn: u64,
    pub patience: i32,
    pub writeups: u32,
    pub pockets: Pockets,
    pub open_doors: Vec<String>,
    pub collected: Vec<String>,
    pub visible: Vec<String>,
    pub explored: Vec<String>,
    pub visited: Vec<String>,
}
impl std::ops::Deref for GameView {
    type Target = ShiftView;
    fn deref(&self) -> &Self::Target {
        &self.shift
    }
}

#[derive(Clone, Debug)]
pub struct Game {
    world: World,
    shift: Shift,
    player: Point,
    cat: Cat,
    turn: u64,
    patience: i32,
    writeups: u32,
    pockets: Pockets,
    open: HashSet<Point>,
    collected: HashSet<String>,
    visible: HashSet<Point>,
    explored: HashSet<Point>,
    visited: HashSet<String>,
}

fn coordinates(points: &HashSet<Point>) -> Vec<String> {
    let mut result: Vec<_> = points.iter().map(|p| format!("{},{}", p.x, p.y)).collect();
    result.sort();
    result
}
fn strings(set: &HashSet<String>) -> Vec<String> {
    let mut result: Vec<_> = set.iter().cloned().collect();
    result.sort();
    result
}

impl Game {
    pub fn cat_position(&self) -> Point {
        self.cat.position
    }
    pub(crate) fn set_shift_number(&mut self, n: u64) {
        self.shift.state.shift_number = n;
    }
    pub fn turn(&self) -> u64 {
        self.turn
    }
    pub fn player(&self) -> Point {
        self.player
    }
    pub fn stage(&self) -> Stage {
        self.shift.state.stage
    }
    pub fn visible(&self) -> &HashSet<Point> {
        &self.visible
    }
    pub fn explored(&self) -> &HashSet<Point> {
        &self.explored
    }
    pub fn is_open(&self, p: Point) -> bool {
        self.open.contains(&p)
    }
    pub fn is_collected(&self, id: &str) -> bool {
        self.collected.contains(id)
    }
    pub fn supply_underfoot(&self) -> bool {
        self.world
            .view()
            .supplies
            .iter()
            .any(|s| s.position == self.player && !self.collected.contains(&s.id))
    }
    pub fn new(seed: u32) -> Self {
        let world = World::new(seed);
        let player = world.view().start;
        let cat = Cat {
            position: world.view().cat_home,
            fed: false,
        };
        let mut game = Self {
            world,
            shift: Shift::new(seed),
            player,
            cat,
            turn: 0,
            patience: 100,
            writeups: 0,
            pockets: Pockets::default(),
            open: HashSet::new(),
            collected: HashSet::new(),
            visible: HashSet::new(),
            explored: HashSet::new(),
            visited: HashSet::new(),
        };
        game.refresh();
        game
    }
    pub fn view(&self) -> GameView {
        GameView {
            shift: self.shift.state.clone(),
            player: self.player,
            cat: self.cat.clone(),
            turn: self.turn,
            patience: self.patience,
            writeups: self.writeups,
            pockets: self.pockets.clone(),
            open_doors: coordinates(&self.open),
            collected: strings(&self.collected),
            visible: coordinates(&self.visible),
            explored: coordinates(&self.explored),
            visited: strings(&self.visited),
        }
    }
    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn required(&self) -> &'static str {
        self.shift.required()
    }
    pub fn message(&self) -> &str {
        &self.shift.message
    }
    pub fn adjacent_station(&self) -> Option<&Station> {
        let mut nearby = self
            .world
            .view()
            .stations
            .iter()
            .filter(|s| s.position.distance(self.player) <= 1);
        nearby
            .clone()
            .find(|s| s.id == self.required())
            .or_else(|| nearby.next())
    }
    pub fn next_shift(&self) -> Self {
        let seed = if self.shift.state.shift_number.is_multiple_of(2) {
            repairs::complementary(self.shift.state.seed)
        } else {
            repairs::remixed(self.shift.state.seed)
        };
        let mut next = Self::new(seed);
        next.shift.state.shift_number = self.shift.state.shift_number + 1;
        next
    }
    fn clear(&self, p: Point) -> bool {
        self.world.traversable(p)
            && (self.world.terrain(p) != Terrain::Door || self.open.contains(&p))
    }
    fn refresh(&mut self) {
        self.visible.clear();
        for y in self.player.y - 9..=self.player.y + 9 {
            for x in self.player.x - 9..=self.player.x + 9 {
                let p = Point { x, y };
                if self.world.terrain(p) != Terrain::Void
                    && (x - self.player.x).pow(2) + (y - self.player.y).pow(2) <= 81
                    && self.world.line_visible(self.player, p, &self.open)
                {
                    self.visible.insert(p);
                    self.explored.insert(p);
                    if let Some(room) = self.world.room(p) {
                        self.visited.insert(room.id.into());
                    }
                }
            }
        }
    }
    fn tick_start(&mut self) {
        self.turn += 1;
        self.shift.advance();
    }
    fn tick_end(&mut self, safe: bool) {
        if !safe && self.turn.is_multiple_of(25) {
            self.patience -= 1;
        }
        if !safe && self.patience <= 0 {
            self.writeups += 1;
            self.patience = 60;
            self.shift.message = "HR issues a write-up and a mandatory second wind.".into();
        }
        self.supervisor_turn();
        self.refresh();
    }
    fn supervisor_turn(&mut self) {
        if !self.turn.is_multiple_of(3) {
            return;
        }
        let old = self.cat.position;
        let mut options: Vec<_> = old
            .cardinal()
            .into_iter()
            .filter(|p| self.clear(*p) && *p != self.player)
            .collect();
        if self.cat.fed {
            if old.distance(self.player) <= 1 {
                return;
            }
            let mut queue = VecDeque::from([(self.player, 0usize)]);
            let mut distances = HashMap::new();
            while let Some((p, d)) = queue.pop_front() {
                if !self.clear(p) || distances.contains_key(&p) {
                    continue;
                }
                distances.insert(p, d);
                queue.extend(p.cardinal().map(|q| (q, d + 1)));
            }
            options.sort_by_key(|p| distances.get(p).copied().unwrap_or(usize::MAX));
            if let Some(p) = options.first().filter(|p| distances.contains_key(p)) {
                self.cat.position = *p;
            }
        } else {
            let home = self.world.room(self.world.view().cat_home).map(|r| r.id);
            options.retain(|p| self.world.room(*p).map(|r| r.id) == home);
            let i = Seeded::new(self.shift.state.seed ^ self.turn as u32).index(options.len() + 1);
            if let Some(p) = options.get(i) {
                self.cat.position = *p;
            }
        }
    }

    /// Returns whether the action consumed a world turn. Invalid interactions and
    /// collisions can still change feedback, but never advance the supervisor.
    pub fn apply(&mut self, action: Action, safe: bool) -> bool {
        if self.shift.state.stage == Stage::Credits {
            return false;
        }
        if self.shift.state.stage == Stage::Ignition {
            self.tick_start();
            self.tick_end(safe);
            return true;
        }
        if let Some((dx, dy)) = action.delta() {
            let p = self.player.offset(dx, dy);
            if dx != 0
                && dy != 0
                && (!self.clear(self.player.offset(dx, 0))
                    || !self.clear(self.player.offset(0, dy)))
            {
                self.shift.message = "No squeezing through corners. Union rules.".into();
                return false;
            }
            if !self.world.traversable(p) {
                self.shift.message = self
                    .world
                    .view()
                    .stations
                    .iter()
                    .find(|s| s.position == p)
                    .map(|s| format!("{}: {}. Press Enter to apply.", s.glyph, s.label))
                    .unwrap_or_else(|| "Solid. Unlike your employment prospects.".into());
                return false;
            }
            self.tick_start();
            if self.world.terrain(p) == Terrain::Door && !self.open.contains(&p) {
                self.open.insert(p);
                self.shift.message = "The door opens. Your first measurable result.".into();
            } else {
                if self.cat.position == p {
                    self.cat.position = self.player;
                }
                self.player = p;
                if self.world.view().hazards.contains(&p) {
                    if !safe {
                        self.patience -= 8;
                    }
                    self.shift.message = if safe {
                        "Training spill. HR is briefly reasonable."
                    } else {
                        "Squelch. An unlabelled spill. -8 patience."
                    }
                    .into();
                }
                if self
                    .world
                    .view()
                    .supplies
                    .iter()
                    .any(|s| s.position == p && !self.collected.contains(&s.id))
                {
                    self.shift.message =
                        "Supply underfoot. Tab: collect. Procurement looks away.".into();
                }
            }
            self.tick_end(safe);
            return true;
        }
        match action {
            Action::Apply => {
                let Some(id) = self.adjacent_station().map(|s| s.id) else {
                    self.shift.message =
                        "Stand beside a fixture, then press Enter. G is the generator.".into();
                    return false;
                };
                self.tick_start();
                self.shift.interact(id);
            }
            Action::Pet | Action::Feed => {
                if self.cat.position.distance(self.player) > 1 {
                    self.shift.message =
                        "Management is out of reach. Find f, the supervisor.".into();
                    return false;
                }
                if action == Action::Feed && self.pockets.biscuit == 0 {
                    self.shift.message =
                        "No biscuits. Your expense claim has been rejected.".into();
                    return false;
                }
                self.tick_start();
                if action == Action::Pet {
                    self.shift.interact("cat");
                } else {
                    self.pockets.biscuit -= 1;
                    self.cat.fed = true;
                    self.shift.message =
                        "The supervisor accepts your bribe. You now have direct oversight.".into();
                }
            }
            Action::Pickup => {
                let Some(supply) = self
                    .world
                    .view()
                    .supplies
                    .iter()
                    .find(|s| s.position == self.player && !self.collected.contains(&s.id))
                    .cloned()
                else {
                    self.shift.message = "Nothing to collect except responsibility.".into();
                    return false;
                };
                self.tick_start();
                self.collected.insert(supply.id);
                match supply.kind {
                    SupplyKind::Coffee => self.pockets.coffee += 1,
                    SupplyKind::Biscuit => self.pockets.biscuit += 1,
                };
                self.shift.message = "Pocketed. Company property is a state of mind.".into();
            }
            Action::Coffee => {
                if self.pockets.coffee == 0 {
                    self.shift.message = "No pocket coffee. An occupational hazard.".into();
                    return false;
                }
                self.tick_start();
                self.pockets.coffee -= 1;
                self.patience = (self.patience + 35).min(100);
                self.shift.message = "Lukewarm ambition. +35 patience.".into();
            }
            Action::Wait => {
                self.tick_start();
                self.shift.message = if self.shift.state.wait > 0 {
                    "The breaker is thinking."
                } else {
                    "You wait. The building remains broadly disappointing."
                }
                .into();
            }
            _ => unreachable!("movement handled before interactions"),
        }
        self.tick_end(safe);
        true
    }
}

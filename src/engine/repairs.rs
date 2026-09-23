use super::rng::Seeded;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Repair {
    Fuel,
    Calibration,
    Compliance,
    Ventilation,
    Reboot,
    Coffee,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Start,
    Repair,
    Starter,
    Ready,
    Ignition,
    Credits,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftView {
    pub seed: u32,
    pub shift_number: u64,
    pub stage: Stage,
    pub repairs: Vec<Repair>,
    pub repair_index: usize,
    pub step: usize,
    pub inventory: Vec<String>,
    pub elapsed: u32,
    pub pets: u32,
    pub dial: u32,
    pub dial_target: u32,
    pub cranks: u32,
    pub crank_goal: u32,
    pub wait: u32,
}

#[derive(Clone, Debug)]
pub(super) struct Shift {
    pub state: ShiftView,
    pub message: String,
}

impl Shift {
    pub fn new(seed: u32) -> Self {
        let mut rng = Seeded::new(seed ^ 0xcaf);
        Self {
            state: ShiftView {
                seed,
                shift_number: 0,
                stage: Stage::Start,
                repairs: for_seed(seed),
                repair_index: 0,
                step: 0,
                inventory: vec![],
                elapsed: 0,
                pets: 0,
                dial: 0,
                dial_target: 1 + rng.index(3) as u32,
                cranks: 0,
                crank_goal: 3 + rng.index(3) as u32,
                wait: 0,
            },
            message: "DISPATCH: Three small problems. One very large generator.".into(),
        }
    }

    pub fn required(&self) -> &'static str {
        match self.state.stage {
            Stage::Start | Stage::Ready => "generator",
            Stage::Starter => "handle",
            Stage::Ignition | Stage::Credits => "",
            Stage::Repair => steps(self.state.repairs[self.state.repair_index])[self.state.step].0,
        }
    }

    pub fn advance(&mut self) {
        self.state.wait = self.state.wait.saturating_sub(1);
        if self.state.stage == Stage::Ignition {
            self.state.elapsed += 1;
            if self.state.elapsed == 5 {
                self.message = "Power restored to generator status sign.".into();
            }
            if self.state.elapsed >= 8 {
                self.state.stage = Stage::Credits;
                self.message =
                    "Great work. The status sign has power. The generator remains a generator."
                        .into();
            }
        }
    }

    pub fn interact(&mut self, id: &str) {
        if matches!(self.state.stage, Stage::Ignition | Stage::Credits) {
            return;
        }
        if id == "cat" {
            self.message = [
                "SUPERVISOR: Prrrp. Performance upgraded to tolerable.",
                "SUPERVISOR: Prrrr. The only warm response from management.",
                "SUPERVISOR: Mrrp. Overtime approved. For petting.",
            ][self.state.pets as usize % 3]
                .into();
            self.state.pets += 1;
            return;
        }
        if id != self.required() {
            self.message = match id {
                "poster" => "Together, we generate results. Someone crossed out 'together'.",
                "locker" => "Employee of the month. Empty. It has won six times.",
                "office-note" => "Appeal this policy using the form prohibited by this policy.",
                "lost" => "A box of missing deadlines. All accounted for.",
                "records" => "Previous shift: tried turning it on. Findings: inconclusive.",
                _ => "Not required yet. Check TASK for the next fixture.",
            }
            .into();
            return;
        }
        match self.state.stage {
            Stage::Start => {
                self.state.stage = Stage::Repair;
                self.message = "CLICK. Three maintenance requirements. Naturally.".into();
            }
            Stage::Starter => {
                self.state.stage = Stage::Ready;
                self.message = "Auxiliary power source: building main power. Yes, really.".into();
            }
            Stage::Ready => {
                self.state.stage = Stage::Ignition;
                self.state.elapsed = 0;
                self.message = "Stand clear. The machine is about to justify its budget.".into();
            }
            Stage::Repair => {
                let repair = self.state.repairs[self.state.repair_index];
                match repair {
                    Repair::Calibration => {
                        self.state.dial = (self.state.dial + 1) % 4;
                        if self.state.dial != self.state.dial_target {
                            self.message = format!(
                                "Tolerance {}. TASK requests {}.",
                                self.state.dial + 1,
                                self.state.dial_target + 1
                            );
                            return;
                        }
                    }
                    Repair::Ventilation => {
                        self.state.cranks += 1;
                        if self.state.cranks < self.state.crank_goal {
                            self.message = "The air is considering relocation.".into();
                            return;
                        }
                    }
                    Repair::Reboot => {
                        if self.state.step == 0 {
                            self.state.wait = 2;
                        } else if self.state.wait > 0 {
                            self.message =
                                "Two turns. Even the machine is entitled to a break.".into();
                            return;
                        }
                    }
                    _ => (),
                }
                let (_, item, message) = steps(repair)[self.state.step];
                self.state.inventory = item.map(|i| vec![i.to_string()]).unwrap_or_default();
                self.message = message.into();
                self.state.step += 1;
                if self.state.step == steps(repair).len() {
                    self.state.step = 0;
                    self.state.repair_index += 1;
                    self.state.inventory.clear();
                }
                if self.state.repair_index == self.state.repairs.len() {
                    self.state.stage = Stage::Starter;
                }
            }
            _ => (),
        }
    }
}

type Step = (&'static str, Option<&'static str>, &'static str);
fn steps(repair: Repair) -> &'static [Step] {
    match repair {
        Repair::Fuel => &[
            (
                "tool",
                Some("Chain release"),
                "A tool for the tool that releases the fuel. Efficient.",
            ),
            (
                "can",
                Some("Empty fuel can"),
                "The emergency fuel was secured against emergencies.",
            ),
            (
                "pump",
                Some("Full fuel can"),
                "One emergency portion. Please do not have a second emergency.",
            ),
            (
                "generator",
                None,
                "The gauge has moved from EMPTY to EMERGENCY EMPTY.",
            ),
        ],
        Repair::Calibration => &[(
            "dial",
            None,
            "You changed the acceptable range. The machine is exactly as broken as before.",
        )],
        Repair::Compliance => &[
            (
                "form",
                Some("Inspection form"),
                "Please confirm the generator is a generator.",
            ),
            (
                "generator",
                Some("Signed form"),
                "Inspection result: definitely a generator.",
            ),
            (
                "stamp",
                None,
                "Approved by you, independently of the person who filled it in. Also you.",
            ),
        ],
        Repair::Ventilation => &[(
            "fan",
            None,
            "The stale air is now over there. Excellent work.",
        )],
        Repair::Reboot => &[
            (
                "breaker",
                None,
                "Wait two turns for the machine to forget its mistakes.",
            ),
            (
                "breaker",
                None,
                "It remembers everything. We are calling that a successful reboot.",
            ),
        ],
        Repair::Coffee => &[
            (
                "mug",
                Some("Empty mug"),
                "The handle is your share of the benefits package.",
            ),
            (
                "kettle",
                Some("Company coffee"),
                "Contains traces of coffee and a full working day of expectations.",
            ),
            (
                "sip",
                None,
                "Break complete. The time has been deducted from your next break.",
            ),
        ],
    }
}

pub fn for_seed(seed: u32) -> Vec<Repair> {
    let mut pool = [
        Repair::Fuel,
        Repair::Calibration,
        Repair::Compliance,
        Repair::Ventilation,
        Repair::Reboot,
        Repair::Coffee,
    ];
    Seeded::new((seed >> 1) ^ 0xa531).shuffle(&mut pool);
    pool[if seed & 1 == 0 { 0..3 } else { 3..6 }].to_vec()
}

pub fn complementary(seed: u32) -> u32 {
    let previous = for_seed(seed);
    (1..=256)
        .map(|n| seed.wrapping_add(n))
        .find(|s| for_seed(*s).iter().all(|r| !previous.contains(r)))
        .unwrap_or(seed ^ 1)
}

pub fn remixed(seed: u32) -> u32 {
    let previous = for_seed(seed);
    (1..=256)
        .map(|n| seed.wrapping_add(n))
        .find(|s| {
            let count = for_seed(*s).iter().filter(|r| previous.contains(r)).count();
            count > 0 && count < 3
        })
        .unwrap_or_else(|| complementary(seed))
}

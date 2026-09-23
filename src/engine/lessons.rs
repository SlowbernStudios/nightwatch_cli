use super::rng::Seeded;
use crate::editor::{EditorView, Mode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Words,
    Bounds,
    Insert,
    Undo,
}
impl Kind {
    pub const ALL: [Self; 4] = [Self::Words, Self::Bounds, Self::Insert, Self::Undo];
}
#[derive(Clone, Debug)]
pub struct Step {
    pub instruction: String,
    pub hint: String,
    pub solution: Vec<String>,
    event: &'static str,
    row: Option<usize>,
    col: Option<usize>,
    lines: Option<Vec<String>>,
}
impl Step {
    pub fn achieved(&self, e: &EditorView, event: Option<&str>) -> bool {
        e.mode == Mode::Normal
            && event == Some(self.event)
            && self.row.is_none_or(|r| r == e.row)
            && self.col.is_none_or(|c| c == e.col)
            && self.lines.as_ref().is_none_or(|l| *l == e.lines)
    }
}
#[derive(Clone, Debug)]
pub struct Lesson {
    pub kind: Kind,
    pub title: &'static str,
    pub intro: &'static str,
    pub lines: Vec<String>,
    pub steps: Vec<Step>,
}
fn cursor(instruction: String, hint: &str, key: &'static str, col: usize) -> Step {
    Step {
        instruction,
        hint: hint.into(),
        solution: vec![key.into()],
        event: key,
        row: Some(0),
        col: Some(col),
        lines: None,
    }
}
fn text(
    instruction: &str,
    hint: &str,
    keys: &[&str],
    event: &'static str,
    lines: Vec<String>,
) -> Step {
    Step {
        instruction: instruction.into(),
        hint: hint.into(),
        solution: keys.iter().map(|s| s.to_string()).collect(),
        event,
        row: None,
        col: None,
        lines: Some(lines),
    }
}
impl Lesson {
    pub fn new(kind: Kind, seed: u32) -> Self {
        let mut rng = Seeded::new(seed ^ 0xbeef);
        let object = ["TURBINE", "GENERATOR", "BOILER", "FAN", "PUMP"][rng.index(5)];
        let boss = ["manager", "supervisor", "committee", "director"][rng.index(4)];
        match kind {
            Kind::Words => {
                let first = ["permit", "memo", "form", "waiver"][rng.index(4)];
                let start = first.len() + 1;
                Self {
                    kind,
                    title: "Corporate terminology",
                    intro: "Locate the person who approved this. They deny being a word.",
                    lines: vec![format!("{first} {boss} {}", object.to_lowercase())],
                    steps: vec![
                        cursor(
                            format!("Reach the start of {boss}."),
                            "w: next word start",
                            "w",
                            start,
                        ),
                        cursor(
                            format!("Reach the end of {boss}."),
                            "e: word end",
                            "e",
                            start + boss.len() - 1,
                        ),
                        cursor(
                            format!("Return to the start of {boss}."),
                            "b: previous word start",
                            "b",
                            start,
                        ),
                    ],
                }
            }
            Kind::Bounds => {
                let line = format!("   {object} LIABILITY ENDS HERE");
                let last = line.len() - 1;
                Self {
                    kind,
                    title: "Terms and conditions",
                    intro: "Read both ends. The middle has been outsourced.",
                    lines: vec![line],
                    steps: vec![
                        cursor(
                            "Reach the last character.".into(),
                            "$: end of line",
                            "$",
                            last,
                        ),
                        cursor(
                            "Reach column 1, including its space.".into(),
                            "0: first column, not first word",
                            "0",
                            0,
                        ),
                    ],
                }
            }
            Kind::Insert => {
                let prefix = format!("{object}=");
                let mut step = text(
                    "Insert ON before ; then press Escape.",
                    &format!(
                        "{}l, i, type ON, Escape. Insert BEFORE the cursor.",
                        prefix.len()
                    ),
                    &[],
                    "escape",
                    vec![format!("{prefix}ON;")],
                );
                step.solution = prefix
                    .len()
                    .to_string()
                    .chars()
                    .map(|c| c.to_string())
                    .chain(["l", "i", "O", "N", "escape"].map(String::from))
                    .collect();
                Self {
                    kind,
                    title: "Missing power setting",
                    intro: "The power setting was removed to reduce overhead.",
                    lines: vec![format!("{prefix};")],
                    steps: vec![step],
                }
            }
            Kind::Undo => {
                let lines = vec![
                    format!("DUPLICATE {object} REPORT"),
                    format!("X{object}=APPROVED"),
                ];
                let valid = vec![lines[1].clone()];
                Self {
                    kind,
                    title: "Recoverable paperwork",
                    intro: "Delete. Regret. Undo. Management calls this a process.",
                    lines: lines.clone(),
                    steps: vec![
                        text(
                            "Delete the duplicate report line.",
                            "dd: delete current line",
                            &["d", "d"],
                            "dd",
                            valid.clone(),
                        ),
                        text(
                            "Restore it. Document the regret.",
                            "u: undo the last change",
                            &["u"],
                            "u",
                            lines,
                        ),
                        text(
                            "Delete the duplicate again. With confidence.",
                            "dd: remove the report again",
                            &["d", "d"],
                            "dd",
                            valid.clone(),
                        ),
                        text(
                            "Remove the stray X from the permit.",
                            "x: delete under cursor",
                            &["x"],
                            "x",
                            vec![valid[0][1..].into()],
                        ),
                    ],
                }
            }
        }
    }
}

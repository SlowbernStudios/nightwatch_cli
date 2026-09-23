use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub lines: Vec<String>,
    pub row: usize,
    pub col: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Normal,
    Insert,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Wanted {
    Column(usize),
    End(End),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum End {
    End,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EditorView {
    #[serde(flatten)]
    pub text: Snapshot,
    pub mode: Mode,
    pub pending: String,
    pub wanted: Wanted,
    pub undo: Vec<Snapshot>,
    pub insertion: Option<Snapshot>,
}
impl std::ops::Deref for EditorView {
    type Target = Snapshot;
    fn deref(&self) -> &Snapshot {
        &self.text
    }
}
#[derive(Clone, Debug)]
pub struct Editor {
    state: EditorView,
    message: String,
}
impl Editor {
    pub fn new(lines: Vec<String>) -> Result<Self, &'static str> {
        if lines.is_empty()
            || lines.len() > 32
            || lines
                .iter()
                .any(|l| l.len() > 120 || !l.bytes().all(|c| (32..=126).contains(&c)))
        {
            return Err("Console requires 1-32 ASCII lines of at most 120 columns.");
        }
        Ok(Self {
            state: EditorView {
                text: Snapshot {
                    lines,
                    row: 0,
                    col: 0,
                },
                mode: Mode::Normal,
                pending: String::new(),
                wanted: Wanted::Column(0),
                undo: vec![],
                insertion: None,
            },
            message: String::new(),
        })
    }
    pub fn view(&self) -> &EditorView {
        &self.state
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    fn clamp(&mut self) {
        let s = &mut self.state;
        s.text.row = s.row.min(s.lines.len() - 1);
        s.text.col = s.col.min(
            s.lines[s.row]
                .len()
                .saturating_sub(usize::from(s.mode == Mode::Normal)),
        );
    }
    fn remember(&mut self, snapshot: Snapshot) {
        self.state.undo.push(snapshot);
        if self.state.undo.len() > 100 {
            self.state.undo.remove(0);
        }
    }
    fn finish_insert_group(&mut self) {
        if let Some(s) = self.state.insertion.take()
            && s.lines != self.state.lines
        {
            self.remember(s);
        }
    }
    fn wanted(&mut self) {
        self.state.wanted = Wanted::Column(self.state.col);
    }
    fn word(&mut self, motion: &str) {
        let text = self.state.lines.join("\n");
        let bytes = text.as_bytes();
        let mut p = self.state.col
            + self.state.lines[..self.state.row]
                .iter()
                .map(|l| l.len() + 1)
                .sum::<usize>();
        let kind = |i: usize| -> u8 {
            match bytes.get(i) {
                None | Some(b' ' | b'\n') => 0,
                Some(c) if c.is_ascii_alphanumeric() || *c == b'_' => 1,
                _ => 2,
            }
        };
        let empty =
            |i: usize| bytes.get(i) == Some(&b'\n') && (i == 0 || bytes.get(i - 1) == Some(&b'\n'));
        match motion {
            "w" => {
                let k = kind(p);
                if k > 0 {
                    while p < bytes.len() && kind(p) == k {
                        p += 1;
                    }
                } else if empty(p) {
                    p += 1;
                }
                while p < bytes.len() && kind(p) == 0 && !empty(p) {
                    p += 1;
                }
            }
            "b" => {
                p = p.saturating_sub(1);
                while p > 0 && kind(p) == 0 && !empty(p) {
                    p -= 1;
                }
                let k = kind(p);
                if k > 0 {
                    while p > 0 && kind(p - 1) == k {
                        p -= 1;
                    }
                }
            }
            _ => {
                p = (p + 1).min(bytes.len().saturating_sub(1));
                while p < bytes.len().saturating_sub(1) && kind(p) == 0 {
                    p += 1;
                }
                let k = kind(p);
                while p < bytes.len().saturating_sub(1) && k > 0 && kind(p + 1) == k {
                    p += 1;
                }
            }
        }
        p = p.min(bytes.len().saturating_sub(1));
        let mut row = 0;
        while row < self.state.lines.len() - 1 && p > self.state.lines[row].len() {
            p -= self.state.lines[row].len() + 1;
            row += 1;
        }
        self.state.text.row = row;
        self.state.text.col = p;
        self.clamp();
        self.wanted();
    }
    /// Returns a completed command, never a pending operator or insert character.
    pub fn apply(&mut self, key: &str) -> Option<String> {
        self.message.clear();
        if key == "escape" {
            self.state.pending.clear();
            if self.state.mode == Mode::Insert {
                self.finish_insert_group();
                self.state.mode = Mode::Normal;
                self.state.text.col = self.state.col.saturating_sub(1);
                self.clamp();
                self.wanted();
            }
            return Some(key.into());
        }
        if self.state.mode == Mode::Insert {
            self.insert(key);
            return None;
        }
        if key.len() == 1
            && key.as_bytes()[0].is_ascii_digit()
            && !(key == "0" && (self.state.pending.is_empty() || self.state.pending.ends_with('d')))
        {
            let next = format!("{}{key}", self.state.pending);
            if next
                .rsplit('d')
                .next()
                .unwrap()
                .parse::<u32>()
                .unwrap_or(100)
                > 99
            {
                self.state.pending.clear();
                self.message = "Training counts are limited to 99.".into();
            } else {
                self.state.pending = next;
            }
            return None;
        }
        if key == "d" && !self.state.pending.contains('d') {
            self.state.pending.push('d');
            return None;
        }
        let pending = std::mem::take(&mut self.state.pending);
        let count: usize = pending
            .split('d')
            .map(|p| p.parse::<usize>().unwrap_or(1))
            .product();
        if count > 99 {
            self.message = "Training counts are limited to 99.".into();
            return None;
        }
        if pending.contains('d') {
            if key != "d" {
                self.message = "Only dd is supported for operators.".into();
                return None;
            }
            self.remember(self.state.text.clone());
            let row = self.state.row;
            let end = (row + count).min(self.state.lines.len());
            self.state.text.lines.drain(row..end);
            if self.state.lines.is_empty() {
                self.state.text.lines.push(String::new());
            }
            self.state.text.row = row.min(self.state.lines.len() - 1);
            self.state.text.col = self.state.lines[self.state.row]
                .bytes()
                .position(|c| c != b' ')
                .unwrap_or(0);
            self.clamp();
            self.wanted();
            return Some("dd".into());
        }
        match key {
            "h" | "l" => {
                self.state.text.col = if key == "h" {
                    self.state.col.saturating_sub(count)
                } else {
                    self.state.col + count
                };
                self.clamp();
                self.wanted();
            }
            "j" | "k" => {
                self.state.text.row = if key == "k" {
                    self.state.row.saturating_sub(count)
                } else {
                    (self.state.row + count).min(self.state.lines.len() - 1)
                };
                self.state.text.col = match self.state.wanted {
                    Wanted::Column(c) => c,
                    Wanted::End(_) => usize::MAX,
                };
                self.clamp();
            }
            "w" | "b" | "e" => {
                for _ in 0..count {
                    self.word(key);
                }
            }
            "0" => {
                self.state.text.col = 0;
                self.wanted();
            }
            "$" => {
                self.state.text.row = (self.state.row + count - 1).min(self.state.lines.len() - 1);
                self.state.text.col = self.state.lines[self.state.row].len().saturating_sub(1);
                self.state.wanted = Wanted::End(End::End);
            }
            "i" if pending.is_empty() => {
                self.state.mode = Mode::Insert;
                self.state.insertion = Some(self.state.text.clone());
            }
            "x" => {
                let row = self.state.row;
                let col = self.state.col;
                if !self.state.lines[row].is_empty() {
                    self.remember(self.state.text.clone());
                    let end = (col + count).min(self.state.lines[row].len());
                    self.state.text.lines[row].replace_range(col..end, "");
                    self.clamp();
                    self.wanted();
                }
            }
            "u" => {
                for _ in 0..count {
                    if let Some(s) = self.state.undo.pop() {
                        self.state.text = s;
                        self.clamp();
                        self.wanted();
                    } else {
                        self.message = "Already at oldest change.".into();
                        break;
                    }
                }
            }
            _ => {
                self.message = "Supported: hjkl w b e 0 $ i x dd u and Escape.".into();
                return None;
            }
        }
        Some(key.into())
    }
    fn insert(&mut self, key: &str) {
        if matches!(key, "left" | "right" | "up" | "down") {
            self.finish_insert_group();
            match key {
                "left" => self.state.text.col = self.state.col.saturating_sub(1),
                "right" => self.state.text.col += 1,
                "up" => self.state.text.row = self.state.row.saturating_sub(1),
                _ => self.state.text.row += 1,
            };
            self.clamp();
            self.wanted();
            self.state.insertion = Some(self.state.text.clone());
            return;
        }
        let row = self.state.row;
        let col = self.state.col;
        match key {
            "backspace" => {
                if col > 0 {
                    self.state.text.lines[row].remove(col - 1);
                    self.state.text.col -= 1;
                } else if row > 0
                    && self.state.lines[row - 1].len() + self.state.lines[row].len() <= 120
                {
                    self.state.text.col = self.state.lines[row - 1].len();
                    let line = self.state.text.lines.remove(row);
                    self.state.text.lines[row - 1].push_str(&line);
                    self.state.text.row -= 1;
                }
            }
            "enter" => {
                if self.state.lines.len() == 32 {
                    self.message = "Console limit: 32 lines.".into();
                } else {
                    let right = self.state.text.lines[row].split_off(col);
                    self.state.text.lines.insert(row + 1, right);
                    self.state.text.row += 1;
                    self.state.text.col = 0;
                }
            }
            _ if key.len() == 1 && (32..=126).contains(&key.as_bytes()[0]) => {
                if self.state.lines[row].len() == 120 {
                    self.message = "Console limit: 120 columns.".into();
                } else {
                    self.state.text.lines[row].insert_str(col, key);
                    self.state.text.col += 1;
                }
            }
            _ => self.message = "Escape returns to NORMAL mode.".into(),
        }
    }
}

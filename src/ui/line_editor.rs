pub struct LineEditor {
    pub text: String,
    pub cursor: usize,
    pub visible: bool,
    blink_counter: f32,
}

impl Default for LineEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl LineEditor {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            visible: true,
            blink_counter: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    pub fn set(&mut self, text: String) {
        self.text = text;
        self.cursor = self.text.len();
    }

    pub fn insert(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn append(&mut self, c: char) {
        self.text.push(c);
        self.cursor = self.text.len();
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let prev = prev_char_boundary(&self.text, self.cursor);
        self.text.drain(prev..self.cursor);
        self.cursor = prev;
    }

    pub fn backspace_end(&mut self) {
        self.text.pop();
        self.cursor = self.text.len();
    }

    pub fn delete(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        let next = next_char_boundary(&self.text, self.cursor);
        self.text.drain(self.cursor..next);
    }

    pub fn left(&mut self) {
        self.cursor = prev_char_boundary(&self.text, self.cursor);
    }

    pub fn right(&mut self) {
        self.cursor = next_char_boundary(&self.text, self.cursor);
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.text.len();
    }

    pub fn tick_blink(&mut self, fps: f32, blink_enabled: bool) {
        if !blink_enabled {
            self.reset_blink();
            return;
        }
        self.blink_counter += 1.0;
        let half_period = (fps * 0.5).max(1.0);
        if self.blink_counter >= half_period {
            self.blink_counter = 0.0;
            self.visible = !self.visible;
        }
    }

    pub fn reset_blink(&mut self) {
        self.visible = true;
        self.blink_counter = 0.0;
    }
}

pub struct CommandHistory {
    entries: Vec<String>,
    pos: Option<usize>,
    draft: String,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            pos: None,
            draft: String::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn record(&mut self, entry: String) {
        self.entries.push(entry);
    }

    pub fn reset(&mut self) {
        self.pos = None;
        self.draft.clear();
    }

    pub fn older(&mut self, current: &str) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }
        match self.pos {
            None => {
                self.draft = current.to_string();
                let last = self.entries.len() - 1;
                self.pos = Some(last);
                Some(self.entries[last].clone())
            }
            Some(0) => None,
            Some(i) => {
                self.pos = Some(i - 1);
                Some(self.entries[i - 1].clone())
            }
        }
    }

    pub fn newer(&mut self) -> Option<String> {
        match self.pos {
            None => None,
            Some(i) if i + 1 < self.entries.len() => {
                self.pos = Some(i + 1);
                Some(self.entries[i + 1].clone())
            }
            Some(_) => {
                self.pos = None;
                Some(self.draft.clone())
            }
        }
    }
}

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }
    let mut i = pos - 1;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    let mut i = pos + 1;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

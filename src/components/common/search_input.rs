use crossterm::event::{KeyCode, KeyEvent};

pub struct SearchInput {
    pub value: String,
    placeholder: String,
}

impl SearchInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: "Type to search...".to_string(),
        }
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                self.value.push(c);
                true
            }
            KeyCode::Backspace => {
                self.value.pop();
                true
            }
            _ => false,
        }
    }
}

impl Default for SearchInput {
    fn default() -> Self {
        Self::new()
    }
}

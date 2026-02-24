use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::prelude::Text;

use crate::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::Component;

#[derive(Debug, Default, Clone)]
pub struct InputWidget {
    pub value: String,
    pub placeholder: String,
    pub cursor_pos: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub is_active: bool,
    pub max_length: Option<usize>,
}

impl InputWidget {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: "Enter text...".to_string(),
            cursor_pos: 0,
            history: Vec::new(),
            history_index: None,
            is_active: false,
            max_length: None,
        }
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn with_initial_value(mut self, value: String) -> Self {
        self.value = value;
        self.cursor_pos = self.value.len();
        self
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_pos = 0;
    }

    pub fn submit(&mut self) -> String {
        let value = self.value.clone();
        if !value.is_empty() {
            self.history.push(value.clone());
            if self.history.len() > 100 {
                self.history.remove(0);
            }
        }
        self.clear();
        self.history_index = None;
        value
    }

    pub fn insert_char(&mut self, c: char) {
        if let Some(max_length) = self.max_length {
            if self.value.len() >= max_length {
                return;
            }
        }

        self.value.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.value.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_pos < self.value.len() {
            self.value.remove(self.cursor_pos);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.value.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn move_cursor_to_start(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn move_cursor_to_end(&mut self) {
        self.cursor_pos = self.value.len();
    }

    pub fn move_cursor_word_left(&mut self) {
        if self.cursor_pos > 0 {
            let mut pos = self.cursor_pos - 1;
            while pos > 0 && !self.value.is_char_boundary(pos) {
                pos -= 1;
            }
            while pos > 0 && self.value.as_bytes()[pos - 1].is_ascii_whitespace() {
                pos -= 1;
                while pos > 0 && !self.value.is_char_boundary(pos) {
                    pos -= 1;
                }
            }
            while pos > 0 && !self.value.as_bytes()[pos - 1].is_ascii_whitespace() {
                pos -= 1;
                while pos > 0 && !self.value.is_char_boundary(pos) {
                    pos -= 1;
                }
            }
            self.cursor_pos = pos;
        }
    }

    pub fn move_cursor_word_right(&mut self) {
        if self.cursor_pos < self.value.len() {
            let mut pos = self.cursor_pos;
            while pos < self.value.len() && !self.value.as_bytes()[pos].is_ascii_whitespace() {
                pos += 1;
                while pos < self.value.len() && !self.value.is_char_boundary(pos) {
                    pos += 1;
                }
            }
            while pos < self.value.len() && self.value.as_bytes()[pos].is_ascii_whitespace() {
                pos += 1;
                while pos < self.value.len() && !self.value.is_char_boundary(pos) {
                    pos += 1;
                }
            }
            self.cursor_pos = pos;
        }
    }

    pub fn history_prev(&mut self) {
        if !self.history.is_empty() {
            let index = self.history_index.map_or(self.history.len(), |i| i);
            if index > 0 {
                let new_index = index - 1;
                self.history_index = Some(new_index);
                self.value = self.history[new_index].clone();
                self.cursor_pos = self.value.len();
            }
        }
    }

    pub fn history_next(&mut self) {
        if !self.history.is_empty() {
            let index = self.history_index.map_or(0, |i| i);
            if index < self.history.len() - 1 {
                let new_index = index + 1;
                self.history_index = Some(new_index);
                self.value = self.history[new_index].clone();
                self.cursor_pos = self.value.len();
            } else {
                self.history_index = None;
                self.value.clear();
                self.cursor_pos = 0;
            }
        }
    }

    pub fn get_cursor_position(&self) -> usize {
        self.cursor_pos
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl Component for InputWidget {
    fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Input")
            .style(Style::default().fg(Color::Cyan));

        let input = if self.value.is_empty() {
            self.placeholder.clone()
        } else {
            self.value.clone()
        };

        let style = if self.is_active {
            Style::default().fg(Color::White).bg(Color::Blue)
        } else {
            Style::default().fg(Color::White)
        };

        let paragraph = Paragraph::new(Text::styled(input, style))
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);

        if self.is_active {
            frame.set_cursor(
                area.x + 1 + self.cursor_pos as u16,
                area.y + 1,
            );
        }
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        if !self.is_active {
            return false;
        }

        match event {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Char(c) => {
                        self.insert_char(c);
                        true
                    }
                    KeyCode::Backspace => {
                        self.delete_char();
                        true
                    }
                    KeyCode::Delete => {
                        self.delete_char_forward();
                        true
                    }
                    KeyCode::Left => {
                        self.move_cursor_left();
                        true
                    }
                    KeyCode::Right => {
                        self.move_cursor_right();
                        true
                    }
                    KeyCode::Up => {
                        self.history_prev();
                        true
                    }
                    KeyCode::Down => {
                        self.history_next();
                        true
                    }
                    KeyCode::Home => {
                        self.move_cursor_to_start();
                        true
                    }
                    KeyCode::End => {
                        self.move_cursor_to_end();
                        true
                    }
                    KeyCode::Tab => {
                        // 处理 Tab 键
                        self.insert_char('\t');
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

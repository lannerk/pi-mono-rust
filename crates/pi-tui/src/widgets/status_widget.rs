use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::Line;

use crate::event::{Event};
use crate::Component;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum StatusLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Default, Clone)]
pub struct StatusItem {
    pub message: String,
    pub level: StatusLevel,
    pub timestamp: String,
}

#[derive(Debug, Default, Clone)]
pub struct StatusWidget {
    items: Vec<StatusItem>,
    max_items: usize,
    auto_clear: bool,
    clear_after: Option<chrono::Duration>,
}

impl StatusWidget {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_items: 10,
            auto_clear: false,
            clear_after: None,
        }
    }

    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items = max;
        self
    }

    pub fn with_auto_clear(mut self, auto: bool) -> Self {
        self.auto_clear = auto;
        self
    }

    pub fn with_clear_after(mut self, duration: chrono::Duration) -> Self {
        self.clear_after = Some(duration);
        self
    }

    pub fn info(&mut self, message: String) {
        self.add_item(StatusLevel::Info, message);
    }

    pub fn success(&mut self, message: String) {
        self.add_item(StatusLevel::Success, message);
    }

    pub fn warning(&mut self, message: String) {
        self.add_item(StatusLevel::Warning, message);
    }

    pub fn error(&mut self, message: String) {
        self.add_item(StatusLevel::Error, message);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn items(&self) -> &Vec<StatusItem> {
        &self.items
    }

    fn add_item(&mut self, level: StatusLevel, message: String) {
        let item = StatusItem {
            message,
            level,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        };

        self.items.push(item);

        if self.items.len() > self.max_items {
            self.items.remove(0);
        }

        if self.auto_clear && self.clear_after.is_some() {
            // 这里可以添加一个定时器来自动清除状态
        }
    }
}

impl Component for StatusWidget {
    fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Status")
            .style(Style::default().fg(Color::Cyan));

        let visible_height = area.height.saturating_sub(2);
        let start_idx = self.items.len().saturating_sub(visible_height as usize);
        let visible_items = &self.items[start_idx..];

        let mut lines = Vec::new();

        for item in visible_items {
            let level_style = match item.level {
                StatusLevel::Info => Style::default().fg(Color::White),
                StatusLevel::Success => Style::default().fg(Color::Green),
                StatusLevel::Warning => Style::default().fg(Color::Yellow),
                StatusLevel::Error => Style::default().fg(Color::Red),
            };

            let line = Line::from(format!("[{}] {}: {}", item.timestamp, self.level_to_string(&item.level), item.message))
                .style(level_style);

            lines.push(line);
        }

        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, _event: &Event) -> bool {
        // StatusWidget 暂时不处理事件
        false
    }
}

impl StatusWidget {
    fn level_to_string(&self, level: &StatusLevel) -> &'static str {
        match level {
            StatusLevel::Info => "INFO",
            StatusLevel::Success => "SUCCESS",
            StatusLevel::Warning => "WARNING",
            StatusLevel::Error => "ERROR",
        }
    }
}

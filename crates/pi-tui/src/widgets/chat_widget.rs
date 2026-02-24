use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::prelude::Text;

use crate::event::{Event};
use crate::Component;

#[derive(Debug, Default, Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
    pub is_user: bool,
}

#[derive(Debug, Default, Clone)]
pub struct ChatWidget {
    messages: Vec<ChatMessage>,
    max_messages: usize,
    show_timestamps: bool,
    show_sender: bool,
}

impl ChatWidget {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 100,
            show_timestamps: true,
            show_sender: true,
        }
    }

    pub fn with_max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }

    pub fn with_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    pub fn with_sender(mut self, show: bool) -> Self {
        self.show_sender = show;
        self
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);

        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        let message = ChatMessage {
            sender: "You".to_string(),
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            is_user: true,
        };
        self.add_message(message);
    }

    pub fn add_ai_message(&mut self, content: String) {
        let message = ChatMessage {
            sender: "AI".to_string(),
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            is_user: false,
        };
        self.add_message(message);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn messages(&self) -> &Vec<ChatMessage> {
        &self.messages
    }
}

impl Component for ChatWidget {
    fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Chat")
            .style(Style::default().fg(Color::Cyan));

        let visible_height = area.height.saturating_sub(2);
        let start_idx = self.messages.len().saturating_sub(visible_height as usize);
        let visible_messages = &self.messages[start_idx..];

        let items = visible_messages.iter().map(|msg| {
            let mut line = String::new();

            if self.show_sender {
                line.push_str(&format!("{}: ", msg.sender));
            }

            line.push_str(&msg.content);

            if self.show_timestamps {
                line.push_str(&format!(" [{}]", msg.timestamp));
            }

            let style = if msg.is_user {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Yellow)
            };

            ListItem::new(Text::styled(line, style))
        }).collect::<Vec<_>>();

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(list, area);
    }

    fn handle_event(&mut self, _event: &Event) -> bool {
        // ChatWidget 暂时不处理事件
        false
    }
}

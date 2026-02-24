use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::Line;

use crate::event::{Event};
use crate::Component;

#[derive(Debug, Default, Clone)]
pub struct TextWidget {
    content: String,
    title: String,
    wrap: bool,
    scrollable: bool,
    scroll_offset: usize,
    max_height: Option<u16>,
}

impl TextWidget {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            title: "Text".to_string(),
            wrap: true,
            scrollable: true,
            scroll_offset: 0,
            max_height: None,
        }
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn with_scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    pub fn with_max_height(mut self, height: u16) -> Self {
        self.max_height = Some(height);
        self
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.scroll_offset = 0;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn scroll_up(&mut self, lines: usize) {
        if self.scroll_offset >= lines {
            self.scroll_offset -= lines;
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset += lines;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self, line_count: usize) {
        self.scroll_offset = line_count.saturating_sub(self.visible_lines() as usize);
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn visible_lines(&self) -> u16 {
        20 // 默认值，实际应该根据区域高度计算
    }
}

impl Component for TextWidget {
    fn draw(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title.as_str())
            .style(Style::default().fg(Color::Cyan));

        let content = self.content.clone();
        let lines = content.lines().collect::<Vec<_>>();
        let visible_height = area.height.saturating_sub(2);

        let start_idx = self.scroll_offset.min(lines.len().saturating_sub(visible_height as usize));
        let end_idx = (start_idx + visible_height as usize).min(lines.len());
        let visible_lines = &lines[start_idx..end_idx];

        let text_lines = visible_lines
            .iter()
            .map(|line| Line::from(line.to_string()))
            .collect::<Vec<_>>();

        let mut paragraph = Paragraph::new(text_lines)
            .block(block)
            .style(Style::default().fg(Color::White));

        if self.wrap {
            paragraph = paragraph.wrap(ratatui::widgets::Wrap::default());
        }

        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        if !self.scrollable {
            return false;
        }

        match event {
            Event::Key(key_event) => {
                match key_event.code {
                    crate::event::KeyCode::Up => {
                        self.scroll_up(1);
                        true
                    }
                    crate::event::KeyCode::Down => {
                        self.scroll_down(1);
                        true
                    }
                    crate::event::KeyCode::PageUp => {
                        self.scroll_up(10);
                        true
                    }
                    crate::event::KeyCode::PageDown => {
                        self.scroll_down(10);
                        true
                    }
                    crate::event::KeyCode::Home => {
                        self.scroll_to_top();
                        true
                    }
                    crate::event::KeyCode::End => {
                        let line_count = self.content.lines().count();
                        self.scroll_to_bottom(line_count);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

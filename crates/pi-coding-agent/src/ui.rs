use pi_agent_core::Agent;
use pi_tui::widgets::chat_widget::ChatMessage;
use pi_tui::widgets::{ChatWidget, InputWidget, StatusWidget};
use pi_tui::{App, Component, Event, KeyCode};
use std::sync::Arc;
use tokio;

pub struct ChatUi {
    agent: Arc<Agent>,
    chat_widget: ChatWidget,
    input_widget: InputWidget,
    status_widget: StatusWidget,
}

impl ChatUi {
    pub fn new(agent: Arc<Agent>) -> Self {
        Self {
            agent,
            chat_widget: ChatWidget::new(),
            input_widget: InputWidget::new(),
            status_widget: StatusWidget::new(),
        }
    }

    async fn send_message(&mut self, message: String) {
        self.chat_widget.add_message(ChatMessage::user(message.clone()));
        
        match self.agent.chat(message).await {
            Ok(result) => {
                if let Some(last_message) = result.messages.last() {
                    self.chat_widget.add_message(ChatMessage::assistant(last_message.content.clone()));
                }
            }
            Err(e) => {
                self.chat_widget.add_message(ChatMessage::assistant(format!("Error: {}", e)));
            }
        }
    }
}

impl Component for ChatUi {
    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) => {
                if self.input_widget.handle_event(event) {
                    if let KeyCode::Enter = key.code {
                        let input = self.input_widget.get_input().to_string();
                        self.input_widget.clear();
                        let mut chat_ui = self.clone();
                        tokio::spawn(async move {
                            chat_ui.send_message(input).await;
                        });
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, frame: &mut pi_tui::Frame) {
        let size = frame.size();
        
        let chat_area = pi_tui::layout::Rect {
            x: 0,
            y: 0,
            width: size.width,
            height: size.height - 3,
        };
        
        let input_area = pi_tui::layout::Rect {
            x: 0,
            y: size.height - 3,
            width: size.width,
            height: 2,
        };
        
        let status_area = pi_tui::layout::Rect {
            x: 0,
            y: size.height - 1,
            width: size.width,
            height: 1,
        };
        
        self.chat_widget.render(frame, chat_area);
        self.input_widget.render(frame, input_area);
        self.status_widget.render(frame, status_area);
    }
}

impl Clone for ChatUi {
    fn clone(&self) -> Self {
        Self {
            agent: self.agent.clone(),
            chat_widget: self.chat_widget.clone(),
            input_widget: self.input_widget.clone(),
            status_widget: self.status_widget.clone(),
        }
    }
}

pub async fn run_chat_ui(agent: Arc<Agent>) -> anyhow::Result<()> {
    let chat_ui = ChatUi::new(agent);
    let mut app = App::new(chat_ui).await?;
    app.run().await?;
    Ok(())
}

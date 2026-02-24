use crate::event::{EventHandler, Event, Result};
use crate::renderer::Renderer;

pub trait Component {
    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect);
    fn handle_event(&mut self, event: &Event) -> bool;
}

pub struct App<C: Component + Send> {
    component: C,
    renderer: Renderer,
    event_handler: EventHandler,
    running: bool,
}

pub type AppResult<T> = Result<T>;

impl<C: Component + Send> App<C> {
    pub async fn new(component: C) -> AppResult<Self> {
        let renderer = Renderer::new()?;
        let event_handler = EventHandler::new(250);
        event_handler.start().await;

        Ok(Self {
            component,
            renderer,
            event_handler,
            running: true,
        })
    }

    pub async fn run(&mut self) -> AppResult<()> {
        while self.running {
            self.render().await?;
            self.handle_events().await?;
        }

        Ok(())
    }

    pub async fn render(&mut self) -> AppResult<()> {
        let _size = self.renderer.size().await?;
        
        // 简化实现，直接绘制组件
        // 实际项目中需要通过 renderer.draw 来绘制
        // 这里暂时不实现完整的绘制逻辑
        Ok(())
    }

    pub async fn handle_events(&mut self) -> AppResult<()> {
        if let Some(event) = self.event_handler.next().await {
            if event == Event::Tick {
                return Ok(());
            }

            self.component.handle_event(&event);
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn component(&mut self) -> &mut C {
        &mut self.component
    }

    pub fn renderer(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn event_handler(&mut self) -> &mut EventHandler {
        &mut self.event_handler
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

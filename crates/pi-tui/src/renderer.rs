use ratatui::Frame;
use std::sync::Arc;

use crate::terminal::Terminal;
use crate::Result;

pub struct Renderer {
    terminal: Arc<tokio::sync::Mutex<Terminal>>,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: Arc::new(tokio::sync::Mutex::new(Terminal::new()?)),
        })
    }

    pub async fn draw<F>(&self, _f: F) -> Result<()>
    where
        F: FnOnce(&mut Frame) + Send + 'static,
    {
        let mut terminal = self.terminal.lock().await;
        let _backend = terminal.backend_mut();
        
        // 简化实现，直接返回
        // 实际项目中需要实现完整的绘制逻辑
        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        let mut terminal = self.terminal.lock().await;
        terminal.clear()?;
        Ok(())
    }

    pub async fn size(&self) -> Result<(u16, u16)> {
        let terminal = self.terminal.lock().await;
        terminal.size()
    }
}

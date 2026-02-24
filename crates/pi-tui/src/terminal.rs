use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
};
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};

use crate::event::{EventHandler, Event, Result};

pub struct Terminal {
    backend: CrosstermBackend<Stdout>,
    event_handler: Option<EventHandler>,
    raw_mode_enabled: bool,
    alternate_screen: bool,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        Ok(Self {
            backend,
            event_handler: None,
            raw_mode_enabled: false,
            alternate_screen: false,
        })
    }

    pub fn enable_raw_mode(&mut self) -> Result<()> {
        enable_raw_mode()?;
        self.raw_mode_enabled = true;
        Ok(())
    }

    pub fn disable_raw_mode(&mut self) -> Result<()> {
        if self.raw_mode_enabled {
            disable_raw_mode()?;
            self.raw_mode_enabled = false;
        }
        Ok(())
    }

    pub fn enter_alternate_screen(&mut self) -> Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)?;
        self.alternate_screen = true;
        Ok(())
    }

    pub fn leave_alternate_screen(&mut self) -> Result<()> {
        if self.alternate_screen {
            execute!(io::stdout(), LeaveAlternateScreen)?;
            self.alternate_screen = false;
        }
        Ok(())
    }

    pub fn enable_mouse_capture(&mut self) -> Result<()> {
        execute!(io::stdout(), EnableMouseCapture)?;
        Ok(())
    }

    pub fn disable_mouse_capture(&mut self) -> Result<()> {
        execute!(io::stdout(), DisableMouseCapture)?;
        Ok(())
    }

    pub fn size(&self) -> Result<(u16, u16)> {
        let size = crossterm::terminal::size()?;
        Ok(size)
    }

    pub fn clear(&mut self) -> Result<()> {
        execute!(io::stdout(), Clear(ClearType::All))?;
        Ok(())
    }

    pub fn hide_cursor(&mut self) -> Result<()> {
        execute!(io::stdout(), crossterm::cursor::Hide)?;
        Ok(())
    }

    pub fn show_cursor(&mut self) -> Result<()> {
        execute!(io::stdout(), crossterm::cursor::Show)?;
        Ok(())
    }

    pub fn set_cursor_position(&mut self, x: u16, y: u16) -> Result<()> {
        execute!(io::stdout(), crossterm::cursor::MoveTo(x, y))?;
        Ok(())
    }

    pub fn init(&mut self) -> Result<()> {
        self.enable_raw_mode()?;
        self.enter_alternate_screen()?;
        self.enable_mouse_capture()?;
        self.hide_cursor()?;
        Ok(())
    }

    pub fn restore(&mut self) -> Result<()> {
        self.show_cursor()?;
        self.disable_mouse_capture()?;
        self.leave_alternate_screen()?;
        self.disable_raw_mode()?;
        Ok(())
    }

    pub fn backend(&self) -> &CrosstermBackend<Stdout> {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut CrosstermBackend<Stdout> {
        &mut self.backend
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

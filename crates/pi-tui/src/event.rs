use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use std::io::{self, Stdout};
use tokio::sync::mpsc;
use tokio::time::{interval, MissedTickBehavior};

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
    FocusGained,
    FocusLost,
    Paste(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Null,
    Esc,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    KeypadBegin,
    Media(MediaKeyCode),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MediaKeyCode {
    Play,
    Pause,
    PlayPause,
    Reverse,
    Stop,
    FastForward,
    Rewind,
    TrackNext,
    TrackPrevious,
    Record,
    LowerVolume,
    RaiseVolume,
    MuteVolume,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
    pub hyper: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    Moved,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub struct EventHandler {
    pub tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    tick_rate: u64,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            tx,
            rx,
            tick_rate,
        }
    }

    pub fn tx(&self) -> mpsc::Sender<Event> {
        self.tx.clone()
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub async fn start(&self) {
        let tx = self.tx.clone();
        let tick_rate = self.tick_rate;

        tokio::spawn(async move {
            let mut interval = interval(tokio::time::Duration::from_millis(tick_rate));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                let Ok(event) = crossterm::event::read() else {
                    break;
                };

                let event = match event {
                    CrosstermEvent::Key(key) => {
                        Event::Key(KeyEvent {
                            code: KeyCode::from(key.code),
                            modifiers: KeyModifiers::from(key.modifiers),
                        })
                    }
                    CrosstermEvent::Mouse(mouse) => {
                        Event::Mouse(MouseEvent {
                            kind: MouseEventKind::from(mouse.kind),
                            column: mouse.column,
                            row: mouse.row,
                            modifiers: KeyModifiers::from(mouse.modifiers),
                        })
                    }
                    CrosstermEvent::Resize(width, height) => {
                        Event::Resize(width, height)
                    }
                    CrosstermEvent::FocusGained => {
                        Event::FocusGained
                    }
                    CrosstermEvent::FocusLost => {
                        Event::FocusLost
                    }
                    CrosstermEvent::Paste(s) => {
                        Event::Paste(s)
                    }
                };

                if tx.send(event).await.is_err() {
                    break;
                }

                tokio::select! {
                    _ = interval.tick() => {
                        if tx.send(Event::Tick).await.is_err() {
                            break;
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(1)) => {
                        // 避免阻塞
                    }
                }
            }
        });
    }
}

impl From<crossterm::event::KeyCode> for KeyCode {
    fn from(code: crossterm::event::KeyCode) -> Self {
        match code {
            crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
            crossterm::event::KeyCode::Enter => KeyCode::Enter,
            crossterm::event::KeyCode::Left => KeyCode::Left,
            crossterm::event::KeyCode::Right => KeyCode::Right,
            crossterm::event::KeyCode::Up => KeyCode::Up,
            crossterm::event::KeyCode::Down => KeyCode::Down,
            crossterm::event::KeyCode::Home => KeyCode::Home,
            crossterm::event::KeyCode::End => KeyCode::End,
            crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
            crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
            crossterm::event::KeyCode::Tab => KeyCode::Tab,
            crossterm::event::KeyCode::BackTab => KeyCode::BackTab,
            crossterm::event::KeyCode::Delete => KeyCode::Delete,
            crossterm::event::KeyCode::Insert => KeyCode::Insert,
            crossterm::event::KeyCode::F(n) => KeyCode::F(n),
            crossterm::event::KeyCode::Char(c) => KeyCode::Char(c),
            crossterm::event::KeyCode::Null => KeyCode::Null,
            crossterm::event::KeyCode::Esc => KeyCode::Esc,
            crossterm::event::KeyCode::CapsLock => KeyCode::CapsLock,
            crossterm::event::KeyCode::ScrollLock => KeyCode::ScrollLock,
            crossterm::event::KeyCode::NumLock => KeyCode::NumLock,
            crossterm::event::KeyCode::PrintScreen => KeyCode::PrintScreen,
            crossterm::event::KeyCode::Pause => KeyCode::Pause,
            crossterm::event::KeyCode::Menu => KeyCode::Menu,
            crossterm::event::KeyCode::KeypadBegin => KeyCode::KeypadBegin,
            crossterm::event::KeyCode::Media(media) => KeyCode::Media(MediaKeyCode::from(media)),
            _ => KeyCode::Null,
        }
    }
}

impl From<crossterm::event::MediaKeyCode> for MediaKeyCode {
    fn from(code: crossterm::event::MediaKeyCode) -> Self {
        match code {
            crossterm::event::MediaKeyCode::Play => MediaKeyCode::Play,
            crossterm::event::MediaKeyCode::Pause => MediaKeyCode::Pause,
            crossterm::event::MediaKeyCode::PlayPause => MediaKeyCode::PlayPause,
            crossterm::event::MediaKeyCode::Reverse => MediaKeyCode::Reverse,
            crossterm::event::MediaKeyCode::Stop => MediaKeyCode::Stop,
            crossterm::event::MediaKeyCode::FastForward => MediaKeyCode::FastForward,
            crossterm::event::MediaKeyCode::Rewind => MediaKeyCode::Rewind,
            crossterm::event::MediaKeyCode::TrackNext => MediaKeyCode::TrackNext,
            crossterm::event::MediaKeyCode::TrackPrevious => MediaKeyCode::TrackPrevious,
            crossterm::event::MediaKeyCode::Record => MediaKeyCode::Record,
            crossterm::event::MediaKeyCode::LowerVolume => MediaKeyCode::LowerVolume,
            crossterm::event::MediaKeyCode::RaiseVolume => MediaKeyCode::RaiseVolume,
            crossterm::event::MediaKeyCode::MuteVolume => MediaKeyCode::MuteVolume,
        }
    }
}

impl From<crossterm::event::KeyModifiers> for KeyModifiers {
    fn from(modifiers: crossterm::event::KeyModifiers) -> Self {
        Self {
            shift: modifiers.contains(crossterm::event::KeyModifiers::SHIFT),
            control: modifiers.contains(crossterm::event::KeyModifiers::CONTROL),
            alt: modifiers.contains(crossterm::event::KeyModifiers::ALT),
            super_key: modifiers.contains(crossterm::event::KeyModifiers::SUPER),
            hyper: modifiers.contains(crossterm::event::KeyModifiers::HYPER),
            meta: modifiers.contains(crossterm::event::KeyModifiers::META),
        }
    }
}

impl From<crossterm::event::MouseEventKind> for MouseEventKind {
    fn from(kind: crossterm::event::MouseEventKind) -> Self {
        match kind {
            crossterm::event::MouseEventKind::Down(button) => {
                MouseEventKind::Down(MouseButton::from(button))
            }
            crossterm::event::MouseEventKind::Up(button) => {
                MouseEventKind::Up(MouseButton::from(button))
            }
            crossterm::event::MouseEventKind::Drag(button) => {
                MouseEventKind::Drag(MouseButton::from(button))
            }
            crossterm::event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
            crossterm::event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
            crossterm::event::MouseEventKind::ScrollLeft => MouseEventKind::ScrollLeft,
            crossterm::event::MouseEventKind::ScrollRight => MouseEventKind::ScrollRight,
            crossterm::event::MouseEventKind::Moved => MouseEventKind::Moved,
        }
    }
}

impl From<crossterm::event::MouseButton> for MouseButton {
    fn from(button: crossterm::event::MouseButton) -> Self {
        match button {
            crossterm::event::MouseButton::Left => MouseButton::Left,
            crossterm::event::MouseButton::Right => MouseButton::Right,
            crossterm::event::MouseButton::Middle => MouseButton::Middle,
        }
    }
}

pub type Result<T> = std::result::Result<T, std::io::Error>;

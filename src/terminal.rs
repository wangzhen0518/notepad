use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyEvent, KeyEventKind},
    terminal,
};

use crate::editor::Position;

#[derive(Debug)]
pub struct Terminal {
    // size: Size,
}

impl Terminal {
    #[must_use]
    #[inline]
    pub fn width(&self) -> u16 {
        // self.size.width
        let (width, _height) = terminal::size().unwrap_or_default();
        width
    }

    #[must_use]
    #[inline]
    pub fn height(&self) -> u16 {
        // self.size.height
        let (_width, height) = terminal::size().unwrap_or_default();
        height.saturating_sub(2)
    }

    #[must_use]
    #[inline]
    pub fn size(&self) -> (u16, u16) {
        // (self.size.width, self.size.height)
        let (width, height) = terminal::size().unwrap_or_default();
        (width, height.saturating_sub(2))
    }
}

impl Terminal {
    #[inline]
    pub fn clear_screen() {
        print!("{}", terminal::Clear(terminal::ClearType::All));
    }

    #[inline]
    pub fn clear_current_line() {
        print!("{}", terminal::Clear(terminal::ClearType::CurrentLine));
    }

    /// # Errors
    #[inline]
    pub fn flush() -> Result<(), io::Error> {
        io::stdout().flush()
    }

    /// # Errors
    #[inline]
    pub fn read_key() -> Result<KeyEvent, io::Error> {
        loop {
            if let Some(Event::Key(key)) = event::read().into_iter().next() {
                if key.kind == KeyEventKind::Press {
                    return Ok(key);
                }
            }
        }
    }
}

impl Terminal {
    #[inline]
    pub fn cursor_hide() {
        print!("{}", cursor::Hide);
    }

    #[inline]
    pub fn cursor_show() {
        print!("{}", cursor::Show);
    }

    /// # Panics
    #[inline]
    #[allow(clippy::unwrap_used)]
    pub fn cursor_set_position(pos: &Position) {
        let x = u16::try_from(pos.x()).unwrap();
        let y = u16::try_from(pos.y()).unwrap();
        print!("{}", cursor::MoveTo(x, y));
    }
}

impl Default for Terminal {
    #[inline]
    #[allow(clippy::expect_used)]
    fn default() -> Self {
        // let (width, height) = terminal::size().expect("fail to get terminal size");
        // terminal::enable_raw_mode().expect("fail to enable raw mode");
        Terminal {
            // size: Size::new(width, height.saturating_sub(2)),
        }
    }
}

impl Drop for Terminal {
    #[inline]
    #[allow(clippy::expect_used)]
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("fail to disable raw mode");
    }
}

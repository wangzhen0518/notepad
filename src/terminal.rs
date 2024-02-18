use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyEvent, KeyEventKind},
    terminal,
};

use crate::editor::Position;

// use crate::Shape;

#[derive(Debug, Default, Clone)]
struct Size {
    width: u16,
    height: u16,
}

impl Size {
    #[must_use]
    fn new(width: u16, height: u16) -> Size {
        Self { width, height }
    }
}

// impl Shape for Size {
//     fn first(&self) -> u16 {
//         self.width
//     }

//     fn second(&self) -> u16 {
//         self.height
//     }
// }

// impl From<WindowSize> for Size {
//     fn from(value: WindowSize) -> Self {
//         Size::new(value.columns, value.rows)
//     }
// }

#[derive(Debug)]
pub struct Terminal {
    size: Size, // _stdout: io::Stdout,
}

impl Terminal {
    #[must_use]
    pub fn height(&self) -> u16 {
        self.size.height
    }

    #[must_use]
    pub fn width(&self) -> u16 {
        self.size.width
    }

    #[must_use]
    pub fn size(&self) -> (u16, u16) {
        (self.size.height, self.size.width)
    }
}

impl Terminal {
    pub fn clear_screen() {
        print!("{}", terminal::Clear(terminal::ClearType::All));
    }

    pub fn clear_current_line() {
        print!("{}", terminal::Clear(terminal::ClearType::CurrentLine));
    }

    /// # Errors
    pub fn flush() -> Result<(), io::Error> {
        io::stdout().flush()
    }

    /// # Errors
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
    pub fn cursor_hide() {
        print!("{}", cursor::Hide);
    }

    pub fn cursor_show() {
        print!("{}", cursor::Show);
    }

    /// # Panics
    pub fn cursor_set_position(pos: &Position) {
        let &Position { x, y } = pos;
        let x = u16::try_from(x).unwrap();
        let y = u16::try_from(y).unwrap();
        print!("{}", cursor::MoveTo(x, y));
    }
}

impl Default for Terminal {
    fn default() -> Self {
        let (mut width, mut height) = terminal::size().expect("fail to get terminal size");
        if cfg!(target_os = "windows") {
            height -= 1;
            width -= 1;
        }
        terminal::enable_raw_mode().expect("fail to enable raw mode");
        Terminal {
            size: Size::new(width, height),
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("fail to disable raw mode");
    }
}

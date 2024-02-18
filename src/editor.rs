use std::io;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::terminal::Terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

// impl From<(u16, u16)> for Position {
//     fn from(value: (u16, u16)) -> Self {
//         Self::new(value.0 as usize, value.1 as usize)
//     }
// }

// impl Shape for Position {
//     fn first(&self) -> u16 {
//         self.x as u16
//     }
//     fn second(&self) -> u16 {
//         self.y as u16
//     }
// }

#[derive(Default, Debug)]
pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
}

impl Editor {
    //! basic functions

    #[must_use]
    pub fn new() -> Editor {
        Editor::default()
    }

    fn die(e: io::Error) {
        Terminal::clear_screen();
        panic!("{e}");
    }

    #[must_use]
    pub fn terminal_size(&self) -> (u16, u16) {
        self.terminal.size()
    }
}

impl Editor {
    //! logistic functions

    /// # Errors
    pub fn run(&mut self) -> Result<(), io::Error> {
        loop {
            if let Err(e) = self.refresh_screen() {
                Editor::die(e);
            }
            if self.should_quit {
                break;
            }

            let key = Terminal::read_key()?;
            self.process_keypress(key);
        }

        Ok(())
    }

    // fn process_keypress(&mut self, key: KeyEvent) -> Result<(), io::Error> {
    fn process_keypress(&mut self, key: KeyEvent) {
        match key.code {
            // KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
            //     let b = c as u8;
            //     println!("{:?} ({})\r", b, c);
            // }
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End => self.move_cursor(key.code),
            _ => (), //println!("{:?}\r", key),
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
            self.should_quit = true;
            // Err(io::Error::new(io::ErrorKind::Interrupted, "finish"))
        }
        //  else {
        //     Ok(())
        // }
    }
}

impl Editor {
    //! draw functions

    /// # Errors
    fn refresh_screen(&self) -> Result<(), io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_set_position(&Position::new(0, 0));
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            Terminal::cursor_set_position(&self.cursor_position);
        }
        Terminal::cursor_show();

        Terminal::flush()
    }

    fn draw_rows(&self) {
        let height = self.terminal.height();
        for row in 0..height {
            Terminal::clear_current_line();
            if row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Editor by Wang Zhen -- version {}\r", VERSION);
        let width = self.terminal.width() as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }
}

impl Editor {
    //! cursor functions

    fn move_cursor(&mut self, key: KeyCode) {
        let (width, height) = self.terminal_size();
        let width = width as usize;
        let height = height as usize;

        let Position { x, y } = &mut self.cursor_position;
        match key {
            KeyCode::Up => *y = y.saturating_sub(1),
            KeyCode::Down => {
                if *y < height {
                    *y = y.saturating_add(1);
                }
            }
            KeyCode::Left => *x = x.saturating_sub(1),
            KeyCode::Right => {
                if *x < width {
                    *x = x.saturating_add(1);
                }
            }
            KeyCode::PageUp => *y = 0,
            KeyCode::PageDown => *y = height,
            KeyCode::Home => *x = 0,
            KeyCode::End => *x = width,
            _ => Editor::die(io::Error::new(
                io::ErrorKind::InvalidData,
                "error argument to move cursor",
            )),
        }
        // self.set_cursor_position((x, y))
    }
}

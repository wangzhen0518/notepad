use std::{cmp, env, io};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{document::Document, row::Row, terminal::Terminal};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug)]
pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
}

impl Default for Editor {
    fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let filename = &args[1];
            Document::open(filename).expect("no such file")
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default(),
            cursor_position: Position::default(),
            offset: Position::default(),
            document,
        }
    }
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
    pub fn terminal_size(&self) -> (usize, usize) {
        let (width, height) = self.terminal.size();
        (width as usize, height as usize)
    }

    #[must_use]
    pub fn terminal_width(&self) -> usize {
        self.terminal.width() as usize
    }

    #[must_use]
    pub fn terminal_height(&self) -> usize {
        self.terminal.height() as usize
    }

    // #[must_use]
    // pub fn document_length(&self) -> usize {
    //     self.document.len()
    // }
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

    fn process_keypress(&mut self, key: KeyEvent) {
        match key.code {
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
        }
    }
}

impl Editor {
    //! draw functions

    /// # Errors
    fn refresh_screen(&self) -> Result<(), io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_set_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            Terminal::cursor_set_position(&self.terminal_cursor_position());
        }
        Terminal::cursor_show();

        Terminal::flush()
    }

    pub fn draw_row(&self, row: &Row) {
        let start = self.offset.x;
        let end = self.offset.x + self.terminal_width();
        let row = row.render(start, end);
        println!("{}\r", row);
        // io::stdout().flush().unwrap();
    }

    fn draw_rows(&self) {
        let height = self.terminal_height();
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Editor by Wang Zhen -- version {}\r", VERSION);
        let width = self.terminal_width();
        let padding = width.saturating_sub(welcome_message.len()) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }
}

impl Editor {
    //! cursor functions

    fn terminal_cursor_position(&self) -> Position {
        Position {
            x: self.cursor_position.x.saturating_sub(self.offset.x),
            y: self.cursor_position.y.saturating_sub(self.offset.y),
        }
    }

    fn move_cursor(&mut self, key: KeyCode) {
        let Position { mut x, mut y } = self.cursor_position;
        let height = self.document.len();
        let mut width = self.document.row_length(y);

        match key {
            KeyCode::Up => y = y.saturating_sub(1),
            KeyCode::Down => {
                if y < height {
                    y += 1;
                }
            }
            KeyCode::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    x = self.document.row_length(y);
                };
            }
            KeyCode::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            KeyCode::PageUp => y = cmp::max(y.saturating_sub(self.terminal_height()), 0),
            KeyCode::PageDown => y = cmp::min(y.saturating_add(self.terminal_height()), height),
            KeyCode::Home => y = 0,
            KeyCode::End => y = height,
            _ => Editor::die(io::Error::new(
                io::ErrorKind::InvalidData,
                "error argument to move cursor",
            )),
        }
        // check x is valid
        width = self.document.row_length(y);
        x = cmp::min(x, width);

        self.cursor_position = Position::new(x, y);
        self.scroll();
    }

    fn scroll(&mut self) {
        // TODO 修改为不移动 cursor，而移动窗口的版本
        let (width, height) = self.terminal_size();

        let cur_pos = &self.cursor_position;
        let offset = &mut self.offset;
        if cur_pos.y < offset.y {
            offset.y = cur_pos.y;
        } else if cur_pos.y >= offset.y.saturating_add(height) {
            offset.y = cur_pos.y.saturating_sub(height).saturating_add(1);
        }
        if cur_pos.x < offset.x {
            offset.x = cur_pos.x;
        } else if cur_pos.x >= offset.x.saturating_add(width) {
            offset.x = cur_pos.x.saturating_sub(width).saturating_add(1);
        }
    }
}

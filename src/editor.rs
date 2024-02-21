use std::{
    cmp, env,
    fmt::Display,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    style::{self, Stylize},
};

use crate::{
    constants::{QUIT_TIMES, STATUS_BG_COLOR, STATUS_FG_COLOR, VERSION},
    document::Document,
    row::Row,
    terminal::Terminal,
};

#[derive(Debug, Default, Clone)]
pub struct Position {
    x: usize,
    y: usize,
}

impl Position {
    #[must_use]
    #[inline]
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    #[must_use]
    #[inline]
    pub fn at_beginning(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    #[must_use]
    #[inline]
    pub fn x(&self) -> usize {
        self.x
    }

    #[must_use]
    #[inline]
    pub fn y(&self) -> usize {
        self.y
    }

    #[inline]
    pub fn set_x(&mut self, x: usize) {
        self.x = x;
    }

    #[inline]
    pub fn set_y(&mut self, y: usize) {
        self.y = y;
    }
}

#[derive(Debug)]
struct StatusMessage {
    time: Instant,
    text: String,
}

impl<T> From<T> for StatusMessage
where
    T: Display,
{
    fn from(value: T) -> Self {
        StatusMessage {
            time: Instant::now(),
            text: format!("{}", value),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Debug)]
pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    quit_times: u8,
    highlighted_word: Option<String>,
}

impl Default for Editor {
    fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status =
            String::from("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");
        let mut document = Document::default();
        if let Some(filename) = args.get(1) {
            match Document::open(filename) {
                Ok(doc) => document = doc,
                Err(e) => {
                    initial_status = format!("ERR: Could not open file: {}", filename);
                    eprintln!("{}", e);
                }
            }
        };

        Self {
            should_quit: false,
            terminal: Terminal::default(),
            cursor_position: Position::default(),
            offset: Position::default(),
            document,
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
            highlighted_word: None,
        }
    }
}

impl Editor {
    //! basic functions

    #[inline]
    #[must_use]
    pub fn new() -> Editor {
        Editor::default()
    }

    #[inline]
    #[must_use]
    #[allow(clippy::as_conversions)]
    pub fn terminal_size(&self) -> (usize, usize) {
        let (width, height) = self.terminal.size();
        (width as usize, height as usize)
    }

    #[inline]
    #[must_use]
    #[allow(clippy::as_conversions)]
    pub fn terminal_width(&self) -> usize {
        self.terminal.width() as usize
    }

    #[inline]
    #[must_use]
    #[allow(clippy::as_conversions)]
    pub fn terminal_height(&self) -> usize {
        self.terminal.height() as usize
    }
}

impl Editor {
    //! logistic functions

    #[allow(clippy::needless_pass_by_value, clippy::panic)]
    fn die(e: io::Error) {
        Terminal::clear_screen();
        panic!("{e}");
    }

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
            KeyCode::Enter => {
                self.document.new_line(&self.cursor_position);
                self.move_cursor(KeyCode::Right);
            }
            KeyCode::Delete => self.document.delete(&self.cursor_position),
            KeyCode::Backspace => {
                if !self.cursor_position.at_beginning() {
                    self.move_cursor(KeyCode::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'f' => self.search(),
                        's' => self.save(),
                        'q' => {
                            if !self.try_quit() {
                                return;
                            }
                        }
                        _ => (),
                    }
                } else {
                    self.document.insert(&self.cursor_position, c);
                    self.move_cursor(KeyCode::Right);
                }
            }
            _ => (),
        }
        self.reset_quit();
        self.scroll();
    }

    /// return whether success to quit
    #[allow(clippy::arithmetic_side_effects)]
    fn try_quit(&mut self) -> bool {
        self.quit_times -= 1;
        if self.quit_times > 0 && self.document.is_dirty() {
            // let quit_check_msg =
            self.status_message = StatusMessage::from(format!(
                "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                self.quit_times
            ));
            false
        } else {
            self.should_quit = true;
            true
        }
    }

    fn reset_quit(&mut self) {
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from("");
        }
    }

    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        // let mut direction = SearchDirection::Forward;
        let query = self
            .prompt(
                "Search (ESC to cancel, Arrows to navigate): ",
                increase_search,
            )
            .unwrap_or_default();
        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
        self.highlighted_word = None;
    }

    fn save(&mut self) {
        if self.document.filename().is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if let Some(new_name) = new_name {
                self.document.set_filename(Some(new_name));
            } else {
                self.status_message = StatusMessage::from("Save aborted");
                return;
            }
        }

        let msg = match self.document.save() {
            Ok(()) => "File saved successfully",
            Err(err) => {
                eprintln!("{}", err);
                "Error writing file!"
            }
        };
        self.status_message = StatusMessage::from(msg);
    }

    fn prompt<T, C>(&mut self, prompt: T, callback: C) -> Result<Option<String>, io::Error>
    where
        T: Display,
        C: Fn(&mut Self, KeyEvent, &str),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let key = Terminal::read_key()?;
            match key.code {
                KeyCode::Enter => break,
                KeyCode::Backspace => result.truncate(result.len().saturating_sub(1)),
                KeyCode::Esc => {
                    result.truncate(0);
                    break;
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    result.push(c);
                }
                _ => (),
            }
            callback(self, key, &result);
        }

        self.status_message = StatusMessage::from("");
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}

impl Editor {
    //! draw functions

    /// # Errors
    fn refresh_screen(&mut self) -> Result<(), io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_set_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.document.highlight(
                self.highlighted_word.as_ref(),
                Some(self.offset.y().saturating_add(self.terminal_height())),
            );
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_set_position(&self.terminal_cursor_position());
        }
        Terminal::cursor_show();

        Terminal::flush()
    }

    pub fn draw_row(&self, row: &Row) {
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(self.terminal_width());
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn draw_rows(&self) {
        let height = self.terminal_height();
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            #[allow(clippy::arithmetic_side_effects, clippy::integer_division)]
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
        #[allow(clippy::integer_division)]
        let padding = width.saturating_sub(welcome_message.len()) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn draw_status_bar(&self) {
        // TODO 构建 StatusInfo 结构，存储相关信息，并实现 Display 用于打印
        // TODO 其中 filename 以什么方式存储？用引用吗
        let width = self.terminal_width();
        // check is modified
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };
        // filename
        let mut filename = String::from("[No Name]");
        if let Some(name) = self.document.filename() {
            filename = name.clone();
            filename.truncate(20);
        }
        // number of lines
        let mut status_info = format!(
            "{} - {} lines{}",
            filename,
            self.document.len(),
            modified_indicator
        );
        let line_indicator = format!(
            "{} | {}/{}",
            self.document.filetype(),
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );

        #[allow(clippy::arithmetic_side_effects)]
        let len = status_info.len() + line_indicator.len();
        status_info.push_str(&" ".repeat(width.saturating_sub(len)));
        status_info = format!("{}{}", status_info, line_indicator);
        status_info.truncate(width);
        println!(
            "{}",
            style::style(status_info)
                .with(STATUS_FG_COLOR)
                .on(STATUS_BG_COLOR)
        );
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        let delta = Instant::now().duration_since(message.time);
        if delta < Duration::from_secs(5) {
            let mut text = message.text.clone();
            text.truncate(self.terminal_width());
            print!("{}", text);
        }
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
            KeyCode::Down =>
            {
                #[allow(clippy::arithmetic_side_effects)]
                if y < height {
                    y += 1;
                }
            }
            KeyCode::Left => {
                #[allow(clippy::arithmetic_side_effects)]
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    x = self.document.row_length(y);
                };
            }
            KeyCode::Right =>
            {
                #[allow(clippy::arithmetic_side_effects)]
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
            // KeyCode::Enter=>
            _ => Editor::die(io::Error::new(
                io::ErrorKind::InvalidData,
                "error argument to move cursor",
            )),
        }
        // check x is valid
        width = self.document.row_length(y);
        x = cmp::min(x, width);

        self.cursor_position = Position::new(x, y);
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

fn increase_search(editor: &mut Editor, key: KeyEvent, query: &str) {
    let mut moved = false;
    let direction = match key.code {
        KeyCode::Right | KeyCode::Down => {
            editor.move_cursor(KeyCode::Right);
            moved = true;
            SearchDirection::Forward
        }
        KeyCode::Left | KeyCode::Up => SearchDirection::Backward,
        _ => SearchDirection::Forward,
    };
    if let Some(pos) = editor
        .document
        .find(query, &editor.cursor_position, direction)
    {
        editor.cursor_position = pos;
        editor.scroll();
    } else if moved {
        editor.move_cursor(KeyCode::Left);
    }
    editor.highlighted_word = Some(query.to_string());
}

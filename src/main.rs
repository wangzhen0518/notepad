use std::io;

use notepad::editor;

fn main() -> Result<(), io::Error> {
    let mut editor = editor::Editor::default();
    editor.run()
}

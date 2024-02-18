use std::error::Error;

use notepad::editor;

fn main() -> Result<(), Box<dyn Error>> {
    let mut editor = editor::Editor::default();
    editor.run()?;

    Ok(())
}

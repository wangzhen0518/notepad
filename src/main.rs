use std::error::Error;

use notepad::editor;

// #[derive(Debug)]
// struct Position {
//     x: usize,
//     y: usize,
// }

// #[derive(Debug)]
// struct MyTest {
//     offset: Position,
// }

fn main() -> Result<(), Box<dyn Error>> {
    let mut editor = editor::Editor::default();
    editor.run()?;

    // let mut x = MyTest {
    //     offset: Position { x: 0, y: 0 },
    // };
    // let offset = &mut x.offset;
    // offset.x = 10;
    // offset.y = 100;

    // println!("{:?}", x);

    Ok(())
}

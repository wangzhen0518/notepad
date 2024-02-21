use std::io;

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

fn main() -> Result<(), io::Error> {
    let mut editor = editor::Editor::default();
    editor.run()

    // let content = fs::read_to_string("cano.txt").unwrap_or_default();
    // let chars: Vec<char> = content.chars().collect();
    // let s_: Vec<&str> = content.graphemes(true).collect();
    // println!("{:?}", chars);
    // println!("{:?}", s_);
    // assert_eq!(chars.len(), s_.len());

    // let mut start = 0;
    // let end = 10;
    // for x in start..=end {
    //     println!("{}", x);
    //     start += 1;
    // }

    // Ok(())
}

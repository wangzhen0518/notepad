#[derive(Debug, Default)]
pub struct HighlightingOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
    multiline_comments: bool,
    primary_keywords: Vec<&'static str>,
    secondary_keywords: Vec<&'static str>,
}

impl HighlightingOptions {
    pub fn numbers(&self) -> bool {
        self.numbers
    }

    pub fn strings(&self) -> bool {
        self.strings
    }

    pub fn characters(&self) -> bool {
        self.characters
    }

    pub fn comments(&self) -> bool {
        self.comments
    }

    pub fn multiline_comments(&self) -> bool {
        self.multiline_comments
    }

    pub fn primary_keywords(&self) -> &Vec<&'static str> {
        &self.primary_keywords
    }

    pub fn secondary_keywords(&self) -> &Vec<&'static str> {
        &self.secondary_keywords
    }
}

#[derive(Debug)]
pub struct FileType {
    name: String,
    hl_opts: HighlightingOptions,
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            hl_opts: HighlightingOptions::default(),
        }
    }
}

impl<T> From<T> for FileType
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        let filename: String = value.into();
        if filename.ends_with(".rs") {
            let primary_keywords = vec![
                "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false",
                "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
                "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
                "true", "type", "unsafe", "use", "where", "while", "dyn", "abstract", "become",
                "box", "do", "final", "macro", "override", "priv", "typeof", "unsized", "virtual",
                "yield", "async", "await", "try",
            ];
            let secondary_keywords = vec![
                "bool", "char", "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64",
                "usize", "f32", "f64",
            ];
            Self {
                name: String::from("Rust"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    multiline_comments: true,
                    primary_keywords,
                    secondary_keywords,
                },
            }
        } else {
            Self::default()
        }
    }
}

impl FileType {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn highlightling_options(&self) -> &HighlightingOptions {
        &self.hl_opts
    }
}

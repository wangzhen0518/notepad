#[derive(Debug, Default, Clone)]
pub struct HighlightingOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
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
}

#[derive(Debug, Clone)]
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
            Self {
                name: String::from("Rust"),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
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

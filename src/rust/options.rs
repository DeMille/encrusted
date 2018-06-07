#[derive(Debug)]
pub struct Options {
    pub save_dir: String,
    pub save_name: String,
    pub log_instructions: bool,
}

impl Options {
    pub fn default() -> Options {
        Options {
            save_dir: String::new(),
            save_name: String::new(),
            log_instructions: false,
        }
    }
}
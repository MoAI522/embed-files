use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Warning {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
}

#[derive(Default)]
pub struct Warnings(Vec<Warning>);

impl Warnings {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, warning: Warning) {
        self.0.push(warning);
    }

    pub fn extend(&mut self, other: Warnings) {
        self.0.extend(other.0);
    }

    pub fn print_all(&self) {
        for warning in &self.0 {
            eprintln!("Warning: {}", warning);
        }
    }
}

impl IntoIterator for Warnings {
    type Item = Warning;
    type IntoIter = std::vec::IntoIter<Warning>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

use crate::error::{Error, Result};
use crate::warning::{Warning, Warnings};
use glob::glob;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct PathResolver {
    base_dir: PathBuf,
    warnings: Warnings,
}

impl PathResolver {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        let base_dir = base_path
            .as_ref()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        Self {
            base_dir,
            warnings: Warnings::new(),
        }
    }

    pub fn resolve_glob(&mut self, pattern: &str) -> Result<Vec<PathBuf>> {
        let full_pattern = if Path::new(pattern).is_absolute() {
            pattern.to_string()
        } else {
            self.base_dir.join(pattern).to_string_lossy().to_string()
        };

        let paths = glob(&full_pattern).map_err(|e| Error::InvalidGlobPattern {
            pattern: pattern.to_string(),
            source: e,
        })?;

        let mut result = Vec::new();
        for entry in paths {
            match entry {
                Ok(path) => {
                    if path.starts_with(&self.base_dir) && self.is_valid_file(&path) {
                        result.push(path);
                    }
                }
                Err(_) => {
                    self.warnings.push(Warning::FileNotFound {
                        path: PathBuf::from(pattern),
                    });
                }
            }
        }

        if result.is_empty() {
            self.warnings.push(Warning::FileNotFound {
                path: PathBuf::from(pattern),
            });
        }

        Ok(result)
    }

    pub fn resolve_regex(&mut self, pattern: &str) -> Result<Vec<PathBuf>> {
        let regex = Regex::new(pattern).map_err(|e| Error::InvalidRegexPattern {
            pattern: pattern.to_string(),
            source: e,
        })?;

        let mut result = Vec::new();
        self.walk_directory(&self.base_dir, &regex, &mut result)?;

        if result.is_empty() {
            self.warnings.push(Warning::FileNotFound {
                path: PathBuf::from(pattern),
            });
        }

        Ok(result)
    }

    fn walk_directory(
        &self,
        dir: &Path,
        regex: &Regex,
        results: &mut Vec<PathBuf>,
    ) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    self.walk_directory(&path, regex, results)?;
                } else if let Some(path_str) = path.to_str() {
                    if regex.is_match(path_str) && self.is_valid_file(&path) {
                        results.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    fn is_valid_file(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        // ファイルの先頭部分を読んでバイナリファイルかどうかを判定
        if let Ok(content) = fs::read(path) {
            !content.iter().take(1024).any(|&byte| byte == 0)
        } else {
            false
        }
    }

    pub fn take_warnings(&mut self) -> Warnings {
        std::mem::take(&mut self.warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup_test_files() -> (tempfile::TempDir, PathResolver) {
        let temp_dir = tempdir().unwrap();

        // テストファイルの作成
        fs::write(temp_dir.path().join("test1.txt"), "Hello, World!").unwrap();
        fs::write(temp_dir.path().join("test2.txt"), "Test content").unwrap();

        let virtual_template_path = temp_dir.path().join("template.txt");
        let resolver = PathResolver::new(virtual_template_path);
        (temp_dir, resolver)
    }

    #[test]
    fn test_glob_resolution() {
        let (_temp_dir, mut resolver) = setup_test_files();
        let paths = resolver.resolve_glob("*.txt").unwrap();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_regex_resolution() {
        let (_temp_dir, mut resolver) = setup_test_files();
        let paths = resolver.resolve_regex(r".*\.txt$").unwrap();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_invalid_glob_pattern() {
        let (_temp_dir, mut resolver) = setup_test_files();
        let result = resolver.resolve_glob("[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let (_temp_dir, mut resolver) = setup_test_files();
        let result = resolver.resolve_regex("[invalid");
        assert!(result.is_err());
    }
}

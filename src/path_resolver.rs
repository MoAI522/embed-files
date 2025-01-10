use crate::error::{Error, Result};
use crate::warning::{Warning, Warnings};
use glob::glob;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

pub struct PathResolver {
    base_dir: PathBuf,
    warnings: Warnings,
}

impl PathResolver {
    pub fn new() -> Result<Self> {
        let base_dir = env::current_dir()?;

        Ok(Self {
            base_dir,
            warnings: Warnings::new(),
        })
    }

    pub fn resolve_glob(&mut self, pattern: &str) -> Result<Vec<PathBuf>> {
        let current_dir = env::current_dir()?;
        let full_pattern = if Path::new(pattern).is_relative() {
            current_dir.join(pattern).to_string_lossy().into_owned()
        } else {
            pattern.to_string()
        };

        let paths = glob(&full_pattern).map_err(|e| Error::InvalidGlobPattern {
            pattern: pattern.to_string(),
            source: e,
        })?;

        let mut result = Vec::new();
        for entry in paths {
            match entry {
                Ok(path) => {
                    if self.is_valid_file(&path) {
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

                // 絶対パスを相対パスに変換
                let relative_path = if let Ok(rel_path) = path.strip_prefix(&self.base_dir) {
                    rel_path.to_path_buf()
                } else {
                    path.clone()
                };

                if path.is_dir() {
                    self.walk_directory(&path, regex, results)?;
                } else if let Some(path_str) = relative_path.to_str() {
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

        if let Ok(content) = fs::read(path) {
            let is_valid = !content.iter().take(1024).any(|&byte| byte == 0);
            is_valid
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
    use tempfile::{tempdir, TempDir};

    struct TestContext {
        _temp_dir: TempDir, // TempDirをドロップされないように保持
        resolver: PathResolver,
    }

    fn setup_test_files() -> Result<TestContext> {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();

        fs::create_dir_all(temp_path).unwrap();
        fs::write(temp_path.join("test1.txt"), "Hello, World!").unwrap();
        fs::write(temp_path.join("test2.txt"), "Test content").unwrap();

        env::set_current_dir(temp_path).unwrap();

        let resolver = PathResolver::new()?;

        Ok(TestContext {
            _temp_dir: temp_dir,
            resolver,
        })
    }

    #[test]
    fn test_glob_resolution() -> Result<()> {
        let mut ctx = setup_test_files()?;
        let paths = ctx.resolver.resolve_glob("*.txt")?;
        assert_eq!(paths.len(), 2);
        Ok(())
    }

    #[test]
    fn test_regex_resolution() -> Result<()> {
        let mut ctx = setup_test_files()?;
        let paths = ctx.resolver.resolve_regex(r".*\.txt$")?;
        assert_eq!(paths.len(), 2);
        Ok(())
    }

    #[test]
    fn test_invalid_glob_pattern() -> Result<()> {
        let mut ctx = setup_test_files()?;
        let result = ctx.resolver.resolve_glob("[invalid");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_invalid_regex_pattern() -> Result<()> {
        let mut ctx = setup_test_files()?;
        let result = ctx.resolver.resolve_regex("[invalid");
        assert!(result.is_err());
        Ok(())
    }
}

use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub struct EfTemplate {
    template: String,
    base_dir: PathBuf,
}

impl EfTemplate {
    pub fn find_and_load<P: AsRef<Path>>(start_path: P) -> Result<Self> {
        let start_path = start_path.as_ref();
        let base_dir = if start_path.is_file() {
            start_path.parent().unwrap_or_else(|| Path::new("."))
        } else {
            start_path
        }
        .to_path_buf();

        if let Some(template_path) = Self::find_template(&start_path)? {
            let content = fs::read_to_string(template_path)?;
            Ok(Self {
                template: content,
                base_dir,
            })
        } else {
            Ok(Self::new_with_default(base_dir))
        }
    }

    fn find_template<P: AsRef<Path>>(start_path: P) -> Result<Option<PathBuf>> {
        let start_path = start_path.as_ref();
        let mut current_dir = if start_path.is_file() {
            start_path.parent().unwrap_or_else(|| Path::new("."))
        } else {
            start_path
        };

        loop {
            let template_path = current_dir.join(".eftemplate");
            if template_path.is_file() {
                return Ok(Some(template_path));
            }

            if let Some(parent) = current_dir.parent() {
                current_dir = parent;
            } else {
                break;
            }
        }

        Ok(None)
    }

    pub fn format(&self, file_path: &Path, content: &str) -> String {
        let language = self.detect_language(file_path);
        // 絶対パスを相対パスに変換
        let relative_path = if file_path.is_absolute() {
            if let Some(rel_path) = pathdiff::diff_paths(file_path, &self.base_dir) {
                rel_path
            } else {
                file_path.to_path_buf()
            }
        } else {
            file_path.to_path_buf()
        };

        self.template
            .replace("{filePath}", &relative_path.to_string_lossy())
            .replace("{language}", &language)
            .replace("{content}", content)
    }

    fn detect_language(&self, path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                crate::language_mapping::get_language_for_extension(ext)
                    .unwrap_or_else(|| "plaintext".to_string())
            })
            .unwrap()
    }
}

impl Default for EfTemplate {
    fn default() -> Self {
        Self::new_with_default(PathBuf::from("."))
    }
}

impl EfTemplate {
    fn new_with_default(base_dir: PathBuf) -> Self {
        Self {
            template: "{filePath}\n```{language}\n{content}\n```\n".to_string(),
            base_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_template() {
        let base_dir = PathBuf::from("/test/base/dir");
        let template = EfTemplate::new_with_default(base_dir);
        let file_path = Path::new("/test/base/dir/test.rs");
        let content = "fn main() {\n    println!(\"Hello\");\n}";

        let result = template.format(file_path, content);

        assert!(result.contains("test.rs")); // 相対パスに変換されていることを確認
        assert!(result.contains("```rust"));
        assert!(result.contains(content));
    }

    #[test]
    fn test_custom_template() {
        let temp_dir = tempdir().unwrap();
        let template_content = "File: {filePath}\nType: {language}\n---\n{content}\n---\n";
        fs::write(temp_dir.path().join(".eftemplate"), template_content).unwrap();

        let template = EfTemplate::find_and_load(temp_dir.path()).unwrap();
        let file_path = temp_dir.path().join("src/main.rs");
        let content = "fn main() {\n    println!(\"Hello\");\n}";

        let result = template.format(&file_path, content);

        assert!(result.contains("File: src/main.rs")); // 相対パスに変換されていることを確認
        assert!(result.contains("Type: rust"));
        assert!(result.contains(content));
    }

    #[test]
    fn test_language_detection() {
        let template = EfTemplate::default();
        assert_eq!(template.detect_language(Path::new("test.rs")), "rust");
        assert_eq!(template.detect_language(Path::new("test.tsx")), "tsx");
        assert_eq!(template.detect_language(Path::new("test.js")), "javascript");
        assert_eq!(
            template.detect_language(Path::new("test.unknown")),
            "plaintext"
        );
    }

    #[test]
    fn test_path_resolution() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path().join("project");
        fs::create_dir(&base_dir).unwrap();
        let template = EfTemplate::new_with_default(base_dir.clone());

        // ケース1: 絶対パスが相対パスに変換される
        let abs_path = base_dir.join("src/main.rs");
        let result = template.format(&abs_path, "content");
        assert!(result.contains("src/main.rs"));
        assert!(!result.contains(base_dir.to_str().unwrap()));

        // ケース2: 既に相対パスの場合はそのまま
        let rel_path = Path::new("src/lib.rs");
        let result = template.format(rel_path, "content");
        assert!(result.contains("src/lib.rs"));

        // ケース3: ベースディレクトリの外のパスの場合
        let outside_path = temp_dir.path().join("outside/file.rs");
        let result = template.format(&outside_path, "content");
        assert!(result.contains("../outside/file.rs"));
    }

    #[test]
    fn test_template_inheritance() {
        let temp_dir = tempdir().unwrap();

        // ルートディレクトリに.eftemplateを作成
        fs::write(
            temp_dir.path().join(".eftemplate"),
            "ROOT: {filePath} ({language})\n{content}",
        )
        .unwrap();

        // サブディレクトリを作成
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(
            &sub_dir.join(".eftemplate"),
            "SUB: {filePath} ({language})\n{content}",
        )
        .unwrap();

        // サブディレクトリからテンプレートを読み込み
        let template = EfTemplate::find_and_load(&sub_dir).unwrap();
        let file_path = sub_dir.join("test.rs");
        let result = template.format(&file_path, "content");

        assert!(result.contains("SUB: "));
        assert!(!result.contains("ROOT: "));
        assert!(result.contains("test.rs")); // 相対パスに変換されていることを確認
    }
}

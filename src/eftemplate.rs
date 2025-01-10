use crate::error::Result;
use std::path::{Path, PathBuf};
use std::{env, fs};

pub struct EfTemplate {
    template: String,
}

impl EfTemplate {
    pub fn find_and_load<P: AsRef<Path>>(start_path: P) -> Result<Self> {
        if let Some(template_path) = Self::find_template(&start_path)? {
            let content = fs::read_to_string(template_path)?;
            Ok(Self { template: content })
        } else {
            Ok(Self::default())
        }
    }

    fn find_template<P: AsRef<Path>>(start_path: P) -> Result<Option<PathBuf>> {
        let start_path = start_path.as_ref();
        let current_dir = if start_path.is_file() {
            start_path.parent().unwrap_or_else(|| Path::new("."))
        } else {
            start_path
        };

        let mut current_dir = current_dir.canonicalize()?;

        loop {
            let template_path = current_dir.join(".eftemplate");
            if template_path.is_file() {
                return Ok(Some(template_path));
            }

            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                break;
            }
        }

        Ok(None)
    }

    pub fn format(&self, file_path: &Path, content: &str) -> String {
        let language = self.detect_language(file_path);
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let relative_path = if file_path.is_absolute() {
            // 絶対パスを現在のディレクトリからの相対パスに変換
            if let Some(stripped_path) = pathdiff::diff_paths(file_path, &current_dir) {
                stripped_path
            } else {
                file_path.to_path_buf()
            }
        } else {
            // 相対パスはそのまま
            file_path.to_path_buf()
        };

        let normalized_path = relative_path
            .components()
            .collect::<PathBuf>()
            .to_string_lossy()
            .into_owned();

        self.template
            .replace("{filePath}", &normalized_path)
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
        Self {
            template: "{filePath}\n```{language}\n{content}\n```\n".to_string(),
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
        let template = EfTemplate::default();
        // カレントディレクトリからの相対パスとして扱われるべき絶対パス
        let file_path = env::current_dir().unwrap().join("test.rs");
        let content = "fn main() {\n    println!(\"Hello\");\n}";

        let result = template.format(&file_path, content);

        assert!(result.contains("test.rs")); // カレントディレクトリからの相対パスに変換されていることを確認
        assert!(result.contains("```rust"));
        assert!(result.contains(content));
    }

    #[test]
    fn test_custom_template() {
        let temp_dir = tempdir().unwrap();
        let template_content = "File: {filePath}\nType: {language}\n---\n{content}\n---\n";
        fs::write(temp_dir.path().join(".eftemplate"), template_content).unwrap();

        // カレントディレクトリを一時ディレクトリに変更
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        let template = EfTemplate::find_and_load(".").unwrap();
        let file_path = Path::new("src/main.rs");
        let content = "fn main() {\n    println!(\"Hello\");\n}";

        let result = template.format(&file_path, content);

        assert!(result.contains("File: src/main.rs"));
        assert!(result.contains("Type: rust"));
        assert!(result.contains(content));

        // テスト終了後にカレントディレクトリを戻す
        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_language_detection() {
        let template = EfTemplate::default();
        assert_eq!(template.detect_language(Path::new("test.rs")), "rust");
        assert_eq!(
            template.detect_language(Path::new("test.tsx")),
            "typescript"
        );
        assert_eq!(template.detect_language(Path::new("test.js")), "javascript");
        assert_eq!(
            template.detect_language(Path::new("test.unknown")),
            "plaintext"
        );
    }

    #[test]
    fn test_path_resolution() {
        let temp_dir = tempdir().unwrap();

        // プロジェクトディレクトリ構造を作成
        let project_dir = temp_dir.path().join("project");
        fs::create_dir(&project_dir).unwrap();
        fs::create_dir(project_dir.join("src")).unwrap();

        // outsideディレクトリを作成
        let outside_dir = temp_dir.path().join("outside");
        fs::create_dir(&outside_dir).unwrap();

        // カレントディレクトリをproject_dirに変更
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&project_dir).unwrap();

        let template = EfTemplate::default();

        // ケース1: プロジェクト内の絶対パスが相対パスに変換される
        let abs_path = project_dir.join("src/main.rs");
        let result = template.format(&abs_path, "content");
        assert!(
            result.contains("src/main.rs"),
            "Expected src/main.rs, got: {}",
            result
        );

        // ケース2: 相対パスはそのまま
        let rel_path = Path::new("src/lib.rs");
        let result = template.format(rel_path, "content");
        assert!(
            result.contains("src/lib.rs"),
            "Expected src/lib.rs, got: {}",
            result
        );

        // ケース3: プロジェクト外のパスは相対パスに変換
        let outside_file = outside_dir.join("file.rs");
        let result = template.format(&outside_file, "content");
        assert!(
            result.contains("../outside/file.rs"),
            "Expected ../outside/file.rs, got: {}",
            result
        );

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_template_inheritance() {
        let temp_dir = tempdir().unwrap();
        let root_dir = temp_dir.path();

        // ディレクトリ構造を作成
        fs::create_dir_all(root_dir.join("subdir")).unwrap();

        // テンプレートファイルを作成
        fs::write(
            root_dir.join(".eftemplate"),
            "ROOT: {filePath} ({language})\n{content}",
        )
        .unwrap();

        fs::write(
            root_dir.join("subdir/.eftemplate"),
            "SUB: {filePath} ({language})\n{content}",
        )
        .unwrap();

        // カレントディレクトリをrootディレクトリに変更
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(root_dir).unwrap();

        // サブディレクトリのテンプレートを使用
        let template = EfTemplate::find_and_load(Path::new("subdir")).unwrap();
        let file_path = Path::new("subdir/test.rs");
        let result = template.format(file_path, "content");

        assert!(result.contains("SUB: "));
        assert!(!result.contains("ROOT: "));
        assert!(result.contains("subdir/test.rs"));

        env::set_current_dir(original_dir).unwrap();
    }
}

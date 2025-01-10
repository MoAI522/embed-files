use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

pub struct TestEnv {
    _temp_dir: TempDir,
    base_path: PathBuf,
}

impl TestEnv {
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        // 基本ディレクトリ構造を作成
        Self::setup_directory_structure(&base_path);

        Self {
            _temp_dir: temp_dir,
            base_path,
        }
    }

    fn setup_directory_structure(base_path: &Path) {
        fs::create_dir_all(base_path.join("src")).expect("Failed to create src directory");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(base_path, fs::Permissions::from_mode(0o755))
                .expect("Failed to set base directory permissions");
        }
    }

    pub fn create_file(&self, path: &str, content: &str) -> PathBuf {
        let full_path = self.base_path.join(path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(&full_path, content).unwrap();

        // カレントディレクトリからの相対パスを返す
        let current_dir = env::current_dir().unwrap();
        pathdiff::diff_paths(&full_path, &current_dir).unwrap_or_else(|| full_path.clone())
    }

    pub fn create_template(&self, content: &str) -> PathBuf {
        self.create_file("template.txt", content)
    }

    pub fn create_eftemplate(&self, content: &str) -> PathBuf {
        self.create_file(".eftemplate", content)
    }

    pub fn path(&self) -> &Path {
        &self.base_path
    }

    pub fn run_test_in_scope<F, R>(&self, test_fn: F) -> R
    where
        F: FnOnce() -> R,
    {
        let original_dir = env::current_dir().expect("Failed to get current directory");

        let absolute_path = self
            .base_path
            .canonicalize()
            .expect("Failed to get absolute path");
        env::set_current_dir(&absolute_path)
            .unwrap_or_else(|e| panic!("Failed to change directory to {:?}: {}", absolute_path, e));

        let result = test_fn();

        env::set_current_dir(original_dir).expect("Failed to restore original directory");

        result
    }
}

// サンプルファイルのセットアップ関数を改善
pub fn setup_sample_files(env: &TestEnv) {
    env.create_file(
        "src/main.rs",
        r#"fn main() {
    println!("Hello, world!");
}"#,
    );

    env.create_file(
        "src/lib.rs",
        r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}"#,
    );
}

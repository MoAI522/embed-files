use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

pub struct TestEnv {
    temp_dir: TempDir,
    base_path: PathBuf,
}

impl TestEnv {
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let base_path = temp_dir.path().to_path_buf();

        // 基本ディレクトリ構造を作成
        Self::setup_directory_structure(&base_path);

        Self {
            temp_dir,
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
            fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("Failed to create directory {:?}: {}", parent, e));
        }

        fs::write(&full_path, content)
            .unwrap_or_else(|e| panic!("Failed to write file {:?}: {}", full_path, e));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&full_path, fs::Permissions::from_mode(0o644))
                .unwrap_or_else(|e| panic!("Failed to set permissions for {:?}: {}", full_path, e));
        }

        full_path
            .strip_prefix(&self.base_path)
            .unwrap_or(&full_path)
            .to_path_buf()
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

    // 新しいメソッド: テスト実行のためのスコープを作成
    pub fn run_test_in_scope<F, R>(&self, test_fn: F) -> R
    where
        F: FnOnce() -> R,
    {
        // 現在のディレクトリを保存
        let original_dir = env::current_dir().expect("Failed to get current directory");

        // テスト用ディレクトリに移動
        env::set_current_dir(&self.base_path).unwrap_or_else(|e| {
            panic!("Failed to change directory to {:?}: {}", self.base_path, e)
        });

        // テスト関数を実行
        let result = test_fn();

        // 元のディレクトリに戻る
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

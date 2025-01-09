以下は現在のプロジェクトのコードの状況です。

Cargo.toml
```
[package]
name = "embed-files"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.5.24", features = ["derive"] }
glob = "0.3.2"
insta = "1.42.0"
regex = "1.11.1"
thiserror = "2.0.10"

```

src/main.rs
```
use cli::parse_cli;
use std::error::Error;

mod cli;
mod error;
mod warning;

fn main() {
    let cli = parse_cli();
    let mut warnings = warning::Warnings::new();
    let debug = cli.debug;

    if let Err(err) = run(cli, &mut warnings) {
        eprintln!("Error: {}", err);
        if debug {
            if let Some(source) = err.source() {
                eprintln!("\nCaused by:");
                let mut source = source;
                while let Some(next) = source.source() {
                    eprintln!("    {}", source);
                    source = next;
                }
                eprintln!("    {}", source);
            }
        }
        warnings.print_all();
        std::process::exit(1);
    }

    warnings.print_all();
}

fn run(cli: cli::Cli, warnings: &mut warning::Warnings) -> error::Result<()> {
    println!("Template path: {}", cli.template_path);
    Ok(())
}

```


src/cli.rs
```
use clap::Parser;

#[derive(Parser)]
#[command(name = "ef")]
#[command(author = "moai")]
#[command(version = "0.1.0")]
#[command(about = "Expands file contents in prompt templates for LLM input", long_about = None)]
pub struct Cli {
    /// Path to prompt template file
    #[arg(value_name = "TEMPLATE")]
    pub template_path: String,

    /// Show debug information
    #[arg(short, long)]
    pub debug: bool,
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_debug_flag() {
        let args = vec!["ef", "template.txt", "--debug"];
        let cli = Cli::parse_from(args);
        assert!(cli.debug);
        assert_eq!(cli.template_path, "template.txt");
    }

    #[test]
    fn test_cli_without_debug() {
        let args = vec!["ef", "template.txt"];
        let cli = Cli::parse_from(args);
        assert!(!cli.debug);
        assert_eq!(cli.template_path, "template.txt");
    }
}

```

src/error.rs
```
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid glob pattern: {pattern}")]
    InvalidGlobPattern {
        pattern: String,
        #[source]
        source: glob::PatternError,
    },

    #[error("Invalid regex pattern: {pattern}")]
    InvalidRegexPattern {
        pattern: String,
        #[source]
        source: regex::Error,
    },

    #[error("Invalid directive placement at line {line}: directive must be at the start of line")]
    DirectivePlacement { line: usize },

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

```

src/warning.rs
```
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

    pub fn print_all(&self) {
        for warning in &self.0 {
            eprintln!("Warning: {}", warning);
        }
    }
}

```

tests/common/mod.rs
```
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

pub struct TestEnv {
    pub temp_dir: TempDir,
}

impl TestEnv {
    pub fn new() -> Self {
        Self {
            temp_dir: tempdir().expect("Failed to create temp directory"),
        }
    }

    pub fn create_file(&self, path: &str, content: &str) -> PathBuf {
        let full_path = self.temp_dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        fs::write(&full_path, content).expect("Failed to write file");
        full_path
    }

    pub fn create_template(&self, content: &str) -> PathBuf {
        self.create_file("template.txt", content)
    }

    pub fn create_eftemplate(&self, content: &str) -> PathBuf {
        self.create_file(".eftemplate", content)
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}

// テスト用のサンプルファイルを作成する関数
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

```

tests/integration/mod.rs
```
use embed_files::*;
mod common;
use common::TestEnv;

#[test]
fn test_basic_template_processing() {
    let env = TestEnv::new();

    // 基本的なテンプレートファイルを作成
    let template = env.create_template(
        r#"Here is the main file:
#ef src/main.rs
"#,
    );

    // テスト用のサンプルファイルを作成
    common::setup_sample_files(&env);

    // デフォルトの.eftemplateを作成
    env.create_eftemplate(
        r#"File: {filePath}
```{language}
{content}
```"#,
    );

    // TODO: プログラムの実行とアサーション
    // この部分は実装が進んでから追加
}

#[test]
fn test_glob_pattern_expansion() {
    let env = TestEnv::new();

    // globパターンを使用したテンプレート
    let template = env.create_template(
        r#"All source files:
#ef src/*.rs
"#,
    );

    common::setup_sample_files(&env);

    // TODO: プログラムの実行とアサーション
}

#[test]
fn test_error_handling() {
    let env = TestEnv::new();

    // 不正なテンプレート（行頭以外の指示子）
    let template = env.create_template(
        r#"This is invalid:
  #ef src/main.rs
"#,
    );

    // TODO: エラーハンドリングのテスト
}

```

次にテンプレートファイル読み込み処理を実装していきます。

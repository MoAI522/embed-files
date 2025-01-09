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
mod template;
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
    let template = template::Template::from_file(&cli.template_path)?;

    // デバッグ出力（後で削除）
    for line in template.lines() {
        println!("{:?}", line);
    }

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

src/template.rs
```
use crate::error::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum Directive {
    Glob(String),
    Regex(String),
}

#[derive(Debug, PartialEq)]
pub struct Template {
    lines: Vec<TemplateLine>,
}

#[derive(Debug, PartialEq)]
pub enum TemplateLine {
    Text(String),
    Directive(Directive),
}

impl Template {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self> {
        let mut lines = Vec::new();

        for (_i, line) in content.lines().enumerate() {
            let line = if let Some(parts) = line.strip_prefix('#') {
                let mut elements = parts.split_whitespace();

                if let Some(directive_name) = elements.next() {
                    let arguments = elements.collect::<Vec<_>>();

                    match directive_name {
                        "ef" => {
                            let argument = arguments.join(" ");
                            if argument.is_empty() {
                                TemplateLine::Text(line.to_string())
                            } else {
                                TemplateLine::Directive(Directive::Glob(argument))
                            }
                        }
                        "efr" => {
                            let argument = arguments.join(" ");
                            if argument.is_empty() {
                                TemplateLine::Text(line.to_string())
                            } else {
                                TemplateLine::Directive(Directive::Regex(argument))
                            }
                        }
                        _ => TemplateLine::Text(line.to_string()),
                    }
                } else {
                    // #だけの行の場合
                    TemplateLine::Text(line.to_string())
                }
            } else {
                // #で始まらない行の場合
                TemplateLine::Text(line.to_string())
            };
            lines.push(line);
        }

        Ok(Self { lines })
    }

    pub fn lines(&self) -> &[TemplateLine] {
        &self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_glob_directive() {
        let template = Template::parse("#ef src/*.rs").unwrap();
        assert_eq!(
            template.lines(),
            &[TemplateLine::Directive(Directive::Glob(
                "src/*.rs".to_string()
            ))]
        );
    }

    #[test]
    fn test_parse_regex_directive() {
        let template = Template::parse("#efr ^src/.*\\.rs$").unwrap();
        assert_eq!(
            template.lines(),
            &[TemplateLine::Directive(Directive::Regex(
                "^src/.*\\.rs$".to_string()
            ))]
        );
    }

    #[test]
    fn test_parse_mixed_content() {
        let content = "Here is the content:\n#ef src/*.rs\nMore text";
        let template = Template::parse(content).unwrap();
        assert_eq!(
            template.lines(),
            &[
                TemplateLine::Text("Here is the content:".to_string()),
                TemplateLine::Directive(Directive::Glob("src/*.rs".to_string())),
                TemplateLine::Text("More text".to_string()),
            ]
        );
    }

    #[test]
    fn test_unknown_directive() {
        let template = Template::parse("#unknown argument").unwrap();
        assert_eq!(
            template.lines(),
            &[TemplateLine::Text("#unknown argument".to_string())]
        );
    }

    #[test]
    fn test_empty_directive() {
        let template = Template::parse("#ef").unwrap();
        assert_eq!(template.lines(), &[TemplateLine::Text("#ef".to_string())]);
    }

    #[test]
    fn test_hash_only_line() {
        let template = Template::parse("#").unwrap();
        assert_eq!(template.lines(), &[TemplateLine::Text("#".to_string())]);
    }

    #[test]
    fn test_directive_with_multiple_arguments() {
        let template = Template::parse("#ef src/*.rs test/*.rs").unwrap();
        assert_eq!(
            template.lines(),
            &[TemplateLine::Directive(Directive::Glob(
                "src/*.rs test/*.rs".to_string()
            ))]
        );
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

次にファイルパスの解決機能を実装していきます。

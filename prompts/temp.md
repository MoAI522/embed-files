以下は現在のプロジェクトのコードの状況です。

Cargo.toml
```
[package]
name = "embed-files"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0.95"
clap = { version = "4.5.24", features = ["derive"] }
colored = "3.0.0"
env_logger = "0.11.6"
glob = "0.3.2"
insta = "1.42.0"
log = "0.4.22"
pathdiff = "0.2.3"
regex = "1.11.1"
reqwest = { version = "0.12.12", features = ["blocking"] }
ron = "0.8.1"
serde = { version = "1.0.217", features = ["derive"] }
serde_yaml = "0.9.34"
tempfile = "3.15.0"
thiserror = "2.0.10"

[[bin]]
name = "update-languages"
path = "scripts/update_languages.rs"

[dev-dependencies]
mockito = "1.6.1"
similar-asserts = "1.6.0"
temp-env = "0.3.6"
tempfile = "3.15.0"

```

src/main.rs
```
mod cli;
mod eftemplate;
mod error;
mod executor;
mod path_resolver;
mod template;
mod warning;

use cli::parse_cli;
use std::error::Error;

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
    executor::execute(cli, &mut std::io::stdout(), warnings)
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

src/path_resolver.rs
```
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

```

src/lib.rs
```
use clap::Parser;

pub mod cli;
pub mod eftemplate;
pub mod error;
mod executor;
pub mod path_resolver;
pub mod template;
pub mod warning;

/// テスト用に公開する実行関数
#[doc(hidden)]
pub fn run_with_args(args: Vec<String>) -> error::Result<String> {
    let mut output = Vec::new();
    let mut warnings = warning::Warnings::new();
    let cli = cli::Cli::parse_from(args);

    executor::execute(cli, &mut output, &mut warnings)?;

    for warning in warnings.into_iter() {
        eprintln!("Warning: {}", warning);
    }

    Ok(String::from_utf8(output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?)
}

```

src/eftemplate.rs
```
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

    /// ファイル拡張子から言語識別子を取得
    fn detect_language(&self, path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext {
                "tsx" => "tsx",
                "jsx" => "jsx",
                "js" => "javascript",
                "ts" => "typescript",
                "rs" => "rust",
                "py" => "python",
                "rb" => "ruby",
                "php" => "php",
                "java" => "java",
                "c" => "c",
                "cpp" => "cpp",
                "h" => "cpp",
                "hpp" => "cpp",
                "css" => "css",
                "html" => "html",
                "xml" => "xml",
                "json" => "json",
                "yaml" => "yaml",
                "yml" => "yaml",
                "md" => "markdown",
                "txt" => "text",
                _ => "plaintext",
            })
            .unwrap_or("plaintext")
            .to_string()
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

```

src/executor.rs
```
use crate::cli;
use crate::eftemplate;
use crate::error;
use crate::path_resolver;
use crate::template;
use crate::warning;
use std::io::Write;

/// コマンドの実行ロジック
pub fn execute<W: Write>(
    cli: cli::Cli,
    writer: &mut W,
    warnings: &mut warning::Warnings,
) -> error::Result<()> {
    // テンプレートを読み込み
    let template = template::Template::from_file(&cli.template_path)?;
    let mut resolver = path_resolver::PathResolver::new(&cli.template_path);
    let eftemplate = eftemplate::EfTemplate::find_and_load(&cli.template_path)?;

    // テンプレートの各行を処理
    for line in template.lines() {
        match line {
            template::TemplateLine::Text(text) => {
                writeln!(writer, "{}", text)?;
            }
            template::TemplateLine::Directive(directive) => match directive {
                template::Directive::Glob(pattern) => {
                    let paths = resolver.resolve_glob(&pattern)?;
                    for path in paths {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let formatted = eftemplate.format(&path, &content);
                            write!(writer, "{}", formatted)?;
                        }
                    }
                }
                template::Directive::Regex(pattern) => {
                    let paths = resolver.resolve_regex(&pattern)?;
                    for path in paths {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let formatted = eftemplate.format(&path, &content);
                            write!(writer, "{}", formatted)?;
                        }
                    }
                }
            },
        }
    }

    // PathResolverからの警告を統合
    warnings.extend(resolver.take_warnings());

    Ok(())
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

tests/integration.rs
```
use embed_files::run_with_args;
mod common;
use common::TestEnv;
use std::env;

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

    env::set_current_dir(env.path()).unwrap();

    let args = vec!["ef".to_string(), template.to_str().unwrap().to_string()];
    let output = run_with_args(args).unwrap();

    assert!(output.contains("Here is the main file:"));
    assert!(output.contains("File: src/main.rs"));
    assert!(output.contains("```rust"));
    assert!(output.contains(
        r#"fn main() {
    println!("Hello, world!");
}"#
    ));
    assert!(output.contains("```"));
}

#[test]
fn test_glob_pattern_expansion() {
    let env = TestEnv::new();

    let template = env.create_template(
        r#"All source files:
#ef src/*.rs
"#,
    );

    common::setup_sample_files(&env);

    env.create_eftemplate(
        r#"=== {filePath} ===
Language: {language}
{content}
==========="#,
    );

    env::set_current_dir(env.path()).unwrap();

    let args = vec!["ef".to_string(), template.to_str().unwrap().to_string()];
    let output = run_with_args(args).unwrap();

    assert!(output.contains("All source files:"));
    assert!(output.contains("=== src/main.rs ==="));
    assert!(output.contains("=== src/lib.rs ==="));
    assert!(output.contains("Language: rust"));
    assert!(output.contains(
        r#"fn main() {
    println!("Hello, world!");
}"#
    ));
    assert!(output.contains(
        r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}"#
    ));
}

#[test]
fn test_regex_pattern_expansion() {
    let env = TestEnv::new();

    let template = env.create_template(
        r#"All rust files:
#efr .*\.rs$
"#,
    );

    common::setup_sample_files(&env);

    env::set_current_dir(env.path()).unwrap();

    let args = vec!["ef".to_string(), template.to_str().unwrap().to_string()];
    let output = run_with_args(args).unwrap();

    assert!(output.contains("All rust files:"));
    assert!(output.contains("src/main.rs"));
    assert!(output.contains("src/lib.rs"));
    assert!(output.contains("```rust"));
}

#[test]
fn test_no_eftemplate_uses_default() {
    let env = TestEnv::new();

    let template = env.create_template(
        r#"Main source:
#ef src/main.rs
"#,
    );

    common::setup_sample_files(&env);

    env::set_current_dir(env.path()).unwrap();

    let args = vec!["ef".to_string(), template.to_str().unwrap().to_string()];
    let output = run_with_args(args).unwrap();

    assert!(output.contains("Main source:"));
    assert!(output.contains("src/main.rs"));
    assert!(output.contains("```rust"));
    assert!(output.contains("```"));
}

#[test]
fn test_eftemplate_inheritance() {
    let env = TestEnv::new();

    env.create_eftemplate("ROOT: {filePath} ({language})\n{content}");

    let template = env.create_file(
        "subdir/template.txt",
        r#"File content:
#ef ../src/main.rs
"#,
    );
    env.create_file(
        "subdir/.eftemplate",
        "SUB: {filePath} ({language})\n{content}",
    );

    common::setup_sample_files(&env);

    env::set_current_dir(env.path()).unwrap();

    let args = vec!["ef".to_string(), template.to_str().unwrap().to_string()];
    let output = run_with_args(args).unwrap();

    assert!(output.contains("SUB: "));
    assert!(!output.contains("ROOT: "));
}

#[test]
fn test_debug_flag() {
    let env = TestEnv::new();

    let template = env.create_template(
        r#"Invalid pattern:
#ef [invalid/*.rs
"#,
    );

    common::setup_sample_files(&env);

    env::set_current_dir(env.path()).unwrap();

    let args = vec![
        "ef".to_string(),
        "--debug".to_string(),
        template.to_str().unwrap().to_string(),
    ];
    let result = run_with_args(args);

    assert!(result.is_err());
    // エラーメッセージは標準エラー出力に出力されるため、
    // 直接的な検証は難しいですが、エラーが発生することは確認できます
}

```



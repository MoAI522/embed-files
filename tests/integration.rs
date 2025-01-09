use ef::run_with_args;
mod common;
use common::TestEnv;
use std::env;
// tests/integration.rs

#[test]
fn test_basic_template_processing() {
    let env = TestEnv::new();
    env.run_test_in_scope(|| {
        let template = env.create_template(
            r#"Here is the main file:
#ef src/main.rs
"#,
        );

        common::setup_sample_files(&env);

        env.create_eftemplate(
            r#"File: {filePath}
```{language}
{content}
```"#,
        );

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
    })
}

#[test]
fn test_glob_pattern_expansion() {
    let env = TestEnv::new();
    env.run_test_in_scope(|| {
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
    })
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
#ef src/main.rs
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

#[test]
fn test_glob_pattern_expansion_with_current_dir() {
    let env = TestEnv::new();

    // テンプレートファイルを別のディレクトリに配置
    let template = env.create_file(
        "templates/template.txt",
        r#"All source files:
#ef src/*.rs
"#,
    );

    // ソースファイルはカレントディレクトリから見た相対パスで配置
    common::setup_sample_files(&env);

    env::set_current_dir(env.path()).unwrap();

    let args = vec!["ef".to_string(), template.to_str().unwrap().to_string()];
    let output = run_with_args(args).unwrap();

    assert!(output.contains("All source files:"));
    assert!(output.contains("src/main.rs")); // カレントディレクトリからの相対パス
    assert!(output.contains("src/lib.rs"));
}

#[test]
fn test_template_in_different_directory() {
    let env = TestEnv::new();

    // テンプレートファイルを深いディレクトリに配置
    let template = env.create_file(
        "templates/subdir/template.txt",
        r#"Source file:
#ef src/main.rs
"#,
    );

    // ソースファイルはカレントディレクトリから見た相対パスで配置
    common::setup_sample_files(&env);

    env::set_current_dir(env.path()).unwrap();

    let args = vec!["ef".to_string(), template.to_str().unwrap().to_string()];
    let output = run_with_args(args).unwrap();

    assert!(output.contains("Source file:"));
    assert!(output.contains("src/main.rs")); // カレントディレクトリからの相対パス
    assert!(output.contains(
        r#"fn main() {
    println!("Hello, world!");
}"#
    ));
}

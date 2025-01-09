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

pub mod cli;
pub mod error;
pub mod warning;

// 後で実装する予定のモジュール
// pub mod template;
// pub mod processor;
// pub mod eftemplate;

// テスト用に公開する実行関数
#[doc(hidden)]
pub fn run_with_args(args: Vec<String>) -> error::Result<String> {
    // TODO: 実装
    Ok(String::new())
}

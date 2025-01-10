以下のrustプロジェクトのテストが落ちる原因を究明してください。

#ef Cargo.toml

#ef src/**/*.rs

#ef tests/**/*.rs

何回か実行した際のテストの落ちた結果は以下の通りです。

```
failures:

---- path_resolver::tests::test_glob_resolution stdout ----
thread 'path_resolver::tests::test_glob_resolution' panicked at src/path_resolver.rs:153:9:
assertion `left == right` failed
  left: 1
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    path_resolver::tests::test_glob_resolution

test result: FAILED. 20 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```


```
failures:

---- test_glob_pattern_expansion stdout ----
thread 'test_glob_pattern_expansion' panicked at tests/integration.rs:61:42:
called `Result::unwrap()` on an `Err` value: IoError(Os { code: 2, kind: NotFound, message: "No such file or directory" })

---- test_basic_template_processing stdout ----
thread 'test_basic_template_processing' panicked at tests/integration.rs:27:42:
called `Result::unwrap()` on an `Err` value: IoError(Os { code: 2, kind: NotFound, message: "No such file or directory" })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- test_no_eftemplate_uses_default stdout ----
thread 'test_no_eftemplate_uses_default' panicked at tests/integration.rs:118:38:
called `Result::unwrap()` on an `Err` value: InvalidGlobPattern { pattern: "[invalid/*.rs", source: PatternError { pos: 0, msg: "invalid range pattern" } }

---- test_regex_pattern_expansion stdout ----
thread 'test_regex_pattern_expansion' panicked at tests/integration.rs:100:5:
assertion failed: output.contains("```rust")


failures:
    test_basic_template_processing
    test_glob_pattern_expansion
    test_no_eftemplate_uses_default
    test_regex_pattern_expansion

test result: FAILED. 4 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```


```
failures:

---- path_resolver::tests::test_glob_resolution stdout ----
thread 'path_resolver::tests::test_glob_resolution' panicked at src/path_resolver.rs:153:9:
assertion `left == right` failed
  left: 1
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    path_resolver::tests::test_glob_resolution

test result: FAILED. 20 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

テストが落ちる原因を特定し、修正してください。

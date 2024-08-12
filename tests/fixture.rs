use std::path::PathBuf;

use swc_ecma_parser::{Syntax, TsConfig};
use swc_core::ecma::{
    visit::as_folder,
};
use swc_ecma_transforms_testing::{test_fixture, FixtureTestConfig};
use s1s_async_import_plugin::MarkExpression;

#[testing::fixture("tests/fixture/**/input.*")]
fn fixture(input: PathBuf) {
    let ext = input.extension().unwrap();
    let output: PathBuf = input.with_file_name("output").with_extension(ext);
    let config_json = r#"
        {
            "title": "MARK_EXPRESSIONS",
            "functions": ["markFnA", "markFnB", "markFnC"],
            "methods": {
                "window": ["markWindowFnA", "markWindowFnB", "markWindowFnC"],
                "this": ["markThisFnA", "markThisFnB", "markThisFnC"]
            },
            "dynamicImports": ["shouldMark"]
        }
    "#;


    test_fixture(
        Syntax::Typescript(TsConfig {
            tsx: true,
            decorators: false,
            dts: false,
            no_early_errors: false,
            disallow_ambiguous_jsx_like: false,
        }),
        &|t| {
            as_folder(MarkExpression::new(
                t.comments.clone()
            ))
        },
        &input,
        &output,
        Default::default(),
    );
}

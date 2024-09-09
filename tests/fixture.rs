use std::path::PathBuf;

use swc_ecma_parser::{Syntax, TsConfig};
use swc_core::ecma::{
    visit::as_folder,
};
use swc_ecma_transforms_testing::{test_fixture, FixtureTestConfig};
use s1s_async_import_plugin::{MarkExpression, Config};

use std::fs::File;
use std::io::{self, Read};
use std::env;

use serde_json::json;

fn read_json_file(file_path: &PathBuf) -> io::Result<String> {
    // 打开文件
    let mut file = File::open(file_path)?;

    // 创建一个字符串来存储文件内容
    let mut contents = String::new();

    // 读取文件内容到字符串
    file.read_to_string(&mut contents)?;

    // 返回读取的内容
    Ok(contents)
}

#[testing::fixture("tests/fixture/**/input.*")]
fn fixture(input: PathBuf) {
    // 指定 JSON 文件的路径
    let current_dir = env::current_dir().unwrap();
    let file_path = current_dir.join("tests/map.json");

    let mut record_str = String::new();

    // 调用 read_json_file 方法
    match read_json_file(&file_path) {
        Ok(contents) => {
            // 打印读取的内容
            println!("File contents: {}", contents.clone());

            record_str = contents;

            // 如果需要，可以在这里解析 JSON
            // let json_value: serde_json::Value = serde_json::from_str(&contents)?;
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }

    let ext = input.extension().unwrap();
    let output: PathBuf = input.with_file_name("output").with_extension(ext);
    let config_json = serde_json::to_string(&json!({
        "record": record_str
    })).unwrap();

    let config = serde_json::from_str::<Option<Config>>(config_json.as_str())
    .expect("Invalid config")
    .unwrap();

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
                t.comments.clone(),
                &config,
            ))
        },
        &input,
        &output,
        Default::default(),
    );
}

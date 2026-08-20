//! devtools —— 开发期工具集（纯前端单机方案的构建期支撑）。
//!
//! 不部署、不进小程序包，仅在开发期使用：
//!
//! ```text
//! cargo run -p devtools -- preprocess <in.imazingapp> <out.imazingapp>
//! cargo run -p devtools -- scan <config_dir> <template.imazingapp>
//! cargo run -p devtools -- golden <config_dir> <template.imazingapp> <out_dir>
//! ```

mod compression;
mod dump;
mod golden;
mod preprocess;
mod scan;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("preprocess") => {
            let (in_path, out_path) = expect_pair(&args[2..], "preprocess <in> <out>");
            match preprocess::run(&in_path, &out_path) {
                Ok(()) => {
                    println!("✓ 预处理完成: {}", out_path);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("✗ 预处理失败: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some("scan") => {
            let (config_dir, template) = expect_pair(&args[2..], "scan <config_dir> <template>");
            match scan::run(&config_dir, &template) {
                Ok(report) => {
                    println!("{report}");
                    println!("✓ 模板扫描通过");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("✗ 模板扫描失败: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some("dump") => {
            let template = args
                .get(2)
                .filter(|a| !a.starts_with("--"))
                .cloned()
                .unwrap_or_else(|| "template-standalone.imazingapp".to_string());
            match dump::run(&template) {
                Ok(report) => {
                    println!("{report}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("✗ dump 失败: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some("golden") => {
            let (config_dir, template) =
                expect_pair(&args[2..], "golden <config_dir> <template> <out_dir>");
            let out_dir = args.get(4).cloned().unwrap_or_else(|| "golden".to_string());
            match golden::run(&config_dir, &template, &out_dir) {
                Ok(summary) => {
                    println!("{summary}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("✗ golden 生成失败: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn expect_pair(args: &[String], usage: &str) -> (String, String) {
    if args.len() < 2 {
        eprintln!("用法: {usage}");
        std::process::exit(2);
    }
    (args[0].clone(), args[1].clone())
}

fn print_help() {
    println!(
        "devtools —— 开发期工具\n\
         \n\
         用法:\n\
         \x20 cargo run -p devtools -- preprocess <in.imazingapp> <out.imazingapp>\n\
         \x20     将模板内目标条目（Active.sav / UserCustom.ini）改为 Stored 存储\n\
         \n\
         \x20 cargo run -p devtools -- scan <config_dir> <template.imazingapp>\n\
         \x20     断言配置引用的 sav 属性 / ini cvar 均存在于模板，输出扫描报告\n\
         \n\
         \x20 cargo run -p devtools -- golden <config_dir> <template.imazingapp> <out_dir>\n\
         \x20     生成各配置组合的 golden fixtures（case-NN.imazingapp + case-NN.json）"
    );
}

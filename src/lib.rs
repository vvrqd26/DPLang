// DPLang 解释器核心库

pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod executor;
pub mod package_loader;
pub mod semantic;
pub mod indicators;
pub mod api;
pub mod streaming;
pub mod orchestration; // 任务编排系统
pub mod backtest; // 回测系统
// pub mod builtin;
// pub mod interpreter;

// 导出公共 API
pub use api::DPLangInterpreter;
pub use api::{parse_csv, format_output_csv};
// pub use indicators::{EMAState, MACDCalculator, KDJCalculator}; // 这些结构体不存在

// pub use interpreter::Interpreter;

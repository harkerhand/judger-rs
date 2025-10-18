use clap::Parser;
use judger::{Config, SeccompRuleName, run};

const VERSION: &str = "0.1.0";

#[derive(Parser, Debug)]
#[command(name = "judger", version = VERSION, about = "A Rust-based code execution judger.")]
pub(crate) struct Args {
    #[arg(long, help = "Max CPU Time (ms)")]
    max_cpu_time: Option<i32>,
    #[arg(long, help = "Max Real Time (ms)")]
    max_real_time: Option<i32>,
    #[arg(long, help = "Max Memory (byte)")]
    max_memory: Option<i64>,
    #[arg(
        long,
        help = "Only check memory usage, do not setrlimit (default: false)"
    )]
    memory_limit_check_only: Option<bool>,
    #[arg(long, help = "Max Stack (byte, default 16M)")]
    max_stack: Option<i64>,
    #[arg(long, help = "Max Process Number")]
    max_process_number: Option<i32>,
    #[arg(long, help = "Max Output Size (byte)")]
    max_output_size: Option<i64>,
    #[arg(long, help = "Exe Path")]
    exe_path: String,
    #[arg(long, help = "Input Path")]
    input_path: Option<String>,
    #[arg(long, help = "Output Path")]
    output_path: Option<String>,
    #[arg(long, help = "Error Path")]
    error_path: Option<String>,
    #[arg(long, help = "Log Path")]
    log_path: Option<String>,
    #[arg(long, help = "Seccomp Rule Name")]
    seccomp_rule_name: Option<SeccompRuleName>,
    #[arg(long, help = "UID (default: 65534)")]
    uid: Option<u32>,
    #[arg(long, help = "GID (default: 65534)")]
    gid: Option<u32>,
    #[arg(long, help = "Arg")]
    args: Vec<String>,
    #[arg(long, help = "Env")]
    env: Vec<String>,
}
fn main() {
    let args = Args::parse();

    let config = Config {
        max_cpu_time: args.max_cpu_time.unwrap_or(-1),
        max_real_time: args.max_real_time.unwrap_or(-1),
        max_memory: args.max_memory.unwrap_or(-1),
        max_stack: args.max_stack.unwrap_or(16 * 1024 * 1024),
        max_process_number: args.max_process_number.unwrap_or(-1),
        max_output_size: args.max_output_size.unwrap_or(-1),
        memory_limit_check_only: args.memory_limit_check_only.unwrap_or(false),
        exe_path: args.exe_path,
        input_path: args.input_path.unwrap_or_else(|| "/dev/stdin".to_string()),
        output_path: args
            .output_path
            .unwrap_or_else(|| "/dev/stdout".to_string()),
        error_path: args.error_path.unwrap_or_else(|| "/dev/stderr".to_string()),
        args: args.args,
        env: args.env,
        log_path: args.log_path.unwrap_or_else(|| "judger.log".to_string()),
        seccomp_rule_name: args.seccomp_rule_name,
        uid: args.uid.unwrap_or(65534),
        gid: args.gid.unwrap_or(65534),
    };

    let result = run(&config);

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}

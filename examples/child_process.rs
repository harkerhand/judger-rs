use judger::{child_process, Config, Logger};

fn main() {
    let config = Config {
        max_cpu_time: 1000,
        max_real_time: 2000,
        max_memory: 128 * 1024 * 1024,
        max_stack: 32 * 1024 * 1024,
        max_process_number: 200,
        max_output_size: 10000,
        memory_limit_check_only: false,
        exe_path: "hello_world".to_string(),
        input_path: "1.in".to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.out".to_string(),
        args: vec![],
        env: vec![],
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some("c_cpp".to_string()),
        uid: 0,
        gid: 0,
    };
    let logger = Logger::new(&config.log_path).expect("Failed to create logger");
    let result = child_process(&config, logger);
    println!("{:?}", result);
}
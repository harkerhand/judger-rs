use judger::{Config, SeccompRuleName, run};
use std::io::Write;

fn main() {
    let tmp_file_path = "./main.c";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let hello_world_code = r#"#include <stdio.h>
int main(int argc, char *argv[]) {
    char input[1000];
    scanf("%s", input);
    printf("Hello %s\n", input);
    return 0;
}"#;
    file.write_all(hello_world_code.as_bytes())
        .expect("Unable to write data");

    let input_file_path = "1.in";
    let mut input_file =
        std::fs::File::create(input_file_path).expect("Unable to create input file");
    let input_data = "World\n";
    input_file
        .write_all(input_data.as_bytes())
        .expect("Unable to write input data");

    let _ = std::process::Command::new("gcc")
        .args([tmp_file_path, "-o", "hello_world"])
        .output();

    let config = Config {
        max_cpu_time: 1000,
        max_real_time: 2000,
        max_memory: 128 * 1024 * 1024,
        max_stack: 32 * 1024 * 1024,
        max_process_number: 200,
        max_output_size: 10000,
        memory_limit_check_only: false,
        exe_path: "hello_world".to_string(),
        input_path: input_file_path.to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.out".to_string(),
        args: vec![],
        env: vec![],
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::CCpp),
        uid: 0,
        gid: 0,
    };

    let result = run(&config);

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
}

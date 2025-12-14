use judger::{Config, SeccompRuleName, run};
use std::io::Write;

#[test]
fn test_syscall_error() {
    let tmp_file_path = "./main.c";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let hello_world_code = r#"#include <stdio.h>
int main() {
    FILE *fp = fopen("unauthorized_write.txt", "w");
    fprintf(fp, "This write operation should be blocked by seccomp!\n");
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
        .args([tmp_file_path, "-o", "syscall_wrong"])
        .output();

    let config = Config {
        exe_path: "syscall_wrong".to_string(),
        input_path: input_file_path.to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.out".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::CCpp),
        ..Default::default()
    };

    let result = run(&config, None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.result, judger::ErrorCode::RuntimeError);
    // clean up
    let _ = std::fs::remove_file(tmp_file_path);
    let _ = std::fs::remove_file(input_file_path);
    let _ = std::fs::remove_file("syscall_wrong");
    let _ = std::fs::remove_file("1.out");
    let _ = std::fs::remove_file("1.err");
    let _ = std::fs::remove_file("judger.log");
}

use judger::{Config, SeccompRuleName, run};
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_interactor() {
    let tmp_file_path = "./user.c";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let hello_world_code = r#"#include <stdio.h>
int main() {
    int a, b;
    while(scanf("%d %d", &a, &b) != EOF) {
        printf("%d\n", a + b);
        fflush(stdout);
    }
    return 0;
}"#;
    file.write_all(hello_world_code.as_bytes())
        .expect("Unable to write data");

    let input_file_path = "user.in";
    let mut input_file =
        std::fs::File::create(input_file_path).expect("Unable to create input file");
    let input_data = "2\n10 20\n100 200\n";
    input_file
        .write_all(input_data.as_bytes())
        .expect("Unable to write input data");

    let _ = std::process::Command::new("gcc")
        .args([tmp_file_path, "-o", "user"])
        .output();

    let config = Config {
        exe_path: "user".to_string(),
        input_path: input_file_path.to_string(),
        output_path: "user.out".to_string(),
        error_path: "user.err".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::CCpp),
        ..Default::default()
    };

    let result = run(&config, Some(PathBuf::from("assets/interactor")));
    // let result = run(&config, None);
    println!("{:?}", result);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.result, judger::ErrorCode::Success);
    // clean up
    let _ = std::fs::remove_file(tmp_file_path);
    let _ = std::fs::remove_file(input_file_path);
    let _ = std::fs::remove_file("user");
    let _ = std::fs::remove_file("user.out");
    let _ = std::fs::remove_file("user.err");
    let _ = std::fs::remove_file("judger.log");
}

#[test]
fn test_interactor_wa() {
    let tmp_file_path = "./user_wrong.c";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let hello_world_code = r#"#include <stdio.h>
int main() {
    int a, b;
    while(scanf("%d %d", &a, &b) != EOF) {
        printf("%d\n", a - b);
        fflush(stdout);
    }
    return 0;
}"#;
    file.write_all(hello_world_code.as_bytes())
        .expect("Unable to write data");

    let input_file_path = "user_wrong.in";
    let mut input_file =
        std::fs::File::create(input_file_path).expect("Unable to create input file");
    let input_data = "2\n10 20\n100 200\n";
    input_file
        .write_all(input_data.as_bytes())
        .expect("Unable to write input data");

    let _ = std::process::Command::new("gcc")
        .args([tmp_file_path, "-o", "user_wrong"])
        .output();

    let config = Config {
        exe_path: "user_wrong".to_string(),
        input_path: input_file_path.to_string(),
        output_path: "user_wrong.out".to_string(),
        error_path: "user_wrong.err".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::CCpp),
        ..Default::default()
    };

    let result = run(&config, Some(PathBuf::from("assets/interactor")));
    println!("{:?}", result);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(
        result.result,
        judger::ErrorCode::WrongAnswer(
            "wrong answer Query 1: expected 30, found -10\n".to_string()
        )
    );
    // clean up
    let _ = std::fs::remove_file(tmp_file_path);
    let _ = std::fs::remove_file(input_file_path);
    let _ = std::fs::remove_file("user_wrong");
    let _ = std::fs::remove_file("user_wrong.out");
    let _ = std::fs::remove_file("user_wrong.err");
    let _ = std::fs::remove_file("judger.log");
}

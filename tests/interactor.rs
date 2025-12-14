use judger::{run, Config, SeccompRuleName};
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_interactor() {
    let tmp_file_path = "./user.c";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let hello_world_code = r#"#include <stdio.h>
int main() {
    int a, b;
    fprintf(stderr, "Debug info1\n");
    while(scanf("%d %d", &a, &b) != EOF) {
        printf("%d\n", a + b);
        fflush(stdout);
        fprintf(stderr, "Debug info %d\n", a + b);
    }
    return 0;
}"#;
    file.write_all(hello_world_code.as_bytes())
        .expect("Unable to write data");

    let input_file_path = "1.in";
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
        output_path: "1.out".to_string(),
        error_path: "1.err".to_string(),
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
    let expected_output = "Begin\n2\n10 20\n30\n100 200\n300";
    let actual_output = std::fs::read_to_string("1.out").expect("Unable to read output file");
    assert_eq!(actual_output.trim(), expected_output);
    // clean up
    let _ = std::fs::remove_file(tmp_file_path);
    let _ = std::fs::remove_file(input_file_path);
    let _ = std::fs::remove_file("user");
    let _ = std::fs::remove_file("1.out");
    let _ = std::fs::remove_file("1.err");
    let _ = std::fs::remove_file("judger.log");
}

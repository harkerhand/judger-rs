use judger::{Config, ErrorCode, SeccompRuleName, run};
use std::io::Write;

#[test]
fn test_tle() {
    let tmp_file_path = "./tle.c";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let hello_world_code = r#"
int main() {
    volatile long long j = 1;
    for(long long i = 0; i < 2000000000LL; i++) {
        j += i;
    }
    return 0;
}"#;
    file.write_all(hello_world_code.as_bytes())
        .expect("Unable to write data");

    let input_file_path = "1.in";
    let mut input_file =
        std::fs::File::create(input_file_path).expect("Unable to create input file");
    let input_data = "\n";
    input_file
        .write_all(input_data.as_bytes())
        .expect("Unable to write input data");

    let _ = std::process::Command::new("gcc")
        .args([tmp_file_path, "-o", "tle"])
        .output();

    let config = Config {
        exe_path: "tle".to_string(),
        input_path: input_file_path.to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.out".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::CCpp),
        ..Default::default()
    };

    let result = run(&config);
    assert!(result.is_ok());
    let result = result.unwrap();
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    assert!(
        result.result == ErrorCode::CpuTimeLimitExceeded
            || result.result == ErrorCode::RealTimeLimitExceeded
    );
    // clean up
    let _ = std::fs::remove_file(tmp_file_path);
    let _ = std::fs::remove_file(input_file_path);
    let _ = std::fs::remove_file("tle");
    let _ = std::fs::remove_file("1.out");
    let _ = std::fs::remove_file("1.err");
    let _ = std::fs::remove_file("judger.log");
}

#[test]
fn test_mle() {
    let tmp_file_path = "./mle.c";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let mle_code = r#"
#include <stdlib.h>
int main() {
    int *arr = (int*)malloc(128 * 1024 * 1024);
    if (arr == NULL) {
        return 1;
    }
    for (long long i = 0; i < 128 * 1024 * 1024 / sizeof(int); i++) {
        arr[i] = i;
        if (i % (1024 * 1024) == 0)
            printf("%d ", arr[i]);
    }
    free(arr);
    return 0;
}"#;
    file.write_all(mle_code.as_bytes())
        .expect("Unable to write data");
    let input_file_path = "1.in";
    let mut input_file =
        std::fs::File::create(input_file_path).expect("Unable to create input file");
    let input_data = "\n";
    input_file
        .write_all(input_data.as_bytes())
        .expect("Unable to write input data");
    let _ = std::process::Command::new("gcc")
        .args([tmp_file_path, "-o", "mle"])
        .output();
    let config = Config {
        exe_path: "mle".to_string(),
        input_path: input_file_path.to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.out".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::CCpp),
        ..Default::default()
    };
    let result = run(&config);
    assert!(result.is_ok());
    let result = result.unwrap();
    println!("{:?}", result);
    assert_eq!(result.result, ErrorCode::MemoryLimitExceeded);
    // clean up
    let _ = std::fs::remove_file(tmp_file_path);
    let _ = std::fs::remove_file(input_file_path);
    let _ = std::fs::remove_file("mle");
    let _ = std::fs::remove_file("1.out");
    let _ = std::fs::remove_file("1.err");
    let _ = std::fs::remove_file("judger.log");
}

#[test]
fn test_ole() {
    let tmp_file_path = "./ole.c";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let stack_overflow_code = r#"
#include <stdio.h>
void recursive_function() {
    int arr[1024 * 1024]; // Allocate a large array on the stack
    for (int i = 0; i < 1024 * 1024; i++) {
        arr[i] = i;
    }
    recursive_function(); // Recursive call
}
int main() {
    recursive_function();
    return 0;
}"#;
    file.write_all(stack_overflow_code.as_bytes())
        .expect("Unable to write data");
    let input_file_path = "1.in";
    let mut input_file =
        std::fs::File::create(input_file_path).expect("Unable to create input file");
    let input_data = "\n";
    input_file
        .write_all(input_data.as_bytes())
        .expect("Unable to write input data");
    let _ = std::process::Command::new("gcc")
        .args([tmp_file_path, "-o", "stack_overflow"])
        .output();
    let config = Config {
        exe_path: "stack_overflow".to_string(),
        input_path: input_file_path.to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.out".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::CCpp),
        ..Default::default()
    };
    let result = run(&config);
    assert!(result.is_ok());
    let result = result.unwrap();
    println!("{:?}", result);
    assert_eq!(result.result, ErrorCode::RuntimeError);
    // clean up
    let _ = std::fs::remove_file(tmp_file_path);
    let _ = std::fs::remove_file(input_file_path);
    let _ = std::fs::remove_file("stack_overflow");
    let _ = std::fs::remove_file("1.out");
    let _ = std::fs::remove_file("1.err");
    let _ = std::fs::remove_file("judger.log");
}

#[test]
fn test_cle() {
    let tmp_file_path = "./cle.c";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let stack_overflow_code = r#"
#include <stdio.h>
int main() {
    while (1) {
        printf("Hello, World!\n");
    }
    return 0;
}"#;
    file.write_all(stack_overflow_code.as_bytes())
        .expect("Unable to write data");
    let input_file_path = "1.in";
    let mut input_file =
        std::fs::File::create(input_file_path).expect("Unable to create input file");
    let input_data = "\n";
    input_file
        .write_all(input_data.as_bytes())
        .expect("Unable to write input data");
    let _ = std::process::Command::new("gcc")
        .args([tmp_file_path, "-o", "char_overflow"])
        .output();
    let config = Config {
        exe_path: "char_overflow".to_string(),
        input_path: input_file_path.to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.out".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::CCpp),
        ..Default::default()
    };
    let result = run(&config);
    assert!(result.is_ok());
    let result = result.unwrap();
    println!("{:?}", result);
    assert_eq!(result.result, ErrorCode::RuntimeError);
    // clean up
    let _ = std::fs::remove_file(tmp_file_path);
    let _ = std::fs::remove_file(input_file_path);
    let _ = std::fs::remove_file("char_overflow");
    let _ = std::fs::remove_file("1.out");
    let _ = std::fs::remove_file("1.err");
    let _ = std::fs::remove_file("judger.log");
}

use judger::{Config, SeccompRuleName, run};
use std::io::Write;

fn main() {
    let tmp_file_path = "./go.go";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let hello_world_code = r#"
package main
import (
    "bufio"
    "fmt"
    "os"
)
func main() {
    reader := bufio.NewReader(os.Stdin)
    name, _ := reader.ReadString('\n')
    fmt.Printf("Hello, %s", name)
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

    let _ = std::process::Command::new("go")
        .args(["build", "-o", "gogo", tmp_file_path])
        .output();

    let config = Config {
        exe_path: "gogo".to_string(),
        // go 更高的内存需求
        max_memory: 512 * 1024 * 1024,
        input_path: input_file_path.to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.err".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::Golang),
        ..Default::default()
    };

    let result = run(&config, None);

    println!("{:?}", result);

    // clean up
    let _ = std::fs::remove_file(tmp_file_path);
    let _ = std::fs::remove_file(input_file_path);
    let _ = std::fs::remove_file("gogo");
    let _ = std::fs::remove_file("1.out");
    let _ = std::fs::remove_file("1.err");
    let _ = std::fs::remove_file("judger.log");
}

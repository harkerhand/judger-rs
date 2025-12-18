use judger::{run, Config, SeccompRuleName};
use std::io::Write;

fn main() {
    let tmp_file_path = "./node.js";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let hello_world_code = r#"
console.log('Hello, ' + require('fs').readFileSync(0, 'utf-8').trim() + '!');
    "#;
    file.write_all(hello_world_code.as_bytes())
        .expect("Unable to write data");
    drop(file);

    let input_file_path = "1.in";
    let mut input_file =
        std::fs::File::create(input_file_path).expect("Unable to create input file");
    let input_data = "World\n";
    input_file
        .write_all(input_data.as_bytes())
        .expect("Unable to write input data");
    drop(input_file);

    let config = Config {
        exe_path: "/usr/bin/node".to_string(),
        args: vec!["/usr/bin/node".to_string(), tmp_file_path.to_string()],
        max_cpu_time: 2000,
        max_real_time: 4000,
        // nodejs需要更多内存
        max_memory: 256 * 1024 * 1024,
        input_path: input_file_path.to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.err".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::Node),
        ..Default::default()
    };

    let result = run(&config, None);

    println!("{:?}", result);

    // clean up
    let _ = std::fs::remove_file(tmp_file_path);
    let _ = std::fs::remove_file(input_file_path);
    let _ = std::fs::remove_file("1.out");
    let _ = std::fs::remove_file("1.err");
    let _ = std::fs::remove_file("judger.log");
}

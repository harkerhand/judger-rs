use judger::{Config, SeccompRuleName, run};
use std::io::Write;

fn main() {
    let tmp_file_path = "./java.java";
    let mut file = std::fs::File::create(tmp_file_path).expect("Unable to create file");
    let hello_world_code = r#"
import java.util.Scanner;
public class Main {
    public static void main(String[] args) {
        Scanner sc = new Scanner(System.in);
        if(sc.hasNextInt()) {
            int n = sc.nextInt();
            while(n-- > 0){
                int a = sc.nextInt();
                int b = sc.nextInt();
                System.out.println(a + b);
            }
        }
    }
}
"#;
    file.write_all(hello_world_code.as_bytes())
        .expect("Unable to write data");

    let input_file_path = "1.in";
    let mut input_file =
        std::fs::File::create(input_file_path).expect("Unable to create input file");
    let input_data = "1 2 3\n";
    input_file
        .write_all(input_data.as_bytes())
        .expect("Unable to write input data");

    let config = Config {
        exe_path: "/usr/bin/java".to_string(),
        args: vec![
            "/usr/bin/java".to_string(),
            "-Xmx256M".to_string(),
            "-XX:CompressedClassSpaceSize=64M".to_string(),
            tmp_file_path.to_string(),
        ],
        // 对于java 真实的内存限制是INFINITY，因为java本身会有额外的内存开销
        // 这里的内存限制主要是为了在程序运行后得到MLE状态
        max_memory: 128 * 1024 * 1024,
        max_cpu_time: 2000,
        max_real_time: 4000,
        input_path: input_file_path.to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.err".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::Java),
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

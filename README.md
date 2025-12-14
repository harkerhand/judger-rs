# Judger

[![Crates.io](https://img.shields.io/crates/v/judger.svg)](https://crates.io/crates/judger)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A secure and efficient sandboxed code execution engine written in Rust. Ideal for online judges, educational platforms,
and other systems requiring isolated execution of untrusted code.

## Features

* **Resource Limiting**: Enforce time and memory limits on the executed process.
* **Secure Sandboxing**: Utilizes Linux namespaces and seccomp for strong process isolation and system call filtering.
* **Flexible Configuration**: Easily configure limits, system call policies, and file access.
* **Cross-platform**: Written in Rust for reliable and efficient execution.

## Getting Started

### Prerequisites

* Rust toolchain (latest stable recommended)
* Linux environment (for seccomp and namespace features)

### Installation

You can install `judger` from Crates.io:

```bash
cargo install judger
```

Or, you can build it from the source:

```bash
git clone https://github.com/harkerhand/judger-rs.git
cd judger-rs
cargo build --release
```

### Example

Here is a simple example of how to use `judger` to run a command with resource limits:

Make sure to add `judger` to your `Cargo.toml`:

```toml
[dependencies]
judger = "0.1"
```

Then, on your main.rs:

```rust
use judger::{Config, SeccompRuleName, run};
fn main() {
    let config = Config {
        exe_path: "hello_world".to_string(),
        input_path: "1.in".to_string(),
        output_path: "1.out".to_string(),
        error_path: "1.err".to_string(),
        log_path: "judger.log".to_string(),
        seccomp_rule_name: Some(SeccompRuleName::CCpp),
        ..Default::default()
    };
    let result = run(&config, None);
    println!("{:?}", result);
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

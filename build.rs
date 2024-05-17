// fn main() {
//     std::process::Command::new("sh")
//         .arg("-c")
//         .arg("cd test-contracts && python build.py && cd ..")
//         .status()
//         .expect("Failed to execute build script");
// }
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
fn main() {
    // Execute the build script and capture the output
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("cd test-contracts && python build.py && cd ..")
        .output()
        .expect("Failed to execute build script");

    // Get the current directory
    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    // Create the build log file in the current directory
    let build_stdout_log_path = current_dir.join("build_stdout.log");
    let build_stderr_log_path = current_dir.join("build_stderr.log");
    let mut build_stdout_log_file = File::create(&build_stdout_log_path).expect("Failed to create build log file");
    let mut build_stderr_log_file = File::create(&build_stderr_log_path).expect("Failed to create build log file");

    // Write the stdout and stderr to the build log file
    build_stdout_log_file
        .write_all(&output.stdout)
        .expect("Failed to write stdout to build log");
    build_stderr_log_file
        .write_all(&output.stderr)
        .expect("Failed to write stderr to build log");
}

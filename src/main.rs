use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::os::unix::process::ExitStatusExt;

#[derive(Debug)]
enum RunStatus {
    Completed,
    TimedOut,
    FailedToStart,
    Signaled,
}

#[derive(Debug)]
struct RunResult {
    command: String,
    args: Vec<String>,
    status: RunStatus,
    exit_code: Option<i32>,
    signal: Option<i32>,
    duration: f64,
    stdout: String,
    stderr: String,
}

fn run_command(command: &str, args: &[&str], timeout_secs: u64) -> RunResult {
    let start = Instant::now();

    let mut child = match Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            return RunResult {
                command: command.to_string(),
                args: args.iter().map(|s| s.to_string()).collect(),
                status: RunStatus::FailedToStart,
                exit_code: None,
                signal: None,
                duration: start.elapsed().as_secs_f64(),
                stdout: String::new(),
                stderr: format!("Failed to start process: {err}"),
            };
        }
    };

    let timeout = Duration::from_secs(timeout_secs);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child
                    .wait_with_output()
                    .expect("Process ended, but output could not be collected");

                let run_status = if status.signal().is_some() {
                    RunStatus::Signaled
                } else {
                    RunStatus::Completed
                };

                return RunResult {
                    command: command.to_string(),
                    args: args.iter().map(|s| s.to_string()).collect(),
                    status: run_status,
                    exit_code: status.code(),
                    signal: status.signal(),
                    duration: start.elapsed().as_secs_f64(),
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                };
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();

                    let output = child
                        .wait_with_output()
                        .expect("Timed out process, but output could not be collected");

                    return RunResult {
                        command: command.to_string(),
                        args: args.iter().map(|s| s.to_string()).collect(),
                        status: RunStatus::TimedOut,
                        exit_code: None,
                        signal: None,
                        duration: start.elapsed().as_secs_f64(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    };
                }

                thread::sleep(Duration::from_millis(10));
            }
            Err(err) => {
                return RunResult {
                    command: command.to_string(),
                    args: args.iter().map(|s| s.to_string()).collect(),
                    status: RunStatus::FailedToStart,
                    exit_code: None,
                    signal: None,
                    duration: start.elapsed().as_secs_f64(),
                    stdout: String::new(),
                    stderr: format!("Error while waiting on process: {err}"),
                };
            }
        }
    }
}

fn print_result(result: &RunResult) {
    println!("=== RUN RESULT ===");
    println!("command: {}", result.command);
    println!("args: {:?}", result.args);

    match result.status {
        RunStatus::Completed => println!("status: completed"),
        RunStatus::TimedOut => println!("status: timed out"),
        RunStatus::FailedToStart => println!("status: failed to start"),
        RunStatus::Signaled => println!("status: terminated by signal"),
    }

    match result.exit_code {
        Some(code) => println!("exit code: {}", code),
        None => println!("exit code: None"),
    }

    match result.signal {
        Some(sig) => println!("signal: {}", sig),
        None => println!("signal: None"),
    }

    println!("duration: {:.4} seconds", result.duration);

    println!("--- stdout ---");
    if result.stdout.is_empty() {
        println!("(empty)");
    } else {
        println!("{}", result.stdout);
    }

    println!("--- stderr ---");
    if result.stderr.is_empty() {
        println!("(empty)");
    } else {
        println!("{}", result.stderr);
    }
}

fn main() {
    let result = run_command("python3", &["test_scripts/hello.py"], 3);
    print_result(&result);
}
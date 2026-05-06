#![allow(dead_code)]

use std::io::Write;
use std::process::{Command, Stdio};

pub fn run_with_stdin(command: &str, input: &str) -> Vec<u8> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg(command)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors");

    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(input.as_bytes())
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait for biors");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    output.stdout
}

pub trait ChildInputExt {
    fn tap_stdin(self, input: &str) -> std::process::Output;
}

impl ChildInputExt for std::process::Child {
    fn tap_stdin(mut self, input: &str) -> std::process::Output {
        self.stdin
            .as_mut()
            .expect("stdin pipe")
            .write_all(input.as_bytes())
            .expect("write stdin");

        self.wait_with_output().expect("wait for biors")
    }
}

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn analyzes_stdin_as_json() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_exitsplain"))
        .args(["analyze", "--format", "json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"fatal: detected dubious ownership in repository")
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let body = String::from_utf8(output.stdout).unwrap();
    assert!(body.contains("\"id\": \"git-dubious-ownership\""));
}

#[test]
fn lists_rules_by_tag() {
    let output = Command::new(env!("CARGO_BIN_EXE_exitsplain"))
        .args(["rules", "--tag", "docker"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let body = String::from_utf8(output.stdout).unwrap();
    assert!(body.contains("docker-daemon-unavailable"));
    assert!(!body.contains("python-module-not-found"));
}

use std::process::Command;

#[test]
fn run_preserves_the_child_exit_code() {
    let mut command = Command::new(env!("CARGO_BIN_EXE_exitsplain"));
    command.args(["run", "--no-color", "--"]);

    #[cfg(windows)]
    command.args(["cmd", "/C", "echo npm ERR! code ERESOLVE 1>&2 & exit /b 7"]);

    #[cfg(not(windows))]
    command.args(["sh", "-c", "echo 'npm ERR! code ERESOLVE' >&2; exit 7"]);

    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("npm found incompatible peer dependencies")
    );
}

use exitsplain::runner;

#[test]
fn streams_and_captures_a_failing_command() {
    let command = if cfg!(windows) {
        vec![
            "cmd".to_owned(),
            "/C".to_owned(),
            "echo hello & echo EADDRINUSE 1>&2 & exit /b 7".to_owned(),
        ]
    } else {
        vec![
            "sh".to_owned(),
            "-c".to_owned(),
            "echo hello; echo EADDRINUSE >&2; exit 7".to_owned(),
        ]
    };

    let outcome = runner::run(&command, 4096).unwrap();
    assert_eq!(outcome.exit_code, 7);
    assert!(outcome.captured.contains("hello"));
    assert!(outcome.captured.contains("EADDRINUSE"));
    assert!(!outcome.truncated);
}

#[test]
fn keeps_the_start_and_end_of_large_logs() {
    let command = if cfg!(windows) {
        vec![
            "powershell".to_owned(),
            "-NoProfile".to_owned(),
            "-Command".to_owned(),
            "'START'; 'x' * 5000; 'END'; exit 9".to_owned(),
        ]
    } else {
        vec![
            "sh".to_owned(),
            "-c".to_owned(),
            "printf START; head -c 5000 /dev/zero | tr '\\0' x; printf END; exit 9".to_owned(),
        ]
    };

    let outcome = runner::run(&command, 1024).unwrap();
    assert!(outcome.truncated);
    assert!(outcome.captured.contains("START"));
    assert!(outcome.captured.contains("END"));
    assert!(outcome.captured.contains("middle of log omitted"));
}

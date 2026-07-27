use std::env;
use std::sync::LazyLock;

use regex::Regex;

static CREDENTIAL_IN_URL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(https?://)[^:/\s]+:[^@\s]+@").expect("valid URL regex"));
static BEARER_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(bearer\s+)[A-Za-z0-9._~+/=-]{12,}").expect("valid bearer regex")
});
static NAMED_SECRET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)((?:token|password|passwd|secret|api[_-]?key)\s*[:=]\s*)[^\s,;]{6,}")
        .expect("valid named secret regex")
});
static KNOWN_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(?:gh[pousr]_[A-Za-z0-9_]{16,}|github_pat_[A-Za-z0-9_]{16,}|sk-[A-Za-z0-9_-]{16,}|AKIA[A-Z0-9]{16})\b",
    )
    .expect("valid token regex")
});

pub fn redact(input: &str) -> String {
    let home = env::var("USERPROFILE")
        .ok()
        .or_else(|| env::var("HOME").ok());
    redact_with_home(input, home.as_deref())
}

pub fn redact_with_home(input: &str, home: Option<&str>) -> String {
    let mut output = input.to_owned();
    if let Some(home) = home.filter(|value| !value.is_empty()) {
        output = output.replace(home, "~");
    }
    output = CREDENTIAL_IN_URL
        .replace_all(&output, "$1[REDACTED]@")
        .into_owned();
    output = BEARER_TOKEN
        .replace_all(&output, "$1[REDACTED]")
        .into_owned();
    output = NAMED_SECRET
        .replace_all(&output, "$1[REDACTED]")
        .into_owned();
    KNOWN_TOKEN.replace_all(&output, "[REDACTED]").into_owned()
}

use exitsplain::redact::redact_with_home;
use exitsplain::{Catalog, Engine};

#[test]
fn matches_a_specific_npm_failure() {
    let catalog = Catalog::builtin().unwrap();
    let diagnostics = Engine::new(&catalog)
        .analyze("npm ERR! code ERESOLVE\nnpm ERR! ERESOLVE unable to resolve dependency tree");

    assert_eq!(diagnostics[0].id, "npm-peer-dependency-conflict");
}

#[test]
fn does_not_guess_when_no_rule_matches() {
    let catalog = Catalog::builtin().unwrap();
    let diagnostics = Engine::new(&catalog).analyze("a completely unknown failure shape");

    assert!(diagnostics.is_empty());
}

#[test]
fn redacts_common_secrets_and_home_paths() {
    let input =
        r"C:\Users\demo\work token=super-secret-value Authorization: Bearer abcdefghijklmnop";
    let redacted = redact_with_home(input, Some(r"C:\Users\demo"));

    assert!(redacted.starts_with(r"~\work"));
    assert!(!redacted.contains("super-secret-value"));
    assert!(!redacted.contains("abcdefghijklmnop"));
    assert_eq!(redacted.matches("[REDACTED]").count(), 2);
}

#[test]
fn evidence_is_deduplicated_and_redacted() {
    let catalog = Catalog::builtin().unwrap();
    let diagnostics = Engine::new(&catalog)
        .analyze("Cannot find module 'left-pad' token=super-secret-value\nCannot find module 'left-pad' token=super-secret-value");

    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.id == "node-module-not-found")
        .unwrap();
    assert_eq!(diagnostic.evidence.len(), 1);
    assert!(diagnostic.evidence[0].contains("[REDACTED]"));
}

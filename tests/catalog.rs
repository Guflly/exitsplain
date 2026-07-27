use std::collections::HashSet;

use exitsplain::Catalog;

#[test]
fn builtin_catalog_is_valid_and_unique() {
    let catalog = Catalog::builtin().expect("built-in catalog should load");
    assert_eq!(catalog.len(), 28);

    let mut ids = HashSet::new();
    for rule in catalog.rules() {
        assert!(ids.insert(&rule.id), "duplicate rule id: {}", rule.id);
        assert!(!rule.actions.is_empty(), "{} has no actions", rule.id);
        assert!(
            rule.docs.iter().all(|url| url.starts_with("https://")),
            "{} has a non-HTTPS documentation link",
            rule.id
        );
    }
}

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use regex::{Regex, RegexBuilder};

use crate::model::{Rule, RulePack};

const PACKS: &[(&str, &str)] = &[
    ("universal.json", include_str!("../rules/universal.json")),
    ("node.json", include_str!("../rules/node.json")),
    ("python.json", include_str!("../rules/python.json")),
    ("git.json", include_str!("../rules/git.json")),
    ("containers.json", include_str!("../rules/containers.json")),
    ("toolchains.json", include_str!("../rules/toolchains.json")),
];

#[derive(Debug)]
pub struct CatalogError(String);

impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for CatalogError {}

#[derive(Debug)]
pub(crate) struct CompiledRule {
    pub rule: Rule,
    pub all: Vec<Regex>,
    pub any: Vec<Regex>,
    pub none: Vec<Regex>,
}

#[derive(Debug)]
pub struct Catalog {
    pub(crate) rules: Vec<CompiledRule>,
}

impl Catalog {
    pub fn builtin() -> Result<Self, CatalogError> {
        let mut compiled = Vec::new();
        let mut ids = HashSet::new();

        for (file, source) in PACKS {
            let pack: RulePack = serde_json::from_str(source)
                .map_err(|error| CatalogError(format!("{file}: invalid JSON: {error}")))?;

            if pack.rules.is_empty() {
                return Err(CatalogError(format!("{file}: rule pack is empty")));
            }

            for rule in pack.rules {
                validate_rule(file, &rule, &mut ids)?;
                compiled.push(compile_rule(file, rule)?);
            }
        }

        compiled.sort_by(|left, right| left.rule.id.cmp(&right.rule.id));
        Ok(Self { rules: compiled })
    }

    pub fn rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter().map(|compiled| &compiled.rule)
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

fn validate_rule(file: &str, rule: &Rule, ids: &mut HashSet<String>) -> Result<(), CatalogError> {
    if rule.id.is_empty()
        || !rule.id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        return Err(CatalogError(format!(
            "{file}: rule id {:?} must use lowercase kebab-case",
            rule.id
        )));
    }
    if !ids.insert(rule.id.clone()) {
        return Err(CatalogError(format!(
            "{file}: duplicate rule id {}",
            rule.id
        )));
    }
    if !(1..=100).contains(&rule.confidence) {
        return Err(CatalogError(format!(
            "{file}: {} confidence must be between 1 and 100",
            rule.id
        )));
    }
    if rule.matchers.all.is_empty() && rule.matchers.any.is_empty() {
        return Err(CatalogError(format!(
            "{file}: {} needs an all or any matcher",
            rule.id
        )));
    }
    if rule.actions.is_empty() {
        return Err(CatalogError(format!(
            "{file}: {} needs at least one action",
            rule.id
        )));
    }
    if let Some(url) = rule.docs.iter().find(|url| !url.starts_with("https://")) {
        return Err(CatalogError(format!(
            "{file}: {} documentation URL must use HTTPS: {url}",
            rule.id
        )));
    }
    Ok(())
}

fn compile_rule(file: &str, rule: Rule) -> Result<CompiledRule, CatalogError> {
    Ok(CompiledRule {
        all: compile_patterns(file, &rule.id, &rule.matchers.all)?,
        any: compile_patterns(file, &rule.id, &rule.matchers.any)?,
        none: compile_patterns(file, &rule.id, &rule.matchers.none)?,
        rule,
    })
}

fn compile_patterns(file: &str, id: &str, patterns: &[String]) -> Result<Vec<Regex>, CatalogError> {
    patterns
        .iter()
        .map(|pattern| {
            RegexBuilder::new(pattern)
                .case_insensitive(true)
                .multi_line(true)
                .build()
                .map_err(|error| {
                    CatalogError(format!(
                        "{file}: {id} has invalid regex {pattern:?}: {error}"
                    ))
                })
        })
        .collect()
}

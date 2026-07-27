use std::collections::HashSet;

use regex::Regex;

use crate::catalog::{Catalog, CompiledRule};
use crate::model::Diagnostic;
use crate::redact::redact;

pub struct Engine<'catalog> {
    catalog: &'catalog Catalog,
}

impl<'catalog> Engine<'catalog> {
    pub fn new(catalog: &'catalog Catalog) -> Self {
        Self { catalog }
    }

    pub fn analyze(&self, log: &str) -> Vec<Diagnostic> {
        let mut diagnostics: Vec<_> = self
            .catalog
            .rules
            .iter()
            .filter(|rule| matches(rule, log))
            .map(|compiled| Diagnostic {
                id: compiled.rule.id.clone(),
                title: compiled.rule.title.clone(),
                summary: compiled.rule.summary.clone(),
                confidence: compiled.rule.confidence,
                causes: compiled.rule.causes.clone(),
                actions: compiled.rule.actions.clone(),
                docs: compiled.rule.docs.clone(),
                tags: compiled.rule.tags.clone(),
                evidence: evidence(compiled, log),
            })
            .collect();

        diagnostics.sort_by(|left, right| {
            right
                .confidence
                .cmp(&left.confidence)
                .then_with(|| left.id.cmp(&right.id))
        });
        diagnostics
    }
}

fn matches(rule: &CompiledRule, log: &str) -> bool {
    rule.all.iter().all(|pattern| pattern.is_match(log))
        && (rule.any.is_empty() || rule.any.iter().any(|pattern| pattern.is_match(log)))
        && rule.none.iter().all(|pattern| !pattern.is_match(log))
}

fn evidence(rule: &CompiledRule, log: &str) -> Vec<String> {
    let patterns: Vec<&Regex> = rule.all.iter().chain(rule.any.iter()).collect();
    let mut seen = HashSet::new();
    let mut lines = Vec::new();

    for line in log.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !patterns.iter().any(|pattern| pattern.is_match(trimmed)) {
            continue;
        }

        let safe = truncate(&redact(trimmed), 240);
        if seen.insert(safe.clone()) {
            lines.push(safe);
        }
        if lines.len() == 3 {
            break;
        }
    }

    if lines.is_empty()
        && let Some(last) = log.lines().rev().find(|line| !line.trim().is_empty())
    {
        lines.push(truncate(&redact(last.trim()), 240));
    }

    lines
}

fn truncate(input: &str, limit: usize) -> String {
    let mut output: String = input.chars().take(limit).collect();
    if input.chars().count() > limit {
        output.push('…');
    }
    output
}

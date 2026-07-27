use std::io::{self, Write};

use crate::model::{Diagnostic, Rule};

pub fn human(
    diagnostics: &[Diagnostic],
    mut writer: impl Write,
    color: bool,
    truncated: bool,
) -> io::Result<()> {
    if diagnostics.is_empty() {
        writeln!(
            writer,
            "\n{}",
            heading("exitsplain: no matching explanation yet", color)
        )?;
        writeln!(
            writer,
            "Add a synthetic fixture at https://github.com/Guflly/exitsplain/issues/new"
        )?;
        return Ok(());
    }

    writeln!(
        writer,
        "\n{}",
        heading(
            &format!(
                "exitsplain: {} likely explanation{}",
                diagnostics.len(),
                if diagnostics.len() == 1 { "" } else { "s" }
            ),
            color
        )
    )?;
    if truncated {
        writeln!(
            writer,
            "The middle of the captured log was omitted before analysis."
        )?;
    }

    for (index, diagnostic) in diagnostics.iter().enumerate() {
        writeln!(
            writer,
            "\n{}. {} [{}%]",
            index + 1,
            bold(&diagnostic.title, color),
            diagnostic.confidence
        )?;
        writeln!(writer, "   {}", diagnostic.summary)?;

        if !diagnostic.evidence.is_empty() {
            writeln!(writer, "   Seen:")?;
            for line in &diagnostic.evidence {
                writeln!(writer, "   > {line}")?;
            }
        }
        if !diagnostic.causes.is_empty() {
            writeln!(writer, "   Check:")?;
            for cause in &diagnostic.causes {
                writeln!(writer, "   - {cause}")?;
            }
        }
        writeln!(writer, "   Try:")?;
        for action in &diagnostic.actions {
            writeln!(writer, "   - {action}")?;
        }
        for url in &diagnostic.docs {
            writeln!(writer, "   {url}")?;
        }
    }

    Ok(())
}

pub fn json(diagnostics: &[Diagnostic], mut writer: impl Write) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut writer, diagnostics).map_err(io::Error::other)?;
    writeln!(writer)
}

pub fn rule_list<'rule>(
    rules: impl Iterator<Item = &'rule Rule>,
    mut writer: impl Write,
) -> io::Result<()> {
    for rule in rules {
        writeln!(
            writer,
            "{}\t{}\t{}",
            rule.id,
            rule.confidence,
            rule.tags.join(",")
        )?;
    }
    Ok(())
}

fn heading(input: &str, color: bool) -> String {
    if color {
        format!("\x1b[1;36m{input}\x1b[0m")
    } else {
        input.to_owned()
    }
}

fn bold(input: &str, color: bool) -> String {
    if color {
        format!("\x1b[1m{input}\x1b[0m")
    } else {
        input.to_owned()
    }
}

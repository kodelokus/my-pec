//! The dream diary, `DREAMS.md`: one entry per sweep with material, in the companion's voice, followed by an exact
//! list of what the dream kept. Written for the person; never a memory source (dreaming R11, DRM-5).

use super::run::Theme;

pub(crate) const INSTRUCTIONS: &str = include_str!("prompts/diary.md");
const HEADING: &str = "# Dream diary";

/// The diary request's one user message.
pub(crate) fn input(themes: &[Theme], gathered: usize, kept: &[String]) -> String {
    let mut out = format!("Tonight you went over {gathered} notes.\n");
    if !themes.is_empty() {
        out.push_str("\nThemes:\n");
        for t in themes {
            out.push_str(&format!("- {}: {}\n", t.title, t.summary));
        }
    }
    out.push_str("\nWhat you kept in your long-term memory:\n");
    if kept.is_empty() {
        out.push_str("- nothing new\n");
    }
    for k in kept {
        out.push_str(&format!("- {k}\n"));
    }
    out
}

/// One entry: the night, the narrative (or a plain one, marked, when the model was unavailable), and the list of
/// what the dream kept.
pub(crate) fn entry(
    night: &str,
    narrative: Option<&str>,
    gathered: usize,
    kept: &[String],
) -> String {
    let narrative = narrative.map(str::trim).filter(|n| !n.is_empty());
    let mut out = match narrative {
        Some(text) => format!("## {night}\n{text}\n"),
        None => format!(
            "## {night} *(written without the model)*\nI dreamt about {gathered} notes and kept {} of them.\n",
            kept.len()
        ),
    };
    if !kept.is_empty() {
        out.push_str("\nKept:\n");
        for k in kept {
            out.push_str(&format!("- {k}\n"));
        }
    }
    out
}

/// The diary with an entry appended after whatever is there.
pub(crate) fn append(current: &str, entry: &str) -> String {
    let mut out = if current.trim().is_empty() {
        format!("{HEADING}\n")
    } else {
        current.trim_end().to_owned() + "\n"
    };
    out.push('\n');
    out.push_str(entry);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_appended_with_what_was_kept() {
        let kept = ["Training for the half marathon in April.".to_owned()];
        let first = append(
            "",
            &entry("2026-11-15", Some(" Running kept coming up. "), 12, &kept),
        );
        assert_eq!(
            first,
            "# Dream diary\n\n## 2026-11-15\nRunning kept coming up.\n\nKept:\n- Training for the half marathon in April.\n"
        );
        // A degraded entry, with nothing kept, after the person's own edits.
        let edited = format!("{first}\nMy own note about the diary.");
        assert_eq!(
            append(&edited, &entry("2026-11-16", None, 3, &[])),
            format!(
                "{first}\nMy own note about the diary.\n\n## 2026-11-16 *(written without the model)*\n\
                 I dreamt about 3 notes and kept 0 of them.\n"
            )
        );
        assert!(entry("2026-11-17", Some("  "), 1, &[]).contains("without the model"));
    }
}

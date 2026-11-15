//! The text side of search: preparing text for the index, chunks, query terms, ranking and auto-recall
//! (memory R10, R11). Pure functions; the database side is in `index.rs`.

use super::files::day_number;

/// A chunk is at most this many characters (memory R11).
const CHUNK_CHARS: usize = 600;
/// Daily-note hits lose half their weight every this many days.
const HALF_LIFE_DAYS: f64 = 30.0;
/// Auto-recall attaches at most this many snippets and characters (architecture section 15).
pub(crate) const RECALL_HITS: usize = 3;
const RECALL_CHARS: usize = 2_000;

/// Scripts written without spaces between words (memory R10).
fn spaceless(c: char) -> bool {
    matches!(c as u32,
        0x0E00..=0x0EFF      // Thai, Lao
        | 0x1000..=0x109F    // Myanmar
        | 0x1780..=0x17FF    // Khmer
        | 0x3040..=0x30FF    // Hiragana, Katakana
        | 0x3400..=0x4DBF    // Han extension A
        | 0x4E00..=0x9FFF    // Han
        | 0xAC00..=0xD7AF    // Hangul
        | 0xF900..=0xFAFF    // Han compatibility
        | 0x20000..=0x2FFFF) // Han extensions B and later
}

/// Rewrites each run of a spaceless script as overlapping pairs (`天気予報` → `天気 気予 予報`), so the
/// `unicode61` tokenizer can match words inside it. Other text is unchanged.
pub(crate) fn prepare(text: &str) -> String {
    fn flush(run: &mut Vec<char>, out: &mut String) {
        if run.len() == 1 {
            out.extend([' ', run[0], ' ']);
        } else {
            for pair in run.windows(2) {
                out.extend([' ', pair[0], pair[1], ' ']);
            }
        }
        run.clear();
    }
    let (mut out, mut run) = (String::with_capacity(text.len() * 2), Vec::new());
    for c in text.chars() {
        if spaceless(c) {
            run.push(c);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
    out
}

/// A query's distinct terms, lowercase, in the index's own token shape. A lone character of a spaceless script
/// is a prefix, since it only ever appears inside a pair.
pub(crate) fn terms(query: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for term in prepare(query)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
    {
        let term = term.to_lowercase();
        if !out.contains(&term) {
            out.push(term);
        }
    }
    out
}

/// The FTS5 query that matches any of `terms`.
pub(crate) fn fts_query(terms: &[String]) -> String {
    terms
        .iter()
        .map(|t| {
            let prefix = t.chars().count() == 1 && t.chars().all(spaceless);
            format!(
                "\"{}\"{}",
                t.replace('"', "\"\""),
                if prefix { "*" } else { "" }
            )
        })
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// A passage of a memory file, with its 1-based line range.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Chunk {
    pub first: usize,
    pub last: usize,
    pub text: String,
}

/// Removes the trailing `<!-- … -->` bookkeeping from a line, so ids and origins aren't searched or shown.
pub(crate) fn without_comment(line: &str) -> &str {
    match line.find("<!--") {
        Some(at) if line.trim_end().ends_with("-->") => line[..at].trim_end(),
        _ => line,
    }
}

/// Splits Markdown at headings, blank lines and bullets into chunks of at most 600 characters. In long-term
/// memory each chunk starts with its topic heading (memory R11).
pub(crate) fn chunks(text: &str, with_topic: bool) -> Vec<Chunk> {
    let mut out = Vec::new();
    let mut topic = String::new();
    let mut current: Option<Chunk> = None;
    fn flush(current: &mut Option<Chunk>, out: &mut Vec<Chunk>) {
        if let Some(chunk) = current.take() {
            out.push(chunk);
        }
    }
    for (i, raw) in text.lines().enumerate() {
        let number = i + 1;
        let line = without_comment(raw).trim_end();
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            flush(&mut current, &mut out);
            if let Some(heading) = trimmed.strip_prefix("## ") {
                topic = heading.trim().to_owned();
            }
            continue;
        }
        if trimmed.is_empty() {
            flush(&mut current, &mut out);
            continue;
        }
        let bullet = trimmed.starts_with("- ") || trimmed.starts_with("* ");
        let length = current.as_ref().map_or(0, |c| c.text.chars().count());
        if bullet || length + trimmed.chars().count() + 1 > CHUNK_CHARS {
            flush(&mut current, &mut out);
        }
        let chars: Vec<char> = trimmed.chars().collect();
        for piece in chars.chunks(CHUNK_CHARS) {
            let piece: String = piece.iter().collect();
            match &mut current {
                Some(chunk) if chunk.text.chars().count() + piece.chars().count() < CHUNK_CHARS => {
                    chunk.text.push('\n');
                    chunk.text.push_str(&piece);
                    chunk.last = number;
                }
                _ => {
                    flush(&mut current, &mut out);
                    let head = if with_topic && !topic.is_empty() {
                        format!("{topic}\n")
                    } else {
                        String::new()
                    };
                    current = Some(Chunk {
                        first: number,
                        last: number,
                        text: format!("{head}{piece}"),
                    });
                }
            }
        }
    }
    flush(&mut current, &mut out);
    out
}

/// A search result.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Hit {
    pub path: String,
    pub day: Option<String>,
    pub first: usize,
    pub last: usize,
    pub text: String,
    /// How many distinct query terms the chunk contains.
    pub matched: usize,
    /// Semantic search: how close it is in meaning, 0 to 1 above the floor; 0 without the model.
    pub meaning: f64,
    pub score: f64,
}

impl Hit {
    /// search R4: the larger of the share of the query's terms it holds and its meaning score, the
    /// 0-to-1 relevance recall evidence keeps.
    pub fn relevance(&self, terms: usize) -> f64 {
        let words = if terms == 0 {
            0.0
        } else {
            (self.matched as f64 / terms as f64).min(1.0)
        };
        words.max(self.meaning)
    }

    /// memory FR-019, search R5: worth recalling when it holds two of the query's terms (or its only one), or
    /// is close enough in meaning.
    pub fn relevant(&self, terms: usize) -> bool {
        (terms > 0 && self.matched >= terms.min(2)) || self.meaning >= RECALL_MEANING
    }
}

/// search R4: cosine similarity at or below the floor is no match in meaning, and at the ceiling a
/// full one; measured with multilingual-e5-small, where unrelated notes score 0.74 to 0.78 and related ones 0.80 to
/// 0.85.
const MEANING_FLOOR: f64 = 0.78;
const MEANING_CEILING: f64 = 0.86;
const KEYWORD_WEIGHT: f64 = 0.35;
const MEANING_WEIGHT: f64 = 0.65;
/// search R5: a hit this close in meaning is recalled without sharing words.
const RECALL_MEANING: f64 = 0.4;

/// A cosine similarity as a meaning score: 0 at the floor and below, 1 at the ceiling and above.
pub(crate) fn meaning_score(cosine: f64) -> f64 {
    ((cosine - MEANING_FLOOR) / (MEANING_CEILING - MEANING_FLOOR)).clamp(0.0, 1.0)
}

/// The blended score from a keyword score (the bm25 share of the query's best, times the share of the query's terms
/// the hit holds, so one common word counts little) and a meaning score.
pub(crate) fn blend(keyword: f64, meaning: f64) -> f64 {
    KEYWORD_WEIGHT * keyword + MEANING_WEIGHT * meaning
}

/// Weighs hits by relevance and, for daily notes, by age; keeps the best of near-duplicates within five lines.
pub(crate) fn rank(mut hits: Vec<Hit>, today: Option<&str>, limit: usize) -> Vec<Hit> {
    if let Some(today) = today.and_then(day_number) {
        for hit in &mut hits {
            if let Some(day) = hit.day.as_deref().and_then(day_number) {
                let age = (today - day).max(0) as f64;
                hit.score *= 0.5_f64.powf(age / HALF_LIFE_DAYS);
            }
        }
    }
    hits.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<Hit> = Vec::new();
    for hit in hits {
        let near = kept
            .iter()
            .any(|k| k.path == hit.path && k.first.abs_diff(hit.first) <= 5);
        if !near {
            kept.push(hit);
        }
        if kept.len() == limit {
            break;
        }
    }
    kept
}

/// What a context already holds, so recall doesn't repeat it: the days of its notes opener, and how many lines
/// of long-term memory its budget let through.
pub(crate) struct Loaded {
    pub days: Vec<String>,
    pub memory_lines: usize,
}

/// memory FR-019: the hits worth attaching to a message. A hit must be relevant (two of the query's terms, or its only
/// term, or close in meaning), must not be in the context already, and the chosen ones fit the recall budget.
pub(crate) fn select_recall(hits: Vec<Hit>, terms: usize, loaded: &Loaded) -> Vec<Hit> {
    let (mut out, mut chars) = (Vec::new(), 0);
    for hit in hits {
        let loaded_already = match &hit.day {
            Some(day) => loaded.days.contains(day),
            None => hit.last <= loaded.memory_lines,
        };
        if !hit.relevant(terms) || loaded_already {
            continue;
        }
        let length = hit.text.chars().count();
        if chars + length > RECALL_CHARS || out.len() == RECALL_HITS {
            break;
        }
        chars += length;
        out.push(hit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spaceless_scripts_become_pairs() {
        assert_eq!(prepare("天気予報"), " 天気  気予  予報 ");
        assert_eq!(prepare("今日は天気"), " 今日  日は  は天  天気 ");
        assert_eq!(prepare("Bimo と散歩"), "Bimo  と散  散歩 ");
        assert_eq!(prepare("rapat besok"), "rapat besok");
        assert_eq!(terms("天気"), ["天気"]);
        assert_eq!(terms("Rapat, RAPAT besok!"), ["rapat", "besok"]);
        assert_eq!(fts_query(&terms("天")), "\"天\"*");
        assert_eq!(fts_query(&terms("tea time")), "\"tea\" OR \"time\"");
    }

    #[test]
    fn chunks_follow_the_markdown_and_keep_their_lines() {
        let text = "# Memory\n\n## Health\n- Allergic to peanuts. <!-- id:m_1 at:2026-11-14 from:user src:1-0 -->\n\
                    - Takes vitamin D.\n\nA paragraph\nthat continues.\n";
        assert_eq!(
            chunks(text, true),
            [
                Chunk {
                    first: 4,
                    last: 4,
                    text: "Health\n- Allergic to peanuts.".into()
                },
                Chunk {
                    first: 5,
                    last: 5,
                    text: "Health\n- Takes vitamin D.".into()
                },
                Chunk {
                    first: 7,
                    last: 8,
                    text: "Health\nA paragraph\nthat continues.".into()
                },
            ]
        );
        assert_eq!(
            chunks("# 2026-11-14\n\n- 09:00 Note.\n", false)[0].text,
            "- 09:00 Note."
        );
        let long = "x".repeat(1_500);
        let pieces = chunks(&long, false);
        assert_eq!(pieces.len(), 3);
        assert!(
            pieces
                .iter()
                .all(|c| c.text.chars().count() <= CHUNK_CHARS && c.first == 1)
        );
    }

    fn hit(path: &str, day: Option<&str>, first: usize, matched: usize, score: f64) -> Hit {
        Hit {
            path: path.into(),
            day: day.map(Into::into),
            first,
            last: first,
            text: format!("text {first}"),
            matched,
            meaning: 0.0,
            score,
        }
    }

    #[test]
    fn older_notes_weigh_less_and_near_duplicates_are_thinned() {
        let hits = vec![
            hit("memory/2026-10-14.md", Some("2026-10-14"), 1, 1, 1.0),
            hit("memory/2026-11-14.md", Some("2026-11-14"), 1, 1, 0.6),
            hit("memory/2026-11-14.md", Some("2026-11-14"), 4, 1, 0.55),
            hit("MEMORY.md", None, 9, 1, 0.4),
        ];
        let ranked = rank(hits, Some("2026-11-13"), 10);
        let order: Vec<_> = ranked.iter().map(|h| (h.path.as_str(), h.first)).collect();
        // The month-old note halved to 0.5; the two lines four apart are one hit.
        assert_eq!(
            order,
            [
                ("memory/2026-11-14.md", 1),
                ("memory/2026-10-14.md", 1),
                ("MEMORY.md", 9)
            ]
        );
    }

    #[test]
    fn recall_needs_two_terms_skips_what_is_loaded_and_fits_the_budget() {
        let loaded = Loaded {
            days: vec!["2026-11-14".into()],
            memory_lines: 10,
        };
        let hits = vec![
            hit("memory/2026-11-01.md", Some("2026-11-01"), 1, 1, 9.0),
            hit("memory/2026-11-14.md", Some("2026-11-14"), 1, 2, 8.0),
            hit("MEMORY.md", None, 5, 2, 7.0),
            hit("MEMORY.md", None, 30, 2, 6.0),
            hit("memory/2026-10-01.md", Some("2026-10-01"), 1, 3, 5.0),
            hit("memory/2026-09-01.md", Some("2026-09-01"), 1, 2, 4.0),
            hit("memory/2026-08-01.md", Some("2026-08-01"), 1, 2, 3.0),
        ];
        let chosen: Vec<_> = select_recall(hits.clone(), 3, &loaded)
            .into_iter()
            .map(|h| h.path)
            .collect();
        assert_eq!(
            chosen,
            ["MEMORY.md", "memory/2026-10-01.md", "memory/2026-09-01.md"]
        );
        // A one-word query needs only that word.
        assert_eq!(select_recall(hits[..1].to_vec(), 1, &loaded).len(), 1);
        assert!(select_recall(Vec::new(), 2, &loaded).is_empty());
        let big = Hit {
            text: "y".repeat(RECALL_CHARS + 1),
            ..hits[5].clone()
        };
        assert!(select_recall(vec![big], 2, &loaded).is_empty());
    }

    #[test]
    fn meaning_and_words_blend_into_one_relevance() {
        // search R4: no meaning at the floor or below, a full one at the ceiling.
        assert_eq!(meaning_score(0.70), 0.0);
        assert_eq!(meaning_score(MEANING_FLOOR), 0.0);
        assert!((meaning_score(0.82) - 0.5).abs() < 1e-9);
        assert_eq!(meaning_score(0.95), 1.0);
        assert!((blend(1.0, 0.0) - KEYWORD_WEIGHT).abs() < 1e-12);
        assert!((blend(0.0, 1.0) - MEANING_WEIGHT).abs() < 1e-12);
        // A hit matching the words is above one that matches neither, whatever the blend.
        assert!(blend(0.01, 0.0) > blend(0.0, 0.0));
        // Relevance is the larger of the words' share and the meaning (search FR-007); recall takes either.
        let mut h = hit("memory/2026-11-01.md", Some("2026-11-01"), 1, 1, 1.0);
        assert_eq!(h.relevance(4), 0.25);
        assert!(!h.relevant(4));
        h.meaning = RECALL_MEANING;
        assert_eq!(h.relevance(4), RECALL_MEANING);
        assert!(h.relevant(4));
        h.matched = 2;
        h.meaning = 0.0;
        assert!(h.relevant(4) && h.relevant(1));
        // A query of only common words recalls by meaning alone.
        h.matched = 0;
        assert!(!h.relevant(0));
        h.meaning = 0.9;
        assert!(h.relevant(0));
    }
}

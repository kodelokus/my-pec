//! The mood marker at the start of each model call's text: read while streaming, removed from everything the
//! person sees, kept in the transcript (avatar interfaces, avatar R2).

use crate::avatar::Mood;

const OPEN: &str = "<mood>";
const CLOSE: &str = "</mood>";
/// At most this many characters come before a marker's closing `>`; held text longer than this is released.
const HOLD: usize = 32;

enum Scan {
    /// A whole marker, its byte length and its mood.
    Marker(usize, Mood),
    /// Could still become a marker.
    Prefix,
    No,
}

fn scan(s: &str) -> Scan {
    let bytes = s.as_bytes();
    let n = bytes.len().min(OPEN.len());
    if bytes[..n] != OPEN.as_bytes()[..n] {
        return Scan::No;
    }
    if bytes.len() < OPEN.len() {
        return Scan::Prefix;
    }
    let rest = &s[OPEN.len()..];
    let (inner, tail) = rest.split_at(rest.find('<').unwrap_or(rest.len()));
    // Spaces, then letters, then spaces.
    let word = inner.trim_matches(' ');
    if word.len() > 20 || !word.bytes().all(|b| b.is_ascii_alphabetic()) {
        return Scan::No;
    }
    if tail.len() < CLOSE.len() {
        return if CLOSE.starts_with(tail) && s.chars().count() <= HOLD {
            Scan::Prefix
        } else {
            Scan::No
        };
    }
    let len = OPEN.len() + inner.len() + CLOSE.len();
    if !tail.starts_with(CLOSE) || word.is_empty() || len > HOLD + 1 {
        return Scan::No;
    }
    Scan::Marker(len, Mood::named(word))
}

/// The length of the line break right after a marker: one `\n` or `\r\n`, or `None` while it can't be told yet.
fn line_break(after: &str, ended: bool) -> Option<usize> {
    match after.as_bytes() {
        [] | [b'\r'] if !ended => None,
        [b'\r', b'\n', ..] => Some(2),
        [b'\r', ..] | [b'\n', ..] => Some(1),
        _ => Some(0),
    }
}

/// One model call's text: holds back its start while it could be a marker.
#[derive(Default)]
pub(crate) struct MoodParser {
    held: String,
    done: bool,
}

impl MoodParser {
    /// The text to show now, and the mood when a marker was just read.
    pub(crate) fn feed(&mut self, delta: &str) -> (String, Option<Mood>) {
        if self.done {
            return (delta.into(), None);
        }
        self.held.push_str(delta);
        self.settle(false)
    }

    /// The call ended: a marker still waiting for its line break is read, anything else held is ordinary text.
    pub(crate) fn finish(&mut self) -> (String, Option<Mood>) {
        if self.done {
            return (String::new(), None);
        }
        self.settle(true)
    }

    fn settle(&mut self, ended: bool) -> (String, Option<Mood>) {
        match scan(&self.held) {
            Scan::Prefix if !ended => (String::new(), None),
            Scan::Marker(len, mood) => match line_break(&self.held[len..], ended) {
                None => (String::new(), None),
                Some(br) => {
                    self.done = true;
                    (self.held.split_off(len + br), Some(mood))
                }
            },
            _ => {
                self.done = true;
                (std::mem::take(&mut self.held), None)
            }
        }
    }
}

/// Stored text without its leading marker, as shown to the person.
pub(crate) fn strip(text: &str) -> &str {
    match scan(text) {
        Scan::Marker(len, _) => {
            let br = line_break(&text[len..], true).unwrap_or(0);
            &text[len + br..]
        }
        _ => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn whole(text: &str) -> (String, Option<Mood>) {
        let mut p = MoodParser::default();
        let (mut out, first) = p.feed(text);
        let (rest, last) = p.finish();
        out.push_str(&rest);
        (out, first.or(last))
    }

    #[test]
    fn markers_follow_the_contract_examples() {
        let cases = [
            (
                "<mood>happy</mood>\nThat's wonderful news!",
                "That's wonderful news!",
                Some(Mood::Happy),
            ),
            ("That's wonderful news!", "That's wonderful news!", None),
            ("<mood>ecstatic</mood>\nYay!", "Yay!", Some(Mood::Neutral)),
            ("<mood happy>\nYay!", "<mood happy>\nYay!", None),
            (
                "I feel <mood>sad</mood> about it.",
                "I feel <mood>sad</mood> about it.",
                None,
            ),
            (
                "<mood> Concerned </mood>\r\nHm.",
                "Hm.",
                Some(Mood::Concerned),
            ),
            ("<mood>SURPRISED</mood>Oh!", "Oh!", Some(Mood::Surprised)),
            (
                "<mood>playful</mood>\n\nTwo breaks.",
                "\nTwo breaks.",
                Some(Mood::Playful),
            ),
            ("<mood></mood>\nEmpty.", "<mood></mood>\nEmpty.", None),
            ("<mood>ha ppy</mood>\nNo.", "<mood>ha ppy</mood>\nNo.", None),
            ("<mood>\nsad</mood>\nNo.", "<mood>\nsad</mood>\nNo.", None),
            (
                " <mood>sad</mood>\nSpace first.",
                " <mood>sad</mood>\nSpace first.",
                None,
            ),
            ("<mood>sad</mood>", "", Some(Mood::Sad)),
            ("<mood>sad", "<mood>sad", None),
        ];
        for (text, visible, mood) in cases {
            assert_eq!(whole(text), (visible.to_owned(), mood), "{text:?}");
            assert_eq!(strip(text), visible, "strip {text:?}");
        }
    }

    #[test]
    fn a_marker_split_anywhere_is_read_the_same() {
        let text = "<mood>sad</mood>\r\nOh no.";
        for cut in 1..text.len() {
            let mut p = MoodParser::default();
            let (mut out, first) = p.feed(&text[..cut]);
            let (rest, second) = p.feed(&text[cut..]);
            let (end, third) = p.finish();
            out.push_str(&rest);
            out.push_str(&end);
            assert_eq!(out, "Oh no.", "cut at {cut}");
            assert_eq!(first.or(second).or(third), Some(Mood::Sad), "cut at {cut}");
        }
    }

    #[test]
    fn held_text_is_released_when_it_cannot_be_a_marker() {
        let mut p = MoodParser::default();
        assert_eq!(p.feed("<mo"), (String::new(), None));
        assert_eq!(
            p.feed("re than that"),
            ("<more than that".into(), None),
            "diverged"
        );
        assert_eq!(
            p.feed(" and on"),
            (" and on".into(), None),
            "nothing held after"
        );

        let long = format!("<mood>{}", " ".repeat(30));
        let mut p = MoodParser::default();
        assert_eq!(p.feed(&long[..32]), (String::new(), None));
        assert_eq!(
            p.feed(&long[32..]),
            (long.clone(), None),
            "past 32 characters"
        );

        let mut p = MoodParser::default();
        assert_eq!(p.feed("<mood>ha"), (String::new(), None));
        assert_eq!(
            p.finish(),
            ("<mood>ha".into(), None),
            "the call ended while holding back"
        );
    }
}

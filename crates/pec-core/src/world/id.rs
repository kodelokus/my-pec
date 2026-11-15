//! The companion's id and birth date (world R1, R2, world data 1).

use crate::memory::files::is_date;
use crate::onboarding::field;
use std::hash::{BuildHasher, Hasher};

/// No `0`, `1`, `I`, `L` or `O`, so the id reads aloud without look-alikes.
const ALPHABET: &[u8; 31] = b"23456789ABCDEFGHJKMNPQRSTUVWXYZ";

/// `len` random characters of the alphabet, from two OS-seeded hashers (up to about 25 characters).
pub fn random_chars(len: usize) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    let mut bits = 0u128;
    for _ in 0..2 {
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write_u64(nanos);
        bits = (bits << 64) | u128::from(hasher.finish());
    }
    (0..len)
        .map(|_| {
            let c = ALPHABET[(bits % 31) as usize] as char;
            bits /= 31;
            c
        })
        .collect()
}

/// A new random id, `K7Q2-M9TX-3HPA`: about 59 bits (world FR-004).
pub(crate) fn new_id() -> String {
    let chars = random_chars(12);
    [&chars[..4], &chars[4..8], &chars[8..]].join("-")
}

/// A well-formed id in its canonical uppercase form.
pub(crate) fn parse_id(text: &str) -> Option<String> {
    let upper = text.trim().to_ascii_uppercase();
    let groups: Vec<&str> = upper.split('-').collect();
    let valid = groups.len() == 3
        && groups
            .iter()
            .all(|g| g.len() == 4 && g.bytes().all(|b| ALPHABET.contains(&b)));
    valid.then_some(upper)
}

/// The readable `- Id:` and `- Born:` of an identity file.
pub(crate) fn identity(text: &str) -> (Option<String>, Option<String>) {
    let id = field(text, "Id").and_then(parse_id);
    let born = field(text, "Born").filter(|d| is_date(d)).map(Into::into);
    (id, born)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_id_is_three_groups_of_four_without_look_alikes() {
        let id = new_id();
        assert_eq!(id.len(), 14);
        assert_eq!(parse_id(&id).as_deref(), Some(id.as_str()));
        assert!(!id.contains(['0', '1', 'I', 'L', 'O']));
        // world FR-004: random, so two ids differ.
        assert_ne!(new_id(), new_id());
    }

    #[test]
    fn ids_are_parsed_strictly() {
        assert_eq!(
            parse_id(" k7q2-m9tx-3hpa ").as_deref(),
            Some("K7Q2-M9TX-3HPA")
        );
        for bad in [
            "K7Q2-M9TX-3HP",
            "K7Q2M9TX3HPA",
            "K7Q2-M9TX-3HP0",
            "K7Q2-M9TX-3HPL",
            "",
        ] {
            assert_eq!(parse_id(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_identity_file_gives_its_id_and_birth_date() {
        let text = "# Identity\n\n- Name: Steve\n- Avatar: pip\n- Id: K7Q2-M9TX-3HPA\n- Born: 2026-11-14\n";
        assert_eq!(
            identity(text),
            (Some("K7Q2-M9TX-3HPA".into()), Some("2026-11-14".into()))
        );
        assert_eq!(identity("- Id: nope\n- Born: someday\n"), (None, None));
    }
}

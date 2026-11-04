//! Nearest-match suggestions for typos (GH #439): "did you mean `status`?".

/// Levenshtein edit distance between `a` and `b` (case-insensitive).
pub fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.to_lowercase().chars().collect();
    let b: Vec<char> = b.to_lowercase().chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != cb);
            cur.push(sub.min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// How far a candidate may be from `word` and still count as a plausible typo.
fn tolerance(word: &str) -> usize {
    match word.chars().count() {
        0..=3 => 1,
        4..=7 => 2,
        _ => 3,
    }
}

/// Up to `limit` candidates nearest to `word`, closest first (ties keep input order).
/// A candidate qualifies when within the typo tolerance, or when one is a prefix of the other.
/// An exact (case-insensitive) match is never "near" — the caller handles that case.
pub fn nearest<'a>(word: &str, candidates: impl IntoIterator<Item = &'a str>, limit: usize) -> Vec<&'a str> {
    if word.is_empty() {
        return Vec::new();
    }
    let w = word.to_lowercase();
    let tol = tolerance(word);
    let mut scored: Vec<(usize, usize, &str)> = candidates
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let cl = c.to_lowercase();
            if cl == w {
                return None;
            }
            let d = distance(word, c);
            let prefix = w.len() >= 3 && (cl.starts_with(&w) || w.starts_with(&cl));
            (d <= tol || prefix).then_some((if prefix { d.min(1) } else { d }, i, c))
        })
        .collect();
    scored.sort();
    scored.into_iter().take(limit).map(|(_, _, c)| c).collect()
}

/// ` Did you mean 'x'?` / ` Did you mean one of 'x', 'y'?` (leading space), or empty.
pub fn did_you_mean<'a>(word: &str, candidates: impl IntoIterator<Item = &'a str>) -> String {
    let near = nearest(word, candidates, 3);
    match near.as_slice() {
        [] => String::new(),
        [one] => format!(" Did you mean '{one}'?"),
        many => format!(" Did you mean one of {}?", many.iter().map(|c| format!("'{c}'")).collect::<Vec<_>>().join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_basics() {
        assert_eq!(distance("status", "status"), 0);
        assert_eq!(distance("stauts", "status"), 2);
        assert_eq!(distance("Claim", "claim"), 0);
        assert_eq!(distance("", "abc"), 3);
    }

    #[test]
    fn a_transposed_field_name_suggests_the_right_one() {
        assert_eq!(nearest("stauts", ["status", "name", "evidence"], 3), ["status"]);
        assert_eq!(did_you_mean("stauts", ["status", "name"]), " Did you mean 'status'?");
    }

    #[test]
    fn nothing_close_suggests_nothing() {
        assert!(nearest("zzzzzz", ["status", "name"], 3).is_empty());
        assert_eq!(did_you_mean("zzzzzz", ["status"]), "");
    }

    #[test]
    fn an_exact_match_is_not_a_suggestion_and_prefixes_are() {
        assert!(nearest("status", ["status"], 3).is_empty());
        assert_eq!(nearest("valid", ["validate", "version"], 3), ["validate"]);
    }

    #[test]
    fn results_are_closest_first_and_limited() {
        let got = nearest("claim", ["claims", "clam", "release", "clime"], 2);
        assert_eq!(got.len(), 2);
        assert!(got.contains(&"claims") && got.contains(&"clam"));
    }
}

//! Shared TF-IDF text analysis over element normative text (name + body).
//!
//! Backs `topics` (REQ-TRS-SEARCH-002), `clusters` (REQ-TRS-SEARCH-003) and the
//! `summarize` "about" labels (REQ-TRS-OUT-023). Deterministic and offline
//! (ADR-SYS-SCAN-002): bag-of-words TF-IDF, no model, no network. The tokeniser is
//! the same one `search-text` uses.

use std::collections::HashMap;

use syscribe_model::element::RawElement;

/// Lowercase alphanumeric-run tokeniser (identical to `ftsearch`).
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

/// English + requirement-boilerplate stop words dropped from TF-IDF analysis so the
/// `topics`/`clusters`/`summarize` labels surface content words, not "the"/"shall".
/// (Kept out of the `search-text` tokeniser, which matches literal terms.)
const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "been", "but", "by", "can", "for", "from",
    "has", "have", "if", "in", "into", "is", "it", "its", "may", "must", "no", "not", "of",
    "on", "or", "over", "shall", "should", "so", "such", "than", "that", "the", "their",
    "then", "there", "these", "this", "to", "up", "when", "which", "will", "with", "within",
    "without", "would", "each", "any", "all", "per", "via", "e", "g", "i",
];

/// Content tokens: the tokeniser output with stop words and single-character tokens
/// removed. Used for TF-IDF analysis (not for `search-text`).
fn content_tokens(text: &str) -> Vec<String> {
    tokenize(text)
        .into_iter()
        .filter(|t| t.len() > 1 && !STOP_WORDS.contains(&t.as_str()))
        .collect()
}

/// The analysable text of an element: its display name plus its Markdown body.
pub fn element_text(e: &RawElement) -> String {
    match &e.frontmatter.name {
        Some(n) => format!("{n}\n{}", e.doc),
        None => e.doc.clone(),
    }
}

/// A TF-IDF corpus over a set of documents (each document a bag of terms).
pub struct Corpus {
    doc_tf: Vec<HashMap<String, u32>>,
    /// document frequency per term.
    df: HashMap<String, usize>,
    n: usize,
}

impl Corpus {
    /// Build the corpus from the raw text of each document.
    pub fn build(texts: &[String]) -> Corpus {
        let mut doc_tf = Vec::with_capacity(texts.len());
        let mut df: HashMap<String, usize> = HashMap::new();
        for text in texts {
            let mut tf: HashMap<String, u32> = HashMap::new();
            for t in content_tokens(text) {
                *tf.entry(t).or_insert(0) += 1;
            }
            for term in tf.keys() {
                *df.entry(term.clone()).or_insert(0) += 1;
            }
            doc_tf.push(tf);
        }
        let n = texts.len();
        Corpus { doc_tf, df, n }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    /// Smoothed inverse document frequency (sklearn-style: never zero, so a term in
    /// every document still carries a small, equal weight).
    pub fn idf(&self, term: &str) -> f64 {
        let df = *self.df.get(term).unwrap_or(&0) as f64;
        ((self.n as f64 + 1.0) / (df + 1.0)).ln() + 1.0
    }

    /// L2-normalised sparse TF-IDF vector for document `i` (so cosine = dot product).
    pub fn tfidf_vec(&self, i: usize) -> HashMap<String, f64> {
        let mut v: HashMap<String, f64> = self.doc_tf[i]
            .iter()
            .map(|(term, &tf)| (term.clone(), tf as f64 * self.idf(term)))
            .collect();
        let norm = v.values().map(|w| w * w).sum::<f64>().sqrt();
        if norm > 0.0 {
            for w in v.values_mut() {
                *w /= norm;
            }
        }
        v
    }

    /// The top-`k` terms of document `i` by TF-IDF weight (ties broken by term).
    pub fn top_terms(&self, i: usize, k: usize) -> Vec<(String, f64)> {
        let mut terms: Vec<(String, f64)> = self.doc_tf[i]
            .iter()
            .map(|(term, &tf)| (term.clone(), tf as f64 * self.idf(term)))
            .collect();
        terms.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.0.cmp(&b.0))
        });
        terms.truncate(k);
        terms
    }
}

/// Cosine similarity of two sparse vectors. When both are L2-normalised this is
/// their dot product. Iterates the smaller map.
pub fn cosine(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    let (small, big) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    small.iter().filter_map(|(t, wa)| big.get(t).map(|wb| wa * wb)).sum()
}

/// The top-`k` terms of an arbitrary summed sparse vector (ties broken by term).
pub fn top_terms_of(vec: &HashMap<String, f64>, k: usize) -> Vec<(String, f64)> {
    let mut terms: Vec<(String, f64)> = vec.iter().map(|(t, w)| (t.clone(), *w)).collect();
    terms.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.0.cmp(&b.0))
    });
    terms.truncate(k);
    terms
}

/// Add sparse vector `src` into `dst` (accumulate).
pub fn add_into(dst: &mut HashMap<String, f64>, src: &HashMap<String, f64>) {
    for (t, w) in src {
        *dst.entry(t.clone()).or_insert(0.0) += w;
    }
}

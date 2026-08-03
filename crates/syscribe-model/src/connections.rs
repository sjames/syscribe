//! Typed parse/add/remove helpers over the untyped `connections:` (and
//! `flow_connections:`/`binding_connections:`/`succession_connections:`) YAML
//! sequences — see `docs/format/frontmatter.md`.
//!
//! `connections:` intentionally has no dedicated Rust struct in `element.rs`;
//! entries are hand-authored `serde_yaml::Value` mappings in one of two shapes:
//! a binary form (`from`/`to` or `left`/`right`) or an n-ary form
//! (`ends: [{end: <role>, binds: <chain>}, ...]`). `graph.rs::build_graph` reads
//! these same two shapes to extract endpoint chains before resolving each chain
//! against the model graph; `parse_entry` here is that same chain-extraction
//! step, factored out so a diagram-driven edit and the graph builder can never
//! drift apart on what counts as a valid entry.

use serde_yaml::{Mapping, Value};

/// One endpoint of a connection entry: an optional role label (`end:`, only
/// present in the n-ary form) and the dotted feature chain or qname/id it
/// binds (`binds:`, or the raw `from`/`to`/`left`/`right` value).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionEndpoint {
    pub role: Option<String>,
    pub chain: String,
}

/// A single `connections:` (or sibling `*_connections:`) sequence entry.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConnectionEntry {
    pub typed_by: Option<String>,
    pub endpoints: Vec<ConnectionEndpoint>,
}

fn key(m: &Mapping, k: &str) -> Option<Value> {
    m.get(Value::from(k)).cloned()
}

fn str_key(m: &Mapping, k: &str) -> Option<String> {
    key(m, k).and_then(|v| v.as_str().map(String::from))
}

/// Parse one `connections:` entry into its `typedBy` and endpoint chains,
/// using the exact key conventions `graph.rs::build_graph` resolves against:
/// binary `from`/`to`/`left`/`right`, or n-ary `ends: [{end?, binds}, ...]`.
/// Returns `None` for a malformed entry (not a mapping, or no chains found).
pub fn parse_entry(entry: &Value) -> Option<ConnectionEntry> {
    let Value::Mapping(m) = entry else { return None };
    let typed_by = str_key(m, "typedBy");
    let mut endpoints = Vec::new();

    for k in ["from", "to", "left", "right"] {
        if let Some(chain) = str_key(m, k) {
            endpoints.push(ConnectionEndpoint { role: None, chain });
        }
    }

    if let Some(Value::Sequence(seq)) = key(m, "ends") {
        for e in seq {
            let Value::Mapping(em) = &e else { continue };
            let Some(chain) = str_key(em, "binds") else { continue };
            let role = str_key(em, "end");
            endpoints.push(ConnectionEndpoint { role, chain });
        }
    }

    if endpoints.is_empty() {
        return None;
    }
    Some(ConnectionEntry { typed_by, endpoints })
}

/// Parse every entry in a `connections:`-style sequence, skipping malformed ones.
pub fn parse_connections(entries: &[Value]) -> Vec<ConnectionEntry> {
    entries.iter().filter_map(parse_entry).collect()
}

/// Append a new binary `{typedBy?, from, to}` entry.
///
/// New connections are always written in the binary `from`/`to` form (the
/// documented canonical shape), even though most hand-authored entries in this
/// model use the n-ary `ends:` form for named-role bindings — a diagram-driven
/// "connect port A to port B" gesture has no natural role label to offer, so
/// there's nothing the n-ary form would add. `parse_entry` still reads both
/// shapes, so an existing `ends:`-form entry remains fully understood.
pub fn add_connection(entries: &mut Vec<Value>, from: &str, to: &str, typed_by: Option<&str>) {
    let mut m = Mapping::new();
    if let Some(tb) = typed_by {
        m.insert(Value::from("typedBy"), Value::from(tb));
    }
    m.insert(Value::from("from"), Value::from(from));
    m.insert(Value::from("to"), Value::from(to));
    entries.push(Value::Mapping(m));
}

/// Remove the first entry whose parsed endpoint chains are exactly `{from, to}`
/// (in either order, regardless of source shape or role labels). Returns
/// `true` if an entry was removed.
pub fn remove_connection(entries: &mut Vec<Value>, from: &str, to: &str) -> bool {
    let pos = entries.iter().position(|e| match parse_entry(e) {
        Some(parsed) if parsed.endpoints.len() == 2 => {
            let chains: Vec<&str> = parsed.endpoints.iter().map(|ep| ep.chain.as_str()).collect();
            (chains[0] == from && chains[1] == to) || (chains[0] == to && chains[1] == from)
        }
        _ => false,
    });
    match pos {
        Some(i) => {
            entries.remove(i);
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_binary_from_to() {
        let entry: Value = serde_yaml::from_str(
            "typedBy: Interfaces::PowerConnectionDef\nfrom: avionics.telemetryOut\nto: telemetryOut\n",
        )
        .unwrap();
        let parsed = parse_entry(&entry).unwrap();
        assert_eq!(parsed.typed_by.as_deref(), Some("Interfaces::PowerConnectionDef"));
        assert_eq!(parsed.endpoints.len(), 2);
        assert_eq!(parsed.endpoints[0].chain, "avionics.telemetryOut");
        assert_eq!(parsed.endpoints[0].role, None);
    }

    #[test]
    fn parses_nary_ends_form() {
        let entry: Value = serde_yaml::from_str(
            "typedBy: Interfaces::PowerConnectionDef\nends:\n  - end: source\n    binds: power.mainPowerOut\n  - end: sink\n    binds: mainPowerIn\n",
        )
        .unwrap();
        let parsed = parse_entry(&entry).unwrap();
        assert_eq!(parsed.endpoints.len(), 2);
        assert_eq!(parsed.endpoints[0].role.as_deref(), Some("source"));
        assert_eq!(parsed.endpoints[0].chain, "power.mainPowerOut");
        assert_eq!(parsed.endpoints[1].role.as_deref(), Some("sink"));
    }

    #[test]
    fn malformed_entry_is_skipped() {
        let entry: Value = serde_yaml::from_str("typedBy: Foo\n").unwrap();
        assert!(parse_entry(&entry).is_none());
    }

    #[test]
    fn add_then_remove_round_trips() {
        let mut entries: Vec<Value> = Vec::new();
        add_connection(&mut entries, "avionics.telemetryOut", "telemetryOut", Some("Interfaces::T"));
        assert_eq!(entries.len(), 1);
        let parsed = parse_connections(&entries);
        assert_eq!(parsed[0].endpoints.len(), 2);

        assert!(remove_connection(&mut entries, "telemetryOut", "avionics.telemetryOut"));
        assert!(entries.is_empty());
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut entries: Vec<Value> = Vec::new();
        assert!(!remove_connection(&mut entries, "a", "b"));
    }
}

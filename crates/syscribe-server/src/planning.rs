//! Board model of the planning dashboard (`REQ-TRS-VIS-027`): every
//! `PlanningItem`, counted by status, grouped by who is working on it and laid
//! out in one column per status. Pure over the element list so it is tested
//! without a server.

use std::collections::{BTreeMap, HashMap};

use syscribe_model::element::{ElementType, RawElement};

/// Statuses that always get a column, in this order; others follow alphabetically.
const STATUS_ORDER: [&str; 4] = ["todo", "in_progress", "blocked", "done"];
/// Shown when an item has no `status:`.
const NO_STATUS: &str = "todo";

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub qname: String,
    pub id: String,
    pub name: String,
    pub item_type: String,
    pub status: String,
    pub parent: String,
    /// Display name from the `[users]` roster, else the username; empty when unassigned.
    pub assignee: String,
    pub claimed_by: String,
    pub claimed_at: String,
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub status: String,
    pub count: usize,
    /// Empty for a hidden `done` column (only its count is shown).
    pub cards: Vec<Card>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkGroup {
    pub who: String,
    /// True when `who` is an agent claimant rather than an assigned person.
    pub is_agent: bool,
    pub cards: Vec<Card>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    pub total: usize,
    pub summary: Vec<(String, usize)>,
    pub working: Vec<WorkGroup>,
    pub columns: Vec<Column>,
    pub who: String,
    pub show_done: bool,
    /// Every assignee/claimant present, for the filter list.
    pub people: Vec<String>,
}

fn card_of(e: &RawElement, users: &HashMap<String, String>) -> Card {
    let f = &e.frontmatter;
    let assigned = f.assigned_to.clone().unwrap_or_default();
    let status = f.status.clone().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| NO_STATUS.to_string());
    Card {
        qname: e.qualified_name.clone(),
        id: f.id.clone().unwrap_or_default(),
        name: f.name.clone().unwrap_or_else(|| e.qualified_name.clone()),
        item_type: f.item_type.clone().unwrap_or_default(),
        status,
        parent: f.parent.clone().unwrap_or_default(),
        assignee: users.get(&assigned).cloned().unwrap_or(assigned),
        claimed_by: f.claimed_by.clone().map(|s| s.trim().to_string()).unwrap_or_default(),
        claimed_at: f.claimed_at.clone().unwrap_or_default(),
        blocked_by: f.blocked_by.clone().unwrap_or_default(),
    }
}

/// Build the board. `who` filters to items assigned to or claimed by that
/// name (empty = everyone); `show_done` keeps done cards (counts always include them).
pub fn build_board(elements: &[RawElement], users: &HashMap<String, String>, who: &str, show_done: bool) -> Board {
    let mut cards: Vec<Card> = elements
        .iter()
        .filter(|e| e.frontmatter.element_type == Some(ElementType::PlanningItem))
        .map(|e| card_of(e, users))
        .collect();
    cards.sort_by(|a, b| a.id.cmp(&b.id).then(a.qname.cmp(&b.qname)));

    let mut people: Vec<String> = cards
        .iter()
        .flat_map(|c| [c.assignee.clone(), c.claimed_by.clone()])
        .filter(|s| !s.is_empty())
        .collect();
    people.sort();
    people.dedup();

    let who = who.trim().to_string();
    let mine = |c: &Card| who.is_empty() || c.assignee == who || c.claimed_by == who;
    let visible: Vec<&Card> = cards.iter().filter(|c| mine(c)).collect();

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for c in &visible {
        *counts.entry(c.status.clone()).or_default() += 1;
    }
    let mut statuses: Vec<String> = STATUS_ORDER.iter().map(|s| s.to_string()).collect();
    statuses.extend(counts.keys().filter(|s| !STATUS_ORDER.contains(&s.as_str())).cloned());
    let summary: Vec<(String, usize)> = statuses.iter().map(|s| (s.clone(), counts.get(s).copied().unwrap_or(0))).collect();

    // Working now: in_progress or claimed, and not already done.
    let mut groups: BTreeMap<(bool, String), Vec<Card>> = BTreeMap::new();
    for c in visible.iter().filter(|c| c.status != "done" && (c.status == "in_progress" || !c.claimed_by.is_empty())) {
        let (key, agent) = if !c.claimed_by.is_empty() {
            (c.claimed_by.clone(), true)
        } else if !c.assignee.is_empty() {
            (c.assignee.clone(), false)
        } else {
            ("Unassigned".to_string(), false)
        };
        groups.entry((!agent, key)).or_default().push((*c).clone());
    }
    let working = groups
        .into_iter()
        .map(|((not_agent, who), cards)| WorkGroup { who, is_agent: !not_agent, cards })
        .collect();

    let columns = statuses
        .iter()
        .map(|s| {
            let in_col: Vec<Card> = visible.iter().filter(|c| &c.status == s).map(|c| (*c).clone()).collect();
            let count = in_col.len();
            let hide = s == "done" && !show_done;
            Column { status: s.clone(), count, cards: if hide { Vec::new() } else { in_col } }
        })
        .collect();

    Board { total: visible.len(), summary, working, columns, who, show_done, people }
}

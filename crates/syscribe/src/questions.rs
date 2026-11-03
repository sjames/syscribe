//! `questions [--open] [--status S] [--item PI] [--json]` — every PlanningItem's questions in one
//! list (GH #436, `REQ-TRS-PIQ-002`). The data function is shared with the MCP `questions` tool.

use syscribe_model::element::{ElementType, RawElement};
use syscribe_model::questions::{self, Question};
use syscribe_model::resolver::Resolver;

/// Filters shared by the CLI and MCP.
#[derive(Debug, Default, Clone)]
pub struct Filter {
    pub status: Option<String>,
    pub item: Option<String>,
}

/// The questions matching `f`, ordered by owning item then entry order.
/// `Err` for an unknown `status`, an `item` that does not resolve, or one that is not a PlanningItem.
pub fn select(elements: &[RawElement], f: &Filter) -> Result<Vec<Question>, String> {
    if let Some(s) = &f.status {
        if !questions::STATUSES.contains(&s.as_str()) {
            return Err(format!(
                "unknown question status '{s}';{} valid statuses: {}",
                crate::suggest::did_you_mean(s, questions::STATUSES.iter().copied()),
                questions::STATUSES.join(", ")
            ));
        }
    }
    let only: Option<String> = match &f.item {
        None => None,
        Some(r) => {
            let resolver = Resolver::new(elements);
            let e = resolver.resolve_ref(elements, r).ok_or_else(|| format!("element not found: {r}"))?;
            if e.frontmatter.element_type != Some(ElementType::PlanningItem) {
                return Err(format!("{r} is not a PlanningItem"));
            }
            Some(e.qualified_name.clone())
        }
    };
    let mut qs: Vec<Question> = elements
        .iter()
        .filter(|e| only.as_deref().is_none_or(|q| e.qualified_name == q))
        .flat_map(|e| questions::parse(e).0)
        .filter(|q| f.status.as_deref().is_none_or(|s| q.status == s))
        .collect();
    qs.sort_by(|a, b| a.item_qualified_name.cmp(&b.item_qualified_name));
    Ok(qs)
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

/// `args` are the tokens after `questions`. Returns the exit code.
pub fn cmd_questions(elements: &[RawElement], args: &[String]) -> i32 {
    let value = |flag: &str| args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone());
    let json = args.iter().any(|a| a == "--json");
    let mut f = Filter { status: value("--status"), item: value("--item") };
    if args.iter().any(|a| a == "--open") {
        if f.status.as_deref().is_some_and(|s| s != "open") {
            eprintln!("questions: --open and --status {} contradict each other", f.status.as_deref().unwrap_or(""));
            return 1;
        }
        f.status = Some("open".into());
    }
    let qs = match select(elements, &f) {
        Ok(q) => q,
        Err(m) => {
            eprintln!("questions: {m}");
            return 1;
        }
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "schemaVersion": "1.0", "questions": qs })).unwrap());
        return 0;
    }
    if qs.is_empty() {
        println!("No questions{}.", if f.status.is_some() || f.item.is_some() { " match" } else { "" });
        return 0;
    }
    println!("# Questions ({})\n", qs.len());
    println!("| Question | Status | Text | Recommended | Answer |");
    println!("|---|---|---|---|---|");
    for q in &qs {
        let answer = match (&q.answer, &q.answered_by) {
            (Some(a), Some(by)) => format!("{a} — {by}"),
            (Some(a), None) => a.clone(),
            _ => String::new(),
        };
        println!(
            "| {} | {} | {} | {} | {} |",
            q.reference(),
            q.status,
            cell(&q.text),
            cell(q.recommended.as_deref().unwrap_or("")),
            cell(&answer)
        );
    }
    0
}

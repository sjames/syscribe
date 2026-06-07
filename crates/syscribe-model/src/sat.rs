//! Minimal deterministic CNF SAT solver (DPLL with unit propagation), pure Rust
//! and dependency-free. Sufficient for the Boolean feature-model analysis
//! queries (satisfiability under assumptions, plus deletion-based unsat cores).
//!
//! Determinism: branching variables are chosen in ascending index order and the
//! `true` branch is always tried before `false`, so a given formula yields the
//! same answer (and the same model) on every run and platform — the property
//! tool qualification requires (REQ-TRS-FMA-006).

/// A literal: a variable index plus a sign (`neg == true` means ¬var).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Lit {
    pub var: usize,
    pub neg: bool,
}

impl Lit {
    pub fn pos(var: usize) -> Lit {
        Lit { var, neg: false }
    }
    pub fn neg(var: usize) -> Lit {
        Lit { var, neg: true }
    }
}

/// A CNF formula over `num_vars` Boolean variables.
#[derive(Clone, Default)]
pub struct Cnf {
    pub num_vars: usize,
    pub clauses: Vec<Vec<Lit>>,
}

impl Cnf {
    pub fn new(num_vars: usize) -> Self {
        Cnf { num_vars, clauses: Vec::new() }
    }

    pub fn add(&mut self, clause: Vec<Lit>) {
        self.clauses.push(clause);
    }

    /// Returns a satisfying assignment under the given assumption literals, or
    /// `None` if unsatisfiable. Unassigned variables in the returned vector
    /// (those that never had to be decided) are filled `false`.
    pub fn solve(&self, assumptions: &[Lit]) -> Option<Vec<bool>> {
        let mut assign: Vec<Option<bool>> = vec![None; self.num_vars];
        for a in assumptions {
            let want = !a.neg;
            match assign[a.var] {
                Some(v) if v != want => return None,
                _ => assign[a.var] = Some(want),
            }
        }
        if dpll(&self.clauses, &mut assign) {
            Some(assign.into_iter().map(|o| o.unwrap_or(false)).collect())
        } else {
            None
        }
    }

    pub fn is_sat(&self, assumptions: &[Lit]) -> bool {
        self.solve(assumptions).is_some()
    }
}

/// Returns `true` and a completed assignment if satisfiable from the current
/// partial assignment, else `false`.
fn dpll(clauses: &[Vec<Lit>], assign: &mut [Option<bool>]) -> bool {
    // Unit propagation to a fixpoint; returns false on conflict.
    loop {
        let mut progress = false;
        for cl in clauses {
            let mut satisfied = false;
            let mut unassigned: Option<Lit> = None;
            let mut n_unassigned = 0;
            for lit in cl {
                match assign[lit.var] {
                    Some(v) => {
                        if v != lit.neg {
                            satisfied = true;
                            break;
                        }
                    }
                    None => {
                        n_unassigned += 1;
                        unassigned = Some(*lit);
                    }
                }
            }
            if satisfied {
                continue;
            }
            if n_unassigned == 0 {
                return false; // conflict
            }
            if n_unassigned == 1 {
                let l = unassigned.unwrap();
                assign[l.var] = Some(!l.neg);
                progress = true;
            }
        }
        if !progress {
            break;
        }
    }

    // Pick the first unassigned variable (deterministic order).
    let branch = assign.iter().position(|a| a.is_none());
    let Some(branch) = branch else {
        // Fully assigned: verify every clause is satisfied.
        return clauses
            .iter()
            .all(|cl| cl.iter().any(|l| assign[l.var] == Some(!l.neg)));
    };

    for val in [true, false] {
        let mut next = assign.to_vec();
        next[branch] = Some(val);
        if dpll(clauses, &mut next) {
            assign.copy_from_slice(&next);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial_sat_unsat() {
        let mut c = Cnf::new(1);
        assert!(c.is_sat(&[]));
        c.add(vec![Lit::pos(0)]); // x0
        c.add(vec![Lit::neg(0)]); // ¬x0
        assert!(!c.is_sat(&[])); // x0 ∧ ¬x0 is UNSAT
    }

    #[test]
    fn implication_and_assumptions() {
        // x0 ⇒ x1  encoded as (¬x0 ∨ x1)
        let mut c = Cnf::new(2);
        c.add(vec![Lit::neg(0), Lit::pos(1)]);
        // Under x0 = true, x1 must be true.
        let m = c.solve(&[Lit::pos(0)]).unwrap();
        assert!(m[1]);
        // x0 ∧ ¬x1 is UNSAT.
        assert!(!c.is_sat(&[Lit::pos(0), Lit::neg(1)]));
    }

    #[test]
    fn exactly_one_of_three() {
        // at-least-one: (a ∨ b ∨ c); at-most-one: pairwise ¬a∨¬b etc.
        let mut c = Cnf::new(3);
        c.add(vec![Lit::pos(0), Lit::pos(1), Lit::pos(2)]);
        c.add(vec![Lit::neg(0), Lit::neg(1)]);
        c.add(vec![Lit::neg(0), Lit::neg(2)]);
        c.add(vec![Lit::neg(1), Lit::neg(2)]);
        assert!(c.is_sat(&[Lit::pos(0)]));
        assert!(!c.is_sat(&[Lit::pos(0), Lit::pos(1)])); // two selected → UNSAT
        assert!(!c.is_sat(&[Lit::neg(0), Lit::neg(1), Lit::neg(2)])); // none → UNSAT
    }

    #[test]
    fn determinism() {
        let mut c = Cnf::new(3);
        c.add(vec![Lit::pos(0), Lit::pos(1), Lit::pos(2)]);
        let a = c.solve(&[]).unwrap();
        let b = c.solve(&[]).unwrap();
        assert_eq!(a, b);
    }
}

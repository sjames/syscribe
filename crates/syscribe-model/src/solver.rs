//! batsat-backed SAT engine for feature-model analysis (ADR-FM-002).
//!
//! batsat (MiniSat-derived CDCL) is the **only** SAT solver used — there is no
//! in-tree solver. This module owns a small clause IR ([`Cnf`]/[`Lit`]) that the
//! encoder builds and hands to batsat. The `#[cfg(test)]` brute-force oracle is
//! an exhaustive truth-table *verifier* (not a solver) used to validate the
//! batsat wrapper on random formulas.

use batsat::{lbool, BasicSolver, Lit as BLit, SolverInterface, Var};

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

/// A CNF formula over `num_vars` Boolean variables — a neutral IR the encoder
/// builds; solving is delegated to batsat.
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
}

/// Map our literal onto a batsat literal. In batsat `Lit::new(v, sign)` with
/// `sign == true` is the **positive** literal, so we pass `!neg`.
fn blit(vars: &[Var], l: Lit) -> BLit {
    BLit::new(vars[l.var], !l.neg)
}

/// A batsat solver primed with a fixed clause set; reusable across assumption
/// queries (batsat is incremental, so the many deep-analysis queries share one
/// solver).
pub struct Solver {
    inner: BasicSolver,
    vars: Vec<Var>,
    ok: bool,
}

impl Solver {
    pub fn from_cnf(cnf: &Cnf) -> Self {
        let mut inner = BasicSolver::default();
        let vars: Vec<Var> = (0..cnf.num_vars).map(|_| inner.new_var_default()).collect();
        let mut ok = true;
        for cl in &cnf.clauses {
            let mut bcl: Vec<BLit> = cl.iter().map(|l| blit(&vars, *l)).collect();
            // add_clause_reuse returns false once the solver is UNSAT.
            ok &= inner.add_clause_reuse(&mut bcl);
        }
        Solver { inner, vars, ok }
    }

    /// Satisfiable under the given assumption literals?
    pub fn is_sat(&mut self, assumptions: &[Lit]) -> bool {
        if !self.ok {
            return false;
        }
        let asm: Vec<BLit> = assumptions.iter().map(|l| blit(&self.vars, *l)).collect();
        self.inner.solve_limited(&asm) == lbool::TRUE
    }
}

/// One-shot satisfiability check against a fresh solver.
pub fn is_sat(cnf: &Cnf, assumptions: &[Lit]) -> bool {
    Solver::from_cnf(cnf).is_sat(assumptions)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exhaustive truth-table oracle (a verifier, not a solver).
    fn brute_sat(cnf: &Cnf, assumptions: &[Lit]) -> bool {
        for mask in 0u64..(1u64 << cnf.num_vars) {
            let assign: Vec<bool> = (0..cnf.num_vars).map(|v| (mask >> v) & 1 == 1).collect();
            if assumptions.iter().any(|a| assign[a.var] == a.neg) {
                continue;
            }
            if cnf.clauses.iter().all(|cl| cl.iter().any(|l| assign[l.var] != l.neg)) {
                return true;
            }
        }
        false
    }

    // Deterministic dependency-free PRNG (xorshift64) for reproducible fuzzing.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    #[test]
    fn basic() {
        assert!(is_sat(&Cnf::new(1), &[]));
        let mut c = Cnf::new(1);
        c.add(vec![Lit::pos(0)]);
        c.add(vec![Lit::neg(0)]);
        assert!(!is_sat(&c, &[])); // x ∧ ¬x
        // implication under assumptions
        let mut d = Cnf::new(2);
        d.add(vec![Lit::neg(0), Lit::pos(1)]); // x0 ⇒ x1
        assert!(is_sat(&d, &[Lit::pos(0)]));
        assert!(!is_sat(&d, &[Lit::pos(0), Lit::neg(1)]));
    }

    #[test]
    fn edge_cases() {
        assert!(is_sat(&Cnf::new(0), &[])); // empty formula SAT
        let mut c = Cnf::new(2);
        c.add(vec![]); // empty clause → UNSAT
        assert!(!is_sat(&c, &[]));
        assert!(!is_sat(&Cnf::new(1), &[Lit::pos(0), Lit::neg(0)])); // contradictory assumptions
    }

    #[test]
    fn pigeonhole_unsat() {
        fn php(p: usize, h: usize) -> Cnf {
            let mut c = Cnf::new(p * h);
            for i in 0..p {
                c.add((0..h).map(|j| Lit::pos(i * h + j)).collect());
            }
            for j in 0..h {
                for i1 in 0..p {
                    for i2 in (i1 + 1)..p {
                        c.add(vec![Lit::neg(i1 * h + j), Lit::neg(i2 * h + j)]);
                    }
                }
            }
            c
        }
        assert!(!is_sat(&php(3, 2), &[]));
        assert!(!is_sat(&php(4, 3), &[]));
        assert!(is_sat(&php(2, 2), &[]));
        assert!(is_sat(&php(3, 3), &[]));
    }

    // Validate the batsat wrapper (sign conventions, clause building, assumptions)
    // against the exhaustive oracle on thousands of random formulas.
    #[test]
    fn batsat_matches_brute_force() {
        let mut rng = Rng(0xD1B5_4A32_D192_ED03);
        for _ in 0..5000 {
            let nv = 1 + rng.below(7);
            let mut cnf = Cnf::new(nv);
            let m = rng.below(4 * nv + 1);
            for _ in 0..m {
                let k = 1 + rng.below(3);
                let cl: Vec<Lit> = (0..k)
                    .map(|_| Lit { var: rng.below(nv), neg: rng.next() & 1 == 0 })
                    .collect();
                cnf.add(cl);
            }
            let na = rng.below(3);
            let asm: Vec<Lit> = (0..na)
                .map(|_| Lit { var: rng.below(nv), neg: rng.next() & 1 == 0 })
                .collect();
            assert_eq!(
                is_sat(&cnf, &asm),
                brute_sat(&cnf, &asm),
                "batsat disagrees with brute force: clauses={:?} asm={:?}",
                cnf.clauses,
                asm
            );
        }
    }
}

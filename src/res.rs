use std::collections::BTreeSet;

use crate::cnf::{Clause, Term};

/// A candidate clause, tracking its complexity. This struct orders clauses by complexity when used
/// in a [BTreeSet], allowing us to prioritise low-complexity clauses.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct CandidateClause {
    complexity: usize, // Note: the order of fields matters for the derived implementation of Ord
    clause: Clause
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
enum Resolvent {
    /// The resolution resolved a nontrivial clause
    Nontrivial(Clause),

    /// The resolution resovled a tautology or an in other way pointless expression
    Tautology,

    /// The resolution resolved a contradiction
    Contradiction
}

impl Resolvent {
    fn cleanup(self) -> Self {
        if let Resolvent::Nontrivial(mut clause) = self {

            // If there is a positive tautology, the clause is a tautology itself
            if clause.pos.iter().any(|e| e.is_tautology()) {
                return Resolvent::Tautology;
            }

            // If there is a negative tautology, it's a contradiction instead, and we remove it
            clause.neg.retain(|e| !e.is_tautology());

            if clause.is_empty() {
                return Resolvent::Contradiction;
            }

            Resolvent::Nontrivial(clause)
        } else {
            self
        }
    }
}



/// Resolves two clauses against eachother. Given two clauses
/// `A1 | A2 | A3 | ... | B` and `!B | C1 | C2 | C3 | ...`, this computes the resolvent clause
/// `A1 | A2 | A3 | ... | C1 | C2 | C3 | ...`.
///
/// The logic of this function stems from the fact that `A1 | A2 | A3 | ... | B` can be written
/// as `!(A1 | A2 | A3 | ...) -> B` and that `!B | C1 | C2 | C3 | ...` can be written as
/// `B -> (C1 | C2 | C3 | ...)`. We now apply the knowledge that `P -> Q, Q -> R |- P -> R`,
/// which results in the statement `!(A1 | A2 | A3 | ...) -> (C1 | C2 | C3 | ...)`. This can be
/// rewritten back to `A1 | A2 | A3 | ... | C1 | C2 | C3 | ...`.
///
/// There is some additional reasoning included in this function:
/// - If the clauses are distinct and do not share a symbol in complementary form, nothing is
///   resolved.
/// - If the clauses share multiple symbols in complementary form, one of these symbols is the
///   symbol that is resolved over. The remaining symbols form tautologies, which cause the
///   entire clause to be a tautology.
///
/// Logically, the function just combines the clauses into one disjunction and removes the
/// complementary pairs. This works, as one of those pairs is the resolved symbol, while the
/// others are tautologies that would otherwise be unnecessarily left in the resolvent. No
/// matter which symbol we pick as symbol to resolve over, the elimination of tautologies will
/// eventually lead to the same resolvent.
/// 
/// If the resolvent has no remaining disjuncts, then a trivial case is reached, and the
/// result is either a tautology or a contradiction:
/// - If the clauses shared no symbol in complementary form, then the clauses must necessarily
///   be two empty clauses, therefore being both contradictions, and the resolvent is therefore
///   also a contradiction.
/// - If the clauses share exactly one symbol in complementary form, then the clauses must
///   necessarily be of the form `A` and `!A`, and `A` must necessarily be the symbol that is
///   resolved over. The statement `A & !A` is contradictory, thus in this case, the resolution
///   lead to a contradiction.
/// - If the clauses share two or more pairs of symbols in complementary form, then the
///   resolvent is a tautology, as described before.
///
/// Thus, the return value of this function is:
/// - [Resolvent::Contradiction] when both clauses are empty;
/// - [Resolvent::Contradiction] when the clauses are of the form `A` and `!A`;
/// - [Resolvent::Tautology] when the clauses share multiple symbols in complementary form, or none at all;
/// - [Resolvent::Nontrivial] in any other case.
fn propositional_resolve(a: &Clause, b: &Clause) -> Resolvent {
    // Collect the unions of the positive and negative sets of the clauses
    let mut pos_u = BTreeSet::<Term>::new();
    let mut neg_u = BTreeSet::<Term>::new();

    pos_u.extend(a.pos.clone());
    pos_u.extend(b.pos.clone());
    neg_u.extend(a.neg.clone());
    neg_u.extend(b.neg.clone());

    // Now find the intersection between these two sets
    let mut isc = BTreeSet::<Term>::new();
    for term in pos_u.intersection(&neg_u) {
        isc.insert(term.clone());
    }

    // Remove intersecting elements from unions
    //
    // One of the intersecting symbols is the symbol we resolved over,
    // the other ones that we remove are tautologies in the resolvent.
    // The difference between these two is only relevant in the case
    // we resolve an empty clause.
    let mut iscs = 0u64;
    for term in isc {
        pos_u.remove(&term);
        neg_u.remove(&term);
        iscs += 1;

        // If we remove more than one intersection, it means there is a tautology in the
        // clause. Any clause with a tautology is per definition a tautology itself.
        if iscs > 1 {
            return Resolvent::Tautology;
        }
    }

    if iscs == 0 {
        return Resolvent::Tautology;
    }

    if pos_u.is_empty() && neg_u.is_empty() {
        // iscs == 0:
        //   The input clauses were empty, so we treat the clauses as `false` and `false`.
        //   As `false & false |- false`, the result must be contradictory.
        //
        // iscs == 1:
        //   Only one intersection was there, thus the only intersecting symbol
        //   was the symbol we resolved over. In other words, we resolved something
        //   along the lines of `P & !P`, which is contradictory.
        //
        // iscs > 1:
        //   Can't happen, we already caught this case before.
        return Resolvent::Contradiction;
    } else {
        // Resulting clause is non-empty and it's a non-trivial case
        return Resolvent::Nontrivial(Clause {
            pos: pos_u,
            neg: neg_u
        })
    }
}


/// Causes a substitution in `base` if `eq` is of the form `a == b`. If it is,
/// all instances of `a` are replaced with `b` in `base`. If it isn't, [None]
/// is returned.
fn equals_resolve(base: &Clause, eq: &Clause) -> Option<Resolvent> {
    match eq.pos_singleton() {
        Some(Term::Equality(l, r)) => {
            let right = base.clone().substitute(l, r);
            Some(Resolvent::Nontrivial(right))
        }
        _ => None
    }
}


/// Finds all resolvents between the two clauses.
fn resolve(a: &Clause, b: &Clause) -> Vec<Resolvent> {
    let mut out = Vec::new();

    out.push(propositional_resolve(a, b));

    if let Some(res) = equals_resolve(a, b) {
        out.push(res);
    }

    if let Some(res) = equals_resolve(b, a) {
        out.push(res);
    }

    out
}


/// The results of a [resolution] operation.
pub struct Resolution {
    /// True when the the input statement can be satisfied, false if it is a contradiction.
    pub satisfied: bool,

    /// The amount of clauses it learned during resolution.
    pub clauses_learned: u64,
}


/// Given a set of [Clause]s representing an expression in CNF, this function determines the satisfiability
/// of that expression by means of resolution.
pub fn resolution(stmt: &BTreeSet<Clause>) -> Resolution {
    // The algorithm is somewhat similar to A*, searching the entire search space but heavily preferring to
    // work with smaller expressions (the smaller, the more likely it is to be a contradiction)

    // Knowledge base
    let mut knowledge = BTreeSet::new();

    // The next resolvents that can be added to the knowledge base
    // The B-Tree helps us keep them sorted by complexity
    let mut next = BTreeSet::new();

    let mut stats = Resolution {
        satisfied: false,
        clauses_learned: 0
    };

    // Start with the input on the "next" set
    for clause in stmt.clone() {
        match Resolvent::Nontrivial(clause).cleanup() {
            // If any input clause is a tautology, it's pointless
            Resolvent::Tautology => {},

            // If any input clause is a contradiction, we're done early
            Resolvent::Contradiction => return stats,

            // Insert with zero complexity since these clauses are trivial knowledge.
            // It is possible to use the clause's actual complexity, but this seems to only
            // slow down resolution.
            Resolvent::Nontrivial(clause) => {
                next.insert(CandidateClause {
                    complexity: 0,
                    clause
                });
            },
        };
    }

    // While there are elements in `next`, we keep adding them to our knowledge base.
    while let Some(cand) = next.pop_first() {
        let new = cand.clause;

        if knowledge.contains(&new) {
            continue; // Old news
        }


        // This is new knowledge, we need to update the `next` set with the new
        // candidate clauses we could learn from learning this clause
        for old in &knowledge {
            let resolvents = resolve(old, &new);

            for resolvent in resolvents {

                match resolvent.cleanup() {
                    // New nontrivial clause: add candidate if we did not already know
                    // about this clause
                    Resolvent::Nontrivial(clause) => {
                        if !knowledge.contains(&clause) {
                            next.insert(CandidateClause {
                                complexity: clause.complexity(),
                                clause
                            });
                        }
                    }

                    // Contradictory clause: this proves unsatisfiability so we are done
                    Resolvent::Contradiction => return stats,

                    // Pointless clause: ignore it
                    Resolvent::Tautology => {}
                }
            }
        }

        // Only add the clause to knowledge now, so that the previous
        // loop does not need to worry about resolving a clause with itself
        knowledge.insert(new);

        stats.clauses_learned += 1;
    }

    // No contradictions were found, thus the expression is satisfiable
    stats.satisfied = true;
    stats
}
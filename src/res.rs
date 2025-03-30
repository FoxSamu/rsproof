use std::collections::BTreeSet;

use crate::cnf::Clause;

/// A candidate clause, tracking its complexity. This struct orders clauses by complexity when used
/// in a [BTreeSet], allowing us to prioritise low-complexity clauses.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct CandidateClause {
    complexity: usize, // Note: the order of fields matters for the derived implementation of Ord
    clause: Clause
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
enum Resolvent {
    /// The resolution resolved a nontrivial clause
    Nontrivial(Clause),

    /// The resolution resovled a tautology
    Tautology,

    /// The resolution resolved a contradiction
    Contradiction
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
/// - If the clauses are distinct and do not share a symbol in complementary form, the
///   disjunction of the clauses is resolved. E.g. `A | B` and `C | D` resolve to
///   `A | B | C | D`. It applies the inference `P, Q |- P | Q`.
/// - If the clauses share multiple symbols in complementary form, none of those symbols will
///   be present in the resolvent. E.g. `A | B | C` and `!B | !C | D` resolve to `A | D`. 
///   This reasoning stems from the fact that the resolvent would otherwise contain a
///   tautology, which we can remove from the clause.
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
/// - If the clauses share two or more pairs of symbols in complementary form, then one of those
///   pairs is of the symbol it resolved over. The remaining pairs are all tautologies, and
///   therefore the resolvent must necessarily be a tautology.
fn resolve(a: &Clause, b: &Clause) -> Resolvent {
    // Collect the unions of the positive and negative sets of the clauses
    let mut pos_u = BTreeSet::<u64>::new();
    let mut neg_u = BTreeSet::<u64>::new();

    pos_u.extend(&a.pos);
    pos_u.extend(&b.pos);
    neg_u.extend(&a.neg);
    neg_u.extend(&b.neg);

    // Now find the intersection between these two sets
    let mut isc = BTreeSet::<u64>::new();
    isc.extend(pos_u.intersection(&neg_u));

    // Remove intersecting elements from unions
    //
    // One of the intersecting symbols is the symbol we resolved over,
    // the other ones that we remove are tautologies in the resolvent.
    // The difference between these two is only relevant in the case
    // we resolve an empty clause.
    let mut iscs = 0u64;
    for sym in isc {
        pos_u.remove(&sym);
        neg_u.remove(&sym);
        iscs += 1;
    }

    if pos_u.is_empty() && neg_u.is_empty() {
        // Resulting clause is empty and we hit a trivial case
        if iscs <= 1 {
            // iscs == 0:
            //   The input clauses were empty, so we treat the clauses as `false` and `false`.
            //   As `false & false |- false`, the result must be contradictory.
            //
            // iscs == 1:
            //   Only one intersection was there, thus the only intersecting symbol
            //   was the symbol we resolved over. In other words, we resolved something
            //   along the lines of `P & !P`, which is contradictory.
            return Resolvent::Contradiction;
        } else {
            // There were multiple intersections, meaning that the resolvent clause would
            // be a disjunction of tautologies of the form `P | !P`. Since all disjuncts
            // are tautologies, the resulting clause is a tautology.
            return Resolvent::Tautology;
        }
    } else {
        // Resulting clause is non-empty and it's a non-trivial case
        return Resolvent::Nontrivial(Clause {
            pos: pos_u,
            neg: neg_u
        })
    }
}

/// Given a set of [Clause]s representing an expression in CNF, this function determines the satisfiability
/// of that expression by means of resolution.
pub fn resolution(stmt: &BTreeSet<Clause>) -> bool {
    // The algorithm is somewhat similar to A*, searching the entire search space but heavily preferring to
    // work with smaller expressions (the smaller, the more likely it is to be a contradiction)

    // Knowledge base
    let mut knowledge = BTreeSet::new();

    // The next resolvents that can be added to the knowledge base
    // The B-Tree helps us keep them sorted by complexity
    let mut next = BTreeSet::new();

    // Start with the input on the "next" set
    for clause in stmt.clone() {
        // If any input clause is a contradiction we are done early
        if clause.is_contradiction() {
            return false;
        }

        // Insert with zero complexity since these clauses are trivial knowledge.
        // It is possible to use the clause's actual complexity, but this seems to only
        // slow down resolution.
        next.insert(CandidateClause {
            complexity: 0,
            clause
        });
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
            match resolve(old, &new) {
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
                Resolvent::Contradiction => return false,

                // Tautology clause: this is useless
                Resolvent::Tautology => {}
            }
        }

        // Only add the clause to knowledge now, so that the previous
        // loop does not need to worry about resolving a clause with itself
        knowledge.insert(new);
    }

    // No contradictions were found, thus the expression is satisfiable
    true
}
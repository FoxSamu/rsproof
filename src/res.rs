use std::collections::BTreeSet;

use crate::cnf::Clause;

/// A resolvent clause, tracking its complexity.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Resolvent {
    complexity: usize, // Note: the order of fields matters for the derived implementation of Ord
    clause: Clause
}

/// Given a set of [Clause]s representing an expression in CNF, this function determines the satisfiability
/// of that expression by means of resolution.
pub fn resolution(stmt: &BTreeSet<Clause>) -> bool {
    // The algorithm is somewhat similar to A*, searching the entire search space but heavily preferring to
    // work with smaller expressions (the smaller, the more likely it is to be a contradiction)

    // Knowledge base
    let mut knowledge = stmt.clone();

    // The next resolvents that can be added to the knowledge base
    // The B-Tree helps us keep them sorted by complexity
    let mut next = BTreeSet::new();

    // Compute all resolvents that follow from the knowledge base and add them
    // to the next queue.
    for i in &knowledge {
        for j in &knowledge {
            if j < i {
                let c = Clause::from_union(i, j);

                // Contradiction -> unsatisfiable
                if c.is_contradiction() {
                    return false;
                }

                // Only add candidate if we did not already learn this
                if !knowledge.contains(&c) {
                    next.insert(Resolvent {
                        complexity: c.complexity(),
                        clause: c
                    });
                }
            }
        }
    }

    // While there are elements in `next`, we keep adding them to our knowledge base.
    while let Some(res) = next.pop_first() {
        if knowledge.contains(&res.clause) {
            continue; // Old news
        }

        // This is new knowledge, we need to update the `next` set with the new
        // potential knowledge we gain from learning this clause
        for k in &knowledge {
            let c = Clause::from_union(k, &res.clause);

            // Contradiction -> unsatisfiable
            if c.is_contradiction() {
                return false;
            }

            // Only add candidate if we did not already learn this
            if !knowledge.contains(&c) {
                next.insert(Resolvent {
                    complexity: c.complexity(),
                    clause: c
                });
            }
        }

        // Only add the clause to knowledge now, so that the previous
        // loop does not need to worry about resolving a clause with itself
        knowledge.insert(res.clause);
    }

    // No contradictions were found, thus the expression is satisfiable
    true
}
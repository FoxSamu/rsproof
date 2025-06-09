use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::nf::Clause;
use crate::expr::Name;

type RcClauses = BTreeSet<Rc<Clause>>;

/// A [KnowledgeBase] systematically stores [Clause]s so that clauses with complementary predicates,
/// e.g. `P` and `!P`, can be quickly matched.
/// 
/// Say a resolution algorithm is supposed to resolve over a knowledge base of `N` clauses, then there
/// are `N^2` possible pairs of clauses to which resolution can be applied. However, most of these
/// pairs will not be a *resolvable pair*, that is, they share no complementary predicate or they cannot
/// be unified.
/// 
/// A naive database would just be a vector or set, requiring one to iterate `O(N^2)` pairs of clauses to
/// find resolvable pairs. By more systematically storing clauses, it is possible to make a significant
/// reduction to the search space. It is quite easy to conclude that clauses never make a resolvable pair
/// if they do not share the usage of a predicate in complementary forms&ndash;and then we have not even
/// done the unification yet.
/// 
/// The [KnowledgeBase] sorts clauses based on which predicates they refer to and whether they are negated,
/// that is, it keeps two multimaps, one mapping any name `P` to a set of clauses which have a non-negated
/// use of a predicate named `P`, and one mapping `P` to a set of clauses which have a negated use of `P`.
/// The database then reduces the search space by matching these maps: if a pair of clauses is resolvable,
/// then one clause appears in the positive map under some name `P` and the other appears in the negative
/// map under the same name `P`. Thus, for any predicate name it knows of, it simply pairs the positive set
/// under that name with the negative set under that name.
#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    // Index stores a set of all clauses, but it also refers to clauses by predicate names
    // that appear in the clause. It also tracks all the predicate names it knows of.

    // E.g. clause `P | Q(:x) | !R(a) | R(:x)` is stored in the field `clauses`, but also
    // under names `P`, `Q`, `R` in `by_pos`, and under name `R` in `by_neg`.

    // Use counted references so we can refer to a clause from multiple locations in
    // the data structure.


    clauses: RcClauses,
    by_pos: BTreeMap<Name, RcClauses>,
    by_neg: BTreeMap<Name, RcClauses>,
    candidates: BTreeSet<(Rc<Clause>, Rc<Clause>)>
}

impl KnowledgeBase {
    /// Instantiates a new [KnowledgeBase] with no clauses.
    pub fn new() -> Self {
        Self {
            clauses: RcClauses::new(),
            by_pos: BTreeMap::new(),
            by_neg: BTreeMap::new(),
            candidates: BTreeSet::new()
        }
    }

    /// Learns a specific clause. The return value is a set of candidates that were freshly
    /// obtained from learning this clause.
    pub fn learn(&mut self, c: Clause) -> BTreeSet<(Rc<Clause>, Rc<Clause>)> {
        let rc = Rc::new(c);

        let mut pos_names = BTreeSet::new();
        let mut neg_names = BTreeSet::new();

        let mut new_candidates = BTreeSet::new();

        // Insert in total clause set
        self.clauses.insert(rc.clone());

        // Map by positive names
        for pos in rc.pos().iter_pred_names() {
            let entry = self.by_pos
                .entry(*pos)
                .or_insert(RcClauses::new());

            entry.insert(rc.clone());

            pos_names.insert(*pos);
        }

        // Map by negative names
        for neg in rc.neg().iter_pred_names() {
            let entry = self.by_neg
                .entry(*neg)
                .or_insert(RcClauses::new());

            entry.insert(rc.clone());

            neg_names.insert(*neg);
        }

        // Map positive names with negative candidates
        for name in pos_names {
            if let Some(set) = self.by_neg.get(&name) {
                for elem in set {
                    if *rc != **elem {
                        self.candidates.insert((rc.clone(), elem.clone()));
                        new_candidates.insert((elem.clone(), rc.clone()));
                    }
                }
            }
        }

        // Map negative names with positive candidates
        for name in neg_names {
            if let Some(set) = self.by_pos.get(&name) {
                for elem in set {
                    if *rc != **elem {
                        self.candidates.insert((elem.clone(), rc.clone()));
                        new_candidates.insert((elem.clone(), rc.clone()));
                    }
                }
            }
        }

        new_candidates
    }

    /// Resolves a set of resolution candidates. These candidates are added to the set `out`.
    pub fn resolution_candidates(&self, out: &mut BTreeSet<(Rc<Clause>, Rc<Clause>)>) {
        for cand in self.candidates.iter() {
            out.insert(cand.clone());
        }
    }
}
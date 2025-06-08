use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::nf::Clause;
use crate::expr::Name;

type RcClauses = BTreeSet<Rc<Clause>>;

#[derive(Debug, Clone)]
pub struct ClauseDatabase {
    // Index stores a set of all clauses, but it also refers to clauses by predicate names
    // that appear in the clause. It also tracks all the predicate names it knows of.

    // E.g. clause `P | Q(:x) | !R(a) | R(:x)` is stored in the field `clauses`, but also
    // under names `P`, `Q`, `R` in `by_pos`, and under name `R` in `by_neg`.

    // Use counted references so we can refer to the memory from multiple locations in
    // the data structure.


    clauses: RcClauses,
    by_pos: BTreeMap<Name, RcClauses>,
    by_neg: BTreeMap<Name, RcClauses>,
    names: BTreeSet<Name>
}

impl ClauseDatabase {
    pub fn new() -> Self {
        Self {
            clauses: RcClauses::new(),
            by_pos: BTreeMap::new(),
            by_neg: BTreeMap::new(),
            names: BTreeSet::new(),
        }
    }

    pub fn learn(&mut self, c: Clause) {
        let rc = Rc::new(c);

        self.clauses.insert(rc.clone());

        for pos in rc.pos().iter_pred_names() {
            let entry = self.by_pos
                .entry(*pos)
                .or_insert(RcClauses::new());

            entry.insert(rc.clone());

            self.names.insert(*pos);
        }

        for neg in rc.neg().iter_pred_names() {
            let entry = self.by_neg
                .entry(*neg)
                .or_insert(RcClauses::new());

            entry.insert(rc.clone());

            self.names.insert(*neg);
        }
    }

    pub fn resolution_candidates(&self, out: &mut BTreeSet<(Rc<Clause>, Rc<Clause>)>) {
        for name in self.names.iter() {
            if let (Some(pos), Some(neg)) = (self.by_pos.get(name), self.by_neg.get(name)) {
                for pos_elem in pos {
                    for neg_elem in neg {
                        if **pos_elem != **neg_elem {
                            out.insert((pos_elem.clone(), neg_elem.clone()));
                        }
                    }
                }
            }
        }
    }
}



#[cfg(test)]
mod test {
    use std::collections::BTreeSet;
    use std::rc::Rc;

    use crate::nf::Clause;
    use crate::res::database::ClauseDatabase;
    use crate::test::TestContext;

    fn pairs(ctx: &mut TestContext, v: Vec<(&str, &str)>) -> BTreeSet<(Rc<Clause>, Rc<Clause>)> {
        let mut new = BTreeSet::new();

        for (l, r) in v {
            new.insert((Rc::new(ctx.clause(l)), Rc::new(ctx.clause(r))));
        }

        new
    }
    
    #[test]
    fn test_1() {
        let mut ctx = TestContext::new();

        let mut db = ClauseDatabase::new();

        db.learn(ctx.clause("P(:x) | P(a) | Q(a)"));
        db.learn(ctx.clause("!P(:x)"));
        db.learn(ctx.clause("!Q(b)"));

        let expected = pairs(&mut ctx, vec![
            ("P(:x) | P(a) | Q(a)", "!P(:x)"),
            ("P(:x) | P(a) | Q(a)", "!Q(b)"),
        ]);

        let mut actual = BTreeSet::new();
        db.resolution_candidates(&mut actual);


        assert_eq!(expected, actual);
    }
}
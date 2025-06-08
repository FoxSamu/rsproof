use crate::expr::{Name, Names};
use crate::fmt::DisplayNamed;
use crate::nf::{Atom, Clause, PredicateIndex};
use crate::uni::Unifier;

#[derive(Debug, PartialEq, Eq)]
pub struct Resolvee {
    pub a: Atom,
    pub b: Atom,
    pub mgu: Unifier
}

fn find_resolvee_by_name(a: &PredicateIndex, b: &PredicateIndex, name: Name) -> Option<Resolvee> {
    let a_preds = a.get_preds(&name)?;
    let b_preds = b.get_preds(&name)?;

    let mut resolvee = None;
    
    for a_pred in a_preds {
        for b_pred in b_preds {
            if let Some(mgu) = Unifier::mgu(a_pred, b_pred) {
                match resolvee {
                    None => resolvee = Some(Resolvee {
                        a: Atom::Pred(name, a_pred.clone()),
                        b: Atom::Pred(name, b_pred.clone()),
                        mgu
                    }),

                    Some(_) => return None
                }
            }
        }
    }

    resolvee
}

fn find_resolvee_index(a: &PredicateIndex, b: &PredicateIndex) -> Option<Resolvee> {
    let mut resolvee = None;

    for name in a.iter_pred_names() {
        if b.contains_preds(name) {
            match resolvee {
                None => resolvee = find_resolvee_by_name(&a, &b, *name),

                Some(_) => return None // Second candidate
            }
        }
    }

    resolvee
}

pub fn find_resolvee(a: &Clause, b: &Clause) -> Option<Resolvee> {
    None
        .or_else(|| find_resolvee_index(a.pos(), b.neg()))
        .or_else(|| find_resolvee_index(a.neg(), b.pos()))
}




impl DisplayNamed for Resolvee {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        write!(f, "Resolvee {{\n")?;
        write!(f, "    a:   {},\n", self.a.with_table(names))?;
        write!(f, "    b:   {},\n", self.b.with_table(names))?;
        write!(f, "    mgu: {}\n", self.mgu.with_table(names))?;
        write!(f, "}}")?;

        Ok(())
    }
}





#[cfg(test)]
mod test {
    use crate::res::clause::{find_resolvee, Resolvee};
    use crate::test::TestContext;

    #[test]
    fn unifies_1() {
        let mut ctx = TestContext::new();

        // Unifies by x|->a
        let a = ctx.clause("P(:x) | Q(:x)");
        let b = ctx.clause("!P(a)");
        
        let expect = Some(Resolvee {
            a: ctx.atom("P(:x)"),
            b: ctx.atom("P(a)"),
            mgu: ctx.mgu([
                ("x", "a")
            ])
        });

        let actual = find_resolvee(&a, &b);

        assert_eq!(expect, actual)
    }

    #[test]
    fn unifies_2() {
        let mut ctx = TestContext::new();

        // Unifies without substitution
        let a = ctx.clause("P(a) | Q");
        let b = ctx.clause("!P(a)");
        
        let expect = Some(Resolvee {
            a: ctx.atom("P(a)"),
            b: ctx.atom("P(a)"),
            mgu: ctx.mgu([])
        });

        let actual = find_resolvee(&a, &b);

        assert_eq!(expect, actual)
    }

    #[test]
    fn unifies_3() {
        let mut ctx = TestContext::new();

        // Unifies by x|->f(a), y|->a
        let a = ctx.clause("P(:x, :y) | Q(:x)");
        let b = ctx.clause("!P(f(:y), a()) | R(:x)");
        
        let expect = Some(Resolvee {
            a: ctx.atom("P(:x, :y)"),
            b: ctx.atom("P(f(:y), a)"),
            mgu: ctx.mgu([
                ("x", "f(a)"),
                ("y", "a")
            ])
        });

        let actual = find_resolvee(&a, &b);

        assert_eq!(expect, actual)
    }

    #[test]
    fn unifies_4() {
        let mut ctx = TestContext::new();

        // Unifies by x|->a, the other two predicates P(a) and P(:y) are not complementary
        let a = ctx.clause("P(a)");
        let b = ctx.clause("!P(:x) | P(a) | P(:y)");
        
        let expect = Some(Resolvee {
            a: ctx.atom("P(a)"),
            b: ctx.atom("P(:x)"),
            mgu: ctx.mgu([
                ("x", "a")
            ])
        });

        let actual = find_resolvee(&a, &b);

        assert_eq!(expect, actual)
    }

    #[test]
    fn does_not_unify_1() {
        let mut ctx = TestContext::new();

        // Does not unify: the predicates are different
        let a = ctx.clause("Q(:x)");
        let b = ctx.clause("!P(a)");
        
        let expect = None;

        let actual = find_resolvee(&a, &b);

        assert_eq!(expect, actual)
    }

    #[test]
    fn does_not_unify_2() {
        let mut ctx = TestContext::new();

        // Does not unify: there is no unifier that unifies a,b
        let a = ctx.clause("P(b)");
        let b = ctx.clause("!P(a)");
        
        let expect = None;

        let actual = find_resolvee(&a, &b);

        assert_eq!(expect, actual)
    }

    #[test]
    fn does_not_unify_3() {
        let mut ctx = TestContext::new();

        // Does not unify: the literals are not complementary
        let a = ctx.clause("P(:x)");
        let b = ctx.clause("P(a)");
        
        let expect = None;

        let actual = find_resolvee(&a, &b);

        assert_eq!(expect, actual)
    }

    #[test]
    fn does_not_unify_4() {
        let mut ctx = TestContext::new();

        // Does not unify: both P(:x),P(a) and P(a),P(a) can unify
        let a = ctx.clause("P(:x) | P(a)");
        let b = ctx.clause("!P(a)");
        
        let expect = None;

        let actual = find_resolvee(&a, &b);

        assert_eq!(expect, actual)
    }

    #[test]
    fn does_not_unify_5() {
        let mut ctx = TestContext::new();

        // Does not unify: both P(a),P(:x) and P(a),P(a) can unify
        let a = ctx.clause("!P(a)");
        let b = ctx.clause("P(:x) | P(a)");
        
        let expect = None;

        let actual = find_resolvee(&a, &b);

        assert_eq!(expect, actual)
    }

    #[test]
    fn does_not_unify_6() {
        let mut ctx = TestContext::new();

        // Does not unify: both P(a),P(:x) and Q(:x),Q(a) can unify
        let a = ctx.clause("P(a) | Q(:x)");
        let b = ctx.clause("P(:x) | Q(a)");
        
        let expect = None;

        let actual = find_resolvee(&a, &b);

        assert_eq!(expect, actual)
    }
}
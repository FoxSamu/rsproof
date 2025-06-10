use std::collections::BTreeSet;
use std::fmt::Debug;

use rsplib::res::Resolvee;
use rsplib::test::TestContext;

fn assert_eq_unordered<T>(a: Vec<T>, b: Vec<T>) where T : Ord + Debug {
    let a_set = BTreeSet::from_iter(a);
    let b_set = BTreeSet::from_iter(b);

    assert_eq!(a_set, b_set);
}

#[test]
fn unifies_1() {
    let mut ctx = TestContext::new();

    // Unifies by x|->a
    let a = ctx.clause("P(:x) | Q(:x)");
    let b = ctx.clause("!P(a)");
    
    let expect = vec![Resolvee {
        a: ctx.atom("P(:x)"),
        b: ctx.atom("P(a)"),
        a_neg: false,
        b_neg: true,
        mgu: ctx.mgu([
            ("x", "a")
        ])
    }];

    let actual = Resolvee::find(&a, &b);

    assert_eq_unordered(expect, actual)
}

#[test]
fn unifies_2() {
    let mut ctx = TestContext::new();

    // Unifies without substitution
    let a = ctx.clause("P(a) | Q");
    let b = ctx.clause("!P(a)");
    
    let expect = vec![Resolvee {
        a: ctx.atom("P(a)"),
        b: ctx.atom("P(a)"),
        a_neg: false,
        b_neg: true,
        mgu: ctx.mgu([])
    }];

    let actual = Resolvee::find(&a, &b);

    assert_eq_unordered(expect, actual)
}

#[test]
fn unifies_3() {
    let mut ctx = TestContext::new();

    // Unifies by x|->f(a), y|->a
    let a = ctx.clause("P(:x, :y) | Q(:x)");
    let b = ctx.clause("!P(f(:y), a()) | R(:x)");
    
    let expect = vec![Resolvee {
        a: ctx.atom("P(:x, :y)"),
        b: ctx.atom("P(f(:y), a)"),
        a_neg: false,
        b_neg: true,
        mgu: ctx.mgu([
            ("x", "f(a)"),
            ("y", "a")
        ])
    }];

    let actual = Resolvee::find(&a, &b);

    assert_eq_unordered(expect, actual)
}

#[test]
fn unifies_4() {
    let mut ctx = TestContext::new();

    // Unifies by x|->a, the other two predicates P(a) and P(:y) are not complementary
    let a = ctx.clause("P(a)");
    let b = ctx.clause("!P(:x) | P(a) | P(:y)");
    
    let expect = vec![Resolvee {
        a: ctx.atom("P(a)"),
        b: ctx.atom("P(:x)"),
        a_neg: false,
        b_neg: true,
        mgu: ctx.mgu([
            ("x", "a")
        ])
    }];

    let actual = Resolvee::find(&a, &b);

    assert_eq_unordered(expect, actual)
}

#[test]
fn unifies_5() {
    let mut ctx = TestContext::new();

    // Unifies: both P(:x),P(a) and P(a),P(a) can unify
    let a = ctx.clause("P(:x) | P(a)");
    let b = ctx.clause("!P(a)");
    
    let expect: Vec<Resolvee> = vec![
        Resolvee {
            a: ctx.atom("P(:x)"),
            b: ctx.atom("P(a)"),
            a_neg: false,
            b_neg: true,
            mgu: ctx.mgu([
                ("x", "a")
            ])
        },
        Resolvee {
            a: ctx.atom("P(a)"),
            b: ctx.atom("P(a)"),
            a_neg: false,
            b_neg: true,
            mgu: ctx.mgu([])
        }
    ];

    let actual = Resolvee::find(&a, &b);

    assert_eq_unordered(expect, actual)
}

#[test]
fn unifies_6() {
    let mut ctx = TestContext::new();

    // Unifies: both P(a),P(:x) and Q(:x),Q(a) can unify
    let a = ctx.clause("P(a) | !Q(:x)");
    let b = ctx.clause("!P(:x) | Q(a)");
    
    let expect: Vec<Resolvee> = vec![
        Resolvee {
            a: ctx.atom("P(a)"),
            b: ctx.atom("P(:x)"),
            a_neg: false,
            b_neg: true,
            mgu: ctx.mgu([
                ("x", "a")
            ])
        },
        Resolvee {
            a: ctx.atom("Q(:x)"),
            b: ctx.atom("Q(a)"),
            a_neg: true,
            b_neg: false,
            mgu: ctx.mgu([
                ("x", "a")
            ])
        },
    ];

    let actual = Resolvee::find(&a, &b);

    assert_eq_unordered(expect, actual)
}

#[test]
fn does_not_unify_1() {
    let mut ctx = TestContext::new();

    // Does not unify: the predicates are different
    let a = ctx.clause("Q(:x)");
    let b = ctx.clause("!P(a)");
    
    let expect: Vec<Resolvee> = vec![];

    let actual = Resolvee::find(&a, &b);

    assert_eq_unordered(expect, actual)
}

#[test]
fn does_not_unify_2() {
    let mut ctx = TestContext::new();

    // Does not unify: there is no unifier that unifies a,b
    let a = ctx.clause("P(b)");
    let b = ctx.clause("!P(a)");
    
    let expect: Vec<Resolvee> = vec![];

    let actual = Resolvee::find(&a, &b);

    assert_eq_unordered(expect, actual)
}

#[test]
fn does_not_unify_3() {
    let mut ctx = TestContext::new();

    // Does not unify: the literals are not complementary
    let a = ctx.clause("P(:x)");
    let b = ctx.clause("P(a)");
    
    let expect: Vec<Resolvee> = vec![];

    let actual = Resolvee::find(&a, &b);

    assert_eq_unordered(expect, actual)
}
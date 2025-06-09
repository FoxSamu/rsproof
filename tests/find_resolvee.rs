use rsplib::res::{find_resolvee, Resolvee};
use rsplib::test::TestContext;

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
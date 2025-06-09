use rsplib::test::TestContext;
use rsplib::uni::Unifier;

#[test]
fn mgu_1() {
    let mut ctx = TestContext::new();

    let a = ctx.aexprs(["f(:x)", ":y"]);
    let b = ctx.aexprs(["f(a)", ":x"]);

    let actual = Unifier::mgu(&a, &b);
    
    let expected = Some(ctx.mgu([
        ("x", "a"),
        ("y", "a")
    ]));

    assert_eq!(expected, actual);
}

#[test]
fn mgu_2() {
    let mut ctx = TestContext::new();

    let a = ctx.aexprs([":x",    "f(:z)", ":z"]);
    let b = ctx.aexprs(["f(:y)", ":y",    "a"]);

    let actual = Unifier::mgu(&a, &b);
    
    let expected = Some(ctx.mgu([
        ("x", "f(f(a))"),
        ("y", "f(a)"),
        ("z", "a"),
    ]));

    assert_eq!(expected, actual);
}

#[test]
fn mgu_3() {
    let mut ctx = TestContext::new();

    let a = ctx.aexprs([":x"]);
    let b = ctx.aexprs([":y"]);

    let actual = Unifier::mgu(&a, &b);
    
    let expected = Some(ctx.mgu([
        ("x", ":y")
    ]));

    assert_eq!(expected, actual);
}

#[test]
fn no_mgu_1() {
    let mut ctx = TestContext::new();

    let a = ctx.aexprs(["g(:x)"]);
    let b = ctx.aexprs(["f(a)"]);

    let actual = Unifier::mgu(&a, &b);
    
    let expected = None;

    assert_eq!(expected, actual);
}

#[test]
fn no_mgu_2() {
    let mut ctx = TestContext::new();

    let a = ctx.aexprs(["f(b)"]);
    let b = ctx.aexprs(["f(a)"]);

    let actual = Unifier::mgu(&a, &b);
    
    let expected = None;

    assert_eq!(expected, actual);
}

#[test]
fn no_mgu_3() {
    let mut ctx = TestContext::new();

    let a = ctx.aexprs([":x"]);
    let b = ctx.aexprs(["f(g(:x))"]);

    let actual = Unifier::mgu(&a, &b);
    
    let expected = None;

    assert_eq!(expected, actual);
}
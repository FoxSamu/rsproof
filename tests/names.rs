use std::collections::BTreeSet;

use rsplib::expr::{Name, Names};
use rsplib::test::TestContext;

fn assert_has_names(exp: impl Names, names: Vec<Name>) {
    let actual = exp.names::<BTreeSet<_>>();
    let expected = names.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(expected, actual);
}

#[test]
fn names_1() {
    let mut ctx = TestContext::new();

    assert_has_names(ctx.aexpr("f(x, :y)"), ctx.names(["f", "x", "y"]));
}

#[test]
fn names_2() {
    let mut ctx = TestContext::new();

    assert_has_names(ctx.aexpr("f(x, g(a, :x, :x))"), ctx.names(["f", "x", "g", "a"]));
}
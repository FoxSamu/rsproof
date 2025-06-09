use std::collections::BTreeSet;

use rsplib::expr::{Name, Vars};
use rsplib::test::TestContext;

fn assert_has_vars(exp: impl Vars, names: Vec<Name>) {
    let actual = exp.vars::<BTreeSet<_>>();
    let expected = names.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(expected, actual);
}

#[test]
fn names_1() {
    let mut ctx = TestContext::new();

    assert_has_vars(ctx.aexpr("f(x, :y)"), ctx.names(["y"]));
}

#[test]
fn names_2() {
    let mut ctx = TestContext::new();

    assert_has_vars(ctx.aexpr("f(:y, g(a, :x, :x, b), :z)"), ctx.names(["x", "y", "z"]));
}
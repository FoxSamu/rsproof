use rsplib::res::find_subexpr_unifiers;
use rsplib::test::TestContext;

#[test]
pub fn random1() {
    let mut ctx = TestContext::new();

    let search = ctx.aexpr("f(a, f(b, c))");
    let elem = ctx.aexpr("f(:y, :z)");

    let unifiers = find_subexpr_unifiers(search, elem);

    for (k, v) in unifiers {
        ctx.display((k, v));
    }
}
use std::collections::BTreeSet;
use std::rc::Rc;

use rsplib::nf::Clause;
use rsplib::res::ClauseDatabase;
use rsplib::test::TestContext;

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
    db.learn(ctx.clause("A"));
    db.learn(ctx.clause("B | C"));
    db.learn(ctx.clause("!A | !B"));

    let expected = pairs(&mut ctx, vec![
        ("P(:x) | P(a) | Q(a)", "!P(:x)"),
        ("P(:x) | P(a) | Q(a)", "!Q(b)"),
        ("A", "!A | !B"),
        ("B | C", "!A | !B"),
    ]);

    let mut actual = BTreeSet::new();
    db.resolution_candidates(&mut actual);

    ctx.display(&expected);
    ctx.display(&actual);

    assert_eq!(expected, actual);
}
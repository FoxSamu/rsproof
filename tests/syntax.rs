use rsplib::parser::ParseContext;


fn test_aexpr(str: &str) {
    let mut ctx = ParseContext::new();

    if let Err(err) = ctx.aexpr_valid(str) {
        panic!("Failed to parse '{str}': {err}")
    }
}

fn test_bexpr(str: &str) {
    let mut ctx = ParseContext::new();

    if let Err(err) = ctx.bexpr_valid(str) {
        panic!("Failed to parse '{str}': {err}")
    }
}

#[allow(unused)]
fn test_stmt(str: &str) {
    let mut ctx = ParseContext::new();

    if let Err(err) = ctx.stmt_valid(str) {
        panic!("Failed to parse '{str}': {err}")
    }
}

#[allow(unused)]
fn test_unifiable(str: &str) {
    let mut ctx = ParseContext::new();

    if let Err(err) = ctx.unifiable_valid(str) {
        panic!("Failed to parse '{str}': {err}")
    }
}


#[test]
fn arithmetic() {
    test_aexpr("3");
    test_aexpr("0xC0FFEE");
    test_aexpr("0b101010");
    test_aexpr("0Xc0ffee");
    test_aexpr("0B101010");
    test_aexpr("a");
    test_aexpr("-a");
    test_aexpr("+a");
    test_aexpr("+-+-a");
    test_aexpr("a + 1");
    test_aexpr("a - 1");
    test_aexpr("a - +1");
    test_aexpr("a + -1");
    test_aexpr("a * -1");
    test_aexpr("a / 2");
    test_aexpr("a % 2");
    test_aexpr("3 + 1*4");
    test_aexpr("b*b - 4*a*c");
    test_aexpr("a * b + c");
    test_aexpr("a * (b + c)");
    test_aexpr("blah * blah");
}

#[test]
fn functions() {
    test_aexpr("f(x) * g(y)");
    test_aexpr("f(g(x))");
    test_aexpr("f()");
    test_aexpr("f(a, b)");
    test_aexpr("f(99, x)");
    test_aexpr("mystery(99, blah)");
}

#[test]
fn equality() {
    test_bexpr("a < b");
    test_bexpr("a <= b");
    test_bexpr("a > b");
    test_bexpr("a >= b");
    test_bexpr("a != b");
    test_bexpr("a == b");

    test_bexpr("3+3 == 6");
    test_bexpr("3*3 == 9");
    test_bexpr("D == b*b - 4*a*c");
}

#[test]
fn logic() {
    test_bexpr("True");
    test_bexpr("False");
    test_bexpr("A");
    test_bexpr("!A");
    test_bexpr("A | B");
    test_bexpr("A & B");
    test_bexpr("A <- B");
    test_bexpr("A -> B");
    test_bexpr("A <-> B");
    test_bexpr("A|B|C|D|E|F");
    test_bexpr("!(A & B) <-> (!A | !B)");
    test_bexpr("3+3 != 5 <-> !(3+3 == 5)");

    test_bexpr("all x, y, z: x+y != z <-> !(x+y == z)");
    test_bexpr("exists x, y, z: x+y != z <-> !(x+y == z)");
    test_bexpr("no x, y, z: x+y == z <-> !(x+y == z)");

    test_bexpr("!exists x: x != x");

    test_bexpr(":x == :x");
}

use crate::cnf::Clause;
use crate::parse::parse_string;
use crate::res::resolution;

fn prove(statement: &str) -> bool {

    // Parse stdin
    let parsed = parse_string(&String::from(statement));

    if let Err(msg) = parsed {
        panic!("Syntax error: {msg}");
    }

    // Convert to CNF
    let cnf = parsed.unwrap().0.to_cnf();
    let clauses = Clause::from_cnf(&cnf);

    // Resolve
    return !resolution(&clauses).satisfied;
}

fn sat(statement: &str) {
    assert!(
        prove(statement),
        "Couldn't prove {statement}"
    )
}

fn unsat(statement: &str) {
    assert!(
        !prove(statement),
        "Couldn't disprove {statement}"
    )
}

#[test]
fn taut() {
    sat("|- *")
}

#[test]
fn cont() {
    unsat("|- ~")
}

#[test]
fn substitute1() {
    sat("a == b, P(b) |- P(a)")
}

#[test]
fn substitute2() {
    sat("a == b, P(b, a) |- P(a, b)")
}

#[test]
fn demorgan1() {
    sat("!(P | Q) |- (!P & !Q)")
}

#[test]
fn demorgan2() {
    sat("!(P & Q) |- (!P | !Q)")
}

#[test]
fn substitute3() {
    sat("P(a), a==b, P(b) <-> Q(b) |- Q(a)")
}

#[test]
fn equiv1() {
    unsat("P <-> Q, P |- !Q")
}

#[test]
fn transitivity1() {
    sat("a==b, b==c |- c==a")
}

#[test]
fn transitivity2() {
    unsat("a==b, b==c, c==d, d==e |- e!=a")
}

#[test]
fn equivs() {
    sat("A <-> (B | C), B <-> (C | D), C <-> (D | A), A, !B |- D")
}

#[test]
fn eq_cont1() {
    unsat("|- a!=a")
}

#[test]
fn eq_cont2() {
    unsat("a==b |- a!=b")
}

#[test]
fn names() {
    sat("Foo -> Bar, Bar -> Baz |- Foo -> Baz")
}

#[test]
fn long() {
    sat("A <-> B,
         B <-> C,
         C <-> D,
         D <-> E,
         E <-> F,
         F <-> G,
         G <-> H,
         H <-> I,
         I <-> J,
         J <-> K,
         K <-> L,
         L <-> M,
         M <-> N,
         N <-> O,
         O <-> P,
         P <-> Q,
         Q <-> R,
         R <-> S,
         S <-> T,
         T <-> U,
         U <-> V,
         V <-> W,
         W <-> X,
         X <-> Y,
         Y <-> Z,

         A |- Z")
}

#[test]
fn cont2() {
    unsat("P |- !P")
}
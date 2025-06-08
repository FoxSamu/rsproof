use crate::nf::NormalForm;
use crate::test::TestContext;


#[test]
fn test_equivalent_complicated() {
    let mut ctx = TestContext::new();

    let exp = ctx.bexpr("!(P & !(Q & R)) | (C & D)");
    
    /*
     * !(P & !(Q & R)) | (C & D)
     * !(P & (!Q | !R)) | (C & D)
     * (!P | (Q & R)) | (C & D)
     * !P | (Q & R) | (C & D)
     * ((!P | Q) & (!P | R)) | (C & D)
     * ((!P | Q | (C & D)) & (!P | R | (C & D)))
     * ((!P | ((Q | C) & (Q | D))) & (!P | ((R | C) & (R | D))))
     * ((((!P | Q | C) & (!P | Q | D))) & (((!P | R | C) & (!P | R | D))))
     */

    let expected = ctx.cnf("(!P | Q | C) & (!P | Q | D) & (!P | R | C) & (!P | R | D)");
    let actual = NormalForm::equiv_cnf(exp);

    assert_eq!(expected, actual);
}

#[test]
fn test_equivalent_distribute() {
    let mut ctx = TestContext::new();

    let exp = ctx.bexpr("P | (Q & R)");
    
    let expected = ctx.cnf("(P | Q) & (P | R)");
    let actual = NormalForm::equiv_cnf(exp);

    assert_eq!(expected, actual);
}

#[test]
fn test_equivalent_demorgan1() {
    let mut ctx = TestContext::new();

    let exp = ctx.bexpr("!(Q & R)");
    
    let expected = ctx.cnf("(!Q | !R)");
    let actual = NormalForm::equiv_cnf(exp);

    assert_eq!(expected, actual);
}

#[test]
fn test_equivalent_demorgan2() {
    let mut ctx = TestContext::new();

    let exp = ctx.bexpr("!(Q | R)");
    
    let expected = ctx.cnf("!Q & !R");
    let actual = NormalForm::equiv_cnf(exp);

    assert_eq!(expected, actual);
}
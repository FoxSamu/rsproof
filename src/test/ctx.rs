use crate::expr::{AExpr, BExpr, Name};
use crate::fmt::DisplayNamed;
use crate::nf::{Atom, Clause, NormalForm};
use crate::parser::ParseContext;
use crate::uni::Unifier;

/// A [TestContext] is a structure that allows one to create various instances that are typically parsed
/// and are hard to obtain by manual instantiation. The problem with these is their deep structure for
/// which many instructions are needed to set them up. The other problem is that names are represented
/// numerically, instead of using strings. The [TestContext] will make sure that names are consistent
/// across different parsing calls. E.g. if one calls [TestContext::aexpr] with `f(:x)` as input and
/// the name `x` gets bound to the number `42`, then a later [TestContext::bexpr] with `P(:x)` as input
/// will bind the name `x` to `42` again, because it notices that they are the same.
pub struct TestContext {
    pc: ParseContext
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            pc: ParseContext::new()
        }
    }

    pub fn display<D>(&self, elem: D) where D : DisplayNamed {
        println!("{}", elem.with_table(self.pc.name_table()))
    }

    pub fn aexpr(&mut self, str: &str) -> AExpr {
        self.pc.aexpr(str).unwrap()
    }

    pub fn aexprs<const N: usize>(&mut self, str: [&str; N]) -> Vec<AExpr> {
        str.into_iter().map(|e| self.aexpr(e)).collect()
    }

    pub fn bexpr(&mut self, str: &str) -> BExpr {
        self.pc.bexpr(str).unwrap()
    }

    pub fn bexprs<const N: usize>(&mut self, str: [&str; N]) -> Vec<BExpr> {
        str.into_iter().map(|e| self.bexpr(e)).collect()
    }

    pub fn atom(&mut self, str: &str) -> Atom {
        to_atom(self.bexpr(str))
    }

    pub fn atoms<const N: usize>(&mut self, str: [&str; N]) -> Vec<Atom> {
        str.into_iter().map(|e| self.atom(e)).collect()
    }

    pub fn clause(&mut self, str: &str) -> Clause {
        to_clause(self.bexpr(str))
    }

    pub fn clauses<const N: usize>(&mut self, str: [&str; N]) -> Vec<Clause> {
        str.into_iter().map(|e| self.clause(e)).collect()
    }

    pub fn cnf(&mut self, str: &str) -> NormalForm {
        to_cnf(self.bexpr(str))
    }

    pub fn cnfs<const N: usize>(&mut self, str: [&str; N]) -> Vec<NormalForm> {
        str.into_iter().map(|e| self.cnf(e)).collect()
    }

    pub fn name(&mut self, str: &str) -> Name {
        self.pc.name(str).unwrap()
    }

    pub fn names<const N: usize>(&mut self, str: [&str; N]) -> Vec<Name> {
        str.into_iter().map(|e| self.name(e)).collect()
    }

    pub fn mgu<const N: usize>(&mut self, str: [(&str, &str); N]) -> Unifier {
        let mut u = Unifier::new();

        for (k, v) in str {
            u.add(self.name(k), self.aexpr(v));
        }

        u
    }
}


fn to_atom(e: BExpr) -> Atom {
    match e {
        BExpr::Pred(name, args) => Atom::Pred(name, args),
        _ => panic!("Invalid input")
    }
}

fn to_literal(e: BExpr) -> Clause {
    match e {
        BExpr::Not(e) => Clause::from_neg(to_atom(*e)),
        e => Clause::from_pos(to_atom(e)),
    }
}

fn to_clause(e: BExpr) -> Clause {
    match e {
        BExpr::Or(lhs, rhs) => Clause::concat(to_clause(*lhs), to_clause(*rhs)),
        e => to_literal(e)
    }
}

fn to_cnf(e: BExpr) -> NormalForm {
    match e {
        BExpr::And(lhs, rhs) => NormalForm::concat(to_cnf(*lhs), to_cnf(*rhs)),
        e => NormalForm::from(to_clause(e))
    }
}
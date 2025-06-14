use std::fmt::{Debug, Display};

use crate::fmt::{write_comma_separated, DisplayNamed, NameTable};

use super::{BExpr, Names, Vars};

/// A logical statement.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct Stmt {
    premises: Vec<BExpr>,
    conclusions: Vec<BExpr>,
}

impl Stmt {
    pub fn new() -> Self {
        Self {
            premises: Vec::new(),
            conclusions: Vec::new(),
        }
    }

    pub fn from_implication(premises: Vec<BExpr>, conclusions: Vec<BExpr>) -> Self {
        Self {
            premises,
            conclusions
        }
    }

    pub fn premises(&self) -> &Vec<BExpr> {
        &self.premises
    }

    pub fn conclusions(&self) -> &Vec<BExpr> {
        &self.conclusions
    }

    pub fn into_premises(self) -> Vec<BExpr> {
        self.premises
    }

    pub fn into_conclusions(self) -> Vec<BExpr> {
        self.conclusions
    }

    /// Returns a refutable [BExpr] representing this statement, that is, it returns
    /// an expression whose unsatisfiability proves this statement.
    pub fn refutable_expr(self) -> BExpr {
        let p = to_conj(self.premises);
        let c = to_conj(self.conclusions);

        return p & !c;
    }

    /// Returns a provable [BExpr] representing this statement, that is, it returns
    /// an expression whose unsatisfiability disproves this statement.
    pub fn provable_expr(self) -> BExpr {
        let p = to_conj(self.premises);
        let c = to_conj(self.conclusions);

        return p & c;
    }
}

fn to_conj(mut expr: Vec<BExpr>) -> BExpr {
    if let Some(mut e) = expr.pop() {
        while let Some(n) = expr.pop() {
            e = e & n
        }

        e
    } else {
        BExpr::True
    }
}

impl Names for Stmt {
    fn names<A>(&self) -> A where A : FromIterator<super::Name> {
        (&self.premises, &self.conclusions).names()
    }
}

impl Vars for Stmt {
    fn vars<A>(&self) -> A where A : FromIterator<super::Name> {
        (&self.premises, &self.conclusions).vars()
    }
}

impl DisplayNamed for Stmt {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        write_comma_separated(f, names, self.premises.iter())?;
        write!(f, " |- ")?;
        write_comma_separated(f, names, self.conclusions.iter())?;
        Ok(())
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.with_table(&NameTable::new()), f)
    }
}

impl Into<(Vec<BExpr>, Vec<BExpr>)> for Stmt {
    fn into(self) -> (Vec<BExpr>, Vec<BExpr>) {
        (self.premises, self.conclusions)
    }
}
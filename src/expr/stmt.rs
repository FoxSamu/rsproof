use std::vec::IntoIter;

use super::BExpr;

/// A lemma in an input program.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub enum Lemma {
    /// A premise to be assumed as true.
    Premise(usize, BExpr),

    /// A statement to prove. If proven it will add to the knowledge base. If disproven, the opposite will add to the knowledge base.
    Prove(usize, BExpr),

    /// A statement to disprove. If proven it will add to the knowledge base. If disproven, the opposite will add to the knowledge base.
    Disprove(usize, BExpr)
}

impl Lemma {
    pub fn line(&self) -> &usize {
        match self {
            Lemma::Premise(ln, _) => ln,
            Lemma::Prove(ln, _) => ln,
            Lemma::Disprove(ln, _) => ln,
        }
    }

    pub fn expr(&self) -> &BExpr {
        match self {
            Lemma::Premise(_, expr) => expr,
            Lemma::Prove(_, expr) => expr,
            Lemma::Disprove(_, expr) => expr
        }
    }
}



#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Hash)]
pub struct Stmt {
    lemmas: Vec<Lemma> 
}

impl Stmt {
    pub fn new() -> Self {
        Self {
            lemmas: Vec::new()
        }
    }

    pub fn from_implication(premises: Vec<BExpr>, conclusions: Vec<BExpr>) -> Self {
        let mut stmt = Self::new();

        for premise in premises {
            stmt.premise(0, premise);
        }

        for conclusion in conclusions {
            stmt.prove(0, conclusion);
        }

        stmt
    }

    pub fn from_lemmas(lemmas: Vec<Lemma>) -> Self {
        Self {
            lemmas
        }
    }

    pub fn premise(&mut self, line: usize, expr: BExpr) {
        self.lemmas.push(Lemma::Premise(line, expr));
    }

    pub fn prove(&mut self, line: usize, expr: BExpr) {
        self.lemmas.push(Lemma::Prove(line, expr));
    }

    pub fn disprove(&mut self, line: usize, expr: BExpr) {
        self.lemmas.push(Lemma::Disprove(line, expr));
    }

    pub fn lemmas(&self) -> &Vec<Lemma> {
        &self.lemmas
    }

    pub fn into_lemmas(self) -> Vec<Lemma> {
        self.lemmas
    }

    pub fn iter(&self) -> impl Iterator<Item = &Lemma> {
        self.lemmas.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Lemma> {
        self.lemmas.iter_mut()
    }
}

impl From<Vec<Lemma>> for Stmt {
    fn from(value: Vec<Lemma>) -> Self {
        Self::from_lemmas(value)
    }
}

impl Into<Vec<Lemma>> for Stmt {
    fn into(self) -> Vec<Lemma> {
        self.into_lemmas()
    }
}

impl IntoIterator for Stmt {
    type Item = Lemma;

    type IntoIter = IntoIter<Lemma>;

    fn into_iter(self) -> Self::IntoIter {
        self.lemmas.into_iter()
    }
}

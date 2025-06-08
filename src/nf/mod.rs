use std::collections::BTreeSet;
use std::fmt::{Debug, Display};

use crate::expr::{AExpr, BExpr, Name, Names, Vars};
use crate::fmt::{write_comma_separated, DisplayNamed, NameTable};
use crate::uni::Unifiable;

pub use index::PredicateIndex;

pub type Atoms = BTreeSet<Atom>;
pub type Clauses = BTreeSet<Clause>;


/// Module for converting expressions to equivalent CNF/DNF.
#[allow(dead_code)]
mod equiv_nf;

/// Module for converting expressions to Tseitin DNF/CNF (an equisatisfiable but inequivalent DNF/CNF).
#[allow(dead_code)]
mod tseitin_nf;

/// Module for atom indexing.
mod index;

#[cfg(test)]
mod test;

/// An atomic expression. Atoms are the leaves of a [BExpr] tree.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Atom {
    Pred(Name, Vec<AExpr>)
}


/// A set of literals, optionally negated. Can be interpreted as a conjunction or disjunction.
/// A clause is represented by two sets of [Atom]s, one representing all the literals without
/// negation, and one representing all the literals with negation. The sets are respectively
/// called the *positive set* and the *negative set*.
/// 
/// For example, the disjunctive clause `P | !Q | R` is represented as `pos: {P, R}, neg: {Q}`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Clause {
    pos: PredicateIndex,
    neg: PredicateIndex
}

/// Normal Form: A conjunction or disjunction of clauses. Depending on context, it can represent
/// either Conjunctive Normal Form (CNF) or Disjunctive Normal Form (DNF).
/// This struct simply represents a set of [Clause]s, therefore it can act both as CNF and DNF.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NormalForm {
    clauses: Clauses
}


impl Clause {
    /// Constructs a new empty clause.
    pub fn new() -> Self {
        Self {
            pos: PredicateIndex::new(),
            neg: PredicateIndex::new()
        }
    }

    /// Constructs a clause with just one positive literal. That is, for an atom `A` it creates
    /// the clause `(A)`.
    pub fn from_pos(atom: Atom) -> Self {
        let mut new = Self::new();
        new.add_pos(atom);
        new
    }

    /// Constructs a clause with just one negative literal. That is, for an atom `A` it creates
    /// the clause `(!A)`.
    pub fn from_neg(atom: Atom) -> Self {
        let mut new = Self::new();
        new.add_neg(atom);
        new
    }

    /// Constructs a clause from two slices of atoms, one representing the positive set, and the
    /// other representing the negative set.
    pub fn from_slices<const P: usize, const N: usize>(pos: [Atom; P], neg: [Atom; N]) -> Self {
        Self {
            pos: PredicateIndex::from(pos),
            neg: PredicateIndex::from(neg)
        }
    }

    /// Inserts an atom into the positive set of this clause. That is, for an atom `A` it
    /// adds the literal `A` to the clause.
    pub fn add_pos(&mut self, atom: Atom) -> bool {
        self.pos.insert(atom)
    }

    /// Inserts an atom into the negative set of this clause. That is, for an atom `A` it
    /// adds the literal `!A` to the clause.
    pub fn add_neg(&mut self, atom: Atom) -> bool {
        self.neg.insert(atom)
    }

    /// Borrows the positive set from this clause.
    pub fn pos(&self) -> &PredicateIndex {
        &self.pos
    }

    /// Borrows the negative set from this clause.
    pub fn neg(&self) -> &PredicateIndex {
        &self.neg
    }

    /// Borrows the positive and negative sets from this clause.
    pub fn atoms(&self) -> (&PredicateIndex, &PredicateIndex) {
        (&self.pos, &self.neg)
    }

    /// Moves out the positive set from this clause.
    pub fn into_pos(self) -> PredicateIndex {
        self.pos
    }

    /// Moves out the negative set from this clause.
    pub fn into_neg(self) -> PredicateIndex {
        self.neg
    }

    /// Moves out the positive and negative sets from this clause.
    pub fn into_atoms(self) -> (PredicateIndex, PredicateIndex) {
        (self.pos, self.neg)
    }

    /// Concatenates two clauses. This creates a new clause whose positive set is the union
    /// of the two positive sets of this and the given clause, and whose negative set is the
    /// union of the two negative sets of this and the given clause.
    pub fn concat(self, other: Self) -> Self {
        Self { 
            pos: PredicateIndex::union(self.pos, other.pos),
            neg: PredicateIndex::union(self.neg, other.neg)
        }
    }

    /// Tests if this clause is empty. In a disjunctive clause (CNF) this means the clause
    /// is a contradiction. In a conjunctive clause (DNF) this means the clause is a
    /// tautology.
    pub fn is_empty(&self) -> bool {
        // A contradiction is an empty clause
        self.pos.is_empty() && self.neg.is_empty()
    }


    /// Tests if this clause is disjoint. That is, it tests if the positive and negative
    /// sets of the clause are disjoint. When these sets are not disjoint, there is at least
    /// one atom that appears as both a positive and a negative literal in this clause.
    /// In a disjunctive clause (CNF) this means the clause is a tautology. In a conjunctive
    /// clause (DNF) this means the clause is contradiction.
    pub fn is_disjoint(&self) -> bool {
        // A tautology is a clause where positive and negative atoms are not disjoint
        self.pos.is_disjoint(&self.neg)
    }

    /// Returns the reverse of this clause, swapping the positive and negative sets. This has
    /// the effect of turning a conjunctive clause into an inverted disjunctive clause, and
    /// a disjunctive clause into an inverted conjunctive clause.
    /// 
    /// E.g. it converts `(P | !Q)` into `(!P & Q)`
    pub fn reverse(mut self) -> Self {
        std::mem::swap(&mut self.pos, &mut self.neg);
        self
    }
}

impl NormalForm {
    /// Constructs a new, empty normal form, with zero clauses.
    pub fn new() -> Self {
        Self {
            clauses: Clauses::new()
        }
    }

    /// Constructs a new clause with a single clause of a single positive literal. That is,
    /// for any atom `A` it generates the normal form `((A))`.
    pub fn from_pos(atom: Atom) -> Self {
        Self::from(Clause::from_pos(atom))
    }

    /// Constructs a new clause with a single clause of a single negative literal. That is,
    /// for any atom `A` it generates the normal form `((!A))`.
    pub fn from_neg(atom: Atom) -> Self {
        Self::from(Clause::from_neg(atom))
    }

    /// Adds a clause to this normal form.
    pub fn add(&mut self, clause: Clause) -> bool {
        self.clauses.insert(clause)
    }

    /// Tests whether the given clause is part of this normal form.
    pub fn contains(&self, clause: &Clause) -> bool {
        self.clauses.contains(clause)
    }

    /// Borrows the set of clauses of this normal form.
    pub fn clauses(&self) -> &Clauses {
        &self.clauses
    }

    /// Moves out the set of clauses of this normal form.
    pub fn into_clauses(self) -> Clauses {
        self.clauses
    }

    /// Concatenates this and the given normal form, creating a normal form with the clauses from
    /// this and from the given other normal form. Duplicate equal clauses will be removed.
    pub fn concat(self, other: Self) -> Self {
        Self {
            clauses: union(self.clauses, other.clauses)
        }
    }

    /// Returns the reverse of this normal form, reversing all clauses as by [Clause::reverse]. This
    /// has the effect of turning CNF into inverted DNF and DNF into inverted CNF.
    pub fn reverse(mut self) -> Self {
        // TODO Handle contradiction and tautology here.
        // TODO Convert to &mut self function (also in Clause).

        self.clauses = self.clauses.into_iter().map(|it| it.reverse()).collect();
        self
    }

    /// Returns whether this normal form is empty, i.e. it has zero clauses. In CNF, this means that
    /// the whole CNF evaluates to a tautology. In DNF, this means that the whole DNF evaluates to
    /// a contradiction.
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    /// Returns whether any clause in this normal form is empty. In CNF, this means the whole CNF is
    /// a contradiction. In DNF, this means the whole DNF is a tautology. See [Clause::is_empty].
    pub fn has_empty_clause(&self) -> bool {
        self.clauses.iter().any(|it| it.is_empty())
    }

    /// Returns whether any clause in this normal form is disjoint. See [Clause::is_disjoint].
    pub fn has_disjoint_clause(&self) -> bool {
        self.clauses.iter().any(|it| it.is_disjoint())
    }

    /// Removes all non-disjoint clauses from this normal form. This has the effect of removing all
    /// tautological clauses from a CNF or all contradictory clauses from a DNF. Disjoint clauses
    /// contribute nothing to a normal form.
    pub fn retain_only_disjoint(&mut self) {
        self.clauses.retain(|it| it.is_disjoint());
    }

    /// Computes an equivalent Conjunctive Normal Form. It does this by
    /// rewriting the expression using DeMorgan's law and distribution
    /// properties. Finding an equivalent CNF is an NP-hard problem, this
    /// operation will take `O(2^n)` time and space complexity.
    pub fn equiv_cnf(expr: BExpr) -> NormalForm {
        equiv_nf::cnf(expr).into()
    }

    /// Computes an equivalent Disjunctive Normal Form. It does this by
    /// rewriting the expression using DeMorgan's law and distribution
    /// properties. Finding an equivalent DNF is an NP-hard problem, this
    /// operation will take `O(2^n)` time and space complexity.
    pub fn equiv_dnf(expr: BExpr) -> NormalForm {
        equiv_nf::dnf(expr).into()
    }

    /// Computes an equisatisfiable, but not equivalent, Conjunctive Normal Form.
    /// It does this using Tseitin's transformation. The resulting CNF is satisfiable
    /// if and only if this expression is satisfiable. Finding the Tseitin
    /// transformation is a P problem, this operation will take `O(n)` time and
    /// space complexity.
    pub fn tseitin_cnf(expr: BExpr) -> NormalForm {
        tseitin_nf::cnf(expr).into()
    }

    /// Computes an equisatisfiable, but not equivalent, Disjunctive Normal Form.
    /// It does this using Tseitin's transformation. The resulting DNF is satisfiable
    /// if and only if this expression is satisfiable. Finding the Tseitin
    /// transformation is a P problem, this operation will take `O(n)` time and
    /// space complexity.
    pub fn tseitin_dnf(expr: BExpr) -> NormalForm {
        tseitin_nf::dnf(expr).into()
    }
}


fn union<T>(l: BTreeSet<T>, r: BTreeSet<T>) -> BTreeSet<T> where T : Ord {
    l.into_iter().chain(r.into_iter()).collect()
}



impl From<Atom> for Clause {
    fn from(value: Atom) -> Self {
        Self::from_pos(value)
    }
}

impl From<Atom> for NormalForm {
    fn from(value: Atom) -> Self {
        Self::from_pos(value)
    }
}

impl From<Clause> for NormalForm {
    fn from(value: Clause) -> Self {
        let mut new = Self::new();
        new.add(value);
        new
    }
}

impl From<Clauses> for NormalForm {
    fn from(value: Clauses) -> Self {
        Self {
            clauses: value
        }
    }
}

impl FromIterator<Clause> for NormalForm {
    fn from_iter<T: IntoIterator<Item = Clause>>(iter: T) -> Self {
        Self {
            clauses: iter.into_iter().collect()
        }
    }
}


impl Names for Atom {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        match self {
            Atom::Pred(name, args) => (name, args).names(),
        }
    }
}

impl Names for Clause {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        (&self.pos, &self.neg).names()
    }
}

impl Names for NormalForm {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        self.clauses.names()
    }
}

impl Vars for Atom {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        match self {
            Atom::Pred(_, args) => args.vars(),
        }
    }
}

impl Vars for Clause {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        (&self.pos, &self.neg).vars()
    }
}

impl Vars for NormalForm {
    fn vars<A>(&self) -> A where A : FromIterator<Name> {
        self.clauses.vars()
    }
}

impl Unifiable for Atom {
    fn unify(self, unifier: &crate::uni::Unifier) -> Self {
        match self {
            Atom::Pred(name, args) => Atom::Pred(name, args.unify(unifier)),
        }
    }
    
    fn can_resolve_mgu(a: &Self, b: &Self) -> bool {
        match (a, b) {
            (Atom::Pred(p, ps), Atom::Pred(q, qs)) => p == q && Vec::can_resolve_mgu(ps, qs),
        }
    }
    
    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        match self {
            Atom::Pred(_, args) => Some(args.clone())
        }
    }
}

impl Unifiable for Clause {
    fn unify(mut self, unifier: &crate::uni::Unifier) -> Self {
        self.pos = self.pos.unify(unifier);
        self.neg = self.neg.unify(unifier);
        self
    }
    
    // There exist no MGUs clauses
    fn can_resolve_mgu(_: &Self, _: &Self) -> bool {
        false
    }
    
    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        None
    }
}

impl Unifiable for NormalForm {
    fn unify(mut self, unifier: &crate::uni::Unifier) -> Self {
        self.clauses = self.clauses.unify(unifier);
        self
    }
    
    // There exist no MGUs between normal forms
    fn can_resolve_mgu(_: &Self, _: &Self) -> bool {
        false
    }
    
    fn mgu_arguments(&self) -> Option<Vec<AExpr>> {
        None
    }
}

impl DisplayNamed for Atom {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        match self {
            Atom::Pred(name, args) => {
                write!(f, "{}(", name.with_table(names))?;
                write_comma_separated(f, names, args.iter())?;
                write!(f, ")")?;
            }
        }

        Ok(())
    }
}

impl DisplayNamed for Clause {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        write!(f, "(")?;
        let mut comma = false;

        for (name, args) in self.pos.iter_preds() {
            if comma { write!(f, ", ")?; } else { comma = true; }

            name.fmt_named(f, names)?;
            write!(f, "(")?;
            write_comma_separated(f, names, args.iter())?;
            write!(f, ")")?;
        };

        for (name, args) in self.neg.iter_preds() {
            if comma { write!(f, ", ")?; } else { comma = true; }

            write!(f, "!")?;
            name.fmt_named(f, names)?;
            write!(f, "(")?;
            write_comma_separated(f, names, args.iter())?;
            write!(f, ")")?;
        };

        write!(f, ")")?;

        Ok(())
    }
}

impl DisplayNamed for NormalForm {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        self.clauses.fmt_named(f, names)
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.with_table(&NameTable::new()), f)
    }
}

impl Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.with_table(&NameTable::new()), f)
    }
}

impl Display for NormalForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.with_table(&NameTable::new()), f)
    }
}

impl Debug for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.with_table(&NameTable::new()), f)
    }
}

impl Debug for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.with_table(&NameTable::new()), f)
    }
}

impl Debug for NormalForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.with_table(&NameTable::new()), f)
    }
}

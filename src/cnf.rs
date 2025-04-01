use std::collections::BTreeSet;
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::expr::Expr;
use crate::expr::Expr::*;


/// A clause is a disjunction of atom expressions. That is, this struct represents a conjunction of symbols,
/// which are optionally negated. In a clean clause, there are no tautologies like `P | !P`.
/// 
/// Clauses are represented by a set of non-negated (positive) symbols and a set of negated (negative)
/// symbols. These sets are disjunct if the clause is clean. If both these sets are empty, the clause
/// represents a contradiction.
#[derive(PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Clause {
    /// The set of positive symbols in this clause
    pub pos: BTreeSet<Term>,

    /// The set of negative symbols in this clause
    pub neg: BTreeSet<Term>
}

// Hashing for clauses.
impl Hash for Clause {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for c in &self.pos {
            c.hash(state);
        }
        for c in &self.neg {
            c.hash(state);
        }
    }
}

// Formatting for clauses, which prints them as nice expressions
impl Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for c in &self.pos {
            if first {
                first = false;
            } else {
                write!(f, " | ")?;
            }

            write!(f, "{c}")?;
        }

        for c in &self.neg {
            if first {
                first = false;
            } else {
                write!(f, " | ")?;
            }

            write!(f, "!{c}")?;
        }

        Ok(())
    }
}

impl Debug for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}


/// Creates a new [BTreeSet] that is the union of the two given sets. That is, the returned set
/// will contain an element `x` if and only if `l` contains `x` or `r` contains `x`.
fn union_b<T>(l: BTreeSet<T>, r: BTreeSet<T>) -> BTreeSet<T> where T : Ord {
    let mut o = BTreeSet::new();
    o.extend(l.into_iter());
    o.extend(r.into_iter());
    o
}


/// Creates a new [BTreeSet] that is the difference of the two given sets. That is, the returned set
/// will contain an element `x` if and only if `l` contains `x` and `r` does not contain `x`.
fn difference_b<T>(l: BTreeSet<T>, r: BTreeSet<T>) -> BTreeSet<T> where T : Ord {
    let mut o = BTreeSet::new();
    o.extend(l.into_iter());
    for e in r {
        o.remove(&e);
    }
    o
}


#[allow(dead_code)]
impl Clause {
    /// Creates a positive [Clause] with just one symbol.
    pub fn from_pos(c: Term) -> Self {
        Self { pos: BTreeSet::from([c]), neg: BTreeSet::from([]) }
    }

    /// Creates a negative [Clause] with just one symbol.
    pub fn from_neg(c: Term) -> Self {
        Self { pos: BTreeSet::from([]), neg: BTreeSet::from([c]) }
    }

    /// Creates a [Clause] from an atomic [Expr]. It optionally negates the
    /// clause if `neg` is `true`.
    /// The method will panic if the expression is not atomic.
    fn from_atom(e: &Expr, neg: bool) -> Clause {
        match e {
            Pred(s, v) => if neg {
                Self::from_neg(Term::Predicate(*s, v.clone()))
            } else {
                Self::from_pos(Term::Predicate(*s, v.clone()))
            },
            Eq(l, r) => if neg {
                Self::from_neg(Term::Equality(*l, *r))
            } else {
                Self::from_pos(Term::Equality(*l, *r))
            },
            Not(e) => Self::from_atom(e, !neg),
            e => panic!("Not in CNF: {e} is not an atom")
        }
    }

    /// Creates a [Clause] from an [Expr].
    /// The method will panic if the expression is not a clause.
    fn from_clause(e: &Expr) -> Clause {
        match e {
            Or(l, r) => Self::from_union(&Self::from_clause(l), &Self::from_clause(r)).cleanup(),
            e => Self::from_atom(e, false)
        }
    }

    /// Creates a set of [Clause]s given an [Expr] in CNF, see [Expr::to_cnf].
    /// The method will panic if the expression is not in CNF.
    pub fn from_cnf(e: &Expr) -> BTreeSet<Clause> {
        match e {
            And(l, r) => union_b(Self::from_cnf(l), Self::from_cnf(r)),
            e => {
                let clause = Self::from_clause(e);
                if !clause.is_empty() {
                    BTreeSet::from([clause])
                } else {
                    // If the clause is empty, it is a tautology in this case, so it does not contribute
                    // to the CNF in any way that makes sense
                    BTreeSet::from([])
                }
            }
        }
    }

    /// Creates a [Clause] as the union of two clauses. The returned clause will have all the terms of both
    /// clauses, but with all tautologies removed. For example, the union clause of `P | Q` and `!Q | R | S` is
    /// `P | R | R` - the terms `Q` and `!Q` formed a tautology and have thus disappeared from the union.
    pub fn from_union(l: &Self, r: &Self) -> Self {
        Self {
            pos: union_b(l.pos.clone(), r.pos.clone()),
            neg: union_b(l.neg.clone(), r.neg.clone())
        }
    }

    /// Cleans up this clause. That is, it removes all tautologies like `P | !P` from the clause.
    pub fn cleanup(self) -> Self {
        Self {
            pos: difference_b(self.pos.clone(), self.neg.clone()),
            neg: difference_b(self.neg.clone(), self.pos.clone())
        }
    }

    /// Tests whether this clause is empty. A clause is empty if and only it does not have any terms.
    pub fn is_empty(&self) -> bool {
        self.neg.is_empty() && self.pos.is_empty()
    }

    /// If this clause is a single-term positive clause, returns its term.
    pub fn pos_singleton(&self) -> Option<Term> {
        if self.neg.is_empty() && self.pos.len() == 1 {
            self.pos.iter().next().cloned()
        } else {
            None
        }
    }

    /// If this clause is a single-term negative clause, returns its term.
    pub fn neg_singleton(&self) -> Option<Term> {
        if self.pos.is_empty() && self.neg.len() == 1 {
            self.neg.iter().next().cloned()
        } else {
            None
        }
    }

    /// Returns the complexity of this clause. The complexity of a clause is the amount of terms of the clause. It
    /// is a measure of how far the clause is away from being a contradiction.
    pub fn complexity(&self) -> usize {
        self.neg.len() + self.pos.len()
    }
    
    /// Substitutes all occurences of the symbol `from` with the symbol `to` in this clause.
    pub fn substitute(self, from: u64, to: u64) -> Self {
        Self {
            pos: self.pos.into_iter().map(|t| t.substitute(from, to)).collect(),
            neg: self.neg.into_iter().map(|t| t.substitute(from, to)).collect(),
        }
    }
}




fn sub_name(n: u64, from: u64, to: u64) -> u64 {
    if from == n {
        to
    } else {
        n
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Term {
    Predicate(u64, Vec<u64>),
    Equality(u64, u64),
}

impl Term {
    pub fn substitute(self, from: u64, to: u64) -> Self {
        match self {
            Term::Predicate(n, v) => Term::Predicate(n, v.into_iter().map(|n| sub_name(n, from, to)).collect()),
            Term::Equality(l, r) => Term::Equality(sub_name(l, from, to), sub_name(r, from, to)),
        }
    }

    pub fn is_tautology(&self) -> bool {
        if let Term::Equality(l, r) = self {
            l == r
        } else {
            false
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Predicate: P(a, b, ...)
            Term::Predicate(n, v) => {
                write!(f, "{n}(")?;
                let mut first = true;
                for e in v {
                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }

                    write!(f, "{e}")?;
                }
                write!(f, ")")?;
            },
            Term::Equality(l, r) => {
                write!(f, "{l} == {r}")?;
            }
        }

        Ok(())
    }
}

impl Debug for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}
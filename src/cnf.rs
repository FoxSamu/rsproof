use std::collections::BTreeSet;
use std::fmt::Display;
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
    pos: BTreeSet<char>,
    neg: BTreeSet<char>
}

// Hashing for clauses.
impl Hash for Clause {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // This hash function works in such a way that order of symbols does not matter
        let mut ph = 0u32;
        let mut nh = 0u32;

        for c in &self.pos {
            ph += *c as u32;
        }
        for c in &self.neg {
            nh += *c as u32;
        }

        ph.hash(state);
        nh.hash(state);
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


impl Clause {
    /// Creates a positive [Clause] with just one symbol.
    pub fn from_pos(c: char) -> Self {
        Self { pos: BTreeSet::from([c]), neg: BTreeSet::from([]) }
    }

    /// Creates a negative [Clause] with just one symbol.
    pub fn from_neg(c: char) -> Self {
        Self { pos: BTreeSet::from([]), neg: BTreeSet::from([c]) }
    }

    /// Creates a [Clause] from an atomic [Expr]. It optionally negates the
    /// clause if `neg` is `true`.
    /// The method will panic if the expression is not atomic.
    fn from_atom(e: &Expr, neg: bool) -> Clause {
        match e {
            Symbol(s) => if neg { Self::from_neg(*s) } else { Self::from_pos(*s) },
            Not(e) => Self::from_atom(e, !neg),
            e => panic!("Not in CNF: {e} is not an atom")
        }
    }

    /// Creates a [Clause] from an [Expr].
    /// The method will panic if the expression is not a clause.
    fn from_clause(e: &Expr) -> Clause {
        match e {
            Or(l, r) => Self::from_union(&Self::from_clause(l), &Self::from_clause(r)),
            e => Self::from_atom(e, false)
        }
    }

    /// Creates a set of [Clause]s given an [Expr] in CNF, see [Expr::to_cnf].
    /// The method will panic if the expression is not in CNF.
    pub fn from_cnf(e: &Expr) -> BTreeSet<Clause> {
        match e {
            And(l, r) => union_b(Self::from_cnf(l), Self::from_cnf(r)),
            e => BTreeSet::from([Self::from_clause(e)])
        }
    }

    /// Creates a [Clause] as the union of two clauses. The returned clause will have all the terms of both
    /// clauses, but with all tautologies removed. For example, the union clause of `P | Q` and `!Q | R | S` is
    /// `P | R | R` - the terms `Q` and `!Q` formed a tautology and have thus disappeared from the union.
    pub fn from_union(l: &Self, r: &Self) -> Self {
        Self {
            pos: union_b(l.pos.clone(), r.pos.clone()),
            neg: union_b(l.neg.clone(), r.neg.clone())
        }.cleanup()
    }

    /// Cleans up this clause. That is, it removes all tautologies like `P | !P` from the clause.
    pub fn cleanup(self) -> Self {
        Self {
            pos: difference_b(self.pos.clone(), self.neg.clone()),
            neg: difference_b(self.neg.clone(), self.pos.clone())
        }
    }

    /// Tests whether this clause is a contradiction. A clause is a contradiction if and only it does not have
    /// any terms. That is, a clause `c` can be seen as `c | false`, so if `c` is empty then `(c | false) -> false`.
    pub fn is_contradiction(&self) -> bool {
        self.neg.is_empty() && self.pos.is_empty()
    }

    /// Returns the complexity of this clause. The complexity of a clause is the amount of terms of the clause. It
    /// is a measure of how far the clause is away from being a contradiction.
    pub fn complexity(&self) -> usize {
        self.neg.len() + self.pos.len()
    }
}
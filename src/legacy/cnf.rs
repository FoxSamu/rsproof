use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Debug, Display, Write};
use std::hash::Hash;

use super::expro::{Expr, Name, Term};
use super::expro::Expr::*;
use super::fmto::NamedDisplay;
use super::unify::{unify, Unifiable, Unifier};


/// A clause is a disjunction of atom expressions. That is, this struct represents a conjunction of symbols,
/// which are optionally negated. In a clean clause, there are no tautologies like `P | !P`.
/// 
/// Clauses are represented by a set of non-negated (positive) symbols and a set of negated (negative)
/// symbols. These sets are disjunct if the clause is clean. If both these sets are empty, the clause
/// represents a contradiction.
#[derive(PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Clause {
    /// The set of positive symbols in this clause
    pub pos: BTreeSet<Atom>,

    /// The set of negative symbols in this clause
    pub neg: BTreeSet<Atom>
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
    pub fn empty() -> Self {
        Self { pos: BTreeSet::from([]), neg: BTreeSet::from([]) }
    }

    /// Creates a positive [Clause] with just one symbol.
    pub fn from_pos(c: Atom) -> Self {
        Self { pos: BTreeSet::from([c]), neg: BTreeSet::from([]) }
    }

    /// Creates a negative [Clause] with just one symbol.
    pub fn from_neg(c: Atom) -> Self {
        Self { pos: BTreeSet::from([]), neg: BTreeSet::from([c]) }
    }

    /// Creates a [Clause] from an atomic [Expr]. It optionally negates the
    /// clause if `neg` is `true`.
    /// The method will panic if the expression is not atomic.
    fn from_atom(e: &Expr, neg: bool) -> Clause {
        match e {
            Pred(s, v) => if neg {
                Self::from_neg(Atom::Predicate(*s, v.clone()))
            } else {
                Self::from_pos(Atom::Predicate(*s, v.clone()))
            },
            Eq(l, r) => if neg {
                Self::from_neg(Atom::Equality(l.clone(), r.clone()))
            } else {
                Self::from_pos(Atom::Equality(l.clone(), r.clone()))
            },
            True => if neg {
                Self::from_neg(Atom::Tautology)
            } else {
                Self::from_pos(Atom::Tautology)
            },
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
            e => {
                let clause = Self::from_clause(e);
                if !clause.is_tautology() {
                    BTreeSet::from([clause])
                } else {
                    // If the clause is a tautology, it does not contribute
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

    /// Checks if the clause is a tautology. The positive and negative sets are not disjunct if the clause is a
    /// tautology.
    pub fn is_tautology(&self) -> bool {
        self.pos.intersection(&self.neg).next() != None
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
    pub fn pos_singleton(&self) -> Option<Atom> {
        if self.neg.is_empty() && self.pos.len() == 1 {
            self.pos.iter().next().cloned()
        } else {
            None
        }
    }

    /// If this clause is a single-term negative clause, returns its term.
    pub fn neg_singleton(&self) -> Option<Atom> {
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
    pub fn substitute(self, from: Name, to: Name) -> Self {
        Self {
            pos: self.pos.into_iter().map(|t| t.substitute(from, to)).collect(),
            neg: self.neg.into_iter().map(|t| t.substitute(from, to)).collect(),
        }
    }

    pub fn write_named(&self, name_table: &BTreeMap<Name, String>, str: &mut String) {
        let mut first = true;
        for c in &self.pos {
            if first {
                first = false;
            } else {
                str.push_str(" | ");
            }

            c.write_named(name_table, str, false);
        }

        for c in &self.neg {
            if first {
                first = false;
            } else {
                str.push_str(" | ");
            }

            c.write_named(name_table, str, true);
        }
    }
}

impl Unifiable for Clause {
    fn apply(self, unifier: &Unifier) -> Self {
        Self { pos: self.pos.apply(unifier), neg: self.neg.apply(unifier) }
    }
}




#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Atom {
    Predicate(Name, Vec<Term>),
    Equality(Term, Term),
    Tautology
}

impl Atom {
    pub fn mgu(&self, other: &Atom) -> Option<Unifier> {
        match (self, other) {
            (Atom::Predicate(ln, la), Atom::Predicate(rn, ra)) => {
                if ln == rn {
                    unify(la, ra)
                } else {
                    None
                }
            }

            (Atom::Equality(ll, lr), Atom::Equality(rl, rr)) => {
                unify(&vec![ll.clone(), lr.clone()], &vec![rl.clone(), rr.clone()])
            }

            (Atom::Tautology, Atom::Tautology) => {
                Some(Unifier::new())
            }

            _ => None
        }
    }

    pub fn substitute(self, from: Name, to: Name) -> Self {
        match self {
            Atom::Predicate(n, v) => Atom::Predicate(n, v.into_iter().map(|n| n.substitute(from, to)).collect()),
            Atom::Equality(l, r) => Atom::Equality(l.substitute(from, to), r.substitute(from, to)),
            Atom::Tautology => Atom::Tautology,
        }
    }

    pub fn is_tautology(&self) -> bool {
        match self {
            Atom::Equality(l, r) => l == r,
            Atom::Predicate(_, _) => false,
            Atom::Tautology => true,
        }
    }

    pub fn write_named(&self, name_table: &BTreeMap<Name, String>, str: &mut String, neg: bool) {
        fn write_name(str: &mut String, name_table: &BTreeMap<Name, String>, name: Name) {
            let s_name = name_table.get(&name);
            match s_name {
                Some(n) => str.extend(n.chars()),
                None => str.extend(format!("{name}").chars()),
            }
        }

        fn write_term(str: &mut String, name_table: &BTreeMap<Name, String>, term: &Term) {
            let s_term = format!("{}", term.named(name_table));
            str.push_str(&s_term);
        }

        match self {
            Atom::Predicate(n, v) => {
                if neg {
                    str.push('!');
                }

                write_name(str, name_table, *n);

                if !v.is_empty() {
                    str.push('(');
                    let mut first = true;
                    for vn in v {
                        if first {
                            first = false;
                        } else {
                            str.push_str(", ");
                        }

                        write_term(str, name_table, vn);
                    }
                    str.push(')');
                }
            },
            Atom::Equality(l, r) => {
                write_term(str, name_table, l);
                if neg {
                    str.push_str(" != ");
                } else {
                    str.push_str(" == ");
                }
                write_term(str, name_table, r);
            },
            Atom::Tautology => str.push('*'),
        }
    }
}

impl Unifiable for Atom {
    fn apply(self, unifier: &Unifier) -> Self {
        match self {
            Atom::Predicate(n, v) => Atom::Predicate(n, v.apply(unifier)),
            Atom::Equality(l, r) => Atom::Equality(l.apply(unifier), r.apply(unifier)),
            Atom::Tautology => Atom::Tautology
        }
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        NamedDisplay::fmt_raw(&self, f)
    }
}

impl Debug for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        NamedDisplay::fmt_raw(&self, f)
    }
}

impl NamedDisplay for Atom {
    fn named_fmt(&self, f: &mut super::fmto::NamedFormatter) -> std::fmt::Result {
        match self {
            // Predicate: P(a, b, ...)
            Atom::Predicate(n, v) => {
                f.write_name(*n)?;
                f.write_char('(')?;

                let mut first = true;
                for e in v {
                    if first {
                        first = false;
                    } else {
                        f.write_str(", ")?;
                    }

                    e.named_fmt(f)?;
                }
                f.write_char(')')?;
            },
            Atom::Equality(l, r) => {
                l.named_fmt(f)?;
                f.write_str(" == ")?;
                r.named_fmt(f)?;
            },
            Atom::Tautology => {
                write!(f, "*")?;
            }
        }

        Ok(())
    }
}
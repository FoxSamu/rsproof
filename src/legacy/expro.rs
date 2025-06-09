use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter, Write};

use Expr::*;
use Term::*;

use super::fmto::{NamedDisplay, NamedFormatter};
use super::unify::{Unifiable, Unifier};


pub type Name = u64;

/// An expression syntax tree.
#[derive(Hash, PartialEq, Eq, Clone)]
pub enum Expr {
    /// A predicate symbol, e.g. `P` or `Q`.
    Pred(Name, Vec<Term>),

    /// An equality, i.e. `... == ...`
    Eq(Term, Term),

    /// A tautology
    True,

    /// An inverted expression, i.e. `!...`.
    Not(Box<Expr>),

    /// A conjunction, i.e. `... & ...`.
    And(Box<Expr>, Box<Expr>),

    /// A disjunction, i.e. `... | ...`.
    Or(Box<Expr>, Box<Expr>)
}

/// A term denotes a non-boolean value.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Term {
    Const(Name),
    Var(Name),
    Func(Name, Vec<Term>)
}

impl Term {
    pub fn contains_var(&self, var: &Name) -> bool {
        match self {
            Const(_) => false,
            Var(v) => *v == *var,
            Func(_, args) => args.iter().any(|t| t.contains_var(var)),
        }
    }

    pub fn vars(&self) -> BTreeSet<Name> {
        match self {
            Const(_) => BTreeSet::new(),
            Var(v) => BTreeSet::from([*v]),
            Func(_, args) => args.iter().flat_map(|t| t.vars()).collect()
        }
    }

    pub fn unify(&self, unifier: &Unifier) -> Term {
        match self {
            Const(cst) => Const(*cst),
            Var(var) => unifier.get(var).cloned().unwrap_or(Var(*var)),
            Func(fun, args) => Func(*fun, args.into_iter().map(|e| e.unify(unifier)).collect()),
        }
    }

    pub fn substitute(&self, from: Name, to: Name) -> Term {
        fn map(name: Name, from: Name, to: Name) -> Name {
            if name == from {
                to
            } else {
                name
            }
        }

        match self {
            Const(cst) => Const(map(*cst, from, to)),
            Var(var) => Var(map(*var, from, to)),
            Func(fun, args) => Func(*fun, args.into_iter().map(|e| e.substitute(from, to)).collect()),
        }
    }
}

impl Unifiable for Term {
    fn apply(self, unifier: &Unifier) -> Self {
        match self {
            Const(cst) => Const(cst),
            Var(var) => unifier.get(&var).cloned().unwrap_or(Var(var)),
            Func(fun, args) => Func(fun, args.apply(unifier)),
        }
    }
}

/// Creates a tautology expression.
pub fn taut() -> Expr {
    True
}

/// Creates a contradiction expression.
pub fn cont() -> Expr {
    not(True)
}

/// Creates a disjunction of two expressions. That is, it creates the expression `l | r`.
pub fn or(l: Expr, r: Expr) -> Expr {
    Or(Box::new(l), Box::new(r))
}

/// Creates a conjunction of two expressions. That is, it creates the expression `l & r`.
pub fn and(l: Expr, r: Expr) -> Expr {
    And(Box::new(l), Box::new(r))
}

/// Creates an inversion of an expression. That is, it creates the expression `!n`.
pub fn not(n: Expr) -> Expr {
    Not(Box::new(n))
}

/// Creates a symbol expression, where the given name `p` is the symbol.
pub fn sym(p: Name) -> Expr {
    Pred(p, vec![])
}

/// Creates an equality expression.
pub fn eq(l: Term, r: Term) -> Expr {
    Eq(l, r)
}

/// Creates an inequality expression.
pub fn neq(l: Term, r: Term) -> Expr {
    not(Eq(l, r))
}

/// Creates a predicate expression.
pub fn pred(p: Name, args: Vec<Term>) -> Expr {
    Pred(p, args)
}

/// Creates an implication of two expressions. That is, it creates the expression `l -> r`, in other words `!l | r`.
pub fn imp(l: Expr, r: Expr) -> Expr {
    or(not(l), r)
}

/// Creates an equivalence of two expressions. That is, it creates the expression `l <-> r`, in other words `(!l | r) & (l | !r)`.
pub fn equiv(l: Expr, r: Expr) -> Expr {
    and(imp(l.clone(), r.clone()), imp(r.clone(), l.clone()))
}

/// Creates an exclusive or of two expressions. That is, it creates the expression `l ^ r`, in other words `(l | r) & !(l & r)`
pub fn xor(l: Expr, r: Expr) -> Expr {
    and(or(l.clone(), r.clone()), not(and(l.clone(), r.clone())))
}

impl Expr {
    /// Tests if the given expression is an atom. That is, the expression must be a symbol or
    /// an inverted atom. `P` and `!!!P` are atoms, but `!(P | !Q)` is not.
    fn is_atom(&self) -> bool {
        match self {
            Pred(_, _) | Eq(_, _) | True => true,
            Not(n) => n.is_atom(),
            _ => false
        }
    }

    /// Tests if the given expression is a clause. That is, the expression must be a disjunction
    /// of atoms. `P` and `P | (!!Q | !R)` are clauses, but `!(P | Q & R)` is not. See [is_atom].
    fn is_clause(&self) -> bool {
        match self {
            Or(l, r) => l.is_clause() && r.is_clause(),
            e => e.is_atom()
        }
    }

    /// Tests if the given expression is in conjunctive normal form (CNF). That is, the expression
    /// must be a conjunction of clauses. `(!P & (R | !S)) & Q` is in CNF, but `(P & Q) | R` is not.
    /// See [is_clause].
    fn is_cnf(&self) -> bool {
        match self {
            And(l, r) => l.is_cnf() && r.is_cnf(),
            e => e.is_clause()
        }
    }

    /// Recursively applies DeMorgan's laws. This causes all negations to propagate down so that all
    /// negations apply only to atoms (see [is_atom]). The returned expression will not have any
    /// negated conjunctions or disjunctions, but it is semantically equivalent.
    fn demorgan_pos(self) -> Self {
        match self {
            And(l, r) => and(l.demorgan_pos(), r.demorgan_pos()),
            Or(l, r) => or(l.demorgan_pos(), r.demorgan_pos()),
            Not(n) => n.demorgan_neg(),
            e => e
        }
    }

    /// Recursively applies DeMorgan's laws like [demorgan_pos], but negates this expression before doing
    /// so. This is different from manually inverting the expression and calling [demorgan_pos]: manually
    /// inverting just wraps the whole expression into an [Expr::Not], whereas this will apply the law of
    /// double negation and DeMorgan's law to perform a negation.
    fn demorgan_neg(self) -> Self {
        match self {
            And(l, r) => or(l.demorgan_neg(), r.demorgan_neg()),
            Or(l, r) => and(l.demorgan_neg(), r.demorgan_neg()),
            Not(n) => n.demorgan_pos(),
            e => not(e)
        }
    }

    /// Attempts to distribute `a` over `b`. If `b` is of the form `l & r`, this
    /// will return an expression of the form `(a | l) & (a | r)`.
    /// [None] will be returned if `b` is not of the form `l & r`.
    fn distribute_base(a: &Box<Self>, b: &Box<Self>) -> Option<Self> {
        let a1 = *a.clone();
        let b1 = *b.clone();
        if let And(l, r) = b1 {
            Some(and(or(a1.clone(), *l), or(a1.clone(), *r)))
        } else {
            None
        }
    }

    /// Attepts to distribute a disjunction over a conjunction. If this expression is of the form
    /// `(a | (l & r)` or `((l & r) | a)`, it will return the expression `(a | l) & (a | r)`. If the expression
    /// is not of that form, it will return this expression unchanged. The returned expression will always be
    /// semantically equivalent to this expression.
    fn distribute(self) -> Self {
        match self {
            Or(l, r) => {
                if let Some(e) = Self::distribute_base(&l, &r) {
                    e
                } else if let Some(e) = Self::distribute_base(&r, &l) {
                    e
                } else {
                    or(*l, *r)
                }
            },
            e => e
        }
    }

    /// Applies [distribute] recursively to every [Expr] in the syntax tree of this expression. When this method
    /// is repeatedly applied to an expression, provided that the expression has been transformed by [demorgan_pos],
    /// the expression will eventually become conjunctive normal form.
    fn recursive_distribute(self) -> Self {
        match self.distribute() {
            Not(n) => not(n.recursive_distribute()),
            And(l, r) => and(l.recursive_distribute(), r.recursive_distribute()),
            Or(l, r) => or(l.recursive_distribute(), r.recursive_distribute()),
            e => e
        }
    }

    /// Creates a new expression that is semantically equivalent to this expression, but that is in conjunctive normal
    /// form (CNF). In conjunctive normal form, disjunctions and conjunctions never appear negated, and the terms of
    /// disjuncts are never a conjunct. That is, an expression in CNF can be seen as a conjunction of disjunctions of
    /// atoms.
    pub fn to_cnf(&self) -> Self {
        let mut this = self.clone();

        this = this.demorgan_pos();

        while !this.is_cnf() {
            this = this.recursive_distribute();
        }

        this
    }
}

impl Unifiable for Expr {
    fn apply(self, unifier: &Unifier) -> Self {
        match self {
            Pred(pred, args) => Pred(pred, args.apply(unifier)),
            Eq(lhs, rhs) => Eq(lhs.apply(unifier), rhs.apply(unifier)),
            True => True,
            Not(rhs) => not((*rhs).apply(unifier)),
            And(lhs, rhs) => and((*lhs).apply(unifier), (*rhs).apply(unifier)),
            Or(lhs, rhs) => or((*lhs).apply(unifier), (*rhs).apply(unifier))
        }
    }
}

// Formatting for Expr, print them as nice expressions.
impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        NamedDisplay::fmt_raw(self, f)
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        NamedDisplay::fmt_raw(self, f)
    }
}

impl NamedDisplay for Expr {
    fn named_fmt(&self, f: &mut NamedFormatter) -> std::fmt::Result {
        match self {
            Pred(s, v) => {
                f.write_name(*s)?;
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
            }
            True => f.write_char('*')?,
            Not(n) => {
                f.write_char('!')?;
                f.write_named(n)?;
            },
            Eq(l, r) => {
                f.write_char('(')?;
                l.named_fmt(f)?;
                f.write_str("==")?;
                r.named_fmt(f)?;
                f.write_char(')')?;
            },
            And(l, r) => {
                f.write_char('(')?;
                f.write_named(l)?;
                f.write_char('&')?;
                f.write_named(r)?;
                f.write_char(')')?;
            },
            Or(l, r) => {
                f.write_char('(')?;
                f.write_named(l)?;
                f.write_char('|')?;
                f.write_named(r)?;
                f.write_char(')')?;
            }
        }

        Ok(())
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        NamedDisplay::fmt_raw(self, f)
    }
}

impl Debug for Term {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        NamedDisplay::fmt_raw(self, f)
    }
}

impl NamedDisplay for Term {
    fn named_fmt(&self, f: &mut NamedFormatter) -> std::fmt::Result {
        match self {
            Const(n) => {
                f.write_name(*n)?;
            },

            Var(n) => {
                f.write_char('$')?;
                f.write_name(*n)?;
            },

            Func(s, v) => {
                f.write_name(*s)?;
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
            }
        }

        Ok(())
    }
}


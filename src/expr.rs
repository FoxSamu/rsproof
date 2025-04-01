/// An expression syntax tree.
#[derive(Hash, PartialEq, Eq, Clone)]
pub enum Expr {
    /// A predicate symbol, e.g. `P` or `Q`.
    Pred(u64, Vec<u64>),

    /// An equality, i.e. `... == ...`
    Eq(u64, u64),

    /// An inverted expression, i.e. `!...`.
    Not(Box<Expr>),

    /// A conjunction, i.e. `... & ...`.
    And(Box<Expr>, Box<Expr>),

    /// A disjunction, i.e. `... | ...`.
    Or(Box<Expr>, Box<Expr>)
}

use std::fmt::{Debug, Display};

use Expr::*;

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
pub fn sym(p: u64) -> Expr {
    Pred(p, vec![])
}

/// Creates an equality expression.
pub fn eq(l: u64, r: u64) -> Expr {
    Eq(l, r)
}

/// Creates an inequality expression.
pub fn neq(l: u64, r: u64) -> Expr {
    not(Eq(l, r))
}

/// Creates a predicate expression.
pub fn pred(p: u64, args: Vec<u64>) -> Expr {
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
            Pred(_, _) => true,
            Eq(_, _) => true,
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

// Formatting for Expr, print them as nice expressions.
impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pred(s, v) => {
                write!(f, "{s}(")?;
                let mut first = true;
                for e in v {
                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }

                    write!(f, "{e}")?;
                }
                write!(f, ")")
            }
            Not(n) => write!(f, "!{n}"),
            Eq(l, r) => write!(f, "({l}=={r})"),
            And(l, r) => write!(f, "({l}&{r})"),
            Or(l, r) => write!(f, "({l}|{r})")
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

use std::collections::VecDeque;

use crate::expr::{AExpr, BExpr, Name, Names, Vars};
use crate::fmt::{write_comma_separated, DisplayNamed};
use crate::uni::{Unifiable, Unifier};

use Quantifier::*;


/// Skolemises the expression. This goes in five steps:
/// - First, any quantifier of which the variable was ignored, is simply
///   removed. E.g., `all x: P` becomes `P` and `some x: P` becomes `P`.
/// - Second, all the negations are moved inwards using DeMorgan's laws.
/// - Third, the expression is converted into Prenex Form, in which all
///   quantifiers come before the rest of the expression.
/// - Fourth, existential quantifiers are replaced with Skolem functions,
///   leaving an expression in Skolem Form, a special case of Prenex Form
///   in which there are only universal quantifiers.
/// - Fifth, all the universal quantifiers are dropped, leaving all
///   variables that were bound by these quantifiers unbound.
pub fn skolemise(e: BExpr) -> BExpr {
    PrenexForm::from(e).skolemise()
}


/// A quantifier in the prenex form.
enum Quantifier {
    Universal(Name),
    Existential(Name),
}

impl Names for Quantifier {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        match self {
            Universal(name) => name.names(),
            Existential(name) => name.names(),
        }
    }
}

impl DisplayNamed for Quantifier {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        match self {
            Universal(name) => write!(f, "all {}", name.with_table(names)),
            Existential(name) => write!(f, "some {}", name.with_table(names)),
        }
    }
}


/// The prenex form is a form of expression in which all the quantifiers are in front of the rest of the
/// expression. Every expression has an equivalent prenex form.
struct PrenexForm {
    // Deque because we push to this structure both in front and in the back.
    // Using regular vec would mean a front-push would take linear time whereas
    // a deque allows this in constant time.

    /// The prefix, i.e. the sequence of quantifiers in front of the expression
    prefix: VecDeque<Quantifier>,

    /// The matrix, i.e. the quantifier-free expression that follows the prefix
    matrix: BExpr
}

impl Names for PrenexForm {
    fn names<A>(&self) -> A where A : FromIterator<Name> {
        (&self.prefix, &self.matrix).names()
    }
}

impl DisplayNamed for PrenexForm {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        write_comma_separated(f, names, self.prefix.iter())?;
        write!(f, " : {}", self.matrix.with_table(names))?;
        Ok(())
    }
}



impl PrenexForm {
    /// Tests if this prenex form is in Skolem form, that is, whether it does not have
    /// any existential quantifiers.
    fn is_skolem_form(&self) -> bool {
        self.prefix.iter().all(|q| matches!(q, Quantifier::Universal(_)))
    }

    /// Quantifies a prenex form with universal quantification.
    fn all(mut self, name: Name) -> Self {
        self.prefix.push_front(Quantifier::Universal(name));
        self
    }

    /// Quantifies a prenex form with existential quantification.
    fn some(mut self, name: Name) -> Self {
        self.prefix.push_front(Quantifier::Existential(name));
        self
    }

    /// Computes the disjunction of two prenex forms, in prenex form.
    /// This disjunction is obtained by concatenating the prefixes and
    /// disjuncting the matrices.
    fn disjunct(mut self, mut other: Self) -> Self {
        // The prenex form of the disjuncts is simply obtained by
        // concatenating the prefixes and disjuncting the matrices.
        //
        // However, if A is in Skolem form and B is not, then it
        // would mean that the existential quantifier of B is in
        // front of all the universal quantifiers of A, causing
        // the resulting Skolem function to unnecessarily have all
        // the variables of A.
        //
        // The solution is this swap, causing the universal quantifiers
        // of A to come after the quantifiers of B and introducing no
        // unecessarily complicated Skolem functions.
        if self.is_skolem_form() && !other.is_skolem_form() {
            std::mem::swap(&mut self, &mut other);
        }

        let mut prefix = VecDeque::new();
        prefix.append(&mut self.prefix);
        prefix.append(&mut other.prefix);

        Self {
            prefix,
            matrix: self.matrix | other.matrix
        }
    }

    /// Computes the conjunction of two prenex forms, in prenex form.
    /// This conjunction is obtained by concatenating the prefixes and
    /// conjuncting the matrices.
    fn conjunct(mut self, mut other: Self) -> Self {
        // For the same reason as in `disjunct`, this swap will avoid
        // unnecessarily complicated Skolem functions.
        if self.is_skolem_form() && !other.is_skolem_form() {
            std::mem::swap(&mut self, &mut other);
        }

        let mut prefix = VecDeque::new();
        prefix.append(&mut self.prefix);
        prefix.append(&mut other.prefix);

        Self {
            prefix,
            matrix: self.matrix & other.matrix
        }
    }


    fn from(mut e: BExpr) -> PrenexForm {
        e = cleanup_useless_quantifiers(e);
        e = demorgan_pos(e);

        Self::from_raw(e)
    }

    fn from_raw(e: BExpr) -> PrenexForm {
        // We don't need to do the whole prenex conversion if
        // there are no quantifiers in this expression.
        if e.is_quantifier_free() {
            return PrenexForm {
                prefix: VecDeque::new(),
                matrix: e
            };
        }

        match e {
            BExpr::And(lhs, rhs) => Self::conjunct(Self::from_raw(*lhs), Self::from_raw(*rhs)),
            BExpr::Or(lhs, rhs) => Self::disjunct(Self::from_raw(*lhs), Self::from_raw(*rhs)),

            BExpr::All(name, rhs) => Self::all(Self::from_raw(*rhs), name),
            BExpr::Some(name, rhs) => Self::some(Self::from_raw(*rhs), name),

            // `Not` is covered here because DeMorgan should've brought all `Not`s inwards
            e => Self {
                prefix: VecDeque::new(),
                matrix: e
            }
        }
    }

    fn skolemise(self) -> BExpr {
        let mut vars = Vec::new();

        let Self {
            mut prefix,
            mut matrix
        } = self;

        while let Some(q) = prefix.pop_front() {
            match q {
                Universal(name) => {
                    vars.push(name)
                },

                Existential(name) => {
                    // We can reuse the variable name as skolem function name
                    let sk_fun = skolem_fun(name, &vars);

                    // Use a unifier that replaces the existential variable
                    // with the skolem function
                    let uni = Unifier::try_from(vec![
                        (name, sk_fun)
                    ]).unwrap();

                    matrix = matrix.unify(&uni);
                },
            }
        }

        matrix
    }
}

fn demorgan_pos(e: BExpr) -> BExpr {
    match e {
        BExpr::And(lhs, rhs) => demorgan_pos(*lhs) & demorgan_pos(*rhs),
        BExpr::Or(lhs, rhs) => demorgan_pos(*lhs) | demorgan_pos(*rhs),
        BExpr::Not(rhs) => demorgan_neg(*rhs),

        BExpr::All(name, rhs) => BExpr::all(name, demorgan_pos(*rhs)),
        BExpr::Some(name, rhs) => BExpr::some(name, demorgan_pos(*rhs)),

        e => e
    }
}

fn demorgan_neg(e: BExpr) -> BExpr {
    match e {
        BExpr::And(lhs, rhs) => demorgan_neg(*lhs) | demorgan_neg(*rhs),
        BExpr::Or(lhs, rhs) => demorgan_neg(*lhs) & demorgan_neg(*rhs),
        BExpr::Not(rhs) => demorgan_pos(*rhs),

        BExpr::All(name, rhs) => BExpr::some(name, demorgan_neg(*rhs)),
        BExpr::Some(name, rhs) => BExpr::all(name, demorgan_neg(*rhs)),

        e => !e
    }
}

fn cleanup_useless_quantifiers(e: BExpr) -> BExpr {
    match e {
        BExpr::And(lhs, rhs) => cleanup_useless_quantifiers(*lhs) & cleanup_useless_quantifiers(*rhs),
        BExpr::Or(lhs, rhs) => cleanup_useless_quantifiers(*lhs) | cleanup_useless_quantifiers(*rhs),
        BExpr::Not(rhs) => !cleanup_useless_quantifiers(*rhs),

        BExpr::All(name, rhs) => {
            let e = cleanup_useless_quantifiers(*rhs);
            if e.has_var(&name) {
                BExpr::all(name, e)
            } else {
                e
            }
        },

        BExpr::Some(name, rhs) => {
            let e = cleanup_useless_quantifiers(*rhs);
            if e.has_var(&name) {
                BExpr::some(name, e)
            } else {
                e
            }
        },

        e => e
    }
}

fn skolem_fun(name: Name, vars: &Vec<Name>) -> AExpr {
    AExpr::fun(
        name,
        vars.iter()
            .map(|name| AExpr::var(*name))
            .collect()
    )
}

#[cfg(test)]
mod test {
    use crate::nf::skolemise::skolemise;
    use crate::test::TestContext;

    #[test]
    fn test() {
        let mut ctx = TestContext::new();

        let expr = ctx.bexpr("all x: (P(x) -> Q(x)) & some x: P(x) & no x: Q(x)");

        let skolemised = skolemise(expr);
        ctx.display(&skolemised);
    }
}
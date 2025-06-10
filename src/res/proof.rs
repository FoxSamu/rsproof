use crate::fmt::DisplayNamed;
use crate::nf::Clause;
use crate::res::Resolvee;


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Proof {
    Proven(Vec<Deduction>),
    Disproven
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Deduction {
    /// Statement was a premise.
    Premise {
        clause: Clause
    },

    /// Statement was resolved.
    Resolve {
        clause: Clause,
        a_line: usize,
        b_line: usize,
        resolvee: Resolvee
    },

    /// Statement that was magically deduced. In most, if not all, cases, this
    /// means something went wrong in the prover.
    Magic {
        clause: Clause
    },

    /// End of proof.
    QED {
        line_with_bottom: usize
    }
}


impl DisplayNamed for Deduction {
    fn fmt_named(&self, f: &mut std::fmt::Formatter<'_>, names: &crate::fmt::NameTable) -> std::fmt::Result {
        match self {
            Deduction::Premise {
                clause
            } => {
                write!(
                    f, "{}   [Premise.]",
                    clause.with_table(names)
                )?;
            },

            Deduction::Resolve { 
                clause, 
                a_line, 
                b_line, 
                resolvee
            } => {
                write!(
                    f, "{}   [By resolution from line #{} and #{} with unifier {}.]",
                    clause.with_table(names),
                    a_line,
                    b_line,
                    resolvee.mgu.with_table(names)
                )?;
            },

            Deduction::Magic {
                clause
            } => {
                write!(
                    f, "{}   [By magic.]",
                    clause.with_table(names)
                )?;
            },

            Deduction::QED {
                line_with_bottom
            } => {
                write!(
                    f, "Q.E.D.   [By refutation on line {}.]",
                    line_with_bottom
                )?;
            },
        }

        Ok(())
    }
}
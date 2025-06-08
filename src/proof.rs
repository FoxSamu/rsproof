use std::collections::BTreeMap;

use crate::cnf::Clause;
use crate::reso::Derivation;

#[derive(PartialEq, Eq, Clone, PartialOrd, Ord, Debug)]
pub enum Step {
    Premise(Clause),
    Resolve(Clause, usize, usize),
    Substitute(Clause, usize, usize)
}

impl Step {
    fn clause(&self) -> &Clause {
        match self {
            Self::Premise(c) => c,
            Self::Resolve(c, _, _) => c,
            Self::Substitute(c, _, _) => c,
        }
    }

    fn format(&self, name_table: &BTreeMap<u64, String>, index: usize) -> String {
        let mut str = String::new();

        str.extend(format!("[{index}] ").chars());

        self.clause().write_named(name_table, &mut str);

        str.push_str("   # ");
        
        let comment = match self {
            Self::Premise(_) => format!("Premise"),
            Self::Resolve(_, a, b) => format!("Resolved from statement {a} and {b}"),
            Self::Substitute(_, a, b) => format!("Substitution in statement {a} by equality {b}"),
        };

        str.extend(comment.chars());

        return str;
    }
}

pub type Proof = Vec<Step>;

pub fn format_proof(proof: &Proof, name_table: &BTreeMap<u64, String>) -> Vec<String> {
    let mut proof_lines = Vec::new();
    let mut index = 0usize;

    for step in proof {
        proof_lines.push(step.format(name_table, index));
        index += 1;
    }

    proof_lines
}

struct ProofBuilder {
    proof: Proof,
    indices: BTreeMap<Clause, usize>
}

impl ProofBuilder {
    fn state(&mut self, step: Step) -> usize {
        let index = self.proof.len();
        self.indices.insert(step.clause().clone(), index);
        self.proof.push(step);
        return index;
    }

    fn derive(&mut self, clause: &Clause, knowledge: &BTreeMap<Clause, Derivation>) -> usize {
        if let Some(i) = self.indices.get(clause) {
            return *i;
        }

        let derivation = knowledge.get(clause).unwrap();

        let cl = clause.clone();

        let step = match derivation {
            Derivation::Premise => Step::Premise(cl),
            Derivation::Resolved(a, b) => {
                let ai = self.derive(a, knowledge);
                let bi = self.derive(b, knowledge);

                Step::Resolve(cl, ai, bi)
            },
            Derivation::Substituted(a, b) => {
                let ai = self.derive(a, knowledge);
                let bi = self.derive(b, knowledge);

                Step::Substitute(cl, ai, bi)
            },
        };

        self.state(step)
    }
}

pub fn write_proof(knowledge: &BTreeMap<Clause, Derivation>) -> Proof {
    let mut builder = ProofBuilder {
        proof: Proof::new(),
        indices: BTreeMap::new()
    };

    for (clause, derivation) in knowledge {
        if *derivation == Derivation::Premise {
            builder.derive(clause, knowledge);
        }
    }

    builder.derive(&Clause::empty(), knowledge);

    return builder.proof;
}
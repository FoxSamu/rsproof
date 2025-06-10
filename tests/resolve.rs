use rsplib::nf::NormalForm;
use rsplib::res::{Proof, Resolver, ResolverResult};
use rsplib::test::TestContext;

const PROOF_STEPS: usize = 10000;

fn assert_proven(ctx: &mut TestContext, mut resolver: Resolver) {
    let proof = resolver.step_n_times(PROOF_STEPS);

    match proof {
        Some(ResolverResult {
            proof: Proof::Proven(deductions),
            deductions_made: n
        }) => {
            let mut line = 0usize;

            for elem in deductions.into_iter() {
                print!("{}: ", line);
                ctx.display(elem);
                line += 1;
            }

            println!("{n} deductions made.")
        },

        Some(ResolverResult {
            proof: Proof::Disproven,
            deductions_made: _
        }) => {
            panic!("Disproven");
        },

        None => {
            panic!("Undecided after {PROOF_STEPS} resolution steps");
        }
    }
}

#[test]
fn simple() {
    let mut ctx = TestContext::new();

    let mut resolver = Resolver::new();

    resolver.assume(ctx.clause("!P(:x) | Q(:x)"));
    resolver.assume(ctx.clause("!Q(:z)"));
    resolver.assume(ctx.clause("P(S)"));

    assert_proven(&mut ctx, resolver);
}

#[test]
fn fancy() {
    let mut ctx = TestContext::new();

    let mut resolver = Resolver::new();

    resolver.assume(ctx.clause("!Person(:x) | !Cat(:y) | !Owns(:x, :y) | Animal(f(:x))"));
    resolver.assume(ctx.clause("!Person(:x) | !Cat(:y) | !Owns(:x, :y) | Loves(:x, f(:x))"));
    resolver.assume(ctx.clause("Person(sophie)"));
    resolver.assume(ctx.clause("Cat(rusty)"));
    resolver.assume(ctx.clause("Owns(sophie, rusty)"));
    resolver.assume(ctx.clause("!Animal(:a) | !Loves(sophie, :a)"));

    assert_proven(&mut ctx, resolver);
}

#[test]
fn non_cnf() {
    let mut ctx = TestContext::new();

    let mut resolver = Resolver::new();

    resolver.assume_cnf(
        NormalForm::equiv_cnf(
            ctx.bexpr("all x: (P(x) -> Q(x)) & some x: P(x) & !(some x: Q(x))")
        )
    );

    assert_proven(&mut ctx, resolver);
}
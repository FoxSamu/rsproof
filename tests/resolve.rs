use rsplib::res::Resolver;
use rsplib::test::TestContext;

const PROOF_STEPS: usize = 10000;

#[test]
fn simple() {
    let mut ctx = TestContext::new();

    let mut resolver = Resolver::new();

    resolver.assume(ctx.clause("!P(:x) | Q(:x)"));
    resolver.assume(ctx.clause("!Q(:z)"));
    resolver.assume(ctx.clause("P(S)"));

    let proof = resolver.step_n_times(PROOF_STEPS);

    match proof {
        Some(rsplib::res::Proof::Proven(deductions)) => {
            let mut line = 0usize;

            for elem in deductions.into_iter() {
                print!("{}: ", line);
                ctx.display(elem);
                line += 1;
            }
        },
        Some(rsplib::res::Proof::Disproven) => {
            panic!("Disproven");
        },
        None => {
            panic!("Undecided after {PROOF_STEPS} resolution steps");
        }
    }
}
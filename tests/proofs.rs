
use rsplib::test::TestContext;
use rsplib::nf::NormalForm;
use rsplib::res::{Proof, Resolver, ResolverResult};

macro_rules! prove {
    ($name:ident, $value:expr) => {
        #[test]
        fn $name() {
            let mut ctx = TestContext::new();

            let expr = ctx.stmt($value).refutable_expr();
            print!("Exp: ");
            ctx.display(&expr);

            let cnf = NormalForm::equiv_cnf(expr);
            print!("CNF: ");
            ctx.display(&cnf);

            let mut resolver = Resolver::new();
            resolver.assume_cnf(cnf);
            resolver.should_skip_proof_derivation(true);

            assert_proven(&mut ctx, resolver);
        }
    };
}

macro_rules! disprove {
    ($name:ident, $value:expr) => {
        #[test]
        fn $name() {
            let mut ctx = TestContext::new();

            let expr = ctx.stmt($value).provable_expr();
            print!("Exp: ");
            ctx.display(&expr);

            let cnf = NormalForm::equiv_cnf(expr);
            print!("CNF: ");
            ctx.display(&cnf);

            let mut resolver = Resolver::new();
            resolver.assume_cnf(cnf);

            assert_proven(&mut ctx, resolver);
        }
    };
}

prove!(basic_fol, "all x: (P(x) -> Q(x)) & some x: P(x) |- some x: Q(x)");

prove!(taut, "|- true");

disprove!(cont, "|- false");

prove!(demorgan1, "!(P | Q) |- (!P & !Q)");

prove!(demorgan2, "!(P & Q) |- (!P | !Q)");

disprove!(equiv1, "P <-> Q, P |- !Q");

prove!(equivs, "A <-> (B | C), B <-> (C | D), C <-> (D | A), A, !B |- D");

prove!(names, "Foo -> Bar, Bar -> Baz |- Foo -> Baz");

prove!(long, "
    A <-> B,
    B <-> C,
    C <-> D,
    D <-> E,
    E <-> F,
    F <-> G,
    G <-> H,
    H <-> I,
    I <-> J,
    J <-> K,
    K <-> L,
    L <-> M,
    M <-> N,
    N <-> O,
    O <-> P,
    P <-> Q,
    Q <-> R,
    R <-> S,
    S <-> T,
    T <-> U,
    U <-> V,
    V <-> W,
    W <-> X,
    X <-> Y,
    Y <-> Z,

    A |- Z
");

disprove!(cont2, "P |- !P");

const PROOF_STEPS: usize = 10000;

fn assert_proven(ctx: &mut TestContext, mut resolver: Resolver) {
    let proof = resolver.step_n_times(PROOF_STEPS);

    match proof {
        Some(ResolverResult {
            proof: Proof::Proven(deductions),
            deductions_made: n,
            learning_order: _
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
            deductions_made: _,
            learning_order: _
        }) => {
            panic!("Disproven");
        },

        None => {
            panic!("Undecided after {PROOF_STEPS} resolution steps");
        }
    }
}
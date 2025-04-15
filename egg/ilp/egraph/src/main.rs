use egg::{*, rewrite as rw};

fn main() {
    // Define the language
    define_language! {
        enum SimpleLang {
            "+" = Add([Id; 2]),
            "*" = Mul([Id; 2]),
            "a" = A,
            "b" = B,
        }
    }

    // Create an empty e-graph
    let mut egraph: EGraph<SimpleLang, ()> = EGraph::default();

    // Add initial expressions
    let a = egraph.add(SimpleLang::A);
    let b = egraph.add(SimpleLang::B);
    let add_a_b = egraph.add(SimpleLang::Add([a, b]));
    let add_b_a = egraph.add(SimpleLang::Add([b, a]));
    let mul_add_a_b_add_b_a = egraph.add(SimpleLang::Mul([add_a_b, add_b_a]));
    let add_mul_add_a_b_add_b_a_mul_add_a_b_add_b_a = egraph.add(SimpleLang::Add([mul_add_a_b_add_b_a, mul_add_a_b_add_b_a]));

    let rules: &[Rewrite<SimpleLang, ()>] = &[
        rw!("commute-add"; "(+ ?x ?y)" => "(+ ?y ?x)"),
        rw!("commute-mul"; "(* ?x ?y)" => "(* ?y ?x)"),
    ];

    // Run the rewrites
    let runner = Runner::default().with_egraph(egraph).run(rules);


    runner.egraph.dot().to_dot("egraph.dot").unwrap();

}
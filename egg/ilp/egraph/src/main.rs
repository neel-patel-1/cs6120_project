use egg::{*, rewrite as rw};

fn main() {
    // Define the language
    define_language! {
        enum SimpleLang {
            "+" = Add([Id; 2]),
            "a" = A,
            "b" = B,
        }
    }

    // Create an empty e-graph
    let mut egraph: EGraph<SimpleLang, ()> = EGraph::default();

    // Add initial expressions
    let a = egraph.add(SimpleLang::A);
    let b = egraph.add(SimpleLang::B);
    let add_ab = egraph.add(SimpleLang::Add([a, b]));

    // Define rewrites that create a loop
    let rules: &[Rewrite<SimpleLang, ()>] = &[
        rw!("add-commute"; "(+ ?x ?y)" => "(+ ?y ?x)"),
        rw!("add-self"; "(+ ?x ?x)" => "?x"),
    ];

    // Run the rewrites
    let runner = Runner::default().with_egraph(egraph).run(rules);

    // Print the e-graph to observe the loop
    runner.egraph.dot().to_dot("egraph.dot").unwrap();
}
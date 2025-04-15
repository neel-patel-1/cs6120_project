use egg::{*, rewrite as rw};

fn main() {
    // Define the language
    define_language! {
        enum SimpleLang {
            "+" = Add([Id; 2]),
            "a" = A,
        }
    }

    // Create an empty e-graph
    let mut egraph: EGraph<SimpleLang, ()> = EGraph::default();

    // Add initial expressions
    let a = egraph.add(SimpleLang::A);
    let add_a_a = egraph.add(SimpleLang::Add([a, a]));

    // Define rewrites that create a loop from child to parent
    let rules: &[Rewrite<SimpleLang, ()>] = &[
        rw!("add-to-self"; "(+ ?x ?x)" => "?x"),
        rw!("self-to-add"; "?x" => "(+ ?x ?x)"),
    ];

    // Run the rewrites
    let runner = Runner::default().with_egraph(egraph).run(rules);

    // Print the e-graph to observe the loop
    runner.egraph.dot().to_dot("egraph.dot").unwrap();
}
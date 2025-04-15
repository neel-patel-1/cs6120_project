use egg::{*};

define_language! {
    enum SimpleLang {
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        "a" = A,
        "b" = B,
    }
}

fn get_egraph() -> EGraph<SimpleLang, ()> {
    // Define the language

    // Create an empty e-graph
    let mut egraph: EGraph<SimpleLang, ()> = EGraph::default();

    // Add initial expressions
    let a = egraph.add(SimpleLang::A);
    let b = egraph.add(SimpleLang::B);
    let add_a_b = egraph.add(SimpleLang::Add([a, b]));
    let add_b_a = egraph.add(SimpleLang::Add([b, a]));
    let mul_add_a_b_add_b_a = egraph.add(SimpleLang::Mul([add_a_b, add_b_a]));
    egraph.add(SimpleLang::Add([mul_add_a_b_add_b_a, mul_add_a_b_add_b_a]));


    return egraph;
}

fn main() {
    let egraph = get_egraph();
}

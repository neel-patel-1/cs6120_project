use egg::{*};
use good_lp::variable::ProblemVariables;
use good_lp::{
    variables
};

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

pub struct EGraphProblem {
    num_classes: usize,
    vars: ProblemVariables,
}

impl EGraphProblem {
    pub fn new<L, N>(egraph : &EGraph<L, N>) -> Self
    where
        L: Language,
        N: Analysis<L>,
    {
        Self {
            num_classes: egraph.classes().len(),
            vars: variables!(),
        }
    }

    pub fn get_num_classes(&self) -> usize {
        self.num_classes
    }
}

fn main() {
    let egraph = get_egraph();
    let egraph_prob = EGraphProblem::new(&egraph);
    let num_classes = egraph_prob.get_num_classes();
    println!("Number of classes in the e-graph: {}", num_classes);
}

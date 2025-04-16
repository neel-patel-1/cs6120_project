use egg::{*};
use good_lp::variable::ProblemVariables;
use good_lp::{
    variables,
    variable,
    Constraint,
    default_solver,
    Solution,
    SolverModel,
    Expression
};




pub struct GoodLpExtractor<'a, L:Language, N:Analysis<L>> {
    egraph: &'a EGraph<L, N>,
    vars: ProblemVariables,
    total_cost: Expression,
}


impl <'a, L, N> GoodLpExtractor<'a, L, N>
where
    L: Language,
    N: Analysis<L>,
{

    pub fn new(egraph: &'a EGraph<L, N>) -> Self
    {
        let mut vars = variables!();
        let mut total_cost = 0.into();

        for class in egraph.classes() {
            for node in &class.nodes {
                let node_var = vars.add(variable().binary());
                total_cost += node_var;
            }
        }
        Self {
            vars,
            egraph,
            total_cost,
        }
    }

    fn solve(self) -> Box<dyn Solution> {
        let objective = self.total_cost;
        println!("Objective: {:?}", objective);
        let solution = self.vars
            .minimise(objective)
            .using(default_solver)
            .with_all(Self::constraints())
            .solve()
            .unwrap();

        Box::new(solution)
    }

    fn constraints() -> Vec<Constraint> {
        let mut constraints = Vec::with_capacity(2);

        constraints

    }

}

define_language! {
    enum SimpleLang {
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        Num(i32),
        Symbol(Symbol),
    }
}

fn main() {
    let expr: RecExpr<SimpleLang> = "(+ (* a b) (* b a))".parse().unwrap();

    let rules = &[
        rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        rewrite!("add-0"; "(+ ?a 0)" => "?a"),
        rewrite!("mul-0"; "(* ?a 0)" => "0"),
        rewrite!("mul-1"; "(* ?a 1)" => "?a"),
    ];
    let runner: Runner<SimpleLang, ()> = Runner::default().with_expr(&expr).run(rules);

    let glpe = GoodLpExtractor::new(&runner.egraph);
    glpe.solve();
}

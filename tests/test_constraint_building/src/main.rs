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
    cost_function: Box<dyn LpCostFunction<L, N>>,
}


impl <'a, L, N> GoodLpExtractor<'a, L, N>
where
    L: Language,
    N: Analysis<L>,
{

    pub fn new(egraph: &'a EGraph<L, N>, mut cost_function: Box<dyn LpCostFunction<L, N>>) -> Self
    {
        Self {
            egraph,
            cost_function
        }
    }

    fn solve(mut self, eclass: Id) -> Box<dyn Solution> {
        let mut vars = variables!();
        let mut constraints = Vec::new();
        let mut total_cost: Expression = 0.into();

        /* pass over e-graph creating binary variables constraints */
        for class in self.egraph.classes() {
            for node in &class.nodes {
                let node_var = vars.add(variable().binary());

                let cost = self.cost_function.node_cost(self.egraph, class.id, node);

                total_cost += cost * node_var;
            }
        }

        /* solve */
        let solution = vars
            .minimise(total_cost)
            .using(default_solver)
            .with_all(constraints)
            .solve()
            .unwrap();

        Box::new(solution)
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

    let glpe = GoodLpExtractor::new(&runner.egraph, Box::new(AstSize));
    glpe.solve(runner.roots[0]);
}

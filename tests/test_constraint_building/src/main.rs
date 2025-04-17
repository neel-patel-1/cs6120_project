use egg::{*};
use good_lp::variable::ProblemVariables;
use good_lp::{
    variables,
    variable,
    Variable,
    Constraint,
    default_solver,
    Solution,
    SolverModel,
    Expression
};
use std::collections::HashMap;




pub struct GoodLpExtractor<'a, L:Language, N:Analysis<L>> {
    egraph: &'a EGraph<L, N>,
    cost_function: Box<dyn LpCostFunction<L, N>>,
    enode_vars: HashMap<(Id, usize), Variable>,
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
            cost_function,
            enode_vars: HashMap::new(),
        }
    }

    fn solve(mut self, eclass: Id) -> Box<dyn Solution> {
        let mut vars = variables!();
        let mut constraints = Vec::new();
        let mut total_cost: Expression = 0.into();

        /* t_m */
        let mut topo_vars: HashMap<Id, Variable> = HashMap::new();
        for class in self.egraph.classes() {
            let t_m = vars.add(variable().min(0.0).max(1.0));
            topo_vars.insert(class.id, t_m);
        }
        const EPSILON: f64 = 1e-3;
        const A: f64 = 2.0;

        /* pass over e-graph creating binary variables constraints */
        for class in self.egraph.classes() {
            let t_parent = topo_vars[&class.id].clone();

            for (node_index, _node) in class.nodes.iter().enumerate() {
                let node_var = *self
                    .enode_vars
                    .entry((class.id, node_index))
                    .or_insert_with(|| vars.add(variable().binary()));

                /* for each child e-class, enforce (c3) that at least one of its e-nodes is selected if this e-node gets selected and (c4) that its e-class acyclicity variable is greater than the threshold if it is selected */
                for child in class.nodes[node_index].children() {
                    let child_class = self.egraph.find(*child);
                    let mut child_vars = Vec::new();

                    /* ensure every enode in the child e‑class has a variable */
                    for (c_idx, _c_node) in self.egraph[child_class].nodes.iter().enumerate() {
                        let var = self
                            .enode_vars
                            .entry((child_class, c_idx))
                            .or_insert_with(|| vars.add(variable().binary()))
                            .clone();
                        child_vars.push(var);
                    }

                    /* node_var ≤ Σ child_vars  */
                    let child_sum: Expression = child_vars.iter().cloned().sum();
                    constraints.push(Into::<Expression>::into(node_var.clone()).leq(child_sum));


                }

                let cost = self.cost_function.node_cost(self.egraph, class.id, _node);

                total_cost += cost * node_var;
            }
        }

        /* add constraint: sum of node_vars for the root e-class must be 1 */
        let root_class_id = self.egraph.find(eclass);
        println!("Num Classes: {} Root Class: {}", self.egraph.classes().len(), root_class_id);
        let root_class = &self.egraph[root_class_id];
        let root_vars = root_class
            .nodes
            .iter()
            .enumerate()
            .map(|(node_index, _)| self.enode_vars[&(root_class_id, node_index)].clone())
            .collect::<Vec<_>>();

        constraints.push(root_vars.iter().cloned().sum::<Expression>().eq(1));

        println!("Root vars: {:?}", root_vars);



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

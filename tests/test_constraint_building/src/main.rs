use egg::{*};
use good_lp::{
    variables,
    variable,
    Variable,
    default_solver,
    Solution,
    SolverModel,
    Expression
};
use std::collections::HashMap;
use smallvec::SmallVec;




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

    pub fn new(egraph: &'a EGraph<L, N>, cost_function: Box<dyn LpCostFunction<L, N>>) -> Self
    {
        Self {
            egraph,
            cost_function,
            enode_vars: HashMap::new(),
        }
    }

    fn solve(mut self, eclass: Id) -> (f64, RecExpr<L>) {
        let mut vars = variables!();
        let mut constraints = Vec::new();
        let mut total_cost: Expression = 0.into();

        /* t_m */
        let mut topo_vars: HashMap<Id, Variable> = HashMap::new();
        for class in self.egraph.classes() {
            let t_m = vars.add(variable().min(0.0).max(1.0));
            topo_vars.insert(class.id, t_m);
        }
        const EPS: f64 = 1e-3;
        const ALPHA: f64 = 2.0;

        /* pass over e-graph creating binary variables constraints */
        for class in self.egraph.classes() {
            let t_parent = topo_vars[&class.id].clone();

            for (node_index, _node) in class.nodes.iter().enumerate() {
                let node_var = *self
                    .enode_vars
                    .entry((class.id, node_index))
                    .or_insert_with(|| vars.add(variable().binary()));

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

                    #[cfg(not(feature = "no_acyclic"))]
                    {
                        let node_expr: Expression = node_var.clone().into();
                        let t_child = topo_vars[&child_class].clone();
                        let big_m_part: Expression = (Into::<Expression>::into(1.0) - node_expr.clone()) * ALPHA;
                        let acyc_expr: Expression = t_parent.clone() - t_child - (Into::<Expression>::into(EPS)) + big_m_part;
                        constraints.push(Into::<Expression>::into(acyc_expr).geq(Into::<Expression>::into(0)));
                    }


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
            .minimise(&total_cost)
            .using(default_solver)
            .with_all(constraints)
            .solve()
            .unwrap();

        let obj_cost = solution.eval(&total_cost);

        let mut cache  = HashMap::<Id, egg::Id>::new();
        let mut rexpr  = RecExpr::<L>::default();

        // depth‑first reconstruction
        fn build<L: Language, N: Analysis<L>>(
            egraph   : &EGraph<L, N>,
            enode_vs : &HashMap<(Id, usize), Variable>,
            sol      : &dyn Solution,
            class_id : Id,
            cache    : &mut HashMap<Id, egg::Id>,
            out_expr : &mut RecExpr<L>,
        ) -> egg::Id {
            if let Some(&id) = cache.get(&class_id) {
                return id;
            }
            // find the enode whose var == 1
            let class = &egraph[class_id];
            let (_, node) = class.nodes.iter().enumerate()
                .find(|(i, _)| sol.value(enode_vs[&(class_id, *i)]) > 0.5)
                .expect("no chosen node for e‑class");

            // recursively build children
            let new_children: SmallVec<[egg::Id; 4]> = node
                .children()
                .iter()
                .map(|c| build(egraph, enode_vs, sol, egraph.find(*c), cache, out_expr))
                .collect();

            // add new node with mapped children
            let mut idx = 0usize;
            let mapped = node.clone().map_children(|_| {
                let id = new_children[idx];
                idx += 1;
                id
            });
            let new_id = out_expr.add(mapped);
            cache.insert(class_id, new_id);
            new_id
        }

        build(
            self.egraph,
            &self.enode_vars,
            &solution,
            root_class_id,
            &mut cache,
            &mut rexpr,
        );

        (obj_cost, rexpr)
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
    let (best_cost, best_expr)
        = glpe.solve(runner.roots[0]);

    println!("Best cost:{} Best expr: {:?}", best_cost, best_expr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egg::{define_language, rewrite, RecExpr, Runner};

    define_language! {
        enum CycLang {
            "matmul" = MatMul([Id; 2]),
            "split"  = Split ([Id; 1]),
            Symbol(Symbol),
        }
    }

    struct PreferSplit;
    impl LpCostFunction<CycLang, ()> for PreferSplit {
        fn node_cost(
            &mut self,
            _egraph: &EGraph<CycLang, ()>,
            _id: Id,
            node: &CycLang,
        ) -> f64 {
            match node {
                CycLang::Split(_) => 1.0,   // cheapest ⇒ ILP wants it
                _                 => 100.0, // expensive
            }
        }
    }

    #[test]
    fn acyclicity_constraint_blocks_cyclic_split() {
        let rewrites = &[
            rewrite!(
                "wrap-split";
                "(matmul ?x ?y)" => "(split (matmul ?x ?y))"
            ),
        ];
        let expr: RecExpr<CycLang> = "(matmul a b)".parse().unwrap();
        let runner: Runner<CycLang, ()> =
            Runner::default().with_expr(&expr).run(rewrites);
        let extractor = GoodLpExtractor::new(&runner.egraph, Box::new(PreferSplit));
        let (_cost, best_expr) = extractor.solve(runner.roots[0]);
        let s = best_expr.to_string();
        assert!(
            !s.contains("split"),
            "A cyclic `split` node was selected ⇒ acyclicity constraint failed.\nExtracted expr: {s}"
        );
    }
}

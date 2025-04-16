use egg::{*};
use good_lp::variable::ProblemVariables;
use good_lp::{
    variables
};



pub struct GoodLpExtractor<'a, L:Language, N:Analysis<L>> {
    egraph: &'a EGraph<L, N>,
    vars: ProblemVariables,
}


impl <'a, L, N> GoodLpExtractor<'a, L, N>
where
    L: Language,
    N: Analysis<L>,
{

    pub fn new(egraph: &'a EGraph<L, N>) -> Self
    {
        Self {
            vars: variables!(),
            egraph: egraph
        }

    }

    pub fn add_root_eclass_selection_constraint(&self, eclass: Id){
        println!("Adding root eclass selection constraint for eclass: {:?}", eclass);
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
    let root = runner.roots[0];
    glpe.add_root_eclass_selection_constraint(root);
}

use egg::{*};

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
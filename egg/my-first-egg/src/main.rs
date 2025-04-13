use egg::{*, rewrite as rw};
fn main() {

    // build egraph
    let mut expr = RecExpr::default();
    let a = expr.add(SymbolLang::leaf("a"));
    let b = expr.add(SymbolLang::leaf("b"));
    expr.add(SymbolLang::new("foo", vec![a, b]));

    let mut egraph: EGraph<SymbolLang, ()> = EGraph::default();
    let a = egraph.add(SymbolLang::leaf("a"));
    let b = egraph.add(SymbolLang::leaf("b"));
    let foo = egraph.add(SymbolLang::new("foo", vec![a, b]));

    let foo2 = egraph.add_expr(&expr);

    assert_eq!(foo, foo2);


    let mut egraph: EGraph<SymbolLang, ()> = EGraph::default();
    let a = egraph.add(SymbolLang::leaf("a"));
    let b = egraph.add(SymbolLang::leaf("b"));
    let _foo = egraph.add(SymbolLang::new("foo", vec![a, b]));
    egraph.rebuild();

    let pat: Pattern<SymbolLang> = "(foo ?x ?x)".parse().unwrap();

    let matches = pat.search(&egraph);
    assert!(matches.is_empty());

    egraph.union(a,b);

    egraph.rebuild();

    let matches = pat.search(&egraph);
    assert!(!matches.is_empty());

    let rules: &[Rewrite<SymbolLang, ()>] = &[
        rw!("commute-add"; "(+ ?x ?y)" => "(+ ?y ?x)"),
        rw!("commute-mul"; "(* ?x ?y)" => "(* ?y ?x)"),

        rw!("add-0"; "(+ 0 ?x)" => "?x"),
        rw!("mul-0"; "(* 0 ?x)" => "0"),
        rw!("mul-1"; "(* 1 ?x)" => "?x"),
    ];

    let start = "(+ 0 (* 1 a))".parse().unwrap();

    /* Run Equality saturation */
    let runner = Runner::default().with_expr(&start).run(rules);

    /* Extract using AstSize as the Cost function */
    let extractor = Extractor::new(&runner.egraph, AstSize);

    let (best_cost, best_expr) = extractor.find_best(runner.roots[0]);

    assert_eq!(best_expr, "a".parse().unwrap());
    assert_eq!(best_cost, 1);

}

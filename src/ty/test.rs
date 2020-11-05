use super::*;

#[test]
fn ty_fv() {
    use Ty::*;

    assert_eq!(
        vec!("x".to_string(), "y".to_string()),
        Tuple(vec!(Var("x".to_string()), Var("y".to_string())))
            .fv()
            .collect::<Vec<Ident>>()
    );
    assert_eq!(
        vec!("x".to_string(), "y".to_string()),
        Fun(
            Box::new(Var("x".to_string())),
            Box::new(Var("y".to_string()))
        )
        .fv()
        .collect::<Vec<Ident>>()
    );
}

#[test]
fn scheme_fv() {
    use Ty::*;

    assert_eq!(
        vec!("x".to_string()),
        (
            vec!("y".to_string()),
            Tuple(vec!(Var("x".to_string()), Var("y".to_string())))
        )
            .fv()
            .collect::<Vec<Ident>>()
    );
    assert_eq!(
        vec!("y".to_string()),
        (
            vec!("x".to_string()),
            Fun(
                Box::new(Var("x".to_string())),
                Box::new(Var("y".to_string()))
            )
        )
            .fv()
            .collect::<Vec<Ident>>()
    );
}

#[test]
fn env_fv() {
    use Ty::*;

    assert_eq!(
        vec!("x".to_string()),
        [(
            "foo".to_string(),
            (
                vec!("y".to_string()),
                Tuple(vec!(Var("x".to_string()), Var("y".to_string())))
            )
        )]
        .iter()
        .cloned()
        .collect::<Env>()
        .fv()
        .collect::<Vec<Ident>>()
    );
}

#[test]
fn unify() {
    use Ty::*;

    assert_eq!(
        Ok(vec!()),
        super::unify(vec!((Int, Int)).into_iter()).collect()
    );

    assert_eq!(
        Ok(vec!(("a".to_string(), Int))),
        super::unify(vec!((Var("a".to_string()), Int)).into_iter()).collect()
    );

    assert_eq!(
        Ok(vec!(("a".to_string(), Int))),
        super::unify(vec!((Int, Var("a".to_string()))).into_iter()).collect()
    );

    assert_eq!(
        Ok(vec!(("b".to_string(), Bool), ("a".to_string(), Int))),
        super::unify(
            vec!((
                Fun(Box::new(Int), Box::new(Bool)),
                Fun(
                    Box::new(Var("a".to_string())),
                    Box::new(Var("b".to_string()))
                )
            ))
            .into_iter()
        )
        .collect()
    );

    assert_eq!(
        Ok(vec!(("a".to_string(), Int))),
        super::unify(
            vec!((
                Fun(Box::new(Int), Box::new(Int)),
                Fun(
                    Box::new(Var("a".to_string())),
                    Box::new(Var("a".to_string()))
                )
            ))
            .into_iter()
        )
        .collect()
    );

    assert_eq!(
        Ok(vec!(("a".to_string(), Fun(Box::new(Int), Box::new(Bool))))),
        super::unify(vec!((Fun(Box::new(Int), Box::new(Bool)), Var("a".to_string()))).into_iter())
            .collect()
    );

    assert!(
        super::unify(vec!((Fun(Box::new(Int), Box::new(Int)), Int)).into_iter())
            .collect::<Result<Vec<_>, std::string::String>>()
            .is_err()
    );

    assert_eq!(
        Ok(vec!(("a".to_string(), Int))),
        super::unify(
            vec!((
                Tuple(vec!(Int, Int)),
                Tuple(vec!(Var("a".to_string()), Var("a".to_string())))
            ))
            .into_iter()
        )
        .collect()
    );

    assert_eq!(
        Ok(vec!(("b".to_string(), Bool), ("a".to_string(), Int))),
        super::unify(
            vec!((
                Tuple(vec!(Int, Bool)),
                Tuple(vec!(Var("a".to_string()), Var("b".to_string())))
            ))
            .into_iter()
        )
        .collect()
    );

    assert_eq!(
        Ok(vec!(("a".to_string(), Int))),
        super::unify(
            vec!((
                Record(vec!(("x".to_string(), Int), ("y".to_string(), Int))),
                Record(vec!(
                    ("x".to_string(), Var("a".to_string())),
                    ("y".to_string(), Var("a".to_string()))
                ))
            ))
            .into_iter()
        )
        .collect()
    );

    assert_eq!(
        Ok(vec!(("b".to_string(), Bool), ("a".to_string(), Int))),
        super::unify(
            vec!((
                Record(vec!(("x".to_string(), Int), ("y".to_string(), Bool))),
                Record(vec!(
                    ("x".to_string(), Var("a".to_string())),
                    ("y".to_string(), Var("b".to_string()))
                ))
            ))
            .into_iter()
        )
        .collect()
    );
}

#[test]
fn infer() {
    assert_eq!(
        Ok(Ty::Fun(
            Box::new(Ty::Var("a_0".to_string())),
            Box::new(Ty::Var("a_0".to_string()))
        )),
        super::infer(
            &mut HashMap::new(),
            &mut NameSource::new(),
            &mut HashMap::new(),
            &Expr::Lambda(
                "a".to_string(),
                Box::new(Expr::Atom(Atom::Ident("a".to_string())))
            )
        )
    );

    assert_eq!(
        Ok(Ty::Int),
        super::infer(
            &mut HashMap::new(),
            &mut NameSource::new(),
            &mut HashMap::new(),
            &Expr::Let(
                vec!(("x".to_string(), Expr::Atom(Atom::Int(42)))),
                Box::new(Expr::Atom(Atom::Ident("x".to_string())))
            )
        )
    );

    assert_eq!(
        Ok(Ty::Bool),
        super::infer(
            &mut HashMap::new(),
            &mut NameSource::new(),
            &mut HashMap::new(),
            &Expr::Let(
                vec!(
                    ("x".to_string(), Expr::Atom(Atom::Int(42))),
                    ("y".to_string(), Expr::Atom(Atom::Bool(true)))
                ),
                Box::new(Expr::Atom(Atom::Ident("y".to_string())))
            )
        )
    );

    // `let id = 𝜆y . y in id`
    assert_eq!(
        Ok(Ty::Fun(
            Box::new(Ty::Var("y_0_1".to_string())),
            Box::new(Ty::Var("y_0_1".to_string()))
        )),
        super::infer(
            &mut HashMap::new(),
            &mut NameSource::new(),
            &mut HashMap::new(),
            &Expr::Let(
                vec!((
                    "id".to_string(),
                    Expr::Lambda(
                        "y".to_string(),
                        Box::new(Expr::Atom(Atom::Ident("y".to_string())))
                    )
                )),
                Box::new(Expr::Atom(Atom::Ident("id".to_string())))
            )
        )
    );

    // `let apply = 𝜆f . 𝜆x . f x in let id = 𝜆y . y in apply id`
    let mut env = HashMap::new();
    let mut subs = HashMap::new();

    let res = super::infer(
        &mut subs,
        &mut NameSource::new(),
        &mut env,
        &Expr::Let(
            vec![
                (
                    "apply".to_string(),
                    Expr::Lambda(
                        "f".to_string(),
                        Box::new(Expr::Lambda(
                            "x".to_string(),
                            Box::new(Expr::Apply(
                                Box::new(Expr::Atom(Atom::Ident("f".to_string()))),
                                Box::new(Expr::Atom(Atom::Ident("x".to_string()))),
                            )),
                        )),
                    ),
                ),
                (
                    "id".to_string(),
                    Expr::Lambda(
                        "y".to_string(),
                        Box::new(Expr::Atom(Atom::Ident("y".to_string()))),
                    ),
                ),
            ],
            Box::new(Expr::Apply(
                Box::new(Expr::Atom(Atom::Ident("apply".to_string()))),
                Box::new(Expr::Atom(Atom::Ident("id".to_string()))),
            )),
        ),
    );

    match res {
        Ok(Ty::Fun(lhs, rhs)) => assert_eq!(lhs, rhs),
        e => panic!("Wrong result: {:?}", e),
    }
}

#[test]
fn infer_and_print() {
    use pest::Parser;
    fn infer(input: &str) -> String {
        let e = crate::parse::parse_exprs(
            crate::parse::Parser::parse(crate::parse::Rule::expr, input)
                .unwrap_or_else(|e| panic!("{}", e))
                .next()
                .unwrap()
                .into_inner(),
        )
        .unwrap();
        let ty = super::infer(
            &mut HashMap::new(),
            &mut NameSource::new(),
            &mut HashMap::new(),
            &e,
        )
        .unwrap();

        format!("{}", ty)
    }

    assert_eq!("Int", infer("4"));

    assert_eq!(
        "(x_0_1 -> x_0_1)",
        infer("let id = lambda x -> x in id end")
    );

    assert_eq!(
        "((x_0_1 -> x_0_1), Int, Bool)",
        infer("(let id = lambda x -> x in id end, 42, True)")
    );

    assert_eq!(
        "{ x: (x_0_1 -> x_0_1), y: Int, z: Bool }",
        infer("{ x = let id = lambda x -> x in id end, y = 42, z = True }")
    );

    assert_eq!("()", infer("()"));

    assert_eq!("String", infer("\"Hello World!\""));

    assert_eq!("Int", infer("case 42 of i => i end"));

    assert_eq!("Int", infer("case (42, True) of (i, j) => i end"));
}

#[test]
fn parse_and_infer() {
    use pest::Parser;
    fn infer(input: &str) -> Result<Ty, String> {
        let e = crate::parse::parse_exprs(
            crate::parse::Parser::parse(crate::parse::Rule::expr, input)
                .unwrap_or_else(|e| panic!("{}", e))
                .next()
                .unwrap()
                .into_inner(),
        )
        .unwrap();
        super::infer(
            &mut HashMap::new(),
            &mut NameSource::new(),
            &mut HashMap::new(),
            &e,
        )
    }

    assert!(infer("lambda x -> x x").is_err());

    assert!(infer("case 42 of (i, j) => i end").is_err());
}

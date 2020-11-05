use super::*;

#[test]
fn display_expr() {
    assert_eq!(
        "(foo (id x))".to_string(),
        format!(
            "{}",
            Expr::Apply(
                Box::new(Expr::Atom(Atom::Ident("foo".to_string()))),
                Box::new(Expr::Apply(
                    Box::new(Expr::Atom(Atom::Ident("id".to_string()))),
                    Box::new(Expr::Atom(Atom::Ident("x".to_string())))
                ))
            )
        )
    );

    assert_eq!(
        "((foo id) x)".to_string(),
        format!(
            "{}",
            Expr::Apply(
                Box::new(Expr::Apply(
                    Box::new(Expr::Atom(Atom::Ident("foo".to_string()))),
                    Box::new(Expr::Atom(Atom::Ident("id".to_string())))
                )),
                Box::new(Expr::Atom(Atom::Ident("x".to_string())))
            )
        )
    );

    assert_eq!(
        "(foo \"Hello World!\")".to_string(),
        format!(
            "{}",
            Expr::Apply(
                Box::new(Expr::Atom(Atom::Ident("foo".to_string()))),
                Box::new(Expr::Atom(Atom::String("Hello World!".to_string())))
            )
        )
    );

    assert_eq!(
        "let foo = 42".to_string(),
        format!(
            "{}",
            Statement::Let("foo".to_string(), Expr::Atom(Atom::Int(42)))
        )
    );
}

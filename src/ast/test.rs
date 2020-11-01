use super::*;

#[test]
fn display_expr() {
    assert_eq!(
        "(foo (id x))".to_string(),
        format!(
            "{}",
            Expr::Apply(
                Box::new(Expr::Ident("foo".to_string())),
                Box::new(Expr::Apply(
                    Box::new(Expr::Ident("id".to_string())),
                    Box::new(Expr::Ident("x".to_string()))
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
                    Box::new(Expr::Ident("foo".to_string())),
                    Box::new(Expr::Ident("id".to_string()))
                )),
                Box::new(Expr::Ident("x".to_string()))
            )
        )
    );

    assert_eq!(
        "(foo \"Hello World!\")".to_string(),
        format!(
            "{}",
            Expr::Apply(
                Box::new(Expr::Ident("foo".to_string())),
                Box::new(Expr::String("Hello World!".to_string()))
            )
        )
    );
}

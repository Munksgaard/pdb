use super::*;

#[test]
fn eval_int() {
    assert_eq!(
        "42",
        format!("{}", eval(&Environment::new(), Expr::Int(42)).unwrap())
    );
}

#[test]
fn eval_bool() {
    assert_eq!(
        "true",
        format!("{}", eval(&Environment::new(), Expr::Bool(true)).unwrap())
    );
}

#[test]
fn eval_tuple() {
    assert_eq!(
        "(false, 43)",
        format!(
            "{}",
            eval(
                &Environment::new(),
                Expr::Tuple(vec!(Expr::Bool(false), Expr::Int(43)))
            )
            .unwrap()
        ),
    );
}

#[test]
fn eval_unit() {
    assert_eq!(
        "()",
        format!("{}", eval(&Environment::new(), Expr::Unit).unwrap())
    );
}

#[test]
fn eval_record() {
    assert_eq!(
        "{x = false, y = 42}",
        format!(
            "{}",
            eval(
                &Environment::new(),
                Expr::Record(vec!(
                    (String::from("x"), Expr::Bool(false)),
                    (String::from("y"), Expr::Int(42))
                ))
            )
            .unwrap()
        )
    );
}

use pest::Parser;
fn parse_and_eval(input: &str) -> Result<Object> {
    let e = crate::parse::parse_exprs(
        crate::parse::Parser::parse(crate::parse::Rule::expr, input)
            .unwrap_or_else(|e| panic!("{}", e))
            .next()
            .unwrap()
            .into_inner(),
    )
    .unwrap();
    eval(&Environment::new(), e)
}

#[test]
fn eval_id() {
    assert_eq!(
        "42",
        format!(
            "{}",
            parse_and_eval("let id = lambda x -> x in id 42 end").unwrap()
        )
    );
}

#[test]
fn eval_first() {
    assert_eq!(
        "42",
        format!(
            "{}",
            parse_and_eval("let first = lambda x -> lambda y -> x in first 42 43 end").unwrap()
        )
    );
}

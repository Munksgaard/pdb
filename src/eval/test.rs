use super::*;

#[test]
fn eval_int() {
    assert_eq!(
        "42",
        format!(
            "{}",
            eval(&Environment::new(), Expr::Atom(Atom::Int(42))).unwrap()
        )
    );
}

#[test]
fn eval_bool() {
    assert_eq!(
        "true",
        format!(
            "{}",
            eval(&Environment::new(), Expr::Atom(Atom::Bool(true))).unwrap()
        )
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
                Expr::Tuple(vec!(
                    Expr::Atom(Atom::Bool(false)),
                    Expr::Atom(Atom::Int(43))
                ))
            )
            .unwrap()
        ),
    );
}

#[test]
fn eval_unit() {
    assert_eq!(
        "()",
        format!(
            "{}",
            eval(&Environment::new(), Expr::Atom(Atom::Unit)).unwrap()
        )
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
                    (String::from("x"), Expr::Atom(Atom::Bool(false))),
                    (String::from("y"), Expr::Atom(Atom::Int(42)))
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

#[test]
fn eval_case() {
    assert_eq!(
        "42",
        format!(
            "{}",
            parse_and_eval("case (42, 43) of (i, j) => i end").unwrap()
        )
    );

    assert_eq!(
        "42",
        format!(
            "{}",
            parse_and_eval("case 1337 of 0 => 0 | 1337 => 42 end").unwrap()
        )
    );
    assert_eq!(
        "42",
        format!(
            "{}",
            parse_and_eval("case (1337, 0) of (1337, _) => 42 | _ => 43 end").unwrap()
        )
    );

    assert_eq!(
        "42",
        format!(
            "{}",
            parse_and_eval("case { x = 42, y = True } of { x = i, y = _ } => i end").unwrap()
        )
    );
}

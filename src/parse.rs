use crate::ast::*;
use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser as _;

#[derive(Parser)]
#[grammar = "pdb.pest"]
pub struct Parser;

fn parse_tyrecord(mut pairs: Pairs<Rule>) -> Result<Ty, Error<Rule>> {
    let mut xs = Vec::new();

    while let Some(ident) = pairs.next() {
        let ty = parse_ty(pairs.next().unwrap().into_inner().next().unwrap())?;
        xs.push((ident.as_str().to_owned(), ty));
    }

    xs.sort_by(|(x, _), (y, _)| x.cmp(y));

    Ok(Ty::Record(xs))
}

pub fn parse_ty(pair: Pair<Rule>) -> Result<Ty, Error<Rule>> {
    match pair.as_rule() {
        Rule::tyident => match pair.as_str() {
            "Int" => Ok(Ty::Int),
            "Bool" => Ok(Ty::Bool),
            "String" => Ok(Ty::String),
            x => Ok(Ty::Var(x.to_string())),
        },
        Rule::tytuple => Ok(Ty::Tuple(
            pair.into_inner()
                .map(|x| parse_ty(x.into_inner().next().unwrap()))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Rule::unit => Ok(Ty::Unit),
        Rule::tyrecord => parse_tyrecord(pair.into_inner()),
        Rule::tyfun => {
            let mut inner = pair.into_inner();
            let lhs = parse_ty(inner.next().unwrap())?;
            let rhs = parse_ty(inner.next().unwrap().into_inner().next().unwrap())?;
            Ok(Ty::Fun(Box::new(lhs), Box::new(rhs)))
        }
        Rule::ty => parse_ty(pair.into_inner().next().unwrap()),
        r => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!(
                    "Unexpected rule {:?}, expected tyindent, tyrecord, unit, tyfun or tytuple",
                    r
                ),
            },
            pair.as_span(),
        )),
    }
}

fn parse_record(mut pairs: Pairs<Rule>) -> Result<Expr, Error<Rule>> {
    let mut xs = Vec::new();

    while let Some(ident) = pairs.next() {
        let expr = parse_exprs(pairs.next().unwrap().into_inner())?;
        xs.push((ident.as_str().to_owned(), expr));
    }

    xs.sort_by(|(x, _), (y, _)| x.cmp(y));

    Ok(Expr::Record(xs))
}

fn parse_let(mut pairs: Pairs<Rule>) -> Result<Expr, Error<Rule>> {
    let mut binds = Vec::new();

    loop {
        if let Some(pair) = pairs.next() {
            match pair.as_rule() {
                Rule::identifier => {
                    let expr = parse_exprs(pairs.next().unwrap().into_inner())?;
                    binds.push((pair.as_str().to_string(), expr));
                }
                Rule::expr => {
                    let expr = parse_exprs(pair.into_inner())?;
                    return Ok(Expr::Let(binds, Box::new(expr)));
                }
                other => {
                    unimplemented!("Invalid pair inside let-rule: {:?}", other);
                }
            }
        } else {
            unimplemented!("Invalid let binding {:?}", pairs)
        }
    }
}

fn parse_lambda(mut pairs: Pairs<Rule>) -> Result<Expr, Error<Rule>> {
    let ident = pairs.next().unwrap().as_str().to_string();
    let expr = parse_exprs(pairs.next().unwrap().into_inner())?;
    Ok(Expr::Lambda(ident, Box::new(expr)))
}

pub fn parse_term(term: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    match term.as_rule() {
        Rule::int => Ok(Expr::Int(term.as_str().parse().unwrap())),
        Rule::bool => Ok(Expr::Bool(term.as_str().parse().unwrap())),
        Rule::tuple => Ok(Expr::Tuple(
            term.into_inner()
                .map(|x| parse_exprs(x.into_inner()))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Rule::unit => Ok(Expr::Unit),
        Rule::string => Ok(Expr::String(
            term.into_inner().next().unwrap().as_str().to_string(),
        )),
        Rule::record => parse_record(term.into_inner()),
        Rule::identifier => Ok(Expr::Ident(term.as_str().to_string())),
        Rule::letbind => parse_let(term.into_inner()),
        Rule::lambda => parse_lambda(term.into_inner()),
        Rule::expr => parse_exprs(term.into_inner()),
        r => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("Unexpected rule {:?}, expected term", r),
            },
            term.as_span(),
        )),
    }
}

pub fn parse_exprs(mut exprs: Pairs<Rule>) -> Result<Expr, Error<Rule>> {
    let mut res = parse_term(exprs.next().unwrap().into_inner().next().unwrap())?;

    for term in exprs {
        res = Expr::Apply(
            Box::new(res),
            Box::new(parse_term(term.into_inner().next().unwrap())?),
        );
    }

    Ok(res)
}

pub fn parse_select(mut pairs: Pairs<Rule>) -> Result<Statement, Error<Rule>> {
    let ident = pairs.next().unwrap().as_str();

    Ok(Statement::Select(ident.to_string()))
}

pub fn parse_insert(mut pairs: Pairs<Rule>) -> Result<Statement, Error<Rule>> {
    let expr = parse_exprs(pairs.next().unwrap().into_inner())?;
    let ident = pairs.next().unwrap().as_str();

    Ok(Statement::Insert(ident.to_string(), expr))
}

pub fn parse_create(mut pairs: Pairs<Rule>) -> Result<Statement, Error<Rule>> {
    let ident = pairs.next().unwrap().as_str();
    let ty = parse_ty(pairs.next().unwrap().into_inner().next().unwrap())?;

    Ok(Statement::Create(ident.to_string(), TableDefinition { ty }))
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement, Error<Rule>> {
    match pair.as_rule() {
        Rule::create => Ok(parse_create(pair.into_inner())?),
        Rule::select => Ok(parse_select(pair.into_inner())?),
        Rule::insert => Ok(parse_insert(pair.into_inner())?),
        _ => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("Unexpected rule {:?}, expected statement", pair),
            },
            pair.as_span(),
        )),
    }
}

pub fn parse(input: &str) -> Result<Statement, Error<Rule>> {
    let statement = Parser::parse(Rule::statement, input)
        .unwrap()
        .next()
        .unwrap();

    parse_statement(statement)
}

#[cfg(test)]
mod test {
    use super::Ty;
    use super::*;

    fn parse_ty_helper(input: &str) -> Ty {
        super::parse_ty(Parser::parse(Rule::ty, input).unwrap().next().unwrap()).unwrap()
    }

    #[test]
    fn parse_ty() {
        use Ty::*;

        assert_eq!(Int, parse_ty_helper(&"((Int))"));
        assert_eq!(Bool, parse_ty_helper(&"Bool"));
        assert_eq!(Tuple(vec!(Int, Bool)), parse_ty_helper(&"((Int, Bool))"));
        assert_eq!(Unit, parse_ty_helper(&"()"));
        assert_eq!(String, parse_ty_helper(&"String"));
        assert_eq!(Var("Foo".to_string()), parse_ty_helper(&"Foo"));
        assert_eq!(Int, parse_ty_helper(&"((Int))"));
        assert_eq!(
            Fun(Box::new(Int), Box::new(Int)),
            parse_ty_helper(&"(Int -> Int)")
        );
        assert_eq!(
            Fun(Box::new(Fun(Box::new(Int), Box::new(Int))), Box::new(Int)),
            parse_ty_helper(&"(Int -> Int) -> Int")
        );
        assert_eq!(
            Fun(Box::new(Int), Box::new(Fun(Box::new(Int), Box::new(Int)))),
            parse_ty_helper(&"Int -> Int -> Int")
        );
        assert_eq!(
            Fun(Box::new(Int), Box::new(Fun(Box::new(Int), Box::new(Int)))),
            parse_ty_helper(&"Int -> (Int -> Int)")
        );
    }

    fn parse_exprs_helper(input: &str) -> Expr {
        super::parse_exprs(
            Parser::parse(Rule::expr, input)
                .unwrap_or_else(|e| panic!("{}", e))
                .next()
                .unwrap()
                .into_inner(),
        )
        .unwrap()
    }

    #[test]
    fn parse_exprs() {
        use Expr::*;

        assert_eq!(Int(4), parse_exprs_helper(&"4"));
        assert_eq!(Ident("x".to_string()), parse_exprs_helper(&"x"));
        assert_eq!(
            Apply(
                Box::new(Ident("x".to_string())),
                Box::new(Ident("y".to_string()))
            ),
            parse_exprs_helper(&"x y")
        );
        assert_eq!(
            Apply(
                Box::new(Apply(
                    Box::new(Ident("x".to_string())),
                    Box::new(Ident("y".to_string()))
                )),
                Box::new(Ident("z".to_string()))
            ),
            parse_exprs_helper(&"x y z")
        );
        assert_eq!(
            Apply(
                Box::new(Apply(
                    Box::new(Ident("x".to_string())),
                    Box::new(Ident("y".to_string()))
                )),
                Box::new(Ident("z".to_string()))
            ),
            parse_exprs_helper(&"(x y) z")
        );
        assert_eq!(
            Apply(
                Box::new(Ident("x".to_string())),
                Box::new(Apply(
                    Box::new(Ident("y".to_string())),
                    Box::new(Ident("z".to_string()))
                ))
            ),
            parse_exprs_helper(&"x (y z)")
        );
        assert_eq!(
            Apply(
                Box::new(Ident("x".to_string())),
                Box::new(Tuple(vec!(Ident("y".to_string()), Ident("z".to_string()))))
            ),
            parse_exprs_helper(&"x (y, z)")
        );
        assert_eq!(
            Lambda("f".to_string(), Box::new(Ident("x".to_string()))),
            parse_exprs_helper(&"lambda f -> x")
        );
        assert_eq!(
            Let(
                vec!(("x".to_string(), Int(42))),
                Box::new(Ident("x".to_string()))
            ),
            parse_exprs_helper(&"let x = 42 in x end")
        );
    }

    #[test]
    fn parse_and_print_is_isomorph() {
        fn isomorph(input: &str) {
            let s = format!("{}", parse_exprs_helper(input));
            assert_eq!(input, s);
        }
        isomorph("42");

        isomorph("let x = 42 in x end");

        isomorph("let id = lambda x -> x in id end");
    }

    #[test]
    fn parse_insert() {
        use Expr::*;
        use Statement::*;

        assert_eq!(
            Insert("x".to_string(), Int(4)),
            super::parse_insert(
                Parser::parse(Rule::insert, &"insert 4 into x")
                    .unwrap()
                    .next()
                    .unwrap()
                    .into_inner(),
            )
            .unwrap()
        );
        assert_eq!(
            Insert(
                "x".to_string(),
                Apply(Box::new(Ident("f".to_string())), Box::new(Int(4)))
            ),
            super::parse_insert(
                Parser::parse(Rule::insert, &"insert f 4 into x")
                    .unwrap()
                    .next()
                    .unwrap()
                    .into_inner(),
            )
            .unwrap()
        );
    }

    #[test]
    fn parse_create_int() {
        assert_eq!(
            Statement::Create(String::from("x"), TableDefinition { ty: Ty::Int }),
            parse("create table x Int").unwrap()
        );
    }

    #[test]
    fn parse_create_bool() {
        assert_eq!(
            Statement::Create(String::from("x"), TableDefinition { ty: Ty::Bool }),
            parse("create table x Bool").unwrap()
        );
    }

    #[test]
    fn parse_create_tuple() {
        assert_eq!(
            Statement::Create(
                String::from("x"),
                TableDefinition {
                    ty: Ty::Tuple(vec!(Ty::Bool, Ty::Int))
                }
            ),
            parse("create table x (Bool, Int)").unwrap()
        );
    }

    #[test]
    fn parse_create_nested_tuple() {
        assert_eq!(
            Statement::Create(
                String::from("x"),
                TableDefinition {
                    ty: Ty::Tuple(vec!(Ty::Bool, Ty::Int, Ty::Tuple(vec!(Ty::Int, Ty::Int))))
                }
            ),
            parse("create table x (Bool, Int, (Int, Int,))").unwrap()
        );
    }

    #[test]
    fn parse_create_unit() {
        assert_eq!(
            Statement::Create(String::from("x"), TableDefinition { ty: Ty::Unit }),
            parse("create table x ()").unwrap()
        );
    }

    #[test]
    fn parse_create_record() {
        assert_eq!(
            Statement::Create(
                String::from("x"),
                TableDefinition {
                    ty: Ty::Record(vec!(
                        (String::from("x"), Ty::Bool),
                        (String::from("y"), Ty::Int)
                    ))
                }
            ),
            parse("create table x { y : Int, x : Bool }").unwrap()
        );
    }

    #[test]
    fn parse_insert_int() {
        assert_eq!(
            Statement::Insert(String::from("x"), Expr::Int(42)),
            parse("insert 42 into x").unwrap()
        )
    }

    #[test]
    fn parse_insert_negative_int() {
        assert_eq!(
            Statement::Insert(String::from("x"), Expr::Int(-42)),
            parse("insert -42 into x").unwrap()
        )
    }

    #[test]
    fn parse_insert_0() {
        assert_eq!(
            Ok(Statement::Insert(String::from("x"), Expr::Int(0))),
            parse("insert 0 into x")
        )
    }

    #[test]
    fn parse_insert_negative_0() {
        assert_eq!(
            Statement::Insert(String::from("x"), Expr::Int(0)),
            parse("insert -0 into x").unwrap()
        )
    }

    #[test]
    fn parse_insert_bool() {
        assert_eq!(
            Statement::Insert(String::from("x"), Expr::Bool(false)),
            parse("insert false into x").unwrap()
        );
    }

    #[test]
    fn parse_insert_tuple() {
        assert_eq!(
            Statement::Insert(
                String::from("x"),
                Expr::Tuple(vec!(Expr::Bool(false), Expr::Bool(true), Expr::Int(42)))
            ),
            parse("insert (false, true, 42) into x").unwrap()
        );
    }

    #[test]
    fn parse_insert_unit() {
        assert_eq!(
            Statement::Insert(String::from("x"), Expr::Unit),
            parse("insert () into x").unwrap()
        );
    }

    #[test]
    fn parse_insert_record() {
        assert_eq!(
            Statement::Insert(
                String::from("x"),
                Expr::Record(vec!(
                    (String::from("x"), Expr::Bool(false)),
                    (String::from("y"), Expr::Int(42)),
                ))
            ),
            parse("insert { y = 42, x = false, } into x").unwrap()
        );
    }

    #[test]
    fn parse_insert_record_2() {
        assert_eq!(
            Statement::Insert(
                String::from("bar"),
                Expr::Record(vec!(
                    (String::from("x"), Expr::Int(0)),
                    (String::from("y"), Expr::Int(42)),
                ))
            ),
            parse("insert { y = 42, x = 0 } into bar").unwrap()
        );
    }

    #[test]
    fn parse_select() {
        assert_eq!(
            Statement::Select(String::from("x")),
            parse("select from x").unwrap()
        );
    }
}

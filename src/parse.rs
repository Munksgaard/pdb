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

fn parse_ty(pair: Pair<Rule>) -> Result<Ty, Error<Rule>> {
    match pair.as_rule() {
        Rule::tyident => match pair.as_str() {
            "Int" => Ok(Ty::Int),
            "Bool" => Ok(Ty::Bool),
            x => Err(Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: format!("Invalid type {}", x),
                },
                pair.as_span(),
            )),
        },
        Rule::tytuple => Ok(Ty::Tuple(
            pair.into_inner()
                .map(|x| parse_ty(x.into_inner().next().unwrap()))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Rule::unit => Ok(Ty::Unit),
        Rule::tyrecord => parse_tyrecord(pair.into_inner()),
        r => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!(
                    "Unexpected rule {:?}, expected tyindent, tyrecord, unit or tytuple",
                    r
                ),
            },
            pair.as_span(),
        )),
    }
}

pub fn parse_tabledef(input: &str) -> Result<TableDefinition, Error<Rule>> {
    let parsed = Parser::parse(Rule::table, input)?.next().unwrap();

    let pair = parsed.into_inner().next().unwrap();

    let ty = match pair.as_rule() {
        Rule::ty => parse_ty(pair.into_inner().next().unwrap()),
        _ => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: "Unexpected rule, expected ty".to_string(),
            },
            pair.as_span(),
        )),
    }?;

    Ok(TableDefinition { ty: ty })
}

fn parse_record(mut pairs: Pairs<Rule>) -> Result<Expr, Error<Rule>> {
    let mut xs = Vec::new();

    while let Some(ident) = pairs.next() {
        let expr = parse_expr(pairs.next().unwrap())?;
        xs.push((ident.as_str().to_owned(), expr));
    }

    xs.sort_by(|(x, _), (y, _)| x.cmp(y));

    Ok(Expr::Record(xs))
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    let expr = pair.into_inner().next().unwrap();
    match expr.as_rule() {
        Rule::int => Ok(Expr::Int(expr.as_str().parse().unwrap())),
        Rule::bool => Ok(Expr::Bool(expr.as_str().parse().unwrap())),
        Rule::tuple => Ok(Expr::Tuple(
            expr.into_inner()
                .map(parse_expr)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Rule::unit => Ok(Expr::Unit),
        Rule::record => parse_record(expr.into_inner()),
        r => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("Unexpected rule {:?}, expected expr", r),
            },
            expr.as_span(),
        )),
    }
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement, Error<Rule>> {
    match pair.as_rule() {
        Rule::select => Ok(Statement::Select),
        Rule::insert => Ok(Statement::Insert(parse_expr(
            pair.into_inner().next().unwrap(),
        )?)),
        _ => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("Unexpected rule {:?}, expected statement", pair),
            },
            pair.as_span(),
        )),
    }
}

pub fn parse(input: &str) -> Result<Statement, Error<Rule>> {
    let statement = Parser::parse(Rule::statement, input)?.next().unwrap();

    parse_statement(statement)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_tabledef_test() {
        use super::Ty;

        assert_eq!(
            TableDefinition { ty: Ty::Int },
            parse_tabledef("table Int").unwrap()
        );

        assert_eq!(
            TableDefinition { ty: Ty::Bool },
            parse_tabledef("table Bool").unwrap()
        );

        assert_eq!(
            TableDefinition {
                ty: Ty::Tuple(vec!(Ty::Bool, Ty::Int))
            },
            parse_tabledef("table (Bool, Int)").unwrap()
        );

        assert_eq!(
            TableDefinition {
                ty: Ty::Tuple(vec!(Ty::Bool, Ty::Int, Ty::Tuple(vec!(Ty::Int, Ty::Int))))
            },
            parse_tabledef("table (Bool, Int, (Int, Int,))").unwrap()
        );

        assert_eq!(
            TableDefinition { ty: Ty::Unit },
            parse_tabledef("table ()").unwrap()
        );

        assert_eq!(
            TableDefinition {
                ty: Ty::Record(vec!(
                    (String::from("x"), Ty::Bool),
                    (String::from("y"), Ty::Int)
                ))
            },
            parse_tabledef("table { y : Int, x : Bool }").unwrap()
        );
    }

    #[test]
    fn parse_statement_test() {
        use super::Expr;
        use super::Statement;

        assert_eq!(
            Statement::Insert(Expr::Int(42)),
            parse("insert 42").unwrap()
        );

        assert_eq!(
            Statement::Insert(Expr::Bool(false)),
            parse("insert false").unwrap()
        );

        assert_eq!(
            Statement::Insert(Expr::Tuple(vec!(
                Expr::Bool(false),
                Expr::Bool(true),
                Expr::Int(42)
            ))),
            parse("insert (false, true, 42)").unwrap()
        );

        assert_eq!(Statement::Insert(Expr::Unit), parse("insert ()").unwrap());

        assert_eq!(
            Statement::Insert(Expr::Record(vec!(
                (String::from("x"), Expr::Bool(false)),
                (String::from("y"), Expr::Int(42)),
            ))),
            parse("insert { y = 42, x = false, }").unwrap()
        );

        assert_eq!(Statement::Select, parse("select").unwrap());
    }
}

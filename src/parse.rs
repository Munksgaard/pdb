use crate::ast::*;
use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser as _;

#[cfg(test)]
mod test;

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
        Rule::bool => Ok(Expr::Bool(match term.as_str() {
            "True" => true,
            _ => false,
        })),
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

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
        let ty = parse_ty(pairs.next().unwrap().into_inner())?;
        xs.push((ident.as_str().to_owned(), ty));
    }

    xs.sort_by(|(x, _), (y, _)| x.cmp(y));

    Ok(Ty::Record(xs))
}

pub fn parse_tyterm(pair: Pair<Rule>) -> Result<Ty, Error<Rule>> {
    match pair.as_rule() {
        Rule::unit => Ok(Ty::Unit),
        Rule::tytuple => Ok(Ty::Tuple(
            pair.into_inner()
                .map(|x| parse_ty(x.into_inner()))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Rule::tyrecord => parse_tyrecord(pair.into_inner()),
        Rule::identifier => Ok(Ty::Var(pair.as_str().to_string())),
        Rule::tyident => match pair.as_str() {
            "Int" => Ok(Ty::Int),
            "Bool" => Ok(Ty::Bool),
            "String" => Ok(Ty::String),
            _ => Ok(Ty::Defined(pair.as_str().to_string(), vec![])),
        },
        Rule::ty => parse_ty(pair.into_inner()),
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

pub fn parse_ty(mut pairs: Pairs<Rule>) -> Result<Ty, Error<Rule>> {
    let t = pairs.next().unwrap();
    match t.as_rule() {
        Rule::longtyident => parse_longtyident(t.into_inner()),
        Rule::tyfun => {
            let mut inner = t.into_inner();
            let lhs = parse_tyterm(inner.next().unwrap().into_inner().next().unwrap())?;
            let rhs = parse_ty(inner.next().unwrap().into_inner())?;
            Ok(Ty::Fun(Box::new(lhs), Box::new(rhs)))
        }
        Rule::tyterm => parse_tyterm(t.into_inner().next().unwrap()),
        r => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("Unexpected rule {:?}, expected tycon, tyfun or tyterm", r),
            },
            t.as_span(),
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

fn parse_case(mut pairs: Pairs<Rule>) -> Result<Expr, Error<Rule>> {
    let expr = parse_exprs(pairs.next().unwrap().into_inner())?;

    let mut matches = Vec::new();

    while let Some(pat) = pairs.next() {
        let pat_expr = pairs.next().unwrap().into_inner();
        matches.push((parse_pat(pat.into_inner())?, parse_exprs(pat_expr)?));
    }

    Ok(Expr::Case(Box::new(expr), matches))
}

fn parse_pat(mut pairs: Pairs<Rule>) -> Result<Pattern, Error<Rule>> {
    let pat = pairs.next().unwrap();
    match pat.as_rule() {
        Rule::atom => Ok(Pattern::Atom(parse_atom(pat.into_inner().next().unwrap())?)),
        Rule::tuple_pat => {
            let mut pats = Vec::new();

            for pair in pat.into_inner().into_iter() {
                pats.push(parse_pat(pair.into_inner())?);
            }

            Ok(Pattern::Tuple(pats))
        }
        Rule::record_pat => {
            let mut xs = Vec::new();
            let mut pairs = pat.into_inner();

            while let Some(ident) = pairs.next() {
                let ty = parse_pat(pairs.next().unwrap().into_inner())?;
                xs.push((ident.as_str().to_owned(), ty));
            }

            xs.sort_by(|(x, _), (y, _)| x.cmp(y));

            Ok(Pattern::Record(xs))
        }
        Rule::wildcard => Ok(Pattern::Wildcard),
        Rule::identifier => Ok(Pattern::Ident(pat.as_str().to_string())),
        r => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("Unexpected rule {:?}, expected pattern", r),
            },
            pat.as_span(),
        )),
    }
}

pub fn parse_atom(atom: Pair<Rule>) -> Result<Atom, Error<Rule>> {
    match atom.as_rule() {
        Rule::int => Ok(Atom::Int(atom.as_str().parse().unwrap())),
        Rule::bool => Ok(Atom::Bool(matches!(atom.as_str(), "True"))),
        Rule::unit => Ok(Atom::Unit),
        Rule::string => Ok(Atom::String(
            atom.into_inner().next().unwrap().as_str().to_string(),
        )),
        r => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("Unexpected rule {:?}, expected atom", r),
            },
            atom.as_span(),
        )),
    }
}

pub fn parse_term(term: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    match term.as_rule() {
        Rule::atom => Ok(Expr::Atom(parse_atom(term.into_inner().next().unwrap())?)),
        Rule::identifier => Ok(Expr::Ident(term.as_str().to_string())),
        Rule::tyident => Ok(Expr::Ident(term.as_str().to_string())),
        Rule::tuple => Ok(Expr::Tuple(
            term.into_inner()
                .map(|x| parse_exprs(x.into_inner()))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Rule::record => parse_record(term.into_inner()),
        Rule::letbind => parse_let(term.into_inner()),
        Rule::lambda => parse_lambda(term.into_inner()),
        Rule::case => parse_case(term.into_inner()),
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
    let ty = parse_ty(pairs.next().unwrap().into_inner())?;

    Ok(Statement::Create(ident.to_string(), TableDefinition { ty }))
}

pub fn parse_letdecl(mut pairs: Pairs<Rule>) -> Result<Statement, Error<Rule>> {
    let ident = pairs.next().unwrap().as_str();
    let expr = parse_exprs(pairs.next().unwrap().into_inner())?;

    Ok(Statement::Let(ident.to_string(), expr))
}

pub fn parse_longtyident(mut pairs: Pairs<Rule>) -> Result<Ty, Error<Rule>> {
    let ident = pairs.next().unwrap().as_str().to_string();

    let mut args = Vec::new();

    while let Some(pair) = pairs.next() {
        args.push(parse_tyterm(pair.into_inner().next().unwrap())?);
    }

    Ok(Ty::Defined(ident, args))
}

pub fn parse_tycon(mut pairs: Pairs<Rule>) -> Result<(Ident, Vec<Ty>), Error<Rule>> {
    let ident = pairs.next().unwrap().as_str().to_string();

    let mut args = Vec::new();

    while let Some(pair) = pairs.next() {
        args.push(parse_tyterm(pair.into_inner().next().unwrap())?);
    }

    Ok((ident, args))
}

pub fn parse_datatype(mut pairs: Pairs<Rule>) -> Result<Statement, Error<Rule>> {
    let ident = pairs.next().unwrap().as_str();

    let mut args = Vec::new();

    while let Some(Rule::identifier) = pairs.peek().map(|x| x.as_rule()) {
        args.push(pairs.next().unwrap().as_str().to_string());
    }

    let mut variants = Vec::new();

    for pair in pairs {
        variants.push(parse_tycon(pair.into_inner())?);
    }

    Ok(Statement::Union(ident.to_string(), args, variants))
}

pub fn parse_statement(pair: Pair<Rule>) -> Result<Statement, Error<Rule>> {
    match pair.as_rule() {
        Rule::create => Ok(parse_create(pair.into_inner())?),
        Rule::select => Ok(parse_select(pair.into_inner())?),
        Rule::insert => Ok(parse_insert(pair.into_inner())?),
        Rule::letdecl => Ok(parse_letdecl(pair.into_inner())?),
        Rule::datatype => Ok(parse_datatype(pair.into_inner())?),
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

use crate::ast::*;
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser as _;

#[derive(Parser)]
#[grammar = "pdb.pest"]
pub struct Parser;

fn parse_ty(pair: Pair<Rule>) -> Result<Ty, Error<Rule>> {
    match pair.as_rule() {
        Rule::typ => match pair.as_str() {
            "Int" => Ok(Ty::Int),
            "Bool" => Ok(Ty::Bool),
            x => Err(Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: format!("Invalid type {}", x),
                },
                pair.as_span(),
            )),
        },
        _ => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: "Unexpected rule, expected ty".to_string(),
            },
            pair.as_span(),
        )),
    }
}

fn parse_tydef(pair: Pair<Rule>) -> Result<TyDef, Error<Rule>> {
    match pair.as_rule() {
        Rule::tydef => Ok(TyDef(parse_ty(pair.into_inner().next().unwrap())?)),
        _ => Err(Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: "Unexpected rule, expected tydef".to_string(),
            },
            pair.as_span(),
        )),
    }
}

pub fn parse_tabledef(input: &str) -> Result<TableDefinition, Error<Rule>> {
    let parsed = Parser::parse(Rule::table, input)?.next().unwrap();

    let pair = parsed.into_inner().next().unwrap();

    Ok(TableDefinition {
        tydef: parse_tydef(pair)?,
    })
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, Error<Rule>> {
    let expr = pair.into_inner().next().unwrap();
    match expr.as_rule() {
        Rule::int => Ok(Expr::Int(expr.as_str().parse().unwrap())),
        Rule::bool => Ok(Expr::Bool(expr.as_str().parse().unwrap())),
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
    fn table_parsing() {
        assert!(Parser::parse(Rule::table, "table ()").is_err());
    }

    #[test]
    fn parse_tabledef_test() {
        use super::Ty;

        assert_eq!(
            TableDefinition {
                tydef: TyDef(Ty::Int)
            },
            parse_tabledef("table Int").unwrap()
        );

        assert_eq!(
            TableDefinition {
                tydef: TyDef(Ty::Bool)
            },
            parse_tabledef("table Bool").unwrap()
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

        assert_eq!(Statement::Select, parse("select").unwrap());
    }
}

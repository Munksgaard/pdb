use crate::ast::Statement;
use crate::eval::eval;
use crate::object::Object;
use crate::parse::{parse, parse_tabledef};
use crate::ty::unify;
use std::io::{BufRead, Write};

const PROMPT: &[u8; 3] = b">> ";

pub fn start<R, W>(reader: &mut R, writer: &mut W) -> Result<(), Box<dyn std::error::Error>>
where
    R: BufRead,
    W: Write,
{
    let mut table: Vec<Object> = Vec::new();

    let tabledef = {
        writer.write_all(b"Enter table definition: ")?;
        writer.flush()?;

        let mut line = String::new();
        reader.read_line(&mut line)?;

        parse_tabledef(&line)?
    };

    loop {
        writer.write_all(PROMPT)?;
        writer.flush()?;

        let mut line = String::new();
        reader.read_line(&mut line)?;

        let ast = parse(&line)?;

        match ast {
            Statement::Insert(expr) => {
                if unify(&expr, &tabledef) {
                    let result = eval(expr);
                    table.push(result);
                    writer.write_all(b"Inserted 1\n")?;
                } else {
                    writer.write_all(
                        format!(
                            "Could not insert {:?} into table with definition {:?}\n",
                            expr, tabledef
                        )
                        .as_bytes(),
                    )?;
                }
                writer.flush()?;
            }
            Statement::Select => {
                writer.write_all(format!("{:?}\n", table).as_bytes())?;
                writer.flush()?;
            }
        }
    }
}

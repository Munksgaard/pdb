use crate::ast::{Ident, Statement, TableDefinition};
use crate::eval::eval;
use crate::object::Object;
use crate::parse::parse;
use crate::ty::matches_type;
use std::io::{BufRead, Write};

const PROMPT: &[u8; 3] = b">> ";

pub fn start<R, W>(reader: &mut R, writer: &mut W) -> Result<(), Box<dyn std::error::Error>>
where
    R: BufRead,
    W: Write,
{
    let mut tables: Vec<(Ident, TableDefinition, Vec<Object>)> = Vec::new();

    loop {
        writer.write_all(PROMPT)?;
        writer.flush()?;

        let mut line = String::new();
        reader.read_line(&mut line)?;

        if &line == "" {
            writer.write_all(b"\n")?;
            writer.flush()?;
            return Ok(());
        }

        match parse(&line) {
            Ok(ast) => match ast {
                Statement::Create(ident, def) => {
                    tables.push((ident, def, Vec::new()));
                    writer.write_all(b"Created\n")?;
                    writer.flush()?;
                }
                Statement::Insert(ident, expr) => {
                    if let Some((_, def, objs)) =
                        tables.iter_mut().find(|(ident2, _, _)| ident2 == &ident)
                    {
                        if matches_type(&expr, &def.ty) {
                            let result = eval(expr);
                            objs.push(result);
                            writer.write_all(b"Inserted 1\n")?;
                        } else {
                            writer.write_all(
                                format!(
                                    "Could not insert {:?} into table {:?} with definition {:?}\n",
                                    expr, ident, &def.ty
                                )
                                .as_bytes(),
                            )?;
                        }
                    } else {
                        writer.write_all(b"No such table\n")?;
                    }
                    writer.flush()?;
                }
                Statement::Select(ident) => {
                    if let Some((_, _, objs)) =
                        tables.iter().find(|(ident2, _, _)| ident2 == &ident)
                    {
                        writer.write_all(format!("{:?}\n", objs).as_bytes())?;
                    } else {
                        writer.write_all(b"No such table\n")?;
                    }
                    writer.flush()?;
                }
            },
            Err(e) => {
                writer.write_all(format!("No parse: {}\n", e).as_bytes())?;
                writer.flush()?;
            }
        }
    }
}

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
        parse("insert False into x").unwrap()
    );
}

#[test]
fn parse_insert_tuple() {
    assert_eq!(
        Statement::Insert(
            String::from("x"),
            Expr::Tuple(vec!(Expr::Bool(false), Expr::Bool(true), Expr::Int(42)))
        ),
        parse("insert (False, True, 42) into x").unwrap()
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
        parse("insert { y = 42, x = False, } into x").unwrap()
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

#[test]
fn parse_letdecl() {
    assert_eq!(
        Statement::Let(String::from("x"), Expr::Int(42)),
        parse("let x = 42").unwrap()
    );
}

use crate::ast::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter;

/// Type substitutions
type Substitution = (Ident, Ty);

type GlobalSub = HashMap<Ident, Ty>;

type Constraint = (Ty, Ty);

/// Type scheme
type Scheme = (Vec<Ident>, Ty);

/// Type Environment
type Env = HashMap<Ident, Scheme>;

pub fn unify(
    mut constraints: impl Iterator<Item = Constraint>,
) -> Box<dyn Iterator<Item = Result<Substitution, String>>> {
    match constraints.next() {
        None => Box::new(iter::empty()),
        Some((t1, t2)) if t1 == t2 => unify(constraints),
        Some((Ty::Var(ident), other)) if other.fv().all(|x| x != ident) => {
            let subst = (ident, other);
            let tmp: Vec<_> = constraints.map(|x| x.apply(&subst)).collect();
            Box::new(unify(tmp.into_iter()).chain(iter::once(Ok(subst))))
        }
        Some((other, Ty::Var(ident))) if other.fv().all(|x| x != ident) => {
            let subst = (ident, other);
            let tmp: Vec<_> = constraints.map(|x| x.apply(&subst)).collect();
            Box::new(unify(tmp.into_iter()).chain(iter::once(Ok(subst))))
        }
        Some((Ty::Fun(lhs1, rhs1), Ty::Fun(lhs2, rhs2))) => {
            let tmp: Vec<_> = iter::once((*lhs1, *lhs2))
                .chain(iter::once((*rhs1, *rhs2)))
                .chain(constraints)
                .collect();
            Box::new(unify(tmp.into_iter()))
        }
        Some((Ty::Tuple(tys1), Ty::Tuple(tys2))) if tys1.len() == tys2.len() => {
            let tmp: Vec<_> = tys1.into_iter().zip(tys2).chain(constraints).collect();
            Box::new(unify(tmp.into_iter()))
        }
        Some((Ty::Record(mut tys1), Ty::Record(mut tys2))) => {
            tys1.sort_by_key(|(k, _)| k.clone());
            tys2.sort_by_key(|(k, _)| k.clone());

            if tys1.len() == tys2.len()
                && tys1.iter().map(|(k, _)| k).eq(tys2.iter().map(|(k, _)| k))
            {
                let tmp: Vec<_> = tys1
                    .into_iter()
                    .map(|(_, x)| x)
                    .zip(tys2.into_iter().map(|(_, x)| x))
                    .chain(constraints)
                    .collect();
                Box::new(unify(tmp.into_iter()))
            } else {
                Box::new(iter::once(Err(format!(
                    "Could not unify records {} and {}",
                    Ty::Record(tys1),
                    Ty::Record(tys2)
                ))))
            }
        }
        Some((t1, t2)) => Box::new(iter::once(Err(format!(
            "Could not unify {} and {}",
            t1, t2
        )))),
    }
}

#[derive(Debug)]
pub struct NameSource {
    counter: i64,
}

impl NameSource {
    fn new() -> Self {
        NameSource { counter: 0 }
    }

    fn fresh(&mut self, name: &str) -> Ident {
        let i = self.counter;
        self.counter += 1;
        format!("{}_{}", name, i)
    }
}

/// `infer` is based on Algorithm J
/// Inspiration from this paper:
/// https://www.cl.cam.ac.uk/teaching/1415/L28/type-inference.pdf
pub fn infer(
    indent: usize,
    global_sub: &mut GlobalSub,
    name_src: &mut NameSource,
    env: &Env,
    expr: &Expr,
) -> Result<Ty, String> {
    match expr {
        Expr::Int(_) => Ok(Ty::Int),
        Expr::Bool(_) => Ok(Ty::Bool),
        Expr::Ident(ident) => {
            let scheme = env
                .get(ident)
                .ok_or_else(|| format!("Identifier {} not found in environment", ident))?;
            Ok(instantiate(scheme, name_src))
        }
        Expr::Let(binds, expr) => {
            let mut env = env.clone();
            for (ident, e) in binds {
                let ty = infer(indent + 1, global_sub, name_src, &env, e)?;

                let scheme = generalize(&env, ty);

                env.insert(ident.clone(), scheme);
            }

            infer(indent + 1, global_sub, name_src, &env, expr)
        }
        Expr::Lambda(ident, e) => {
            let freshvar = name_src.fresh(&ident);
            let mut env = env.clone();
            env.insert(ident.clone(), (vec![], Ty::Var(freshvar.clone())));
            let rhs = infer(indent + 1, global_sub, name_src, &env, e)?;

            let lhs = global_sub
                .get(&freshvar)
                .cloned()
                .unwrap_or(Ty::Var(freshvar));

            Ok(Ty::Fun(Box::new(lhs), Box::new(rhs)))
        }
        Expr::Apply(e1, e2) => {
            let t1 = infer(indent + 1, global_sub, name_src, env, e1)?;
            let t2 = infer(indent + 1, global_sub, name_src, env, e2)?;
            let fresh = name_src.fresh(&"arg");
            let substs = unify(iter::once((
                t1,
                Ty::Fun(Box::new(t2), Box::new(Ty::Var(fresh.clone()))),
            )))
            .collect::<Result<Vec<_>, String>>()?;

            // Apply substs in global substitution
            for (ident, ty) in substs {
                global_sub.insert(ident, ty);
            }

            let lhs = global_sub.get(&fresh).cloned().unwrap_or(Ty::Var(fresh));

            Ok(lhs)
        }

        e => unimplemented!("Expr {:?} not supported", e),
    }
}

pub fn matches_type(expr: &Expr, ty: &Ty) -> bool {
    match (expr, ty) {
        (Expr::Int(_), Ty::Int) => true,
        (Expr::Bool(_), Ty::Bool) => true,
        (Expr::Tuple(exprs), Ty::Tuple(tys)) => exprs
            .iter()
            .zip(tys.iter())
            .all(|(x, y)| matches_type(x, y)),
        (Expr::Unit, Ty::Unit) => true,
        (Expr::String(_), Ty::String) => true,
        (Expr::Record(expr_pairs), Ty::Record(ty_pairs)) => expr_pairs
            .iter()
            .zip(ty_pairs.iter())
            .all(|((exprident, expr), (tyident, ty))| {
                exprident == tyident && matches_type(expr, ty)
            }),
        _ => false,
    }
}

fn instantiate(scheme: &Scheme, name_src: &mut NameSource) -> Ty {
    let subs = scheme
        .0
        .iter()
        .map(|ident| (ident.clone(), Ty::Var(name_src.fresh(&ident))));
    let mut res = scheme.1.clone();
    for sub in subs {
        res = res.apply(&sub);
    }
    res
}

fn generalize(env: &Env, ty: Ty) -> Scheme {
    let env_fvs: HashSet<_> = env.fv().collect();
    (
        ty.fv()
            .collect::<HashSet<_>>()
            .difference(&env_fvs)
            .into_iter()
            .cloned()
            .collect(),
        ty,
    )
}

trait Substitute {
    fn apply(&self, substitution: &Substitution) -> Self;
}

impl Substitute for Ty {
    fn apply(&self, substitution: &Substitution) -> Self {
        match self {
            Ty::Int => Ty::Int,
            Ty::Bool => Ty::Bool,
            Ty::Tuple(tys) => Ty::Tuple(tys.iter().map(|x| x.apply(substitution)).collect()),
            Ty::Unit => Ty::Unit,
            Ty::String => Ty::String,
            Ty::Record(recs) => Ty::Record(
                recs.iter()
                    .map(|(ident, ty)| (ident.clone(), ty.apply(substitution)))
                    .collect(),
            ),
            Ty::Var(ident) => {
                if ident == &substitution.0 {
                    substitution.1.clone()
                } else {
                    Ty::Var(ident.clone())
                }
            }
            Ty::Fun(lhs, rhs) => Ty::Fun(
                Box::new(lhs.apply(substitution)),
                Box::new(rhs.apply(substitution)),
            ),
        }
    }
}

impl Substitute for Constraint {
    fn apply(&self, substitution: &Substitution) -> Self {
        (self.0.apply(substitution), self.1.apply(substitution))
    }
}

trait SubstituteMut {
    fn apply_mut(&mut self, substitution: &Substitution);
}

impl SubstituteMut for Ty {
    fn apply_mut(&mut self, substitution: &Substitution) {
        match self {
            Ty::Tuple(tys) => {
                for ty in tys.iter_mut() {
                    ty.apply_mut(substitution)
                }
            }
            Ty::Record(recs) => {
                for (_, ty) in recs.iter_mut() {
                    ty.apply_mut(substitution)
                }
            }
            Ty::Var(ident) => {
                if ident == &substitution.0 {
                    *self = substitution.1.clone();
                }
            }
            Ty::Fun(lhs, rhs) => {
                lhs.apply_mut(substitution);
                rhs.apply_mut(substitution);
            }
            _ => (),
        }
    }
}

impl SubstituteMut for Constraint {
    fn apply_mut(&mut self, substitution: &Substitution) {
        self.0.apply_mut(substitution);
        self.1.apply_mut(substitution);
    }
}

impl SubstituteMut for Scheme {
    fn apply_mut(&mut self, substitution: &Substitution) {
        if !self.0.contains(&substitution.0) {
            self.1.apply_mut(substitution);
        }
    }
}

impl SubstituteMut for Env {
    fn apply_mut(&mut self, substitution: &Substitution) {
        for x in self.values_mut() {
            x.apply_mut(substitution);
        }
    }
}

/// An iterator over free variables
trait FreeVars {
    fn fv(&self) -> Box<dyn Iterator<Item = Ident> + '_>;
}

impl FreeVars for Ty {
    fn fv(&self) -> Box<dyn Iterator<Item = Ident> + '_> {
        match self {
            Ty::Int => Box::new(iter::empty()),
            Ty::Bool => Box::new(iter::empty()),
            Ty::Tuple(tys) => Box::new(tys.iter().flat_map(|x| x.fv())),
            Ty::Unit => Box::new(iter::empty()),
            Ty::String => Box::new(iter::empty()),
            Ty::Record(recs) => Box::new(recs.iter().flat_map(|(_, x)| x.fv())),
            Ty::Var(ident) => Box::new(iter::once(ident.clone())),
            Ty::Fun(lhs, rhs) => Box::new(lhs.fv().chain(rhs.fv())),
        }
    }
}

impl FreeVars for Scheme {
    fn fv(&self) -> Box<dyn Iterator<Item = Ident> + '_> {
        Box::new(self.1.fv().filter(move |x| !self.0.contains(x)))
    }
}

impl FreeVars for Env {
    fn fv(&self) -> Box<dyn Iterator<Item = Ident> + '_> {
        Box::new(self.iter().flat_map(|(_, x)| x.fv()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matches_type_int() {
        assert!(matches_type(&Expr::Int(42), &Ty::Int));
    }

    #[test]
    fn matches_type_int_bool_fails() {
        assert!(!matches_type(&Expr::Int(42), &Ty::Bool));
    }

    #[test]
    fn matches_type_bool() {
        assert!(matches_type(&Expr::Bool(true), &Ty::Bool));
    }

    #[test]
    fn matches_type_tuple() {
        assert!(matches_type(
            &Expr::Tuple(vec!(Expr::Bool(true), Expr::Int(42))),
            &Ty::Tuple(vec!(Ty::Bool, Ty::Int))
        ));
    }

    #[test]
    fn matches_type_unit() {
        assert!(matches_type(&Expr::Unit, &Ty::Unit));
    }

    #[test]
    fn matches_type_record() {
        assert!(matches_type(
            &Expr::Record(vec!(
                (String::from("x"), Expr::Bool(true)),
                (String::from("y"), Expr::Int(42))
            )),
            &Ty::Record(vec!(
                (String::from("x"), Ty::Bool),
                (String::from("y"), Ty::Int)
            ))
        ));
    }

    #[test]
    fn ty_fv() {
        use Ty::*;

        assert_eq!(
            vec!("x".to_string(), "y".to_string()),
            Tuple(vec!(Var("x".to_string()), Var("y".to_string())))
                .fv()
                .collect::<Vec<Ident>>()
        );
        assert_eq!(
            vec!("x".to_string(), "y".to_string()),
            Fun(
                Box::new(Var("x".to_string())),
                Box::new(Var("y".to_string()))
            )
            .fv()
            .collect::<Vec<Ident>>()
        );
    }

    #[test]
    fn scheme_fv() {
        use Ty::*;

        assert_eq!(
            vec!("x".to_string()),
            (
                vec!("y".to_string()),
                Tuple(vec!(Var("x".to_string()), Var("y".to_string())))
            )
                .fv()
                .collect::<Vec<Ident>>()
        );
        assert_eq!(
            vec!("y".to_string()),
            (
                vec!("x".to_string()),
                Fun(
                    Box::new(Var("x".to_string())),
                    Box::new(Var("y".to_string()))
                )
            )
                .fv()
                .collect::<Vec<Ident>>()
        );
    }

    #[test]
    fn env_fv() {
        use Ty::*;

        assert_eq!(
            vec!("x".to_string()),
            [(
                "foo".to_string(),
                (
                    vec!("y".to_string()),
                    Tuple(vec!(Var("x".to_string()), Var("y".to_string())))
                )
            )]
            .iter()
            .cloned()
            .collect::<Env>()
            .fv()
            .collect::<Vec<Ident>>()
        );
    }

    #[test]
    fn unify() {
        use Ty::*;

        assert_eq!(
            Ok(vec!()),
            super::unify(vec!((Int, Int)).into_iter()).collect()
        );

        assert_eq!(
            Ok(vec!(("a".to_string(), Int))),
            super::unify(vec!((Var("a".to_string()), Int)).into_iter()).collect()
        );

        assert_eq!(
            Ok(vec!(("a".to_string(), Int))),
            super::unify(vec!((Int, Var("a".to_string()))).into_iter()).collect()
        );

        assert_eq!(
            Ok(vec!(("b".to_string(), Bool), ("a".to_string(), Int))),
            super::unify(
                vec!((
                    Fun(Box::new(Int), Box::new(Bool)),
                    Fun(
                        Box::new(Var("a".to_string())),
                        Box::new(Var("b".to_string()))
                    )
                ))
                .into_iter()
            )
            .collect()
        );

        assert_eq!(
            Ok(vec!(("a".to_string(), Int))),
            super::unify(
                vec!((
                    Fun(Box::new(Int), Box::new(Int)),
                    Fun(
                        Box::new(Var("a".to_string())),
                        Box::new(Var("a".to_string()))
                    )
                ))
                .into_iter()
            )
            .collect()
        );

        assert_eq!(
            Ok(vec!(("a".to_string(), Fun(Box::new(Int), Box::new(Bool))))),
            super::unify(
                vec!((Fun(Box::new(Int), Box::new(Bool)), Var("a".to_string()))).into_iter()
            )
            .collect()
        );

        assert!(
            super::unify(vec!((Fun(Box::new(Int), Box::new(Int)), Int)).into_iter())
                .collect::<Result<Vec<_>, std::string::String>>()
                .is_err()
        );

        assert_eq!(
            Ok(vec!(("a".to_string(), Int))),
            super::unify(
                vec!((
                    Tuple(vec!(Int, Int)),
                    Tuple(vec!(Var("a".to_string()), Var("a".to_string())))
                ))
                .into_iter()
            )
            .collect()
        );

        assert_eq!(
            Ok(vec!(("b".to_string(), Bool), ("a".to_string(), Int))),
            super::unify(
                vec!((
                    Tuple(vec!(Int, Bool)),
                    Tuple(vec!(Var("a".to_string()), Var("b".to_string())))
                ))
                .into_iter()
            )
            .collect()
        );

        assert_eq!(
            Ok(vec!(("a".to_string(), Int))),
            super::unify(
                vec!((
                    Record(vec!(("x".to_string(), Int), ("y".to_string(), Int))),
                    Record(vec!(
                        ("x".to_string(), Var("a".to_string())),
                        ("y".to_string(), Var("a".to_string()))
                    ))
                ))
                .into_iter()
            )
            .collect()
        );

        assert_eq!(
            Ok(vec!(("b".to_string(), Bool), ("a".to_string(), Int))),
            super::unify(
                vec!((
                    Record(vec!(("x".to_string(), Int), ("y".to_string(), Bool))),
                    Record(vec!(
                        ("x".to_string(), Var("a".to_string())),
                        ("y".to_string(), Var("b".to_string()))
                    ))
                ))
                .into_iter()
            )
            .collect()
        );
    }

    #[test]
    fn infer() {
        assert_eq!(
            Ok(Ty::Fun(
                Box::new(Ty::Var("a_0".to_string())),
                Box::new(Ty::Var("a_0".to_string()))
            )),
            super::infer(
                0,
                &mut HashMap::new(),
                &mut NameSource::new(),
                &mut HashMap::new(),
                &Expr::Lambda("a".to_string(), Box::new(Expr::Ident("a".to_string())))
            )
        );

        assert_eq!(
            Ok(Ty::Int),
            super::infer(
                0,
                &mut HashMap::new(),
                &mut NameSource::new(),
                &mut HashMap::new(),
                &Expr::Let(
                    vec!(("x".to_string(), Expr::Int(42))),
                    Box::new(Expr::Ident("x".to_string()))
                )
            )
        );

        assert_eq!(
            Ok(Ty::Bool),
            super::infer(
                0,
                &mut HashMap::new(),
                &mut NameSource::new(),
                &mut HashMap::new(),
                &Expr::Let(
                    vec!(
                        ("x".to_string(), Expr::Int(42)),
                        ("y".to_string(), Expr::Bool(true))
                    ),
                    Box::new(Expr::Ident("y".to_string()))
                )
            )
        );

        // `let id = 𝜆y . y in id`
        assert_eq!(
            Ok(Ty::Fun(
                Box::new(Ty::Var("y_0_1".to_string())),
                Box::new(Ty::Var("y_0_1".to_string()))
            )),
            super::infer(
                0,
                &mut HashMap::new(),
                &mut NameSource::new(),
                &mut HashMap::new(),
                &Expr::Let(
                    vec!((
                        "id".to_string(),
                        Expr::Lambda("y".to_string(), Box::new(Expr::Ident("y".to_string())))
                    )),
                    Box::new(Expr::Ident("id".to_string()))
                )
            )
        );

        // `let apply = 𝜆f . 𝜆x . f x in let id = 𝜆y . y in apply id`
        let mut env = HashMap::new();
        let mut subs = HashMap::new();

        let res = super::infer(
            0,
            &mut subs,
            &mut NameSource::new(),
            &mut env,
            &Expr::Let(
                vec![
                    (
                        "apply".to_string(),
                        Expr::Lambda(
                            "f".to_string(),
                            Box::new(Expr::Lambda(
                                "x".to_string(),
                                Box::new(Expr::Apply(
                                    Box::new(Expr::Ident("f".to_string())),
                                    Box::new(Expr::Ident("x".to_string())),
                                )),
                            )),
                        ),
                    ),
                    (
                        "id".to_string(),
                        Expr::Lambda("y".to_string(), Box::new(Expr::Ident("y".to_string()))),
                    ),
                ],
                Box::new(Expr::Apply(
                    Box::new(Expr::Ident("apply".to_string())),
                    Box::new(Expr::Ident("id".to_string())),
                )),
            ),
        );

        match res {
            Ok(Ty::Fun(lhs, rhs)) => assert_eq!(lhs, rhs),
            e => panic!("Wrong result: {:?}", e),
        }
    }

    #[test]
    fn infer_and_print() {
        use pest::Parser;
        fn infer(input: &str) -> String {
            let e = crate::parse::parse_exprs(
                crate::parse::Parser::parse(crate::parse::Rule::expr, input)
                    .unwrap_or_else(|e| panic!("{}", e))
                    .next()
                    .unwrap()
                    .into_inner(),
            )
            .unwrap();
            let ty = super::infer(
                0,
                &mut HashMap::new(),
                &mut NameSource::new(),
                &mut HashMap::new(),
                &e,
            )
            .unwrap();

            format!("{}", ty)
        }

        assert_eq!("Int", infer("4"));

        assert_eq!(
            "(x_0_1 -> x_0_1)",
            infer("let id = lambda x -> x in id end")
        );
    }
}

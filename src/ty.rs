use crate::ast::*;
use std::iter;

/// Type substitutions
type Substitution = (Ident, Ty);

type Constraint = (Ty, Ty);

/// Type scheme
type Scheme = (Vec<Ident>, Ty);

/// Type Environment
type Env = Vec<(Ident, Scheme)>;

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
        Some((Ty::Tuple(tys1), Ty::Tuple(tys2))) => unimplemented!(),
        Some((Ty::Record(tys1), Ty::Record(tys2))) => unimplemented!(),
        Some((t1, t2)) => Box::new(iter::once(Err(format!(
            "Could not unify {} and {}",
            t1, t2
        )))),
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

trait Substitutable {
    fn apply(&self, substitution: &Substitution) -> Self;
}

impl Substitutable for Ty {
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

impl Substitutable for Constraint {
    fn apply(&self, substitution: &Substitution) -> Self {
        (self.0.apply(substitution), self.1.apply(substitution))
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
            vec!((
                "foo".to_string(),
                (
                    vec!("y".to_string()),
                    Tuple(vec!(Var("x".to_string()), Var("y".to_string())))
                )
            ))
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
            Ok(vec!(("a".to_string(), Int), ("b".to_string(), Bool))),
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
            Ok(vec!(("a".to_string(), Int), ("b".to_string(), Bool))),
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
}

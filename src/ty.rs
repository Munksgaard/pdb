use crate::ast::*;
use crate::name_source::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter;

#[cfg(test)]
mod test;

/// Type substitutions
type Substitution = (Ident, Ty);

type GlobalSub = HashMap<Ident, Ty>;

type Constraint = (Ty, Ty);

/// Type scheme
type Scheme = (Vec<Ident>, Ty);

/// Type Environment
pub type Env = HashMap<Ident, Scheme>;

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

/// `infer` is based on Algorithm J
/// Inspiration from this paper:
/// https://www.cl.cam.ac.uk/teaching/1415/L28/type-inference.pdf
pub fn infer(
    global_sub: &mut GlobalSub,
    name_src: &mut NameSource,
    env: &Env,
    expr: &Expr,
) -> Result<Ty, String> {
    match expr {
        Expr::Atom(Atom::Int(_)) => Ok(Ty::Int),
        Expr::Atom(Atom::Bool(_)) => Ok(Ty::Bool),
        Expr::Atom(Atom::String(_)) => Ok(Ty::String),
        Expr::Ident(ident) => {
            let scheme = env
                .get(ident)
                .ok_or_else(|| format!("Identifier {} not found in environment", ident))?;
            Ok(instantiate(scheme, name_src))
        }
        Expr::Let(binds, expr) => {
            let mut env = env.clone();
            for (ident, e) in binds {
                let ty = infer(global_sub, name_src, &env, e)?;

                let scheme = generalize(&env, ty);

                env.insert(ident.clone(), scheme);
            }

            infer(global_sub, name_src, &env, expr)
        }
        Expr::Lambda(ident, e) => {
            let freshvar = name_src.fresh(&ident);
            let mut env = env.clone();
            env.insert(ident.clone(), (vec![], Ty::Var(freshvar.clone())));
            let rhs = infer(global_sub, name_src, &env, e)?;

            let lhs = global_sub
                .get(&freshvar)
                .cloned()
                .unwrap_or(Ty::Var(freshvar));

            Ok(Ty::Fun(Box::new(lhs), Box::new(rhs)))
        }
        Expr::Apply(e1, e2) => {
            let t1 = infer(global_sub, name_src, env, e1)?;
            let t2 = infer(global_sub, name_src, env, e2)?;
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
        Expr::Tuple(exprs) => {
            let mut res = Vec::new();

            for expr in exprs {
                res.push(infer(global_sub, name_src, env, expr)?);
            }

            Ok(Ty::Tuple(res))
        }
        Expr::Record(recs) => {
            let mut res = Vec::new();
            for (ident, expr) in recs {
                res.push((ident.clone(), infer(global_sub, name_src, env, expr)?));
            }

            Ok(Ty::Record(res))
        }
        Expr::Atom(Atom::Unit) => Ok(Ty::Unit),
        Expr::Case(expr, matches) => {
            // Find the type of expr
            let ty = infer(global_sub, name_src, env, expr)?;

            // Bind the result to a fresh type variable
            let fresh = name_src.fresh("case");

            for (pat, e) in matches {
                // verify that pat unifies with ty
                let pat_substs = unify(unify_pat(name_src, &ty, pat).into_iter())
                    .collect::<Result<Vec<_>, String>>()?;

                let mut env = env.clone();
                for (ident, ty) in pat_substs {
                    // Insert the result both in global_sub and in the local
                    // environment? Is it necessary to add it to global_sub?
                    global_sub.insert(ident.clone(), ty.clone());

                    let scheme = generalize(&env, ty);

                    env.insert(ident.to_string(), scheme);
                }

                let result_ty = global_sub
                    .get(&fresh)
                    .cloned()
                    .unwrap_or_else(|| Ty::Var(fresh.clone()));

                // the type of e
                let e_ty = infer(global_sub, name_src, &env, e)?;

                // Verify e_ty unifies with result_ty and insert substs?
                let e_substs =
                    unify(iter::once((e_ty, result_ty))).collect::<Result<Vec<_>, String>>()?;

                // Insert substs in global_sub?
                for (ident, ty) in e_substs {
                    global_sub.insert(ident, ty);
                }
            }

            let res = global_sub.get(&fresh).cloned().unwrap_or(Ty::Var(fresh));

            Ok(res)
        }
    }
}

fn unify_pat(name_src: &mut NameSource, ty: &Ty, pat: &Pattern) -> Vec<Constraint> {
    match pat {
        Pattern::Atom(Atom::Unit) => vec![(ty.clone(), Ty::Unit)],
        Pattern::Atom(Atom::Bool(_)) => vec![(ty.clone(), Ty::Bool)],
        Pattern::Atom(Atom::Int(_)) => vec![(ty.clone(), Ty::Int)],
        Pattern::Atom(Atom::String(_)) => vec![(ty.clone(), Ty::String)],
        Pattern::Ident(ident) => vec![(ty.clone(), Ty::Var(ident.clone()))],
        Pattern::Tuple(pats) => {
            let mut constraints = Vec::new();
            let mut freshvars = Vec::new();

            for pat in pats {
                // make fresh variables and add that to the unify chain
                let fresh = name_src.fresh("case");
                let result_ty = Ty::Var(fresh);
                freshvars.push(result_ty.clone());
                constraints.append(&mut unify_pat(name_src, &result_ty, &pat));
            }
            constraints.push((ty.clone(), Ty::Tuple(freshvars)));

            constraints
        }
        Pattern::Record(recs) => {
            let mut constraints = Vec::new();
            let mut freshvars = Vec::new();

            for (ident, pat) in recs {
                // make fresh variables and add that to the unify chain
                let fresh = name_src.fresh("case");
                let result_ty = Ty::Var(fresh);
                freshvars.push((ident.clone(), result_ty.clone()));
                constraints.append(&mut unify_pat(name_src, &result_ty, &pat));
            }
            constraints.push((ty.clone(), Ty::Record(freshvars)));

            constraints
        }
        Pattern::Wildcard => vec![],
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

pub fn generalize(env: &Env, ty: Ty) -> Scheme {
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
            Ty::Defined(name, args) => Ty::Defined(
                name.clone(),
                args.iter().map(|x| x.apply(substitution)).collect(),
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
            Ty::Defined(_, args) => Box::new(args.iter().flat_map(|arg| arg.fv())),
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

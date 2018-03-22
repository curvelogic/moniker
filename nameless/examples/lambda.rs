//! An example of using the `nameless` library to implement the untyped lambda
//! calculus

#[macro_use]
extern crate nameless;

use std::rc::Rc;
use nameless::{Bound, GenId, Pattern, PatternIndex, Scope, ScopeState, Term, Var};

/// The name of a free variable
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Name {
    User(String),
    Gen(GenId),
}

impl Name {
    pub fn user<S: Into<String>>(name: S) -> Name {
        Name::User(name.into())
    }
}

impl Term for Name {
    type Free = Name;

    fn term_eq(&self, other: &Name) -> bool {
        match (self, other) {
            (&Name::User(ref lhs), &Name::User(ref rhs)) => lhs == rhs,
            (&Name::Gen(ref lhs), &Name::Gen(ref rhs)) => lhs == rhs,
            _ => false,
        }
    }

    fn close_term<P: Pattern<Free = Name>>(&mut self, _: ScopeState, _: &P) {}
    fn open_term<P: Pattern<Free = Name>>(&mut self, _: ScopeState, _: &P) {}
}

impl Pattern for Name {
    type Free = Name;

    fn pattern_eq(&self, _other: &Name) -> bool {
        true
    }

    fn freshen(&mut self) -> Vec<Name> {
        *self = match *self {
            Name::User(_) => Name::Gen(GenId::fresh()),
            Name::Gen(_) => return vec![self.clone()],
        };
        vec![self.clone()]
    }

    fn rename(&mut self, perm: &[Name]) {
        assert_eq!(perm.len(), 1); // FIXME: assert
        *self = perm[0].clone(); // FIXME: double clone
    }

    fn close_pattern<P: Pattern<Free = Name>>(&mut self, _: ScopeState, _: &P) {}

    fn open_pattern<P: Pattern<Free = Name>>(&mut self, _: ScopeState, _: &P) {}

    fn on_free(&self, state: ScopeState, name: &Name) -> Option<Bound> {
        match name == self {
            true => Some(Bound {
                scope: state.depth(),
                pattern: PatternIndex(0),
            }),
            false => None,
        }
    }

    fn on_bound(&self, state: ScopeState, name: Bound) -> Option<Self::Free> {
        match name.scope == state.depth() {
            true => {
                assert_eq!(name.pattern, PatternIndex(0));
                Some(self.clone())
            },
            false => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Env {
    Empty,
    Extend(Rc<Env>, Name, Rc<Expr>),
}

fn extend(env: Rc<Env>, name: Name, expr: Rc<Expr>) -> Rc<Env> {
    Rc::new(Env::Extend(env, name, expr))
}

fn lookup<'a>(mut env: &'a Rc<Env>, name: &Name) -> Option<&'a Rc<Expr>> {
    while let Env::Extend(ref next_env, ref curr_name, ref expr) = **env {
        if Name::term_eq(curr_name, name) {
            return Some(expr);
        } else {
            env = next_env;
        }
    }
    None
}

#[derive(Debug, Clone, Term)]
pub enum Expr {
    Var(Var<Name>),
    Lam(Scope<Name, Rc<Expr>>),
    App(Rc<Expr>, Rc<Expr>),
}

pub fn eval(env: &Rc<Env>, expr: &Rc<Expr>) -> Rc<Expr> {
    match **expr {
        Expr::Var(Var::Free(ref name)) => lookup(env, name).unwrap_or(expr).clone(),
        Expr::Var(Var::Bound(ref name, _)) => panic!("encountered a bound variable: {:?}", name),
        Expr::Lam(_) => expr.clone(),
        Expr::App(ref fun, ref arg) => match *eval(env, fun) {
            Expr::Lam(ref scope) => {
                let (name, body) = nameless::unbind(scope.clone());
                eval(&extend(env.clone(), name, eval(env, arg)), &body)
            },
            _ => expr.clone(),
        },
    }
}

#[test]
fn test_eval() {
    // expr = (\x -> x) y
    let expr = Rc::new(Expr::App(
        Rc::new(Expr::Lam(Scope::bind(
            Name::user("x"),
            Rc::new(Expr::Var(Var::Free(Name::user("x")))),
        ))),
        Rc::new(Expr::Var(Var::Free(Name::user("y")))),
    ));

    assert_term_eq!(
        eval(&Rc::new(Env::Empty), &expr),
        Rc::new(Expr::Var(Var::Free(Name::user("y")))),
    );
}

fn main() {}

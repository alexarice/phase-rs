use crate::{
    syntax::{raw::TermR, typed::TermT},
    typecheck::{Env, TypeCheckError},
};

#[derive(Clone, Debug)]
pub struct Command<S> {
    pub gates: Vec<(String, TermR<S>)>,
    pub term: TermR<S>,
}

impl<S: Clone> Command<S> {
    pub fn check(&self) -> Result<(Env, TermT), TypeCheckError<S>> {
        let mut env = Env::default();
        for (name, tm) in &self.gates {
            let t = tm.check(&env, None)?;
            env.0.insert(name.clone(), t);
        }
        let tm = self.term.check(&env, None)?;
        Ok((env, tm))
    }
}

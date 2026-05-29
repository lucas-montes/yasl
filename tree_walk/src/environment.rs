use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use super::{syntax_tree::Literal, tokens::TokenLexem};

#[derive(Default, Debug)]
pub struct InternalEnv(HashMap<TokenLexem, Literal>);

impl Deref for InternalEnv {
    type Target = HashMap<TokenLexem, Literal>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InternalEnv {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct Environment(Vec<InternalEnv>);

impl Default for Environment {
    fn default() -> Self {
        Self(vec![InternalEnv::default()])
    }
}

impl Environment {
    pub fn define(&mut self, key: TokenLexem, value: Literal) {
        if let Some(current_scope) = self.0.last_mut() {
            current_scope.insert(key.into(), value);
        }
    }

    pub fn get(&self, key: &TokenLexem) -> Option<&Literal> {
        for scope in self.0.iter().rev() {
            if let Some(value) = scope.get(key) {
                return Some(value);
            }
        }
        None
    }

    pub fn assing(&mut self, key: TokenLexem, value: Literal) -> Option<Literal> {
        for scope in self.0.iter_mut().rev() {
            if scope.contains_key(&key) {
                scope.insert(key, value.clone());
                return Some(value);
            }
        }
        None
    }

    pub fn push_scope(&mut self) {
        self.0.push(InternalEnv::default());
    }

    pub fn pop_scope(&mut self) {
        if self.0.len() > 1 {
            self.0.pop();
        }
    }
}

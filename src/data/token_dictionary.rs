use std::{collections::HashMap, ops::Index};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenDictionary {
    tokens: Vec<String>,
    token_indices: HashMap<String, u32>,
}

impl TokenDictionary {
    pub fn new(tokens: Vec<String>) -> Self {
        let token_indices = tokens
            .iter()
            .enumerate()
            .map(|(index, token)| (token.clone(), index as u32))
            .collect::<HashMap<_, _>>();
        Self {
            tokens,
            token_indices,
        }
    }

    pub fn tokens(&self) -> &[String] {
        &self.tokens
    }

    pub fn token_indices(&self) -> &HashMap<String, u32> {
        &self.token_indices
    }

    pub fn token_index(&self, token: &str) -> Option<u32> {
        self.token_indices.get(token).copied()
    }
}

impl Index<&str> for TokenDictionary {
    type Output = u32;

    fn index(&self, token: &str) -> &Self::Output {
        &self.token_indices[token]
    }
}

impl Index<u32> for TokenDictionary {
    type Output = String;

    fn index(&self, index: u32) -> &Self::Output {
        &self.tokens[index as usize]
    }
}

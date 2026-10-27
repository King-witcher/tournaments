//! Textos do organizador (nome, descrição e regras em Markdown), no original
//! e nas traduções. Os limites são os mesmos dos checks das tabelas.

use crate::shared::{
    ApiError,
    text::{clean_long_text, clean_text},
};

const NAME_MAX: usize = 80;
const DESCRIPTION_MAX: usize = 2000;
const RULES_MAX: usize = 20000;

pub fn name(raw: &str) -> Result<String, ApiError> {
    clean_text(raw, NAME_MAX, "tournament.name_invalid")
}

pub fn description(raw: &str) -> Result<String, ApiError> {
    clean_long_text(raw, DESCRIPTION_MAX, "tournament.description_invalid")
}

pub fn rules(raw: &str) -> Result<String, ApiError> {
    clean_long_text(raw, RULES_MAX, "tournament.rules_invalid")
}

/// Nome, descrição e regras já validados.
#[derive(Debug)]
pub struct Texts {
    pub name: String,
    pub description: String,
    pub rules: String,
}

impl Texts {
    pub fn clean(name_raw: &str, description_raw: &str, rules_raw: &str) -> Result<Self, ApiError> {
        Ok(Self {
            name: name(name_raw)?,
            description: description(description_raw)?,
            rules: rules(rules_raw)?,
        })
    }
}

use std::{collections::BTreeMap, io::Read};

use indexmap::IndexMap;
use serde::Deserialize;

use super::model::{Block, Id, PropertyVariant, PropertyVariants, State, States};
use crate::name::Name;

#[derive(Debug, Deserialize)]
struct SerdeId(u32);

#[derive(Debug, Deserialize)]
struct SerdeBlock {
    #[serde(default)]
    properties: BTreeMap<String, Vec<String>>,
    states: Vec<SerdeState>,
}

#[derive(Debug, Deserialize)]
struct SerdeState {
    id: SerdeId,
    #[serde(default)]
    default: bool,
    #[serde(default)]
    properties: BTreeMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("malformed JSON data")]
    Json(#[from] serde_json::Error),
}

pub fn parse_block_registry(source: &mut impl Read) -> Result<IndexMap<Name, Block>, ParseError> {
    let registry: BTreeMap<String, SerdeBlock> = serde_json::from_reader(source)?;

    fn property_variants(prop_vars: Vec<String>) -> PropertyVariants {
        enum Kind {
            Bool,
            Numeric,
            Enum,
        }

        let vars = prop_vars.iter().map(String::as_str);
        let option = None
            .or_else(|| prop_vars.is_empty().then_some(Kind::Enum))
            .or_else(|| all_bools(vars.clone()).then_some(Kind::Bool))
            .or_else(|| all_nums(vars).then_some(Kind::Numeric))
            .unwrap_or(Kind::Enum);

        match option {
            Kind::Bool => PropertyVariants::Bool,
            Kind::Numeric => {
                let vars = prop_vars.into_iter().map(|s| s.parse().unwrap()).collect();
                PropertyVariants::Numeric(vars)
            }
            Kind::Enum => {
                let vars = prop_vars.into_iter().map(Name::from_snake_case).collect();
                PropertyVariants::Enum(vars)
            }
        }
    }

    fn state(
        all_properties: &IndexMap<Name, PropertyVariants>,
        state_id: u32,
        state_props: BTreeMap<String, String>,
    ) -> (Id, State) {
        let properties = state_props
            .into_iter()
            .map(|(name_raw, variant_raw)| {
                let prop_name = Name::from_snake_case(name_raw);
                let variants = all_properties.get(&prop_name).unwrap();
                let prop_var = PropertyVariant::from(variant_raw, variants);
                (prop_name, prop_var)
            })
            .collect();

        let id = Id(state_id);
        let block_state = State { properties };
        (id, block_state)
    }

    let result = registry
        .into_iter()
        .map(|(block_name_raw, serde_block)| {
            let block_name = consume_until_colon(&block_name_raw);
            let block_name = Name::from_snake_case(block_name);

            let block_props = serde_block
                .properties
                .into_iter()
                .map(|(prop_raw, vars_raw)| {
                    let name = Name::from_snake_case(prop_raw);
                    let vars = property_variants(vars_raw);
                    (name, vars)
                })
                .collect();

            let def_state_id = serde_block
                .states
                .iter()
                .find_map(|x| x.default.then(|| Id(x.id.0)))
                .expect("default state exists");

            let block_states = serde_block
                .states
                .into_iter()
                .map(|s| state(&block_props, s.id.0, s.properties))
                .collect();

            let block = Block {
                properties: block_props,
                states: States {
                    states: block_states,
                    default: def_state_id,
                },
            };

            (block_name, block)
        })
        .collect();

    Ok(result)
}

fn consume_until_colon(input: &'_ str) -> &'_ str {
    &input[input.find(':').map_or(0, |pos| pos + 1)..]
}

fn all_nums(variants: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
    variants
        .into_iter()
        .all(|s| s.as_ref().chars().all(|c| c.is_numeric()))
}

fn all_bools<'a, I, II>(variants: II) -> bool
where
    I: AsRef<str> + ?Sized + 'a,
    II: IntoIterator<Item = &'a I>,
{
    let mut variants = variants.into_iter();
    let mut next = || variants.next().map(|x| x.as_ref());

    match (next(), next(), next()) {
        (Some("true"), Some("false"), None) => true,
        (Some("false"), Some("true"), None) => true,
        _ => false,
    }
}

use std::collections::BTreeMap;

use serde::Deserialize;
use thiserror::Error;

use super::model::{Block, BlockProperty, BlockPropertyVariant, Id, State, StateProperty};

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

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("malformed JSON data: {0}")]
    Json(#[from] serde_json::Error),
}

pub(super) fn parse_block_registry(json_data: String) -> Result<Vec<Block>, ParseError> {
    let registry: BTreeMap<String, SerdeBlock> = serde_json::from_str(&json_data)?;

    fn block_property(
        block_name_pc: String,
        prop_name_sc: String,
        prop_variants: Vec<String>,
    ) -> BlockProperty {
        let prop_name_sc = defuse_property_name(&prop_name_sc);
        let prop_name_pc = transform_to_pascal_case(&prop_name_sc);

        let all_bools = all_bools(&prop_variants);
        let all_nums = !all_bools && all_nums(&prop_variants);

        let prop_variants = prop_variants
            .iter()
            .map(|prop_variant| {
                let var_name = transform_to_pascal_case(&prop_variant);
                let var_name = defuse_variant_name(var_name);

                match (all_bools, all_nums) {
                    (true, false) => {
                        let value = prop_variant.parse().expect(&prop_variant);
                        BlockPropertyVariant::Bool(value, var_name)
                    }
                    (false, true) => {
                        let ord = prop_variant.parse().unwrap();
                        BlockPropertyVariant::Numeric(ord, var_name)
                    }
                    (_, _) => BlockPropertyVariant::Regular(var_name),
                }
            })
            .collect();

        BlockProperty {
            block_name: block_name_pc, // PascalCase
            name_pc: prop_name_pc,     // PascalCase
            name_sc: prop_name_sc,     // snake_case
            variants: prop_variants,   // snake_case
        }
    }

    fn block_state(
        block_name_pc: String,
        state_id: u32,
        state_props: BTreeMap<String, String>,
        state_def: bool,
    ) -> (Id, State) {
        let state_props = state_props
            .into_iter()
            .map(|(prop_name_sc, prop_variant)| block_state_property(prop_name_sc, prop_variant))
            .collect();

        let block_state = State {
            block_name: block_name_pc,
            properties: state_props,
            default: state_def,
        };

        (Id(state_id), block_state)
    }

    fn block_state_property(prop_name_sc: String, prop_variant_sc: String) -> StateProperty {
        let prop_name_sc = defuse_property_name(&prop_name_sc);
        let prop_name_pc = transform_to_pascal_case(&prop_name_sc);
        let prop_variant_pc = transform_to_pascal_case(&prop_variant_sc);
        let prop_variant_pc = defuse_variant_name(&prop_variant_pc);

        StateProperty {
            field: prop_name_sc,
            prop_enum: prop_name_pc,
            variant: prop_variant_pc,
        }
    }

    let result = registry
        .into_iter()
        .map(|(block_name, serde_block)| {
            let block_name_sc = consume_until_colon(&block_name).to_string();
            let block_name_pc = transform_to_pascal_case(&block_name_sc);

            let block_props = serde_block
                .properties
                .into_iter()
                .map(|(prop_name, prop_variants)| {
                    block_property(block_name_pc.clone(), prop_name, prop_variants)
                })
                .collect();

            let block_states: Vec<_> = serde_block
                .states
                .into_iter()
                .map(|state| {
                    block_state(
                        block_name_pc.clone(),
                        state.id.0,
                        state.properties,
                        state.default,
                    )
                })
                .collect();

            let def_block_state = block_states
                .iter()
                .find(|(_, v)| v.default)
                .map(|(_, v)| v.clone())
                .expect("default state exists");

            Block {
                name_sc: block_name_sc,
                name_pc: block_name_pc,
                properties: block_props,
                states: block_states,
                default_state: def_block_state,
            }
        })
        .collect();

    Ok(result)
}

fn consume_until_colon(input: &'_ str) -> &'_ str {
    &input[input.find(':').map_or(0, |pos| pos + 1)..]
}

fn transform_to_pascal_case(input: impl AsRef<str>) -> String {
    use convert_case::{Case, Casing as _};
    input.as_ref().to_case(Case::Pascal)
}

fn defuse_property_name(property_name: impl AsRef<str>) -> String {
    match property_name.as_ref() {
        "type" => "kind".to_string(),
        _ => property_name.as_ref().to_string(),
    }
}

fn defuse_variant_name(variant_name: impl AsRef<str>) -> String {
    let variant_name = variant_name.as_ref();
    let first_char = variant_name.chars().next().expect("variant name not empty");

    if !first_char.is_numeric() {
        variant_name.into()
    } else {
        if variant_name.chars().all(|c| c.is_numeric()) {
            convert_into_roman(variant_name.parse().unwrap())
        } else {
            format!("_{}", variant_name)
        }
    }
}

fn convert_into_roman(n: u32) -> String {
    use numerals::roman::Roman;
    match n {
        0 => "O".to_string(),
        _ => format!("{:X}", Roman::from(n as i16)),
    }
}

fn all_nums(variants: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
    variants
        .into_iter()
        .all(|s| s.as_ref().chars().all(|c| c.is_numeric()))
}

fn all_bools(variants: impl AsRef<[String]>) -> bool {
    const TF: [&str; 2] = ["true", "false"];
    const FT: [&str; 2] = ["false", "true"];
    variants.as_ref() == TF || variants.as_ref() == FT
}

use std::collections::BTreeMap;

use serde::Deserialize;
use thiserror::Error;

use super::model::{Block, BlockProperty, BlockState, BlockStateProperty, Id};

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
        let prop_variants = prop_variants
            .iter()
            .map(|prop_variant| transform_to_pascal_case(prop_variant))
            .collect();

        BlockProperty {
            block_name: block_name_pc, // PascalCase
            prop_name: prop_name_sc,   // snake_case
            variants: prop_variants,   // snake_case
        }
    }

    fn block_state(
        block_name_pc: String,
        state_id: u32,
        state_props: &BTreeMap<String, String>,
        state_def: bool,
    ) -> (Id, BlockState) {
        let state_props = state_props
            .into_iter()
            .map(|(prop_name, prop_variant)| block_state_property(prop_name.clone(), &prop_variant))
            .collect();

        let block_state = BlockState {
            block_name: block_name_pc,
            properties: state_props,
            default: state_def,
        };
        (Id(state_id), block_state)
    }

    fn block_state_property(prop_name_sc: String, prop_variant: &String) -> BlockStateProperty {
        let prop_variant_pc = transform_to_pascal_case(&prop_variant);
        BlockStateProperty {
            name: prop_name_sc,
            variant_name: prop_variant_pc,
        }
    }

    let result = registry
        .into_iter()
        .map(|(block_name, serde_block)| {
            let block_name = consume_until_colon(&block_name);
            let block_name_pc = transform_to_pascal_case(block_name);

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
                        &state.properties,
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
                name: block_name_pc,
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

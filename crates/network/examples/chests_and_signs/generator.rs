use rjacraft_macro::chat;
use rjacraft_protocol::{chunk, types::*};

const BLOCKSTATE_CHEST: u32 = 2955;
const BLOCKSTATE_SIGN: u32 = 4319;

pub fn generate_blocks() -> chunk::Column<16> {
    let mut result = chunk::Column::default();

    for x in 0..16 {
        for z in 0..16 {
            if x % 2 == 0 && z % 2 == 0 {
                result.blockstates[4][4][z][x] = BLOCKSTATE_CHEST;
                result.blockstates[4][5][z][x] = BLOCKSTATE_SIGN;
            }
        }
    }

    result.sky_light.world = [[[[15; 16]; 16]; 16]; 16];

    result
}

pub fn generate_sign(contents: &[ItemStackProto]) -> BlockEntity {
    let non_empty = contents
        .iter()
        .filter(|x| {
            if let ItemStackProto::None = x {
                false
            } else {
                true
            }
        })
        .count();

    BlockEntity::Sign(Nbt(block_entity::Sign {
        is_waxed: false,
        front_text: block_entity::SignText {
            has_glowing_text: false,
            color: block_entity::Dye::Black,
            messages: vec![
                JsonString(chat!("Non-empty slots:")),
                JsonString(chat!("{non_empty}")),
                JsonString(chat!()),
                JsonString(chat!()),
            ],
        },
        back_text: block_entity::SignText {
            has_glowing_text: false,
            color: block_entity::Dye::Blue,
            messages: vec![JsonString(chat!("Back text")); 4],
        },
    }))
}

pub fn generate_block_entities(
    super::Chests(chests): &super::Chests,
) -> Vec<(BlockPosColumn, BlockEntity)> {
    let mut result = Vec::new();

    for (pos, chest) in chests {
        result.push((
            BlockPosColumn {
                x: pos.x() as u8,
                y: pos.y(),
                z: pos.z() as u8,
            },
            BlockEntity::Chest(Nbt(block_entity::Chest {})),
        ));
        result.push((
            BlockPosColumn {
                x: pos.x() as u8,
                y: pos.y() + 1,
                z: pos.z() as u8,
            },
            generate_sign(&chest.items),
        ));
    }

    result
}

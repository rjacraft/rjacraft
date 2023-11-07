use std::{array, collections::HashMap};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_macro::*;
use rjacraft_network::*;
use rjacraft_protocol::{chunk, packets::s2c, types::*};
use tracing::*;

mod generator;

const CHEST_SIZE: usize = 3 * 9;

pub struct Chest {
    users: u8,
    items: [ItemStackProto; CHEST_SIZE],
}

#[derive(Resource)]
pub struct Chests(HashMap<BlockPos, Chest>);

#[derive(Component)]
pub struct ChestWindowUp(BlockPos);

#[derive(Component)]
struct WindowUp {
    sync_id: u8,
    state_id: i32,
    carried_item: ItemStackProto,
}

fn create_chests() -> HashMap<BlockPos, Chest> {
    let mut result = HashMap::new();

    for x in 0..16 {
        for z in 0..16 {
            if x % 2 == 0 && z % 2 == 0 {
                result.insert(
                    BlockPos::new().with_x(x).with_y(68).with_z(z),
                    Chest {
                        users: 0,
                        items: array::from_fn(|_| ItemStackProto::None),
                    },
                );
            }
        }
    }

    result
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .init();

    App::new()
        .insert_resource(Runtime(
            tokio::runtime::Runtime::new().expect("Failed to create a Tokio runtime"),
        ))
        .add_plugins((
            NetworkPlugin {
                addr: "0.0.0.0:25565",
                n2b_system: IntoSystem::into_system(n2b_system(UserSystems {
                    status: status_system,
                    authenticate: auth_system,
                    brand: server_brand_system,
                })),
            },
            bevy_app::ScheduleRunnerPlugin {
                run_mode: bevy_app::RunMode::Loop { wait: None },
            },
        ))
        .add_systems(
            Update,
            (
                brand_system,
                init_play_system,
                open_system,
                window_input_system,
            ),
        )
        .insert_resource(Registries(prebuilt_registries::simple()))
        .insert_resource(Tags(vec![
            s2c::TagType {
                name: id!("block"),
                tags: vec![].into(),
            },
            s2c::TagType {
                name: id!("entity_type"),
                tags: vec![].into(),
            },
            s2c::TagType {
                name: id!("fluid"),
                tags: vec![].into(),
            },
            s2c::TagType {
                name: id!("game_event"),
                tags: vec![].into(),
            },
            s2c::TagType {
                name: id!("item"),
                tags: vec![].into(),
            },
        ]))
        .insert_resource(Chests(create_chests()))
        .run();
}

fn status_system(_peer: In<Entity>) -> server_status::ServerStatus {
    server_status::ServerStatus {
        version: server_status::Version {
            name: "Snapshot whatever".into(),
            protocol: rjacraft_protocol::SUPPORTED_PROTOCOL,
        },
        players: server_status::Players {
            max: 100,
            online: 0,
            sample: vec![],
        },
        description: chat!("Example: " (b "chests and signs")),
        favicon: None,
        enforces_secure_chat: false,
        previews_chat: false,
    }
}

fn auth_system(In((_, username, uuid)): In<(Entity, String, Uuid)>) -> AuthOutcome {
    AuthOutcome::Success(username, uuid, vec![])
}

fn server_brand_system(_peer: In<Entity>) -> Option<BrandString> {
    Some("rjacraft-derivative".try_into().unwrap())
}

fn brand_system(mut events_in: EventReader<C2sPacket<packet::ClientBrand>>) {
    for C2sPacket(_, data) in events_in.into_iter() {
        info!("client brand: {}", data.brand);
    }
}

const MENU_GENERIC_9X3: u32 = 2;

fn init_play_system(chests: Res<Chests>, players: Query<(Entity, &Play), Added<Play>>) {
    for (entity, play) in players.iter() {
        let (heightmaps, palettes, light) = chunk::to_network(&generator::generate_blocks());

        play.send_packet(&s2c::PlayPacket::Login {
            entity_id: entity.index().into(),
            is_hardcore: false.into(),
            dimensions: vec![id!["overworld"]].into(),
            max_players: 20.into(),
            load_distance: 8.into(),
            simulation_distance: 8.into(),
            reduced_debug_info: false.into(),
            enable_respawn_screen: false.into(),
            dimension_type: id!("overworld"),
            dimension_name: id!("overworld"),
            hashed_seed: 0.into(),
            gamemode: s2c::GameMode::Creative,
            previous_gamemode: s2c::PreviousGameMode::None,
            is_debug: false.into(),
            is_flat: false.into(),
            died: None.into(),
            portal_cooldown: 0.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::PlayerAbilities {
            flags: s2c::PlayerAbilities::new()
                .with_flying(true)
                .with_can_fly(true),
            flying_speed: 0.05.into(),
            fov_modifier: 0.1.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::WorldRespawn {
            position: BlockPos::new().with_x(0).with_y(0).with_z(0),
            pitch: 0.0.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::PlayerTeleport {
            x: 7.5.into(),
            y: 70.0.into(),
            z: 0.0.into(),
            yaw: 0.0.into(),
            pitch: 30.0.into(),
            relative: s2c::TeleportRelative::new(),
            id: 0.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::ChunkData {
            chunk_x: 0.into(),
            chunk_z: 0.into(),
            heightmaps: Nbt(heightmaps),
            palettes,
            block_entities: generator::generate_block_entities(&chests).into(),
            light,
        })
        .unwrap();

        for (position, chest) in &chests.0 {
            play.send_packet(&s2c::PlayPacket::ChunkBlockEvent {
                position: *position,
                event: BlockEvent::ChestUsers(chest.users),
            })
            .unwrap();
        }
    }
}

fn open_system(
    players: Query<&Play>,
    mut chests: ResMut<Chests>,
    mut events: EventReader<C2sPacket<packet::Item>>,
    mut commands: Commands,
) {
    for C2sPacket(entity, data) in events.iter() {
        let play = players.get(*entity).unwrap();

        if let packet::Item::OnBlock {
            block_pos: chest_pos,
            ..
        } = data
        {
            if let Some(chest) = chests.0.get_mut(chest_pos) {
                let sync_id = 1;
                let state_id = 0;
                chest.users += 1;

                play.send_packet(&s2c::PlayPacket::ContainerOpen {
                    sync_id: VarInt(sync_id as i32),
                    kind: VarInt(MENU_GENERIC_9X3 as i32),
                    title: chat!("{chest_pos:?}").into(),
                })
                .unwrap()
                .send_packet(&s2c::PlayPacket::ContainerSlots {
                    sync_id: sync_id.into(),
                    state_id: state_id.into(),
                    slots: Vec::from(chest.items.clone()).into(),
                    carried_item: ItemStackProto::None,
                })
                .unwrap();

                for play_other in players.iter() {
                    play_other
                        .send_packet(&s2c::PlayPacket::ChunkBlockEvent {
                            position: *chest_pos,
                            event: BlockEvent::ChestUsers(chest.users),
                        })
                        .unwrap()
                        .send_packet(&s2c::PlayPacket::SoundPositioned {
                            id: s2c::SoundId::Identifier {
                                id: id!("block.chest.open"),
                                range: None,
                            },
                            category: s2c::SoundCategory::Block,
                            x: Primitive(chest_pos.x() * 8),
                            y: Primitive(chest_pos.y() as i32 * 8),
                            z: Primitive(chest_pos.z() * 8),
                            volume: 1.0.into(),
                            pitch: 1.0.into(),
                            seed: 0.into(),
                        })
                        .unwrap();
                }

                commands.entity(*entity).insert((
                    ChestWindowUp(*chest_pos),
                    WindowUp {
                        sync_id,
                        state_id,
                        carried_item: ItemStackProto::None,
                    },
                ));
            }
        }
    }
}

fn window_input_system(
    mut players_window: Query<(&Play, &ChestWindowUp, &mut WindowUp)>,
    players: Query<&Play>,
    mut chests: ResMut<Chests>,
    mut events: EventReader<C2sPacket<packet::Window>>,
    mut commands: Commands,
) {
    for C2sPacket(entity, data) in events.iter() {
        if let Ok((_, &ChestWindowUp(chest_pos), mut window)) = players_window.get_mut(*entity) {
            let chest = chests.0.get_mut(&chest_pos).unwrap();

            match &data.action {
                packet::WindowAction::Button(_) => {}
                // the rest of the properties are for checking whether this transaction is legal
                packet::WindowAction::Click {
                    new_slots,
                    carried_item,
                    ..
                } => {
                    window.state_id += 1;
                    window.carried_item = carried_item.clone();

                    for (slot, stack) in new_slots {
                        let slot = slot.0 as usize;

                        if slot < CHEST_SIZE {
                            chest.items[slot] = stack.clone();

                            for play_other in players.iter() {
                                play_other
                                    .send_packet(&s2c::PlayPacket::ChunkBlockEntity {
                                        position: BlockPos::new()
                                            .with_x(chest_pos.x())
                                            .with_y(chest_pos.y() + 1)
                                            .with_z(chest_pos.z()),
                                        block_entity: generator::generate_sign(&chest.items),
                                    })
                                    .unwrap();
                            }
                        }
                    }

                    for (play_other, &ChestWindowUp(chest_pos_other), window_other) in
                        players_window.iter()
                    {
                        if chest_pos_other == chest_pos {
                            play_other
                                .send_packet(&s2c::PlayPacket::ContainerSlots {
                                    sync_id: window_other.sync_id.into(),
                                    state_id: window_other.state_id.into(),
                                    slots: Vec::from(chest.items.clone()).into(),
                                    carried_item: window_other.carried_item.clone(),
                                })
                                .unwrap();
                        }
                    }
                }
                packet::WindowAction::Close => {
                    chest.users -= 1;
                    for play_other in players.iter() {
                        play_other
                            .send_packet(&s2c::PlayPacket::ChunkBlockEvent {
                                position: chest_pos,
                                event: BlockEvent::ChestUsers(chest.users),
                            })
                            .unwrap()
                            .send_packet(&s2c::PlayPacket::SoundPositioned {
                                id: s2c::SoundId::Identifier {
                                    id: id!("block.chest.close"),
                                    range: None,
                                },
                                category: s2c::SoundCategory::Block,
                                x: Primitive(chest_pos.x() * 8),
                                y: Primitive(chest_pos.y() as i32 * 8),
                                z: Primitive(chest_pos.z() * 8),
                                volume: 1.0.into(),
                                pitch: 1.0.into(),
                                seed: 0.into(),
                            })
                            .unwrap();
                    }

                    commands
                        .entity(*entity)
                        .remove::<ChestWindowUp>()
                        .remove::<WindowUp>();
                }
            }
        }
    }
}

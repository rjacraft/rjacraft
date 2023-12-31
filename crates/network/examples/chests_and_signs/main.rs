use std::{array, collections::HashMap};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_macro::*;
use rjacraft_network::*;
use rjacraft_protocol::{chunk, packets::s2c, types::*, ProtocolType};

mod generator;

const CHEST_SIZE: usize = 3 * 9;

pub struct Chest {
    users: u8,
    items: [Option<ItemStack<i32>>; CHEST_SIZE],
}

#[derive(Resource)]
pub struct Chests(HashMap<(i32, i16, i32), Chest>);

#[derive(Component)]
pub struct ChestWindowUp((i32, i16, i32));

#[derive(Component)]
struct WindowUp {
    sync_id: u8,
    state_id: i32,
    carried_item: Option<ItemStack<i32>>,
}

fn create_chests() -> HashMap<(i32, i16, i32), Chest> {
    let mut result = HashMap::new();

    for x in 0..16 {
        for z in 0..16 {
            if x % 2 == 0 && z % 2 == 0 {
                result.insert(
                    (x, 68, z),
                    Chest {
                        users: 0,
                        items: array::from_fn(|_| None),
                    },
                );
            }
        }
    }

    result
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .pretty()
        .init();

    App::new()
        .insert_resource(Runtime(
            tokio::runtime::Runtime::new().expect("Failed to create a Tokio runtime"),
        ))
        .add_plugins((
            NetworkPlugin {
                addr: "0.0.0.0:25565",
                n2b_system: IntoSystem::into_system(n2b_system(NetworkConfig {
                    status_system,
                    auth_system,
                    brand_system,
                })),
            },
            bevy_app::ScheduleRunnerPlugin {
                run_mode: bevy_app::RunMode::Loop { wait: None },
            },
        ))
        .add_systems(
            Update,
            (
                init_play_system,
                open_system,
                window_input_system,
                disconnect_system,
            ),
        )
        .insert_resource(Registries(prebuilt_registries::simple()))
        .insert_resource(Tags(prebuilt_registries::clean_tags()))
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
        description: text!("Example: " (b "chests and signs")),
        favicon: None,
        enforces_secure_chat: false,
        previews_chat: false,
    }
}

async fn auth_system(In(auth): In<auth::Handle>) -> auth::Result {
    Ok((
        auth.uuid,
        player_info::Profile {
            username: auth.username.clone(),
            properties: vec![].into(),
        },
    ))
}

fn brand_system(_peer: In<Entity>) -> Option<BrandString> {
    Some("rjacraft-derivative".try_into().unwrap())
}

const MENU_GENERIC_9X3: u32 = 2;

fn init_play_system(chests: Res<Chests>, players: Query<(Entity, &Play), Added<Play>>) {
    for (entity, play) in players.iter() {
        let (heightmaps, palettes, light) = chunk::to_network(&generator::generate_blocks());

        play.send(
            s2c::PlayPacket::Login {
                entity_id: Primitive(entity.index() as i32),
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
            }
            .to_encoded_expect(),
        )
        .send(
            s2c::PlayPacket::PlayerAbilities {
                flags: s2c::PlayerAbilities::new()
                    .with_flying(true)
                    .with_can_fly(true),
                flying_speed: 0.05.into(),
                fov_modifier: 0.1.into(),
            }
            .to_encoded_expect(),
        )
        .send(
            s2c::PlayPacket::WorldRespawn {
                position: BlockPos::new().with_x(0).with_y(0).with_z(0),
                pitch: 0.0.into(),
            }
            .to_encoded_expect(),
        )
        .send(
            s2c::PlayPacket::PlayerTeleport {
                x: 7.5.into(),
                y: 70.0.into(),
                z: 0.0.into(),
                yaw: 0.0.into(),
                pitch: 30.0.into(),
                relative: s2c::TeleportRelative::new(),
                id: 0.into(),
            }
            .to_encoded_expect(),
        )
        .send(
            s2c::PlayPacket::ChunkData {
                chunk_x: 0.into(),
                chunk_z: 0.into(),
                heightmaps: Nbt(heightmaps).to_encoded_expect(),
                palettes: palettes.to_encoded_expect(),
                block_entities: generator::generate_block_entities(&chests).into(),
                light: light.to_encoded_expect(),
            }
            .to_encoded_expect(),
        );

        for (&(x, y, z), chest) in &chests.0 {
            play.send(
                s2c::PlayPacket::ChunkBlockEvent {
                    position: BlockPos::new().with_x(x).with_y(y).with_z(z),
                    event: BlockEvent::ChestUsers(chest.users),
                }
                .to_encoded_expect(),
            );
        }
    }
}

fn open_system(
    players: Query<&Play>,
    mut chests: ResMut<Chests>,
    mut events: EventReader<C2sPacket<packet::Interact>>,
    mut commands: Commands,
) {
    for C2sPacket(entity, data) in events.iter() {
        let play = players.get(*entity).unwrap();

        if let packet::Interact::Block {
            block_pos: chest_pos_proto,
            ..
        } = data
        {
            let chest_pos @ (x, y, z) = (
                chest_pos_proto.x(),
                chest_pos_proto.y(),
                chest_pos_proto.z(),
            );

            if let Some(chest) = chests.0.get_mut(&(x, y, z)) {
                let sync_id = 1;
                let state_id = 0;
                chest.users += 1;

                play.send(
                    s2c::PlayPacket::ContainerOpen {
                        sync_id: VarInt(sync_id as i32),
                        kind: VarInt(MENU_GENERIC_9X3 as i32),
                        title: text!("{chest_pos:?}").into(),
                    }
                    .to_encoded_expect(),
                )
                .send(
                    s2c::PlayPacket::ContainerSlots {
                        sync_id: sync_id.into(),
                        state_id: state_id.into(),
                        slots: Vec::from(chest.items.clone())
                            .into_iter()
                            .map(|x| x.into())
                            .collect(),
                        carried_item: BoolOption(None),
                    }
                    .to_encoded_expect(),
                );

                for play_other in players.iter() {
                    play_other
                        .send(
                            s2c::PlayPacket::ChunkBlockEvent {
                                position: *chest_pos_proto,
                                event: BlockEvent::ChestUsers(chest.users),
                            }
                            .to_encoded_expect(),
                        )
                        .send(
                            s2c::PlayPacket::SoundPositioned {
                                id: s2c::SoundId::Identifier {
                                    id: id!("block.chest.open"),
                                    range: None,
                                },
                                category: s2c::SoundCategory::Block,
                                x: Primitive(x * 8),
                                y: Primitive(y as i32 * 8),
                                z: Primitive(z * 8),
                                volume: 1.0.into(),
                                pitch: 1.0.into(),
                                seed: 0.into(),
                            }
                            .to_encoded_expect(),
                        );
                }

                commands.entity(*entity).insert((
                    ChestWindowUp(chest_pos),
                    WindowUp {
                        sync_id,
                        state_id,
                        carried_item: None,
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

            match data {
                packet::Window::ContainerButton { .. } => {}
                // the rest of the properties are for checking whether this transaction is legal
                packet::Window::ContainerClick {
                    new_slots,
                    carried_item,
                    ..
                } => {
                    window.state_id += 1;
                    window.carried_item = carried_item.clone();

                    for &(slot, ref stack) in new_slots {
                        if (slot as usize) < CHEST_SIZE {
                            chest.items[slot as usize] = stack.clone();

                            for play_other in players.iter() {
                                play_other.send(
                                    s2c::PlayPacket::ChunkBlockEntity {
                                        position: BlockPos::new()
                                            .with_x(chest_pos.0)
                                            .with_y(chest_pos.1 + 1)
                                            .with_z(chest_pos.2),
                                        block_entity: generator::generate_sign(&chest.items),
                                    }
                                    .to_encoded_expect(),
                                );
                            }
                        }
                    }

                    for (play_other, &ChestWindowUp(chest_pos_other), window_other) in
                        players_window.iter()
                    {
                        if chest_pos_other == chest_pos {
                            play_other.send(
                                s2c::PlayPacket::ContainerSlots {
                                    sync_id: window_other.sync_id.into(),
                                    state_id: window_other.state_id.into(),
                                    slots: Vec::from(chest.items.clone()).into(),
                                    carried_item: window_other.carried_item.clone().into(),
                                }
                                .to_encoded_expect(),
                            );
                        }
                    }
                }
                packet::Window::ContainerClose { .. } => {
                    chest.users -= 1;
                    for play_other in players.iter() {
                        play_other
                            .send(
                                s2c::PlayPacket::ChunkBlockEvent {
                                    position: BlockPos::new()
                                        .with_x(chest_pos.0)
                                        .with_y(chest_pos.1)
                                        .with_z(chest_pos.2),
                                    event: BlockEvent::ChestUsers(chest.users),
                                }
                                .to_encoded_expect(),
                            )
                            .send(
                                s2c::PlayPacket::SoundPositioned {
                                    id: s2c::SoundId::Identifier {
                                        id: id!("block.chest.close"),
                                        range: None,
                                    },
                                    category: s2c::SoundCategory::Block,
                                    x: Primitive(chest_pos.0 * 8),
                                    y: Primitive(chest_pos.1 as i32 * 8),
                                    z: Primitive(chest_pos.2 * 8),
                                    volume: 1.0.into(),
                                    pitch: 1.0.into(),
                                    seed: 0.into(),
                                }
                                .to_encoded_expect(),
                            );
                    }

                    commands
                        .entity(*entity)
                        .remove::<ChestWindowUp>()
                        .remove::<WindowUp>();
                }
                _ => {}
            }
        }
    }
}

fn disconnect_system(
    mut chests: ResMut<Chests>,
    players: Query<&ChestWindowUp>,
    mut events: EventReader<PeerDisconnected>,
) {
    for &PeerDisconnected { peer } in events.iter() {
        if let Ok(ChestWindowUp(chest_pos)) = players.get(peer) {
            chests.0.get_mut(chest_pos).unwrap().users -= 1;
        }
    }
}

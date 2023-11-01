use std::{collections::HashMap, ops::Range};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_macro::*;
use rjacraft_network::{packet::Movement, *};
use rjacraft_protocol::{chunk, packets::s2c, types::*};
use tracing::*;

fn generate_number(sec: &mut chunk::Section<u32, 16>, n: i32, mut x: usize, y: usize, z: usize) {
    const BLOCK: u32 = 15;

    let string = n.to_string();

    for c in string.chars() {
        match c {
            '-' => {
                //
                //
                // xxx
                //
                //
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
            }
            '0' => {
                // xxx
                // x x
                // x x
                // x x
                // xxx
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 0] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            '1' => {
                //  x
                //  x
                //  x
                //  x
                //  x
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 3][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 1][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
            }
            '2' => {
                // xxx
                //   x
                // xxx
                // x
                // xxx
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
            }
            '3' => {
                // xxx
                //   x
                // xxx
                //   x
                // xxx
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            '4' => {
                // x x
                // x x
                // xxx
                //   x
                //   x
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            '5' => {
                // xxx
                // x
                // xxx
                //   x
                // xxx
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
            }
            '6' => {
                // xxx
                // x
                // xxx
                // x x
                // xxx
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 0] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            '7' => {
                // xxx
                //   x
                //   x
                //  x
                //  x
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
            }
            '8' => {
                // xxx
                // x x
                // xxx
                // x x
                // xxx
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            '9' => {
                // xxx
                // x x
                // xxx
                //   x
                // xxx
                sec[y + 4][z][x + 0] = BLOCK;
                sec[y + 4][z][x + 1] = BLOCK;
                sec[y + 4][z][x + 2] = BLOCK;
                sec[y + 3][z][x + 0] = BLOCK;
                sec[y + 3][z][x + 2] = BLOCK;
                sec[y + 2][z][x + 0] = BLOCK;
                sec[y + 2][z][x + 1] = BLOCK;
                sec[y + 2][z][x + 2] = BLOCK;
                sec[y + 1][z][x + 2] = BLOCK;
                sec[y + 0][z][x + 0] = BLOCK;
                sec[y + 0][z][x + 1] = BLOCK;
                sec[y + 0][z][x + 2] = BLOCK;
            }
            _ => panic!(),
        }

        x += 4;
    }
}

fn generate(output: &mut chunk::Column<16>, chunk_x: i32, chunk_z: i32) {
    for sx in 0..16 {
        for y in 0..256 {
            for sz in 0..16 {
                let x = 16 * chunk_x + sx as i32;
                let z = 16 * chunk_z + sz as i32;
                let s = y / 16;
                let sy = y % 16;

                if y <= 40 {
                    // stone base
                    output.blockstates[s][sy][sz][sx] = 1;
                } else if y == 41 {
                    // planks
                    // 0123456789abcdef
                    //  xx xx xx xx xx
                    if sx % 3 != 0 && sz % 3 != 0 {
                        output.blockstates[s][sy][sz][sx] = 15;
                    }
                } else if y == 42 {
                    // a stained glass pattern with a prime interval
                    output.blockstates[s][sy][sz][sx] = 5946 + i32::unsigned_abs((x + z) % 11);
                }

                output.sky_light.world[s][sy][sz][sx] =
                    if sx < 8 { 8 + sx as u8 } else { 22 - sx as u8 };
            }
        }
    }

    generate_number(&mut output.blockstates[4], chunk_x, 1, 7, 1);
    generate_number(&mut output.blockstates[4], chunk_z, 1, 1, 1);
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();

    App::new()
        .insert_resource(Runtime(
            tokio::runtime::Runtime::new().expect("Failed to create a Tokio lifetime"),
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
                movement_system,
                client_info_system,
                init_play_system,
                chunk_send_system
                    .after(movement_system)
                    .after(client_info_system)
                    .after(init_play_system),
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
        description: chat!("Example: " (b "procedural limbo")),
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

fn brand_system(mut events: EventReader<C2sPacket<packet::ClientBrand>>) {
    for C2sPacket(_, data) in events.into_iter() {
        info!("client brand: {}", data.brand);
    }
}

#[derive(Component)]
struct Position {
    x: f64,
    _y: f64,
    z: f64,
}

fn movement_system(
    mut query: Query<&mut Position>,
    mut events: EventReader<C2sPacket<packet::Movement>>,
) {
    for C2sPacket(entity, data) in events.into_iter() {
        if let &Movement::Position(x, y, z) = data {
            *query.get_mut(*entity).unwrap() = Position { x, _y: y, z };
        }
    }
}

#[derive(Component)]
struct ClientInfo(packet::ClientInfo);

fn client_info_system(
    mut commands: Commands,
    mut events: EventReader<C2sPacket<packet::ClientInfo>>,
) {
    for C2sPacket(entity, data) in events.into_iter() {
        commands.entity(*entity).insert(ClientInfo(data.clone()));
    }
}

pub const MIN_DISTANCE: u32 = 3;
pub const MAX_DISTANCE: u32 = 32;

#[derive(Component, Debug, PartialEq)]
enum ViewFilter {
    Nothing,
    Square {
        center_x: i32,
        center_z: i32,
        radius: u32,
    },
}

impl ViewFilter {
    fn contains(&self, chunk_x: i32, chunk_z: i32) -> bool {
        match self {
            ViewFilter::Nothing => false,
            &ViewFilter::Square {
                center_x,
                center_z,
                radius,
            } => {
                i32::abs_diff(chunk_x, center_x) <= radius
                    && i32::abs_diff(chunk_z, center_z) <= radius
            }
        }
    }

    fn iter(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let (range, dx, dz) = match self {
            ViewFilter::Nothing => (0..0, 0, 0),
            &ViewFilter::Square {
                center_x,
                center_z,
                radius,
            } => (
                Range {
                    start: -(radius as i32),
                    end: radius as i32 + 1,
                },
                center_x,
                center_z,
            ),
        };

        range
            .clone()
            .flat_map(move |x| range.clone().map(move |z| (x + dx, z + dz)))
    }
}

#[derive(Default)]
struct ChunkCache {
    generator_output: chunk::Column<16>,
    net_chunks: HashMap<
        (i32, i32),
        (
            net_chunk::ColumnHeightmaps,
            net_chunk::ColumnPalettes,
            net_chunk::ColumnLight,
        ),
    >,
}

impl ChunkCache {
    fn load_player(&mut self, x: i32, z: i32, play: &Play) {
        let (heightmaps, palettes, light) = self.net_chunks.entry((x, z)).or_insert_with(|| {
            self.generator_output = Default::default();
            generate(&mut self.generator_output, x, z);
            chunk::to_network(&self.generator_output)
        });

        play.send_packet(&s2c::PlayPacket::ChunkData {
            chunk_x: x.into(),
            chunk_z: z.into(),
            heightmaps: Nbt(heightmaps.clone()),
            palettes: palettes.clone(),
            block_entities: vec![].into(),
            light: light.clone(),
        })
        .unwrap();
    }
}

fn chunk_send_system(
    mut query: Query<
        (&Play, &Position, Option<&ClientInfo>, &mut ViewFilter),
        Or<(Changed<Position>, Changed<ClientInfo>)>,
    >,
    mut chunk_cache: Local<ChunkCache>,
) {
    for (play, position, client_info, mut filter_old) in query.iter_mut() {
        let center_x = f64::floor(position.x / 16.0) as i32;
        let center_z = f64::floor(position.z / 16.0) as i32;
        let filter_new = ViewFilter::Square {
            center_x,
            center_z,
            // when the client asks for n you give it n + 1
            radius: if let Some(ClientInfo(info)) = client_info {
                u32::clamp(info.view_distance as u32 + 1, MIN_DISTANCE, MAX_DISTANCE)
            } else {
                MIN_DISTANCE
            },
        };

        if filter_old.as_ref() != &filter_new {
            info!(
                "filter changed from {:?} to {filter_new:?}",
                filter_old.as_ref()
            );

            play.send_packet(&s2c::PlayPacket::ChunkCenter {
                chunk_x: center_x.into(),
                chunk_z: center_z.into(),
            })
            .unwrap();

            for (x, z) in filter_old.iter() {
                if !filter_new.contains(x, z) {
                    play.send_packet(&s2c::PlayPacket::ChunkUnload {
                        chunk_z: z.into(),
                        chunk_x: x.into(),
                    })
                    .unwrap();
                }
            }

            for (x, z) in filter_new.iter() {
                if !filter_old.contains(x, z) {
                    chunk_cache.load_player(x, z, play);
                }
            }

            *filter_old = filter_new;
        }
    }
}

fn init_play_system(players: Query<(Entity, &Play), Added<Play>>, mut commands: Commands) {
    for (entity, play) in players.iter() {
        play.send_packet(&s2c::PlayPacket::Login {
            entity_id: entity.index().into(),
            is_hardcore: false.into(),
            dimensions: vec![id!["overworld"]].into(),
            max_players: 20.into(),
            load_distance: VarInt(MAX_DISTANCE as i32),
            simulation_distance: VarInt(MAX_DISTANCE as i32),
            reduced_debug_info: false.into(),
            enable_respawn_screen: false.into(),
            dimension_type: id!("overworld"),
            dimension_name: id!("overworld"),
            hashed_seed: 0.into(),
            gamemode: s2c::GameMode::Survival,
            previous_gamemode: s2c::PreviousGameMode::None,
            is_debug: false.into(),
            is_flat: true.into(), // to make the sky not black at y = 40
            died: None.into(),
            portal_cooldown: 0.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::PlayerAbilities {
            flags: 0b00001110.into(),
            flying_speed: 0.05.into(),
            fov_modifier: 0.1.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::PlayerTeleport {
            x: 0.0.into(),
            y: 45.0.into(),
            z: 0.0.into(),
            yaw: 180.0.into(),
            pitch: 0.0.into(),
            flags: 0.into(),
            id: 0.into(),
        })
        .unwrap()
        .send_packet(&s2c::PlayPacket::WorldRespawn {
            position: Position(0, 45, 0),
            pitch: 0.0.into(),
        })
        .unwrap();

        commands.entity(entity).insert((
            Position {
                x: 0.0,
                _y: 45.0,
                z: 0.0,
            },
            ViewFilter::Nothing,
        ));
    }
}

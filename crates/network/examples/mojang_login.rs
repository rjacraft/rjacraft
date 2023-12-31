use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_macro::*;
use rjacraft_network::*;
use rjacraft_protocol::{chunk, packets::s2c, types::*, ProtocolType};
use tracing::*;

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
        .add_systems(Update, init_play_system)
        .insert_resource(Registries(prebuilt_registries::simple()))
        .insert_resource(Tags(prebuilt_registries::clean_tags()))
        .run();
}

fn status_system(In(_): In<Entity>) -> server_status::ServerStatus {
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
        description: text!("Example: " (b "Mojang login")),
        favicon: None,
        enforces_secure_chat: false,
        previews_chat: false,
    }
}

#[derive(Component)]
struct Profile {
    uuid: Uuid,
    username: rjacraft_authlib::profile::Name,
    properties: Vec<rjacraft_authlib::profile::Property>,
}

async fn auth_system(In(mut auth): In<auth::Handle>, world: WorldMutex) -> auth::Result {
    let api = rjacraft_authlib::AnonymousApi::new(rjacraft_authlib::MOJANG_PROD).unwrap();
    let private_key = rsa::RSAPrivateKey::new(&mut rand::thread_rng(), 1024).unwrap();

    let shared_secret = auth.encrypt(&private_key).await.map_err(|e| text!("{e}"))?;
    auth.compress(Some(256));

    let server_hash = rjacraft_authlib::encryption::ServerHash::new(shared_secret, &private_key);
    let session = api
        .has_joined(&auth.username, &server_hash, None)
        .await
        .map_err(|e| {
            error!("{e}");
            text!("Can't reach Mojang")
        })?
        .ok_or_else(|| text!("Mojang says you're not here"))?;

    world.lock().await.entity_mut(auth.peer).insert(Profile {
        uuid: session.id,
        username: auth.username.clone(),
        properties: session.properties.clone(),
    });

    Ok((
        session.id,
        player_info::Profile {
            username: auth.username.clone(),
            properties: session.properties.into(),
        },
    ))
}

fn brand_system(_peer: In<Entity>) -> Option<BrandString> {
    Some("rjacraft-derivative".try_into().unwrap())
}

fn init_play_system(players: Query<(&Play, &Profile), Added<Play>>) {
    for (play, profile) in players.iter() {
        let (heightmaps, palettes, light) = chunk::to_network(&chunk::Column::<16>::default());

        play.send(
            s2c::PlayPacket::Login {
                entity_id: 0.into(),
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
                gamemode: s2c::GameMode::Adventure,
                previous_gamemode: s2c::PreviousGameMode::None,
                is_debug: false.into(),
                is_flat: false.into(),
                died: None.into(),
                portal_cooldown: 0.into(),
            }
            .to_encoded_expect(),
        )
        .send(
            s2c::PlayPacket::ServerPlayerInfo(player_info::Updates {
                players: vec![profile.uuid].into(),
                profile: vec![player_info::Profile {
                    username: profile.username.clone(),
                    properties: profile.properties.clone().into(),
                }]
                .into(),
                gamemode: None,
                listed: vec![true.into()].into(),
                ping: None,
                nickname: None,
            })
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
                position: BlockPos::new().with_x(0).with_y(40).with_z(0),
                pitch: 0.0.into(),
            }
            .to_encoded_expect(),
        )
        .send(
            s2c::PlayPacket::PlayerTeleport {
                x: 0.0.into(),
                y: 40.0.into(),
                z: 0.0.into(),
                yaw: 0.0.into(),
                pitch: 0.0.into(),
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
                block_entities: vec![].into(),
                light: light.to_encoded_expect(),
            }
            .to_encoded_expect(),
        );
    }
}

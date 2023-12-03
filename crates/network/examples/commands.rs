use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use rjacraft_macro::*;
use rjacraft_network::*;
use rjacraft_protocol::{chunk, packets::s2c, types::*, ProtocolType};

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
                    compress: None,
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
            (init_play_system, hi_system, google_system, tp_system),
        )
        .insert_resource(Registries(prebuilt_registries::simple()))
        .insert_resource(Tags(prebuilt_registries::clean_tags()))
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
        description: text!("Example: " (b "commands")),
        favicon: None,
        enforces_secure_chat: false,
        previews_chat: false,
    }
}

#[derive(Component)]
struct Profile {
    username: UsernameString,
}

fn auth_system(
    In((entity, username, uuid)): In<(Entity, UsernameString, Uuid)>,
    mut commands: Commands,
) -> AuthOutcome {
    commands.entity(entity).insert(Profile {
        username: username.clone(),
    });

    AuthOutcome::Success(
        uuid,
        player_info::Profile {
            username,
            properties: vec![].into(),
        },
    )
}

fn brand_system(_peer: In<Entity>) -> Option<BrandString> {
    Some("rjacraft-derivative".try_into().unwrap())
}

fn init_play_system(players: Query<&Play, Added<Play>>) {
    for play in players.iter() {
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
        )
        .send(
            s2c::PlayPacket::ChatCommands {
                nodes: vec![
                    command::Node::Root {
                        children: vec![1, 3, 5],
                    },
                    command::Node::Literal {
                        children: vec![2],
                        executable: true,
                        redirect: None,
                        name: "hi".into(),
                    },
                    command::Node::Argument {
                        children: vec![],
                        executable: true,
                        redirect: None,
                        name: "message".into(),
                        parser: command::Parser::String(command::StringType::Greedy),
                        suggestions: None,
                    },
                    command::Node::Literal {
                        children: vec![4],
                        executable: false,
                        redirect: None,
                        name: "google".into(),
                    },
                    command::Node::Argument {
                        children: vec![],
                        executable: true,
                        redirect: None,
                        name: "query".into(),
                        parser: command::Parser::String(command::StringType::Greedy),
                        suggestions: None,
                    },
                    command::Node::Literal {
                        children: vec![6, 7],
                        executable: false,
                        redirect: None,
                        name: "tp".into(),
                    },
                    command::Node::Argument {
                        children: vec![7],
                        executable: true,
                        redirect: None,
                        name: "position".into(),
                        parser: command::Parser::Vec3,
                        suggestions: None,
                    },
                    command::Node::Argument {
                        children: vec![],
                        executable: true,
                        redirect: None,
                        name: "entity".into(),
                        parser: command::Parser::Entity(command::EntityFlags::new()),
                        suggestions: None,
                    },
                ]
                .into(),
                root: 0.into(),
            }
            .to_encoded_expect(),
        );
    }
}

fn hi_system(
    players: Query<(&Play, &Profile)>,
    mut events: EventReader<C2sPacket<packet::Command>>,
) {
    for C2sPacket(from, data) in events.iter() {
        let to = match &data.tokens[..] {
            [hi] if hi == "hi" => "everybody".into(),
            [hi, to @ ..] if hi == "hi" => to.join(" "),
            _ => continue,
        };

        let (_, profile) = players.get(*from).unwrap();

        let packet = s2c::PlayPacket::ChatUnsignedMessage {
            content: text!(("{} says hi to {to}!", profile.username)).into(),
            overlay: false.into(),
        }
        .to_encoded_expect();

        for (play, _) in players.iter() {
            play.send(packet.clone());
        }
    }
}

fn google_system(players: Query<&Play>, mut events: EventReader<C2sPacket<packet::Command>>) {
    for C2sPacket(from, data) in events.iter() {
        let query = match &data.tokens[..] {
            [google, query @ ..] if google == "google" && !query.is_empty() => query.join(" "),
            _ => continue,
        };

        let play = players.get(*from).unwrap();
        let click = text::ClickEvent::OpenUrl(
            url::Url::parse_with_params("https://google.com/search", [("q", &query)]).unwrap(),
        );
        let hover = text::HoverEvent::ShowText(Box::new(
            text!(("Search ") (b "{query}") (" in your web browser")),
        ));

        play.send(
            s2c::PlayPacket::ChatUnsignedMessage {
                content: text!(("Google: ") (u,ce[click],he[hover] "{query}")).into(),
                overlay: false.into(),
            }
            .to_encoded_expect(),
        );
    }
}

fn tp_system(players: Query<&Play>, mut events: EventReader<C2sPacket<packet::Command>>) {
    for C2sPacket(from, data) in events.iter() {
        let (x, y, z, selector) = match &data.tokens[..] {
            [tp, x, y, z] if tp == "tp" => (x, y, z, "you".into()),
            [tp, x, y, z, selector] if tp == "tp" => (x, y, z, selector.clone()),
            _ => continue,
        };

        let play = players.get(*from).unwrap();

        play.send(
            s2c::PlayPacket::ChatUnsignedMessage {
                content: text!(("Teleporting {selector} to [{x}, {y}, {z}]!")).into(),
                overlay: false.into(),
            }
            .to_encoded_expect(),
        );
    }
}

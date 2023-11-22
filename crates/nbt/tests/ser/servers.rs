use super::*;

/// Test serialization of [servers.dat](https://wiki.vg/NBT#servers.dat).
#[test]
fn test() {
    #[derive(Serialize)]
    struct Servers {
        servers: Vec<Server>,
    }
    #[derive(Serialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Server {
        #[serde(skip_serializing_if = "Option::is_none")]
        accept_textures: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<String>,
        ip: String,
        name: String,
    }

    #[rustfmt::skip]
    assert_eq!(
        to_bytes(&Servers {
            servers: vec![
                Server {
                    accept_textures: Some(true),
                    ip: "199.167.132.229:25620".to_string(),
                    name: "Dainz1 - Creative".to_string(),
                    ..Default::default()
                },
                Server {
                    // since there are no validations on the string being a valid Base64,
                    // we can use the original `...` in the end
                    icon: Some("iVBORw0KGgoAAAANUhEUgAAAEAAAABACA...".to_string()),
                    ip: "76.127.122.65:25565".to_string(),
                    name: "minstarmin4".to_string(),
                    ..Default::default()
                },
            ]
        })
        .unwrap(),
        concat_byte_vec![
            [tag::COMPOUND], U16LenStr::new(""), concat_byte_vec![
                [tag::LIST], U16LenStr::new("servers"), [tag::COMPOUND], &2i32.to_be_bytes(),
                concat_byte_vec![
                    [tag::BYTE], U16LenStr::new("acceptTextures"), [1],
                    [tag::STRING], U16LenStr::new("ip"), U16LenStr::new("199.167.132.229:25620"),
                    [tag::STRING], U16LenStr::new("name"), U16LenStr::new("Dainz1 - Creative"),
                    [tag::END],
                ],
                concat_byte_vec![
                    [tag::STRING],
                        U16LenStr::new("icon"),
                        U16LenStr::new("iVBORw0KGgoAAAANUhEUgAAAEAAAABACA..."),
                    [tag::STRING], U16LenStr::new("ip"), U16LenStr::new("76.127.122.65:25565"),
                    [tag::STRING], U16LenStr::new("name"), U16LenStr::new("minstarmin4"),
                    [tag::END],
                ],
            ],
            [tag::END],
        ],
    );
}

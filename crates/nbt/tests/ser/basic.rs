use super::*;

/// Test serialization of [test.nbt](https://wiki.vg/NBT#test.nbt).
#[test]
fn test() {
    #[derive(Serialize)]
    struct TestNbt {
        name: String,
    }

    #[rustfmt::skip]
    assert_eq!(
        to_bytes_named(
            "hello world",
            &TestNbt {
                name: "Bananrama".to_string(),
            }
        )
        .unwrap(),
        concat_byte_vec![
            [tag::COMPOUND], U16LenStr::new("hello world"),
            concat_byte_vec![
                [tag::STRING], U16LenStr::new("name"), U16LenStr::new("Bananrama"),
            ],
            [tag::END],
        ]
    );
}

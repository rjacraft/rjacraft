use std::env;

use rjacraft_authlib::profile;

#[tokio::main]
async fn main() {
    let name: profile::Name = env::args()
        .skip(1)
        .next()
        .unwrap_or("Notch".into())
        .parse()
        .expect("invalid name");
    let client = rjacraft_authlib::AnonymousApi::new(rjacraft_authlib::MOJANG_PROD).unwrap();

    if let Some(user) = client.get_user(&name).await.unwrap() {
        let response = client.get_profile(user.id).await.unwrap().unwrap();

        println!("{:#?}", response);

        if let [profile::Property::Textures { value, .. }] = &response.properties[..] {
            println!(
                "{:#?}",
                serde_json::from_slice::<profile::PropertyValue::<profile::PropertyValueTextures>>(
                    &value
                )
                .unwrap()
            );
        }
    } else {
        println!("user doesn't exist");
    }
}

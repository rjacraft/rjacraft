use std::env;

#[tokio::main]
async fn main() {
    let uuid = env::args()
        .skip(1)
        .next()
        .expect("pass a parameter")
        .parse()
        .expect("uuid is invalid");
    let client = rjacraft_authlib::AnonymousApi::new(rjacraft_authlib::MOJANG_PROD).unwrap();
    let response = client.get_profile(uuid).await;

    println!("{:#?}", response);
}

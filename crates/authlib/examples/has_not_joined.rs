use rjacraft_authlib::*;

#[tokio::main]
async fn main() {
    let username: profile::Name = "hjksdfhjk88".parse().unwrap();
    let server_hash = encryption::ServerHash([0; 20]);
    let client = AnonymousApi::new(MOJANG_PROD).unwrap();
    let response = client.has_joined(&username, &server_hash, None).await;

    println!("{:#?}", response);
}

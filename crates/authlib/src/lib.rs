use std::net;

use http::uri;
use hyper::{body::Buf, client::connect::HttpConnector};
use hyper_tls::HttpsConnector;
use serde::Deserialize;

pub mod encryption;
pub mod profile;

mod urls;

pub use urls::{BaseUrls, MOJANG_PROD};

#[derive(Debug, Clone, Deserialize)]
pub struct UserRecord {
    pub name: profile::Name,
    pub id: uuid::Uuid,
    pub legacy: Option<bool>,
    pub demo: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRecord {
    pub id: uuid::Uuid,
    pub name: profile::Name,
    pub properties: Vec<profile::Property>,
    pub profile_actions: Vec<profile::Action>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Uri(String),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("unrecognized status code {0}")]
    UnrecognizedStatus(http::StatusCode),
}

pub struct AnonymousApi {
    urls: BaseUrls<uri::Parts>,
    http: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl AnonymousApi {
    pub fn new(urls: BaseUrls<&'static str>) -> Result<Self, uri::InvalidUri> {
        Ok(Self {
            urls: urls.parse()?,
            http: hyper::Client::builder().build(HttpsConnector::new()),
        })
    }

    pub async fn get_user(&self, name: &profile::Name) -> Result<Option<UserRecord>, Error> {
        let resp = self
            .http
            .get(
                urls::Builder::new(&self.urls.accounts)
                    .path("users")
                    .path("profiles")
                    .path("minecraft")
                    .path(name)
                    .build()
                    .map_err(Error::Uri)?,
            )
            .await?;

        match resp.status() {
            http::StatusCode::OK => Ok(Some(serde_json::from_reader(
                hyper::body::aggregate(resp).await?.reader(),
            )?)),
            http::StatusCode::NO_CONTENT => Ok(None),
            http::StatusCode::NOT_FOUND => Ok(None),
            s => Err(Error::UnrecognizedStatus(s)),
        }
    }

    pub async fn get_profile(&self, uuid: uuid::Uuid) -> Result<Option<ProfileRecord>, Error> {
        let resp = self
            .http
            .get(
                urls::Builder::new(&self.urls.session)
                    .path("session")
                    .path("minecraft")
                    .path("profile")
                    .path(uuid)
                    .build()
                    .map_err(Error::Uri)?,
            )
            .await?;

        match resp.status() {
            http::StatusCode::OK => Ok(Some(serde_json::from_reader(
                hyper::body::aggregate(resp).await?.reader(),
            )?)),
            http::StatusCode::NO_CONTENT => Ok(None),
            http::StatusCode::NOT_FOUND => Ok(None),
            s => Err(Error::UnrecognizedStatus(s)),
        }
    }

    pub async fn has_joined(
        &self,
        username: &profile::Name,
        server_hash: &encryption::ServerHash,
        player_ip: Option<net::IpAddr>,
    ) -> Result<Option<ProfileRecord>, Error> {
        let resp = self
            .http
            .get(
                urls::Builder::new(&self.urls.session)
                    .path("session")
                    .path("minecraft")
                    .path("hasJoined")
                    .param("username", username)
                    .param("serverId", server_hash)
                    .param_maybe("ip", player_ip)
                    .build()
                    .map_err(Error::Uri)?,
            )
            .await?;

        match resp.status() {
            http::StatusCode::OK => Ok(Some(dbg!(serde_json::from_reader(
                hyper::body::aggregate(resp).await?.reader(),
            )?))),
            http::StatusCode::NO_CONTENT => Ok(None),
            s => Err(Error::UnrecognizedStatus(s)),
        }
    }
}

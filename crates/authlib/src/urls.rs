use std::str::FromStr;

use http::uri;

#[derive(Debug, Clone, Copy)]
pub struct BaseUrls<U> {
    pub accounts: U,
    pub session: U,
    pub services: U,
}

pub const MOJANG_PROD: BaseUrls<&'static str> = BaseUrls {
    accounts: "https://api.mojang.com",
    session: "https://sessionserver.mojang.com",
    services: "https://api.minecraftservices.com",
};

impl BaseUrls<&'static str> {
    pub fn parse(&self) -> Result<BaseUrls<uri::Parts>, uri::InvalidUri> {
        Ok(BaseUrls {
            accounts: self.accounts.parse::<uri::Uri>()?.into_parts(),
            session: self.session.parse::<uri::Uri>()?.into_parts(),
            services: self.services.parse::<uri::Uri>()?.into_parts(),
        })
    }
}

pub struct Builder {
    scheme: Option<uri::Scheme>,
    authority: Option<uri::Authority>,
    path: String,
    has_query: bool,
    query: form_urlencoded::Serializer<'static, String>,
}

impl Builder {
    pub fn new(base: &uri::Parts) -> Self {
        Builder {
            scheme: base.scheme.clone(),
            authority: base.authority.clone(),
            path: base
                .path_and_query
                .as_ref()
                .map(|x| x.path().to_string())
                .unwrap_or_default(),
            has_query: base
                .path_and_query
                .as_ref()
                .map(|x| x.query().is_some())
                .unwrap_or_default(),
            query: form_urlencoded::Serializer::new(
                base.path_and_query
                    .as_ref()
                    .and_then(|x| x.query())
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
            ),
        }
    }

    pub fn path(mut self, segment: impl ToString) -> Self {
        if !self.path.ends_with('/') {
            self.path.push('/');
        }

        self.path.push_str(&segment.to_string());

        self
    }

    pub fn param(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.has_query = true;
        self.query
            .append_pair(&name.to_string(), &value.to_string());

        self
    }

    pub fn param_maybe(mut self, name: impl ToString, value: Option<impl ToString>) -> Self {
        if let Some(value) = value {
            self.has_query = true;
            self.query
                .append_pair(&name.to_string(), &value.to_string());
        }

        self
    }

    pub fn build(mut self) -> Result<uri::Uri, String> {
        let mut pq = self.path;
        if self.has_query {
            pq.push('?');
        }
        pq.push_str(&self.query.finish());

        let mut parts = uri::Parts::default();
        parts.scheme = self.scheme;
        parts.authority = self.authority;
        parts.path_and_query = Some(uri::PathAndQuery::from_str(&pq).map_err(|e| e.to_string())?);

        uri::Uri::from_parts(parts).map_err(|e| e.to_string())
    }
}

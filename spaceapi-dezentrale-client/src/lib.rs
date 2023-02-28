#[macro_use]
extern crate rocket;

use reqwest::StatusCode;
use spaceapi_dezentrale::Status;

#[derive(Default)]
pub struct ClientBuilder<'a> {
    api_key: Option<&'a str>,
    base_url: Option<&'a str>,
}

pub const USER_AGENT: &str = concat!(std::env!("CARGO_PKG_NAME"), "/", std::env!("CARGO_PKG_VERSION"));

impl<'a> ClientBuilder<'a> {
    pub fn new() -> Self {
        ClientBuilder {
            api_key: None,
            base_url: None,
        }
    }

    pub fn build(self) -> Result<Client, String> {
        let api_key = self.api_key.ok_or("api_key must be set".to_string())?;
        let base_url = self.base_url.ok_or("base_url must be set".to_string())?;

        reqwest::ClientBuilder::new()
            .user_agent(USER_AGENT)
            .build()
            .map(|client| Client {
                api_key: api_key.to_string(),
                base_url: base_url.to_string(),
                client,
            })
            .map_err(|err| format!("Can't build client: {err:?}"))
    }

    pub fn base_url(mut self, url: &'a str) -> Self {
        self.base_url = Some(url);
        self
    }

    pub fn api_key(mut self, key: &'a str) -> Self {
        self.api_key = Some(key);
        self
    }
}

pub struct Client {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl Client {
    pub async fn open(&self) -> Result<(), String> {
        let url = format!(
            "{}{}",
            self.base_url,
            uri!(spaceapi_dezentrale_server::open_space())
        );
        let result = self
            .client
            .post(url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|err| format!("Can't open space: {err:?}"))?;
        match result.status() {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED => Err("Wrong API-Key provided, request denied".to_string()),
            other => Err(format!("Unexpected status code return: {other}")),
        }
    }

    pub async fn close(&self) -> Result<(), String> {
        let url = format!(
            "{}{}",
            self.base_url,
            uri!(spaceapi_dezentrale_server::close_space())
        );
        let result = self
            .client
            .post(url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|err| format!("Can't open space: {err:?}"))?;
        match result.status() {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED => Err("Wrong API-Key provided, request denied".to_string()),
            other => Err(format!("Unexpected status code return: {other}")),
        }
    }

    pub async fn status(&self) -> Result<Status, String> {
        let url = format!(
            "{}{}",
            self.base_url,
            uri!(spaceapi_dezentrale_server::get_status_v14())
        );
        self.client
            .get(url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|err| format!("Can't get space status: {err:?}"))?
            .json::<Status>()
            .await
            .map_err(|err| format!("Can't parse space status: {err}"))
    }

    pub async fn is_open(&self) -> Result<bool, String> {
        let url = format!(
            "{}/{}",
            self.base_url,
            uri!(spaceapi_dezentrale_server::get_status_v14())
        );
        let status = self
            .client
            .get(url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|err| format!("Can't get space status: {err:?}"))?
            .json::<Status>()
            .await
            .map_err(|err| format!("Can't parse space status: {err}"))?;

        let status = if let Some(state) = status.state {
            state.open.unwrap_or(false)
        } else {
            false
        };
        Ok(status)
    }
}

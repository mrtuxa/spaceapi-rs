#[macro_use]
extern crate rocket;

use rand::RngCore;
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest, Request},
    serde::{json::Json, Deserialize, Serialize},
    tokio::sync::RwLock,
    Build, Rocket, State,
};
use std::{
    io::Read,
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Unix time")
        .as_secs()
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ApiKey(String);

impl FromStr for ApiKey {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(ApiKey(s.to_string()))
    }
}

impl From<&str> for ApiKey {
    fn from(s: &str) -> Self {
        ApiKey(s.to_string())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match req.headers().get_one("X-API-Key") {
            Some(api_key) => Outcome::Success(ApiKey(api_key.to_string())),
            None => Outcome::Failure((Status::Unauthorized, "Api key missing")),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AdminConfig {
    #[serde(default, rename = "api_key")]
    api_key: Option<ApiKey>,
    #[serde(default, rename = "enable")]
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceConfig {
    #[serde(rename = "publish")]
    pub publish: spaceapi_dezentrale::Status,
    #[serde(default, rename = "admin")]
    pub admin: AdminConfig,
}

impl SpaceConfig {
    pub fn from_file<P>(path: P) -> Result<SpaceConfig, String>
    where
        P: AsRef<std::path::Path> + std::fmt::Display,
    {
        log::info!("Read config file `{}`", path);
        let mut file = std::fs::File::open(path).map_err(|err| format!("Can't open file: {err:?}"))?;
        let mut file_buf = vec![];
        file.read_to_end(&mut file_buf)
            .map_err(|err| format!("Can't read file: {err:?}"))?;

        let mut config: SpaceConfig =
            serde_yaml::from_slice(&file_buf).map_err(|err| format!("Can't parse space config: {err}"))?;
        config.publish.state = Some(spaceapi_dezentrale::State {
            open: Some(false),
            lastchange: Some(unix_timestamp()),
            ..spaceapi_dezentrale::State::default()
        });

        if config.admin.enabled && config.admin.api_key.is_none() {
            log::error!("API key isn't set. Generating a random one");
            let mut rng = rand::thread_rng();
            let key = format!("{:16x}{:16x}", rng.next_u64(), rng.next_u64());
            println!("Generated key is: {key}");
            config.admin.api_key = Some(ApiKey(key));
        }

        Ok(config)
    }
}

pub struct SpaceGuard(Arc<RwLock<SpaceConfig>>);

impl SpaceGuard {
    pub fn new(space: SpaceConfig) -> Self {
        SpaceGuard(Arc::new(RwLock::new(space)))
    }

    pub async fn open(&self, key: ApiKey) -> Result<(), ()> {
        let mut space = self.0.write().await;

        if &key != space.admin.api_key.as_ref().unwrap() {
            return Err(());
        }

        let state = spaceapi_dezentrale::State {
            open: Some(true),
            lastchange: Some(unix_timestamp()),
            ..spaceapi_dezentrale::State::default()
        };
        space.publish.state = Some(state);
        Ok(())
    }

    pub async fn close(&self, key: ApiKey) -> Result<(), ()> {
        let mut space = self.0.write().await;

        if &key != space.admin.api_key.as_ref().unwrap() {
            return Err(());
        }

        let state = spaceapi_dezentrale::State {
            open: Some(false),
            lastchange: Some(unix_timestamp()),
            ..spaceapi_dezentrale::State::default()
        };
        space.publish.state = Some(state);
        Ok(())
    }

    pub async fn spaceapi_v14(&self) -> spaceapi_dezentrale::Status {
        let space = self.0.read().await;
        let mut status = space.publish.clone();
        status.api_compatibility = Some(vec![spaceapi_dezentrale::ApiVersion::V14]);
        status
    }
}

#[post("/admin/publish/space-open")]
pub async fn open_space(api_key: ApiKey, space: &State<SpaceGuard>) -> rocket::http::Status {
    match space.open(api_key).await {
        Ok(_) => rocket::http::Status::Ok,
        Err(_) => rocket::http::Status::Unauthorized,
    }
}

#[post("/admin/publish/space-close")]
pub async fn close_space(api_key: ApiKey, space: &State<SpaceGuard>) -> rocket::http::Status {
    match space.close(api_key).await {
        Ok(_) => rocket::http::Status::Ok,
        Err(_) => rocket::http::Status::Unauthorized,
    }
}

#[get("/spaceapi/v14")]
pub async fn get_status_v14(space: &State<SpaceGuard>) -> Json<spaceapi_dezentrale::Status> {
    Json(space.spaceapi_v14().await)
}

pub fn serve(config: SpaceConfig) -> Rocket<Build> {
    let mut routes = routes![get_status_v14,];

    if config.admin.enabled {
        routes.extend(routes![open_space, close_space,]);
    }

    rocket::build().manage(SpaceGuard::new(config)).mount("/", routes)
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::{
        http::{Header, Status},
        local::asynchronous::Client,
        tokio,
    };

    pub(crate) fn sample_config(admin_enabled: bool) -> SpaceConfig {
        let admin = if admin_enabled {
            AdminConfig {
                api_key: Some("sesame-open".into()),
                enabled: true,
            }
        } else {
            AdminConfig {
                api_key: None,
                enabled: false,
            }
        };

        SpaceConfig {
            publish: spaceapi_dezentrale::StatusBuilder::v14("test")
                .logo("some_logo")
                .url("http://localhost")
                .contact(Default::default())
                .location(Default::default())
                .build()
                .unwrap(),
            admin,
        }
    }

    pub(crate) async fn tester(config: SpaceConfig) -> Client {
        let rocket = serve(config).ignite().await.expect("A server");
        let client = Client::tracked(rocket).await.expect("A client");
        client
    }

    #[tokio::test]
    async fn check_space_name() {
        let client = tester(sample_config(false)).await;
        let response = client.get(uri!(get_status_v14())).dispatch().await;
        assert_eq!(Status::Ok, response.status());

        let response: spaceapi_dezentrale::Status = response.into_json().await.unwrap();
        assert_eq!("test", response.space);
    }

    fn admin_routes() -> Vec<String> {
        vec![uri!(open_space()).to_string(), uri!(close_space()).to_string()]
    }

    #[tokio::test]
    async fn check_enabled_admin_api_not_authorized_without_api_key() {
        let client = tester(sample_config(true)).await;
        for route in admin_routes() {
            let response = client.post(route).dispatch().await;
            assert_eq!(Status::Unauthorized, response.status());
        }
    }

    #[tokio::test]
    async fn check_enabled_admin_api_not_authorized_with_invalid_api_key() {
        let client = tester(sample_config(true)).await;

        for route in admin_routes() {
            let response = client
                .post(route)
                .header(Header::new("X-API-KEY", "sesame"))
                .dispatch()
                .await;
            assert_eq!(Status::Unauthorized, response.status());
        }
    }

    #[tokio::test]
    async fn check_enabled_admin_api_authorized_with_valid_api_key() {
        let client = tester(sample_config(true)).await;

        for route in admin_routes() {
            let response = client
                .post(route)
                .header(Header::new("X-API-KEY", "sesame-open"))
                .dispatch()
                .await;
            assert_eq!(Status::Ok, response.status());
        }
    }

    #[tokio::test]
    async fn check_disabled_admin_api() {
        let client = tester(sample_config(false)).await;

        for route in admin_routes() {
            let response = client.post(route).dispatch().await;
            assert_eq!(Status::NotFound, response.status());
        }
    }
}

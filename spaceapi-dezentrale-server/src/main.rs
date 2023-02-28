#[macro_use]
extern crate rocket;

use spaceapi_dezentrale_server::{serve, SpaceConfig};

#[launch]
fn launch() -> _ {
    env_logger::init();
    let config_file = std::env::var("CONFIG_FILE").unwrap_or("config.yml".to_string());
    let config = SpaceConfig::from_file(config_file).expect("Invalid config");
    serve(config)
}

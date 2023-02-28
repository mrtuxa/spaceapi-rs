use clap::Command;

#[tokio::main]
async fn main() {
    let prog = Command::new(std::env!("CARGO_PKG_NAME"))
        .author("Alexander Böhm <alexander.boehm@malbolge.net>")
        .version(std::env!("CARGO_PKG_VERSION"))
        .about("A client for changing space status")
        .subcommand(Command::new("open"))
        .subcommand(Command::new("close"))
        .subcommand(Command::new("is-open"));
    let args = prog.get_matches();

    let base_url = std::env::var("SPACEAPI_URL").expect("Set SPACEAPI_URL");
    let api_key = std::env::var("API_KEY").expect("Set API_KEY");
    let client = spaceapi_dezentrale_client::ClientBuilder::new()
        .api_key(&api_key)
        .base_url(&base_url)
        .build()
        .expect("Can't build spaceapi client");

    match args.subcommand_name() {
        Some("open") => {
            client.open().await.expect("Open failed");
        }
        Some("close") => {
            client.close().await.expect("Close failed");
        }
        Some("is-open") => {
            if client.is_open().await.expect("Request failed") {
                println!("open");
            } else {
                println!("closed");
            }
        }
        Some(other) => {
            println!("Unknown command `{other}`");
        }
        None => {
            println!("Specifiy one command");
        }
    };
}

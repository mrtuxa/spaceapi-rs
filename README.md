# Rust SpaceAPI Implementation of dezentrale

This is an implementation of the [SpaceAPI](https://spaceapi.io/) v14 in Rust. It contains following parts

- `spaceapi-dezentrale`: Serialization and deserialization to/from JSON using Serde
- `spaceapi-dezentrale-client`: Client to access the server via API
- `spaceapi-dezentrale-server`: Server which provides the API 

## Build

```
cargo build --release
```

## Usage

### Server

Create a config file (see `config.sample.yml`) which describes some basic information about the space, see [below](#Configuration).

Start the server.

```
CONFIG_FILE=config.sample.yml RUST_LOG=INFO \
    spaceapi-dezentrale-server
```

#### Configuration file

The `publish` section is a representation of the [`Status` struct of the SpaceAPI](https://spaceapi.io/docs/), which will be used as a template for publishing the status.

The server doesn't use much custom logic. See [Rocket documentation](https://rocket.rs/v0.5-rc/guide/configuration/#configuration) how to change parts like ports, limits, etc.

The log level can be changed with the default mechanism of [`RUST_LOG` of `env_logger`](https://docs.rs/env_logger/0.10.0/env_logger/#enabling-logging).

### Client

Open the space

```
SPACEAPI_URL=http://localhost:8000 API_KEY=not-very-secure \
    spaceapi-dezentrale-client open
```

Close the space

```
SPACEAPI_URL=http://localhost:8000 API_KEY=not-very-secure \
    spaceapi-dezentrale-client close
```

Check if the space is open

```
SPACEAPI_URL=http://localhost:8000 API_KEY=not-very-secure \
    spaceapi-dezentrale-client is-open
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

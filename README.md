# Sōzu traffic generator

> Sends a lot of routing instructions to Sōzu, for testing purposes

[Sōzu](https://github.com/sozu-proxy/sozu) is a reverse proxy written in Rust,
this repo is used to test it.

## Status

The generator is meant for developping purposes and is NOT meant to be ever deployed.

## Configuration

We suggest you rename [`example.config.toml`](./example.config.toml) :

```
cp example.config.toml config.toml
```

Set the values. You can set these things:

- the metrics server's address
- the path to Sōzu's configuration

## Usage

Once you have installed the `sozu-traffic-generator` and followed the configuration indication,
you can start it with the following command.

```
cargo run -vvv -c config.toml
```

The number of `v` increasing logging verbosity.

## License

See the [`LICENSE`](./LICENSE) file

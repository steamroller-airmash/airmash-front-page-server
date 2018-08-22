
# Front Page Airmash Server

This server is responsible for powering all the
non-static endpoints for the airmash main server.
More specifically, it is responsible for the
`/games`, `/clienterror` and `/login` endpoints.

If you're looking for the actual game server
see [this repo](https://github.com/steamroller-airmash/airmash-server)
instead. If you're looking for the static pages
for the front page see *TBD* instead.

## Configuration
The configuration for regions and the servers within
them is specified within
[`config.json`](https://github.com/steamroller-airmash/airmash-front-page-server/blob/master/config.json)
at compile time. Within this file, servers are
within regions, with each server having a URL,
game type, and various IDs. These will be migrated
to be provided by the actual servers over time.

## Running
There are two ways to start up the server

# Using cargo
Once you have a recent rust nightly installed,
running

```sh
cargo +nighly run
```
within the root directory of the repository
will start the server on port 9000.

# Using docker
If you don't have rust installed, then running

```sh
docker compose up -d
```
within the root directory of the repository will
bring up a working server on port 8000.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

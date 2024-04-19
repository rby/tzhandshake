![rust-build](https://github.com/rby/tzhandshake/actions/workflows/rust.yml/badge.svg)

# Tezos Handshake

## Status:

Currently just a CLI to perform a handshake with a [Tezos](https://tezos.gitlab.io/index.html) ghostnet node.
I also intend to use it as a playground for learning more about Tezos, and improving my Rust and OCaml.

## Build & run

### Generate an identity file:
In order to handshake with a Tezos node, we're required to generate a minor proof of work. The proof of work when post-concatenated to the public key should hash to a value that has a number (controlled by a difficulty param defaulting to 36) of leading zero bytes.
This is not implemented in the CLI for now, so you need to use octez-node docker image or a local build of octez-node to do that.
Example with docker:
```shell
docker run \
       --rm \
       -ti \
       -v /tmp:/home/tezos/ \
       --entrypoint "/usr/local/bin/tezos-node" \
       tezos/tezos:v14.1 identity generate
# file is written in /tmp/.tezos_node/identiy.json .
```
### Build
```shell
cargo b
```

### Run
```shell
tzhandhsake --identity-path /tmp/.tezos_node/identity.json
# connecting to ghostnet.tzinit.org:9732
# received metadata: Metadata([0, 0])
# received ack: Ack(true)
# end of handshake
# ^C
```

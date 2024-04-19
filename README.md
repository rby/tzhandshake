# Tezos Handshake

## Status:

Currently just a CLI to perform a handshake with a [Tezos](https://tezos.gitlab.io/index.html) ghostnet node.
I also intend to use it as a playground for learning more about Tezos, and improving my Rust and OCaml.

## Build & run

### Generate an identity file:
Not implemented in the CLI, but you can use octez-node docker image or a local build of octez-node to do that.
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

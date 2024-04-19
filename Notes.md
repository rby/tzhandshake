# Notes

- For Tezos, the handshake is best described in this [blog post](https://medium.com/tezedge/tezos-rust-node-a-deep-dive-into-the-tezos-p2p-layer-98e3b3e3b704)
- I missed the part about the metadata, and I tried to increment the local nonce as indicated in the illustation and send/receive the Ack. This became quickly a time sink of frustration. My IP address got temporarly banned also from the Ghostnet nodes I was using due to this error.
- I switched to testing with the octez-node on the tezos docker image `tezos/tezos:v14.1` which I should have done first. This allowed to better control the debugging log in using env variables such as `TEZOS_LOG="p2p.connect* -> debug"`
- It wasn't clear in the code that the nonces are incremented *after* and not before exchanging the messages.
```Ocaml
    (* p2p_socket.ml *)

    let local_nonce = cryptobox_data.local_nonce in
    cryptobox_data.local_nonce <-
      Tezos_crypto.Crypto_box.increment_nonce local_nonce ;
    Tezos_crypto.Crypto_box.fast_box_noalloc
      cryptobox_data.channel_key
      local_nonce
      tag
      msg ;
```
- My last resort was to *compile* octez locally! On a macOS with nix-darwin. This was probably the most challenging part, understading exactly what variables affect what and how each package is an island in its own:
    - some relying on pkg-config: this requires PKG_CONFIG_PATH to be updated accordingly, but also that pkg-config is affected by any C_LIBRARY_PATH for instance and will make some packages fail.
    - dependency on MacOS frameworks (CoreFoundation, IOKit??, Security) which need to be resolved by passing `OCAMLPARAM="cclib=-F nix-path-to-framework,cclib=-f Security"`
- I was then finally able to put my own logging to make sure what nonces are used on both sides etc.
- Once I've figured out the issue of incrementing after and not before, I was able to move forward and successfully do the handshake. Then I tried it with ghostnet node multiple times without being banned.

- I had also a lot of pain points around de/serialization using serde.
    - I tried using binary-serde but it doesn't support arbitrary length strings.
    - I think I should have just worked with a simple handcrafted binary serialization to make it easier, but it was very tempting to make it work with serde, and I wanted to benefit from the experience of writing Serializer/Deserializer. The end result is not satisfying as I'm now iteration over bytes one by one that are wrapped in Option. With more time I can focus on solving this problem while allowing the possibility to just use macro `derives` to deduce the same encoding that is in OCaml code. 
- Proving that the code is effectively completing the handshake:
    - One way is to look at the network messages received through a tool like wireshark:
        - one clear message `ConnectionMessage` that contains the public_key from the node (bytes: 2-34)
        - once metadata message, that is signed by the node.
        - one ack message: the only possible way for this message to be sent by the node and to be an ack is that it first verified the metadata message we've sent, and then from the length 17 `(2bytes (header) + 16 bytes (tag) + 1 byte (Ack variant=0, empty))` we can deduce that's it's not a nack.
    - Using the `Channel` in the `main` function to write and read other legitimates protocol messages.

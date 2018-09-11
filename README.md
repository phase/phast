**Minecraft Server Implementation written in Rust, supporting Java Edition & Bedrock Edition**

The main goals of this implementation are compatibility & performance. Accepting clients using
any Java or Bedrock version should be possible.

**Currently Implemented**

* TCP & UDP Servers run on different threads.
    * There are actually 2 threads for TCP, one for accepting new connections and one for reading from 
      existing connections.
* `Connection`s are created that hold state about a connected client.
    * There is a `ConnectionManager` that is thread-safe. This is currently created in `main.rs`,
      but it will be moved into a `Server` struct once it is created.
* Packets are received and parsed given the user's current protocol.
    * Multiple protocols are supported!
    * Adding a new protocol is simple if it only changes a couple packets from an old version.
* Protocols & Packets are generated using macros.
    * Packets are disassociated from protocols so that they can be used in many different protocols.
    * Protocol definitions are generated with read & write functions that are simple to use.

**TODO**

* Nothing works nor will it work for quite a while. Please don't ask about it.
* Server structure
    * Received Packet queue
    * Game Loop
    * Blocks / Items / Tiles / Entities for specific versions
    * World & Chunk Structure
    * Packet sending queue
        * Put this on its own thread?
    * Wow there's a lot in here
* Anvil World Loader (There are lots of open source implementations already)
* Plugin system? I've got no clue. I think using WASM module could be cool, and it would allow for
any language to be used.

**License**

The source code is licensed under the Mozilla Public License Version 2.0.
You can find more information [here](https://choosealicense.com/licenses/mpl-2.0/).

# RORI v4.0.0

![](https://travis-ci.org/AmarOk1412/rori_core.svg?branch=master)

RORI is a modulable open-source chatterbot platform. The first version was written in 2011 (2.0 in September 2012). I rewrote it in Rust in 2017, and I'm currently migrating the whole communication to use [GNU Ring](https://ring.cx).

A complete RORI chain needs 4 things:

![processus](docs/img/process.png?raw=true)

1. An **entry point** is a application which get commands from an user and send it to `rori_server`. For example, a chat where the entry point reads what users says.
2. An **endpoint** is a application which performs actions requested by RORI. For example, it can execute a shell command or write something in a chat.
3. A **module** is a script activated when a condition is fulfilled and send actions for endpoints to RORI.
4. The **rori_server** which get data from entries, call modules, and send data to endpoints.

# Why RORI?

I run a lot of chatterbots on multiple services (IRC, Discord, Websites, my computer). Some bots do the exact same thing but run on a different service. The idea is to avoid to rewrite the core of each chatterbot and use the same base. Now, I just have to write an interface to communicate with this core on each services.

This is some examples of what I will do with **RORI** (as soon as the migration is finished):
+ Ask **RORI** to launch music on the *best* device (on my computer, or stream on a discord server for example).
+ Ask **RORI** to be alarmed at 7:40.
+ Ask **RORI** to send messages to a friend.
+ Ask **RORI** to shutdown a device.
+ Send a picture to **RORI** and ask to store this pict in the *best* folder.
+ Ask **RORI** to send me a notification before a rendez-vous.

## Acronym

**RORI** is for **RORI O**n **RI**ng.
Where **RORI** is for **R**eally **O**bvious **R**eally **I**ntelligent.


## How it works

Please, see [wiki](https://github.com/AmarOk1412/rori_core/wiki/)

+ [Definitions](https://github.com/AmarOk1412/rori_core/wiki/Definitions)
+ [API documentation](https://github.com/AmarOk1412/rori_core/wiki/API)
+ [Authentification system and discovery](https://github.com/AmarOk1412/rori_core/wiki/Authentification-system-and-discovery)
+ [Interaction handling](https://github.com/AmarOk1412/rori_core/wiki/Interaction-handling)
+ [Modules](https://github.com/AmarOk1412/rori_core/wiki/Modules)
+ [Emotions](https://github.com/AmarOk1412/rori_core/wiki/Emotions)

## Build instructions

`make dependencies` will install the following packages:
1. `ring-daemon`: https://ring.cx for the communication
2. `cargo`: https://crates.io/ for rust
3. `sqlite3` for the database
4. `libdbus` for the communication between the client and `ring-daemon`
5. `libncurses` for the UI
6. `openssl` to generate keys for the API
7. `python > 3.6` for modules.


## Run your instance

`./launch-rori.sh` will:

1. Generate keys for the API
2. Then run RORI to generate the config file and the database
3. For now, close and launch manually `python3 scripts/generate_modules.py` to fill the database
4. `make run`

`config.json` looks something like:
```
{
  "ring_id":"xxxxxxxxxxxxxxxxx",
  "api_listener":"0.0.0.0:1412",
  "cert_path":"keys/api.p12",
  "cert_pass":""
}
```

Another way is to use docker... this is for now, how I run it:

```bash
make docker # build the image
docker run -it --rm --privileged -e DISPLAY=$(DISPLAY) --net=host rori_core bash # not make docker-run for now
sh scripts/launch-rori.sh # will init the keys/ and config.json
python3 scripts/generate_modules.py # when this line will be removed, make docker-run will be sufficient
sh scripts/launch-rori.sh and it's done
```


## Contribute

Please, feel free to contribute to this project in submitting patches, corrections, opening issues, etc.

If you don't know what you can do, you can look the [good-first-issue](https://github.com/AmarOk1412/rori_core/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) label, or still creates modules.

For more infos and ideas read [CONTRIBUTING.md](/CONTRIBUTING.md) (this file doesn't exists for now) and [CODE_OF_CONDUCT.md](/CODE_OF_CONDUCT.md).

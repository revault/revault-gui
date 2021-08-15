# Tutorial: using Revault on testnet

This tutorial is going to outline how to deploy a **demo** of Revault.
We'll do everything on one computer, and we'll use testnet. 
Please note that this tutorial is **not** suitable for mainnet, 
as further security precautions would be needed for real-world deployment.

This tutorial has been tested on Linux, it *might* work on Unix and it won't work for sure on Windows :)

## Table of Contents

- [Prerequisites](#Prerequisites)
- [Downloading the repositories](#downloading-the-repositories)
- [Spinning up Bitcoin core](#Spinning-up-bitcoin-core)
- [Starting the coordinator](#starting-the-coordinator)
- [Getting started as a stakeholder](#getting-started-as-a-stakeholder)
- [Getting started as a manager](#getting-started-as-a-manager)
- [Playing with Revault](#playing-with-revault)

## Prerequisites

### Understanding the Revault architecture

Please refer to [practical-revault](https://github.com/revault/practical-revault/) for in-depth explanation of the architecture.

Every stakeholder will need:
- `revaultd`
- `cosignerd`
- `revault-gui`

Every manager will need:
- `revaultd`
- `revault-gui` (optional)

Also, there must be **one** `coordinatord`

### Bitcoin Core version >= 0.21.0

See https://bitcoin.org/en/download

`revaultd` **won't work** with previous versions! Check your version with `bitcoind --version`

### Rust

`revaultd`, `cosignerd` and `coordinatord` are written in Rust. Since we don't publish compiled binaries (yet!), 
you'll need to have the Rust toolchain installed to be able to compile the projects.

Please refer to https://www.rust-lang.org/tools/install for instructions on how to install Rust.

Add `export PATH="$PATH:$HOME/.cargo/bin"` to your `.bashrc` or
`.zshrc`.

Double check that your rust toolchain is **at least** 1.53.0:

```
cargo --version
```

### Some text editor

There are various keys that you'll need to keep during the whole configuration. 
Obviously a text editor is not a good choice for storing real keys, but
hey, we're in testnet. Open your favorite note-taking app.

### The Revault ceremony

The Revault setup requires to generate secrets for each participants in a safely way.
For this testnet setup, the security is not the primary goal, so the
process of generating and sharing keys is left as an exercise to the readers.

The required secrets are

**if you are a stakeholder**:

- one bip32 xpriv and its xpub.
- one noise private key and its public key
- one cosigner bitcoin private key with its public key
- one cosigner noise private key and its public key

**if you are a manager**:

- one bip32 xpriv and its xpub.
- one noise private key and its public key

**if you are a stakeholder and a manager**

- two bip32 xpriv and their xpubs (one key pair called the stakeholder
  keys and the other one the manager keys)
- one noise private key and its public key
- one cosigner bitcoin private key with its public key
- one cosigner noise private key and its public key

You can generate bip32 key pair on bip32.org
For a noise key pair and a cosigner bitcoin key pair you can do:

`python3 -c 'import os;print(os.urandom(32).hex())'`

### Docker (kinda optional)

We need docker to spin up the `coordinatord`'s Postgre database.
You can avoid using docker though, and just spin up a Postgre database yourself.

## Downloading the repositories

Let's start by creating a dedicated folder - this way if you don't like Revault it will be easier to erase everything :D

```
mkdir revault_tutorial
cd revault_tutorial
```

### If you are a stakeholder:

Download all the needed repositories: `cosignerd`, `revaultd`, `revault-gui`:

```
git clone -b 0.2 https://github.com/revault/cosignerd
git clone -b 0.2 https://github.com/revault/revaultd
git clone -b 0.2 https://github.com/revault/revault-gui
```

Then in each of the repositories:

```
cargo build
```

Binaries will be installed in `~/.cargo/bin`.

### If you are a manager:

Download all the needed repositories: `revaultd`, `revault-gui`:

```
git clone -b 0.2 https://github.com/revault/revaultd
git clone -b 0.2 https://github.com/revault/revault-gui
```

Then in each of the repositories:

```
cargo build
```

## Spinning up Bitcoin Core

Start a Bitcoin Core node using

```
bitcoind -testnet -daemon
```

## Starting the coordinator

As we said, we need just one coordinator running, no matter how many stakeholders/managers there are.
We'll properly update the coordinator configuration later. For now, let's just retrieve the coordinator's noise key.
Cd into the coordinator:

```
cd coordinatord
```

Coordinatord needs a Postgre database running, we'll spin it up using docker:
```
docker run --rm -d -p 5432:5432 --name postgres-coordinatord -e POSTGRES_PASSWORD=revault -e POSTGRES_USER=revault -e POSTGRES_DB=coordinator_db postgres:alpine

```

Now duplicate the config provided:

```
cp contrib/config.toml coordinatord_config.toml
```

Make sure that the `postgres_uri` is `postgresql://revault:revault@localhost:5432/coordinator_db`.
change the noise keys with the public noise keys of the participants.

Your config file now should look like this (with your keys in it instead of those dummy keys, obviously):
```
daemon=true
data_dir = "./revault_coordinatord"
log_level = "debug"

postgres_uri = "postgresql://revault:revault@localhost:5432/coordinator_db"

managers = [
    "d2deeb8398f47789e1f5118c42834031e3722817a432192b74c363fdb36cc634",
]

stakeholders = [
    "eecd2a93f5b09b88519f38d620aa127333d1934987d18001132f07ffa3596c65",
    "614ad96890c309b8da6915ddb9eb1135caf228120178833e70798f20e0783b16",
]

watchtowers = []
```

start the `coordinatord`:

```
~/revault_tutorial/coordinatord$
cargo run -- --config coordinatord_config.toml
```

Then please keep note of the noise key printed at startup, it will ask
by the install screen of each participants as the coordinator public
noise key.

In order to permit other user to send request to the coordinator you can
give them a [ngrok](https://ngrok.com/) endpoint `ngrok tcp 8383`.

## Getting started as a stakeholder.

### Setting up the cosigner.

We'll need one cosigner for each stakeholder.
First of all, create a directory to store all the cosignerd data:

```
mkdir cosigner 
```

You can find an [example
config](https://github.com/revault/cosignerd/tree/master/contrib/config.toml)
to begin with. Copy it to `./cosigner/config.toml`

```
cp cosignerd/contrib/config.toml cosigner/config.toml
```

We'll need to modify it a bit:
- Update the data dir: we'll use `./cosigner`
- Make sure the `listen` field is `127.0.0.1:20001`

Write the bitcoin key of the cosigner into a file `bitcoin_secret`
(change key in the example):

```
cd cosigner
python3 -c 'open("bitcoin_secret", "wb").write(bytes.fromhex("bb87c5d1ea53030843217d42d4dc7f7229c922864af99e7d4180b4e01bd81bbf"))'
cd ..
```

Now start the project:
```
./cosignerd/target/debug --conf cosigner/config.toml
```

In order to permit other user to send request to your cosigner you can
give them a [ngrok](https://ngrok.com/) endpoint `ngrok tcp 20001`.


### Setting up Revault.

Start the install of the gui with:

```
./revault-gui/target/debug/revault-gui --datadir .
```

And follow the instructions.

Once the installation is done, you should have `revaultd` and
`revault-gui` running, two new files: `revault_gui_testnet.toml` and
`revaultd_testnet.toml` and a directory `tesnet`.
To start again the revault setup, do:

```
./revault-gui/target/debug/revault-gui --conf revault_gui_testnet.toml
```

## Getting started as a manager.

Start the install of the gui with:

```
./revault-gui/target/debug/revault-gui --datadir .
```

And follow the instructions.

Once the installation is done, you should have `revaultd` and
`revault-gui` running, two new files: `revault_gui_testnet.toml` and
`revaultd_testnet.toml` and a directory `tesnet`.
To start again the revault setup, do:

```
./revault-gui/target/debug/revault-gui --conf revault_gui_testnet.toml
```

## Playing with Revault

We do not have a user manual yet at this stage of development. 
We tried to have a self explanatory interface for the user and we hope to
have feedback from you about it. Here is a simple todo list 
you can follow in order to understand Revault usage according to your
role.

**If you are a stakeholder:**

- Make some deposits
- Create a vault from a deposit by signing its revocation transactions.
- Delegate a vault to the managers.
- Watch funds being moved by managers and cancel some of their move.
- At the end, trigger the emergency and watch all funds going to the
  emergency address. 

**If you are a manager:**

- Wait for funds being delegated to you by stakeholders.
- Spend some funds: create spend transactions and
  Share them to other managers so they sign them.
- Pray that the stakeholders do not cancel your spend attempt.


### The dummy signer

In order to successfully sign, `revault-gui` has a little tool called
`dummysigner` that takes as argument the xpriv required by the targetted
transaction. This tool is not for real usage with real funds, it only
simulates the expected signing process of a connected hardware wallet in
communication with the `revault-gui`.

```
~/revault_tutorial/revault-gui/contrib/tools/dummysigner$
cargo run -- <xpriv>
```

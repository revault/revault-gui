# Tutorial: using Revault on testnet

This tutorial is going to outline how to deploy a **demo** of Revault.

It's going to be deployed on testnet, using various computers.
You can follow up using a single computer, but it's funnier with friends :)

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
- [Updating the configurations](#updating-the-configurations)
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
- `revault-gui`

Also, there must be **one** `coordinatord`

### Bitcoin Core version >= 0.21.0

See https://bitcoin.org/en/download

`revaultd` **won't work** with previous versions! Check your version with `bitcoind --version`

### Rust

`revaultd`, `cosignerd` and `coordinatord` are written in Rust. Since we don't publish compiled binaries (yet!), 
you'll need to have the Rust toolchain installed to be able to compile the projects.

Please refer to https://www.rust-lang.org/tools/install for instructions on how to install Rust.

Double check that your rust toolchain is **at least** 1.53.0:

```
cargo --version
```

### Some text editor

There are various keys that you'll need to keep during the whole configuration. 
Obviously a text editor is not a good choice for storing real keys, but
hey, we're in testnet. Open your favorite note-taking app.

### Define your setup

Gather all your friends (and their laptops!) - you should be at least 2 people for a Revault deployment.

Then, define who's going to be a stakeholder and who's going to be a manager -
you can also have stakeholder-managers, which act as both!

You must have at least 2 stakeholders and 1 manager - since stakeholder-managers
exist, the minimum number of people needed is 2: one stakeholder-manager
and one stakeholder.

### The Revault ceremony

The Revault setup requires to generate secrets for each participant in a safe way.
For this testnet setup the security is not the primary goal, so we'll throw away
all our best practices and use bip32.org.

The required secrets are:

**If you are a stakeholder**:

- One bip32 xpriv and its xpub.
- One noise private key
- One Bitcoin private key for the cosigner

**If you are a manager**:

- One bip32 xpriv and its xpub.
- One noise private key

**If you are a stakeholder-manager**

- Two bip32 xpriv and their xpubs (one key pair called the stakeholder
  keys and the other one the manager keys)
- One noise private key
- One Bitcoin private key for the cosigner

You can generate bip32 keypairs on bip32.org.

For a noise secret key you can do:
```
python3 -c 'import os;print(os.urandom(32).hex())'
```

Don't worry about the cosigner private key yet, we'll explain how to
generate it when needed.

Also, you need to generate an emergency address, where all your funds will be sent in
case of an emergency. Since it's just a tutorial you can put any P2WSH.

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

and build them:
```
cd cosignerd
cargo build
cd ../revaultd
cargo build
cd ../revault-gui
cargo build
cd ..
```

### If you are a manager:

Download all the needed repositories: `revaultd`, `revault-gui`:

```
git clone -b 0.2 https://github.com/revault/revaultd
git clone -b 0.2 https://github.com/revault/revault-gui
```

and build them:
```
cd revaultd
cargo build
cd ../revault-gui
cargo build
cd ..
```

## Spinning up Bitcoin Core

Every participant should start a Bitcoin Core node using

```
bitcoind -testnet -daemon
```

## Starting the coordinator

As we said, we need just one coordinator running, no matter how many stakeholders/managers there are.

Clone the `coordinatord`:
```
git clone -b 0.2 https://github.com/revault/coordinatord
```

Cd into the coordinatord, create a directory for all the data and build the project:

```
cd coordinatord
mkdir coordinatord_data
cargo build
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

Also, update the `data_dir` to `./coordinatord_data`.

Your config file now should look like this:
```
daemon=true
data_dir = "./coordinatord_data"
log_level = "debug"

postgres_uri = "postgresql://revault:revault@localhost:5432/coordinator_db"

managers = []

stakeholders = []

watchtowers = []
```
We'll update the coordinator configuration later, when we'll have the
noise public keys of the participants.

For now, let's just retrieve the coordinator's noise public key - start the `coordinatord`:

```
cargo run -- --config coordinatord_config.toml
```

Then keep note of the noise key printed at startup - the revault installer
will ask for it soon.

Make sure that users not in your network can reach the coordinator.
[ngrok](https://ngrok.com/) may be of help :)

Now kill the cosigner, we don't need it for now.

## Getting started as a stakeholder

### Setting up the cosigner

We'll need one cosigner for each stakeholder.
First of all, under the `cosigner` directory create a
directory to store all the cosignerd data, and generate the
bitcoin private key:

```
cd cosignerd
mkdir cosigner_data
cd cosigner_data
python3 -c 'import os;open("bitcoin_secret", "wb").write(bytes(1) + os.urandom(31))'
cd ..
```

You can find an [example
config](https://github.com/revault/cosignerd/tree/master/contrib/config.toml)
to begin with. Copy it to `./cosignerd/cosigner_data/config.toml`

```
cp contrib/config.toml config.toml
```

We'll need to modify it a bit:
- Update the data dir: we'll use `./cosigner_data`
- Make sure the `listen` field is `0.0.0.0:20001`

The cosigner also needs a noise key, but the daemon
will generate it once you start it.
Also, the cosigner needs the public noise keys of all the managers. We
still don't have them at the moment, so we're going to modify them later.

Now start the project:
```
./target/debug/cosignerd --conf ./config.toml
```

This will print the Bitcoin public key and the Noise public key. Put them
in your notes, you'll need them later.

Make sure that users not in your network can reach the cosigner.
[ngrok](https://ngrok.com/) may be of help :)


### Setting up Revault

Go back to the `revault_tutorial` directory.

We need to instruct the GUI where revaultd is:

```
export REVAULTD_PATH=./revaultd/target/debug/revaultd
```

Start the installation of the gui with:

```
./revault-gui/target/debug/revault-gui --datadir .
```

And follow the instructions. The installer should guide you into
successfully installing `revaultd` and `revault_gui` :)
New files will be created to store the configuration (`revault_gui_testnet.toml`
and `revaultd_testnet.toml`), and a directory to store all the data (`testnet`).

Once you start the GUI, the public noise key will be printed. Save it for later,
as we'll need to give it to the coordinator.

To start again the revault setup, do:

```
./revault-gui/target/debug/revault-gui --conf revault_gui_testnet.toml
```

## Getting started as a manager
### Setting up Revault
Go back to the `revault_tutorial` directory.

First of all, we need to instruct the GUI where revaultd is:

```
export REVAULTD_PATH = ./revaultd/target/debug/revaultd
```

Start the install of the gui with:

```
./revault-gui/target/debug/revault-gui --datadir .
```

And follow the instructions. The installer should guide you into
successfully installing `revaultd` and `revault_gui` :)
New files will be created to store the configuration (`revault_gui_testnet.toml`
and `revaultd_testnet.toml`), and a directory to store all the data (`testnet`).

Once you start the GUI, the public noise key will be printed. Save it for later,
as we'll need to give it to the coordinator and to the cosigners.

To start again the revault setup, do:

```
./revault-gui/target/debug/revault-gui --conf revault_gui_testnet.toml
```

## Updating the configurations

Now that you collected all the public noise keys, we need to fix the coordinatord
and cosignerd configurations.

### Fixing the coordinatord configuration

Go back to the coordinatord config file and update the participants
noise keys (you just obtained them when starting `revault-gui`).
Then start the coordinatord again:
```
./coordinatord/target/debug -- --config coordinatord_config.toml
```

### Fixing the cosignerd configuration
Go back to the cosignerd config file and update the managers
noise keys (you just obtained them when starting `revault-gui`).
Then start the cosignerd again:
```
./cosignerd/target/debug --conf ./cosignerd/cosigner_data/config.toml
```

## Playing with Revault

We do not have a user manual yet at this stage of development. 
We tried to have a self explanatory interface for the user and we hope to
have feedback from you about it. Here is a simple todo list 
you can follow in order to understand the Revault usage according to your
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
  share them to other managers so they sign them.
- Pray that the stakeholders do not cancel your spend attempt.


### The dummy signer

In order to successfully sign, `revault-gui` has a little tool called
`dummysigner` that takes as argument the xpriv required by the targetted
transaction. This tool is not for real usage with real funds, it only
simulates the expected signing process of a connected hardware wallet in
communication with the `revault-gui`.

```
cd ./revault-gui/contrib/tools/dummysigner
cargo run -- <xpriv>
```

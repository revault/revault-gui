# revault-gui

Revault GUI is an user graphical interface written in rust for the 
[Revault daemon](https://github.com/revault/revaultd).

## Dependencies

- [`pkg-config`](https://www.freedesktop.org/wiki/Software/pkg-config/) (On Debian/Ubuntu `apt install pkg-config`)
- [`libxkbcommon`](https://xkbcommon.org/) for the dummy signer (On Debian/Ubuntu `apt install libxkbcommon-dev`)
- [Vulkan Loader](https://github.com/KhronosGroup/Vulkan-Loader) (On Debian/Ubuntu `apt install libvulkan-dev`)
- `libudev-dev` (On Debian/Ubuntu `apt install libudev-dev`)

We are striving to remove dependencies, especially the 3D ones.

## Usage

`cargo run --release -- --conf revault_gui.toml` or
`cargo run --release -- --datadir revault`

If no argument is provided, the GUI checks for the configuration file
in the default revaultd `datadir` (`~/.revault` for linux).

If the provided `datadir` is empty or does not exist, the GUI starts with
the installer mode.

After start up, The GUI will connect to the running revaultd.
A command starting revaultd is launched if no connection is made.

## Get started

See [aquarium](https://github.com/revault/aquarium) for trying out a
Revault deployment in no time. 

See [doc/DEMO_TESTNET.md](doc/DEMO_TESTNET.md) for instructions on how
to setup Revault on testnet (more involved and likely needs more
participants).

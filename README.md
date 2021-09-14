# revault-gui

Revault GUI is an user graphical interface written in rust for the 
[Revault daemon](https://github.com/revault/revaultd).

## Usage

`revault-gui --datadir <datadir> --<network>`

The default `datadir` is the default `revaultd` `datadir` (`~/.revault`
for linux) and the default `network` is the bitcoin mainnet.

If no argument is provided, the GUI checks in the default `datadir` 
the configuration file for the bitcoin mainnet.

If the provided `datadir` is empty or does not have the configuration
file for the targeted `network`, the GUI starts with the installer mode.

Instead of using `--datadir` and `--<network>`, a direct path to
the GUI configuration file can be provided with `--conf`.

After start up, The GUI will connect to the running revaultd.
A command starting revaultd is launched if no connection is made.

## Get started

See [aquarium](https://github.com/revault/aquarium) for trying out a
Revault deployment in no time. 

See [doc/DEMO_TESTNET.md](doc/DEMO_TESTNET.md) for instructions on how
to setup Revault on testnet (more involved and likely needs more
participants).

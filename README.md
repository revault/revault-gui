# revault-gui

Revault GUI is an user graphical interface written in rust for the 
[Revault daemon](https:://github.com/revault/revaultd).

## Usage

`cargo run --release -- --conf revault_gui.toml`

If no argument is provided, the GUI checks for the configuration file
in the default revaultd datadir (`~/.revault` for linux).

## Get started

See [doc/DEMO.md](doc/DEMO.md) for instructions on how to start the GUI

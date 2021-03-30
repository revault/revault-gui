# revault-gui

Revault GUI is an user graphical interface written in rust for the 
[Revault daemon](https:://github.com/revault/revaultd).

## Get started

### Installation requirements

The GUI needs the following libraries in order to work:
- `libvulkan-dev`
- `cmake`
- `pkg-config`

### Starting revaultd

The GUI needs:
- Access to the revaultd configuration file. This file
location can be specified by the env var `REVAULTD_CONF`, if not the
default `revaultd` configuration location (`~/.revault/revault.toml`) is checked.

- A running revaultd instance. If it's not already running, the GUI will try to run the daemon
at `REVAULTD_PATH`; if `REVAULTD_PATH` is unset, it will try to run the `revaultd`
command (will fail if `revaultd` is not installed globally). For this reason, it is suggested to
either start `revaultd` before starting the GUI, or set `REVAULTD_PATH` to
`/path/to/revaultd/repository/target/debug/revaultd`. For specific instruction on how to run
revaultd, please refer to the [Revaultd repository](https://github.com/revault/revaultd).

### Starting the GUI

```
cargo run --release
```

## ENV vars:

| Var                   | Description                                                                                                                                                              |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `REVAULTD_CONF`       | Path to the [revaultd](https://github.com/revault/revaultd) configuration path                                                                                          |
| `REVAULTGUI_DEBUG`    | If `true`, the interface will use `iced` debug feature to display current layout and set log level to `debug`                                                            |
| `REVAULTGUI_LOG`      | Enable the [tracing env filter](https://docs.rs/tracing-subscriber/0.2.15/tracing_subscriber/filter/struct.EnvFilter.html) example: `revault_gui::revault::client=debug` |
| `REVAULTD_PATH`       | Path to the [revaultd](https://github.com/revault/revaultd) binary                                                                                                      |

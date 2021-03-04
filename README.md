# revault-gui

Revault GUI is an user graphical interface written in rust for the 
[Revault daemon](https:://github.com/revault/revaultd).

## Get started

The GUI need to get access to the revaultd configuration file. This file
location can be specified by the env var `REVAULTD_CONF`, if not the
default `revaultd` configuration location is checked.

## ENV vars:

| Var                   | Description                                                                                                                                                              |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `REVAULTD_CONF`       | Path to the [revaultd](https://github.com/revault/revaultd) configuration path                                                                                          |
| `REVAULTGUI_DEBUG`    | If `true`, the interface will use `iced` debug feature to display current layout and set log level to `debug`                                                            |
| `REVAULTGUI_LOG`      | Enable the [tracing env filter](https://docs.rs/tracing-subscriber/0.2.15/tracing_subscriber/filter/struct.EnvFilter.html) example: `revault_gui::revault::client=debug` |
| `REVAULTD_PATH`       | Path to the [revaultd](https://github.com/revault/revaultd) binary                                                                                                      |

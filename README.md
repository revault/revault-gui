# revault-gui

Revault GUI is a user graphical interface written in rust for the 
[Revault daemon](https:://github.com/re-vault/revaultd).

## Get started

The GUI need to get access to the revaultd configuration file. This file
location can be specified by the env var `REVAULTD_CONF`, if not the
default `revaultd` configuration location is checked.

## ENV vars:

| Var                   | Description                                                                              |
| --------------------- | ---------------------------------------------------------------------------------------- |
| `REVAULTD_CONF`       | Path to the revaultd configuration path                                                  |
| `REVAULTGUI_DEBUG`    | If `true`, the interface will used `iced` debug feature to display current layout        |
| `REVAULTGUI_LOGLEVEL` | Enable the chosen level for logging, can be `debug`, `trace`, `error`, `warning`, `info` |

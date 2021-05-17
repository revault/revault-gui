# Tutorial: using revault-gui for the first time!

## Prerequisites
### Having revaultd, cosignerd and coordinatord up and running
Check out the [revaultd tutorial](https://github.com/revault/revaultd/tree/master/doc/DEMO.md)!
### Libraries
The following libraries are needed:
- `libvulkan-dev`
- `cmake`
- `pkg-config`

The installation method varies depending on the OS.

## Let's get started!

Go back to your `revault_tutorial` directory and clone the GUI:
```
git clone -b 0.1 https://github.com/revault/revault-gui
```

You working directory should look like:
```
.
├── coordinatord
├── cosignerd
├── revaultd
├── revault-gui
```

Starting the GUI should be quite straightforward if the daemon is running correctly. Let's make sure that the daemon is not dead:
```
cd revaultd
stake_1_cli getinfo
```
If you get an error, go back to the [revaultd tutorial](https://github.com/revault/revaultd/tree/master/doc/DEMO.md). Otherwise, let's start!
```
cd ../revault-gui
```

Building the GUI...
```
cargo build --release
```

### 1. Create the configuration

```
cd ..
echo "revaultd_config_path=$(pwd)/revaultd/stake_1_config.toml" > revault-gui/stake_1_config_ui.toml
echo "revaultd_config_path=$(pwd)/revaultd/stake_2_config.toml" > revault-gui/stake_2_config_ui.toml
echo "revaultd_config_path=$(pwd)/revaultd/stake_3_config.toml" > revault-gui/stake_3_config_ui.toml
echo "revaultd_config_path=$(pwd)/revaultd/man_1_config.toml" > revault-gui/man_1_config_ui.toml
```

### 2. Providing the configuration

Let's create some aliases:
```
cd ./revault-gui
alias stake_1_gui=cargo run --release -- --conf stake_1_config_ui.toml"
alias stake_2_gui=cargo run --release -- --conf stake_2_config_ui.toml"
alias stake_3_gui=cargo run --release -- --conf stake_3_config_ui.toml"
alias man_1_gui="cargo run --release -- --conf  man_1_config_ui.toml"
```

### 3. Starting the GUI

```
stake_1_gui
stake_2_gui
man_1_gui
```

Et voilá!

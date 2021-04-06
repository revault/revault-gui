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

### 1. Providing the configuration
Let's create some aliases:
```
alias stake_1_gui="REVAULTD_CONF=\"../revaultd/stake_1_config.toml\" cargo run --release"
alias stake_2_gui="REVAULTD_CONF=\"../revaultd/stake_2_config.toml\" cargo run --release"
alias man_1_gui="REVAULTD_CONF=\"../revaultd/man_1_config.toml\" cargo run --release"
```

### 2. Let the GUI start the binary (optional)
If you don't want to start `revaultd` manually each time you run the config, you can set the `REVAULTD_PATH` environment variable.
This will start `revaultd` **only**, you'll need to start yourself `coordinatord` and `cosignerd` anyways.
Double check that the `revaultd` configuration has `daemon=true`, otherwise the gui won't be able to start.
Then:
```
export REVAULTD_PATH="../revaultd/target/debug/revaultd"
```

### 3. Starting the GUI

```
stake_1_gui
stake_2_gui
man_1_gui
```

Et voilá!

# Linux Installer

Build the self-extracting installer:

```sh
./installers/linux/build-installer.sh
```

Run the generated installer with sudo:

```sh
sudo ./target/installers/basscript-*-linux-x86_64.run
```

The installer writes the app to `/opt/basscript`, creates `/usr/local/bin/basscript` when that path is free or already a symlink, writes `/usr/local/share/applications/basscript.desktop` for launchers such as rofi `drun`, and chowns `/opt/basscript` to the sudo-invoking user. If you run it from a root shell, set the target owner explicitly:

```sh
BASSCRIPT_USER=dude sudo ./target/installers/basscript-*-linux-x86_64.run
```

On reinstall, bundled defaults in `settings/` and `scripts/` are copied only when missing so local editor state is not overwritten.

If rofi was already open or has a stale cache, restart rofi and use `rofi -show drun`. The plain `run` mode uses `$PATH` and should find `basscript` only after the sudo installer has created `/usr/local/bin/basscript`.

Desktop launches write diagnostics to:

```sh
~/.local/state/basscript/basscript.log
```

Use that log when `basscript` works in a terminal but fails from rofi or another launcher.

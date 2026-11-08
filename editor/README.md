# Plottery Editor

The editor lets you manage and run **Plottery** projects.

Check out the [releases on github](https://github.com/TobiasGrothmann/plottery/releases) for the bundled MacOS desktop app.

## Install on macOS

This downloads the newest release and puts `PlotteryEditor.app` into `/Applications`, replacing an older version. If `cargo` is installed, it also installs the [`plottery` CLI](../cli) of the same version:

```sh
curl -fsSL https://raw.githubusercontent.com/TobiasGrothmann/plottery/main/editor/scripts/install_macos.sh | bash
```

From a checkout, `./editor/scripts/install_macos.sh` does the same; pass a directory to install somewhere else. The dmg is not codesigned, so the script also removes the quarantine flag that would make macOS report the app as damaged.

<img width="100%" src="https://github.com/user-attachments/assets/ee596420-4315-4596-a015-b93ff821fefd" />
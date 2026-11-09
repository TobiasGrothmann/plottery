# Plottery Server

Part of [**Plottery**](../README.md), a generative art engine for pen-plotters.

The controller for one specific DIY pen-plotter: a Raspberry Pi driving stepper motors over GPIO. It serves an HTTP interface that the [editor](../editor) sends plots and remote commands to. It is not meant as a general plotter driver.

## Build on the Raspberry Pi

Install the dependencies:

```sh
# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# install ssl
sudo apt install libssl-dev
```

The GPIO code is behind the `raspi` feature. Without it the server builds anywhere but does not move motors:

```sh
cargo build --release -p plottery_server --features raspi
```

## Deploy

The scripts in [`scripts/raspi/`](../scripts/raspi) build the server on the Pi and run it, either directly or as the `plottery` systemd service ([`plottery.service`](plottery.service)).

The plan and measurements for reliable step timing are in [docs/step_timing.md](docs/step_timing.md).

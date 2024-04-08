# NVIDIA Fan Control - Wayland Compatible Utility

## Overview
The `nvidia-fan-control` utility is a user-friendly tool designed for controlling the fan speed on NVIDIA GPUs. It seamlessly integrates with Wayland display servers, making it an ideal choice for modern Linux environments.

### Features
- **Automatic Fan Curve**: The utility implements a pre-configured fan curve that intelligently adjusts fan speeds according to GPU temperature, ensuring optimal cooling without any user intervention.
- **Easy Execution**: Run the utility with a simple command: `sudo ./nvidia-rust`.
- **Autostart Capabilities**: Can be configured to automatically start with your Linux distribution, providing hassle-free operation from boot.

### Compatibility
- **Proprietary Driver Support**: Tailored for the NVIDIA proprietary driver.
- **Open Source Driver**: Compatibility with the open-source `nouveau` driver has not been tested but could potentially work.

### Usage Instructions
To engage the automatic fan control:

```bash
sudo ./nvidia-rust
```

## Compilation Instructions
For compiling the Rust version, execute these commands:

```Bash
git clone https://github.com/UnknownSuperficialNight/nvidia-fan-control.git
cd nvidia-fan-control
cargo build --release
sudo ./target/release/nvidia-rust
```
An optional Rust binary is available in the releases, optimized for minimal binary size.


For the Bash version, use these commands:
```Bash
git clone https://github.com/UnknownSuperficialNight/nvidia-fan-control.git
cd nvidia-fan-control
sudo ./Bash_version/nvidia-fan-ctrl.sh
```

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

# HOW TO CUSTOMIZE THE SPEEDS

The speed array in both versions is used to customize the speed. It will by default find the closest value relative to the current temperature. So, if the lowest value in the array is `59°C` and your current temp is `30°C`, then `59%` speed will be selected. However, if in the array you have the current temp of `30°C` and in the speed array you have these values `[26, 35, 59, 80]`, then in this case, it will choose `26` as it's closest to the current temp.

Thus, change the array how you see fit to make it work for you. I've set it up to work for my GPU as it can only be at 59 speed at minimum.

Also, you can change the code. Currently, I have it set so that if the temp is greater than 80°C, then it adds 20 to the speed_output variable, thus making the fan speed 100%. It also ensures the fan speed does not go over 100%.

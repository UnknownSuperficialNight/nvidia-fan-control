# NVIDIA Fan Control - Wayland Compatible Utility

## Overview
The `nvidia-fan-control` utility is a user-friendly tool designed for controlling the fan speed on NVIDIA GPUs. It seamlessly integrates with Wayland display servers, making it an ideal choice for modern Linux environments.

### Features
- **Automatic Fan Curve**: The utility implements a pre-configured fan curve that intelligently adjusts fan speeds according to GPU temperature, ensuring optimal cooling without any user intervention.
- **Easy Execution**: Run the utility with a simple command: `sudo ./Rust-gpu-fan-control`.
- **Autostart Capabilities**: Can be configured to automatically start with your Linux distribution, providing hassle-free operation from boot.

### Compatibility
- **Proprietary Driver Support**: Tailored for the NVIDIA proprietary driver.
- **Open Source Driver**: Compatibility with the open-source `nouveau` driver has not been tested but could potentially work.

### AMDGPU
- **Monitoring support for amdgpu**: Tested on (RDNA3/RDNA2)

### Usage Instructions
To start the automatic fan control:

```bash
chmod +x Rust-gpu-fan-control # used for allowing execution permissions
sudo ./Rust-gpu-fan-control
```
If you need help with flags just type `sudo ./Rust-gpu-fan-control --help`

#### Download and use bash version

For the Bash version, use these commands:
```Bash
wget 'https://github.com/UnknownSuperficialNight/nvidia-fan-control/raw/main/Bash_version/nvidia-fan-ctrl.sh'
chmod +x nvidia-fan-ctrl.sh # used for allowing execution permissions
sudo ./nvidia-fan-ctrl.sh
```

## Compilation Instructions
For compiling the Rust version, execute these commands:

```Bash
git clone https://github.com/UnknownSuperficialNight/nvidia-fan-control.git
cd nvidia-fan-control
cargo build --release
sudo ./target/release/nvidia-rust
```
Rust binary's in the releases are optimized for minimal binary size, while also being optimized for speed. <!-- (Not available currently due to issues with rendering on nightly builds) -->

# HOW TO CUSTOMIZE THE SPEEDS

Clone the repo:

```Bash
git clone https://github.com/UnknownSuperficialNight/nvidia-fan-control.git
cd nvidia-fan-control
```

Edit `./src/main.rs` inside find a variable you would like to change options below:
- [REFRESH_TIME](https://github.com/UnknownSuperficialNight/nvidia-fan-control/blob/main/src/main.rs#L31)
- [FAN_AMOUNT](https://github.com/UnknownSuperficialNight/nvidia-fan-control/blob/main/src/compile_flag_helper.rs)
- [GPU_NUMBER](https://github.com/UnknownSuperficialNight/nvidia-fan-control/blob/main/src/main.rs#L43)
- [SPEED](https://github.com/UnknownSuperficialNight/nvidia-fan-control/blob/main/src/main.rs#L47)

### REFRESH_TIME:
REFRESH_TIME is how responsive the terminal is to resizing and the speed at which it will update the tui:

### FAN_AMOUNT:
FAN_AMOUNT is the amount of fans on your gpu you wish to target

### GPU_NUMBER:
GPU_NUMBER is the target nvidia gpu if you have one gpu its typically 0

Use this command below to list gpus
```bash
lspci | grep -i vga
```
The line at the top is 0 each line down is then incremented by one so if your gpu is on line 2 then its 1 since we count from 0 up

### SPEED:
SPEED array is the most difficult to explain but here is a TLDR:

It finds the nearest number to your current gpu temp and selects the fan speed to run at that speed 

Example:

`[25 50 75 100]`: In this case if your gpu temp is `63` the closest number in the array is `50` thus the speed is `50%` until it passes over the threshold of `65` since `66` is closer to 75 than 50 it changes the gpu speed to `75%`.

# HOW IT WORKS 

The speed array in both versions is used to customize the speed. It will by default find the closest value relative to the current temperature. So, if the lowest value in the array is `59°C` and your current temp is `30°C`, then `59%` speed will be selected. However, if in the array you have the current temp of `30°C` and in the speed array you have these values `[26, 35, 59, 80]`, then in this case, it will choose `26` as it's closest to the current temp.

Thus, change the array how you see fit to make it work for you. I've set it up to work for my GPU as mine can only be at 59 speed at minimum.

Also, you can change the code. Currently, I have it set so that if the temp is greater than `80°C`, then it adds 20 to the speed_output variable, thus making the fan speed `100%`. It also ensures the fan speed does not go over 100%.

Thus, if you want to remove or change it, going to 100% once it hits 80°C, then the code is at this [Line](https://github.com/UnknownSuperficialNight/nvidia-fan-control/blob/main/src/calculations.rs#L17)

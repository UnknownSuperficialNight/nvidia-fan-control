//  ╔═══════════════════════════════════════════════════════════════════╗
//  ║   Define build flags for quick compilation of different fan_args  ║
//  ╠═══════════════════════════════════════════════════════════════════╣
#[cfg(feature = "fan_amount_2")]
pub const FAN_AMOUNT: u8 = 2;
#[cfg(feature = "fan_amount_2")]
pub const CAPITALIZED_BINARY_NAME: &str = {
    if cfg!(target_feature = "crt-static") {
        "Rust-gpu-fan-control-2-fans-static"
    } else {
        "Rust-gpu-fan-control-2-fans"
    }
};

#[cfg(feature = "fan_amount_3")]
pub const FAN_AMOUNT: u8 = 3;
#[cfg(feature = "fan_amount_3")]
pub const CAPITALIZED_BINARY_NAME: &str = {
    if cfg!(target_feature = "crt-static") {
        "Rust-gpu-fan-control-3-fans-static"
    } else {
        "Rust-gpu-fan-control-3-fans"
    }
};

#[cfg(feature = "fan_amount_4")]
pub const FAN_AMOUNT: u8 = 4;
#[cfg(feature = "fan_amount_4")]
pub const CAPITALIZED_BINARY_NAME: &str = {
    if cfg!(target_feature = "crt-static") {
        "Rust-gpu-fan-control-4-fans-static"
    } else {
        "Rust-gpu-fan-control-4-fans"
    }
};

// Input your gpu fan amount here
#[cfg(not(any(feature = "fan_amount_2", feature = "fan_amount_3", feature = "fan_amount_4")))]
pub const FAN_AMOUNT: u8 = 1; // Default value when none of the other build options are specified
#[cfg(not(any(feature = "fan_amount_2", feature = "fan_amount_3", feature = "fan_amount_4")))]
pub const CAPITALIZED_BINARY_NAME: &str = {
    if cfg!(target_feature = "crt-static") {
        "Rust-gpu-fan-control-static"
    } else {
        "Rust-gpu-fan-control"
    }
};

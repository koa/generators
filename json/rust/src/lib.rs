#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]
#![allow(unstable_name_collisions)]
#![allow(deprecated)]
#![allow(array_into_iter)]
#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]
#![doc(html_root_url = "https://docs.rs/tinkerforge/2.0.20")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/Tinkerforge/generators/master/rust/logo_small.png")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/Tinkerforge/generators/master/rust/logo_small.png")]

//! Rust API bindings for [Tinkerforge](https://www.tinkerforge.com) bricks and bricklets.
//! See also the additional documentation and examples [here](https://www.tinkerforge.com/en/doc/Software/API_Bindings_Rust.html)
pub use crate::bindings::*;

//#[allow(unused_imports)]
mod bindings;

extern crate core;

pub mod base58;
pub mod byte_converter;
pub mod converting_callback_receiver;
pub mod converting_high_level_callback_receiver;
pub mod converting_receiver;
pub mod device;
pub mod error;
pub mod ip_connection;
pub mod low_level_traits;

//mod generator;

//! ## ** THIS IS WORK IN PROGRESS **
//!
//! This crate is in development and is potentially unstable. Basic rendering works. If you find any bugs, please open an issue or
//! create a pull request.
//!
//! # Overview
//!
//! Provides a `SystemBundle` and `RenderPlugin` to integrate [Egui](https://github.com/emilk/egui) into Amethyst.
//!
//! Please refer to the [examples](https://github.com/jgraef/amethyst_egui/tree/main/examples) about how to use `amethyst_egui`.
//!
//! ![Screenshot](https://raw.githubusercontent.com/jgraef/amethyst_egui/main/examples/screenshot.png)
//!
//!

pub mod bundle;
pub mod pass;
pub mod plugin;
pub mod pod;
pub mod system;

pub use bundle::EguiBundle;
pub use system::{EguiConfig, EguiContext, EguiInputGrab};
pub use plugin::RenderEgui;
pub use egui;

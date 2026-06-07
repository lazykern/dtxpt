pub mod app;
pub mod audio;
pub mod config;
pub mod current_song;
pub mod gameplay;
pub mod overlays;
pub mod persistence;
pub mod screens;
pub mod ui;

use anyhow::Result;
use app::DtxptPlugin;
use bevy::prelude::*;

fn main() -> Result<()> {
    App::new().add_plugins(DtxptPlugin).run();
    Ok(())
}

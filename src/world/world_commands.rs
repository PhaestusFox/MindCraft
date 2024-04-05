use bevy::prelude::*;
use bevy_console::{AddConsoleCommand, ConsoleCommand};
use clap::Parser;

use crate::prelude::ChunkId;

use super::Map;

pub fn setup(app: &mut App) {
    app.add_console_command::<ChunkCommand, _>(clear_command);
}

/// Example command
#[derive(Parser, ConsoleCommand)]
#[command(name = "chunk")]
struct ChunkCommand {
    action: Actions,
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum Actions {
    Clear,
    Regen,
}

fn clear_command(mut log: ConsoleCommand<ChunkCommand>, map: Res<Map>) {
    if let Some(Ok(ChunkCommand { action, x, y, z })) = log.take() {
        let mut map = map.0.write().unwrap();
        match action {
            Actions::Clear => map.clear_chunk(ChunkId::new(x, y, z)),
            Actions::Regen => {
                map.clear_chunk(ChunkId::new(x, y, z));
                map.regen_chunk(ChunkId::new(x, y, z));
            },
        }
        
    }
}
//! Module that handles CLI to supply input files as arguments to the executable.
use bevy::prelude::{App, Entity, FileDragAndDrop};
use bevy::window::PrimaryWindow;
use std::env;
use std::path::PathBuf;

pub struct CliArgs {
    pub map_path: Option<PathBuf>,
    pub data_path: Option<PathBuf>,
}

pub fn parse_args() -> CliArgs {
    let args: Vec<String> = env::args().collect();
    // the last args take priority
    let (map_path, data_path) = args.iter().skip(1).zip(args.iter().skip(2)).fold(
        (None, None),
        |(map, data), (arg, next)| match arg.as_str() {
            "--map" => (Some(PathBuf::from(next)), data),
            "--data" => (map, Some(PathBuf::from(next))),
            _ => (map, data),
        },
    );

    CliArgs {
        map_path,
        data_path,
    }
}

/// Generate `FileDragAndDrop` such that the map and/or data
/// if supplied as CLI args are later loaded.
pub fn handle_cli_args(app: &mut App, cli_args: CliArgs) {
    let Some((win, _)) = app
        .world_mut()
        .query::<(Entity, &PrimaryWindow)>()
        .iter(app.world())
        .next()
    else {
        return;
    };
    // paths are canonicalized so that they are not interpreted
    // to be in the assets directory by bevy's `AssetLoader`.
    if let Some(map_path) = cli_args.map_path {
        app.world_mut().send_event(FileDragAndDrop::DroppedFile {
            window: win,
            path_buf: map_path.canonicalize().unwrap(),
        });
    }

    if let Some(data_path) = cli_args.data_path {
        app.world_mut().send_event(FileDragAndDrop::DroppedFile {
            window: win,
            path_buf: data_path.canonicalize().unwrap(),
        });
    }
}

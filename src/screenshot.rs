use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;

pub struct ScreenShotPlugin;

impl Plugin for ScreenShotPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScreenshotEvent>()
            .add_systems(Update, screenshot_on_spacebar);
    }
}

#[derive(Event)]
pub struct ScreenshotEvent {
    pub path: String,
}

fn screenshot_on_spacebar(
    mut save_events: EventReader<ScreenshotEvent>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut counter: Local<u32>,
) {
    for ScreenshotEvent { path } in save_events.iter() {
        let suffix = if path.ends_with(".png") { "" } else { ".png" };
        let path = format!("{path}{suffix}");
        *counter += 1;
        screenshot_manager
            .save_screenshot_to_disk(main_window.single(), path)
            .unwrap();
    }
}

//! Information to show in the UI.
use crate::funcplot::{lerp, IgnoreSave};
use bevy::color::palettes::css::DARK_GRAY;
use bevy::color::Srgba;
use std::time::Duration;

use bevy::prelude::*;

pub struct InfoPlugin;
impl Plugin for InfoPlugin {
    fn build(&self, app: &mut App) {
        let app = app
            .insert_resource(Info {
                msg: None,
                timer: Timer::new(Duration::from_secs(3), TimerMode::Once),
            })
            .add_systems(Update, (pop_infobox, display_information));

        // display the info messages in different positions for native and WASM
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            Startup,
            |asset_server: Res<AssetServer>, commands: Commands| {
                spawn_info_box(asset_server, commands, 2.0, 1.0)
            },
        );

        #[cfg(target_arch = "wasm32")]
        app.add_systems(
            Startup,
            |asset_server: Res<AssetServer>, commands: Commands| {
                spawn_info_box(asset_server, commands, 6.5, 0.5)
            },
        );
    }
}

#[derive(Resource)]
/// Information about IO.
pub struct Info {
    msg: Option<&'static str>,
    timer: Timer,
}

impl Info {
    /// Sends a message to be logged in the CLI and displayed in the GUI.
    pub fn notify(&mut self, msg: &'static str) {
        info!(msg);
        self.msg = Some(msg);
        self.timer.reset();
    }
    pub fn close(&mut self) {
        self.msg = None;
    }
    pub fn displaying(&self) -> bool {
        self.msg.is_some()
    }
}

#[derive(Component)]
pub struct InfoBox;

/// Spawn the UI components to show I/O feedback to the user.
/// The top argument is the top of the screen in percent to allow for different
/// positioning on WASM (would collide with the buttons otherwise).
fn spawn_info_box(asset_server: Res<AssetServer>, mut commands: Commands, top: f32, right: f32) {
    let font = asset_server.load("fonts/Assistant-Regular.ttf");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(right),
                top: Val::Percent(top),
                padding: UiRect {
                    right: Val::Px(8.),
                    left: Val::Px(8.),
                    top: Val::Px(3.),
                    bottom: Val::Px(3.),
                },
                ..Default::default()
            },
            bevy::ui::FocusPolicy::Block,
            GlobalZIndex(10),
            BackgroundColor(Color::Srgba(DARK_GRAY)),
        ))
        .insert(InfoBox)
        .insert(Interaction::default())
        .with_children(|p| {
            p.spawn((
                Text(String::new()),
                bevy::ui::FocusPolicy::Block,
                GlobalZIndex(12),
                IgnoreSave,
                TextFont::from_font(font).with_font_size(20.),
                TextColor(Color::Srgba(Srgba::hex("F49596").unwrap())),
            ));
        });
}

/// Show information about I/O in a popup.
fn display_information(
    mut writer: TextUiWriter,
    info_state: Res<Info>,
    mut info_query: Query<&Children, With<InfoBox>>,
) {
    for child in info_query.single_mut().iter() {
        let msg = info_state.msg.unwrap_or_default();
        *writer.text(*child, 0) = msg.to_string();
    }
}

/// Popup-like mouse interactions for the infobox.
fn pop_infobox(
    time: Res<Time>,
    mut info_state: ResMut<Info>,
    mut hover_query: Query<(&mut Node, &Interaction, &mut BackgroundColor), With<InfoBox>>,
) {
    if info_state.timer.tick(time.delta()).just_finished() {
        info_state.close();
    }

    for (mut ui_node, interaction, mut color) in hover_query.iter_mut() {
        if !info_state.displaying() {
            ui_node.display = Display::None;
            return;
        }
        ui_node.display = Display::Flex;
        match *interaction {
            Interaction::Hovered => {
                info_state.timer.reset();
                info_state.timer.pause();
            }
            _ => {
                info_state.timer.unpause();
            }
        }
        // fade out
        color.0.set_alpha(lerp(
            info_state.timer.elapsed_secs(),
            0.,
            info_state.timer.duration().as_secs_f32(),
            1.,
            0.,
        ));
    }
}

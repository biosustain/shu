//! Information to show in the UI.
use crate::funcplot::lerp;
use std::time::Duration;

use bevy::prelude::*;

pub struct InfoPlugin;
impl Plugin for InfoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Info {
            msg: None,
            timer: Timer::new(Duration::from_secs(3), TimerMode::Once),
        })
        .add_startup_system(spawn_info_box)
        .add_system(pop_infobox)
        .add_system(display_information);
    }
}

#[derive(Resource)]
/// Information about IO.
pub struct Info {
    msg: Option<&'static str>,
    timer: Timer,
}

impl Info {
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

fn spawn_info_box(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                padding: UiRect {
                    right: Val::Px(8.),
                    left: Val::Px(8.),
                    top: Val::Px(3.),
                    bottom: Val::Px(3.),
                },
                ..Default::default()
            },
            focus_policy: bevy::ui::FocusPolicy::Block,
            z_index: ZIndex::Global(10),
            background_color: BackgroundColor(Color::DARK_GRAY),
            ..Default::default()
        })
        .insert(InfoBox)
        .insert(Interaction::default())
        .with_children(|p| {
            p.spawn(TextBundle {
                focus_policy: bevy::ui::FocusPolicy::Block,
                z_index: ZIndex::Global(12),
                ..Default::default()
            });
        });
}

/// Show information about I/O in a popup.
fn display_information(
    info_state: Res<Info>,
    asset_server: Res<AssetServer>,
    mut info_query: Query<&Children, With<InfoBox>>,
    mut text_query: Query<&mut Text>,
) {
    for child in info_query.single_mut().iter() {
        if let Ok(mut info_box) = text_query.get_mut(*child) {
            let font = asset_server.load("fonts/Assistant-Regular.ttf");
            let msg = info_state.msg.unwrap_or_default();
            *info_box = Text::from_section(
                msg.to_string(),
                TextStyle {
                    font: font.clone(),
                    font_size: 20.,
                    color: Color::hex("F49596").unwrap(),
                },
            );
        }
    }
}

/// Popup-like mouse interactions for the infobox.
fn pop_infobox(
    time: Res<Time>,
    mut info_state: ResMut<Info>,
    mut hover_query: Query<(&mut Style, &Interaction, &mut BackgroundColor), With<InfoBox>>,
) {
    if info_state.timer.tick(time.delta()).just_finished() {
        info_state.close();
    }

    for (mut style, interaction, mut color) in hover_query.iter_mut() {
        if !info_state.displaying() {
            style.display = Display::None;
            return;
        }
        style.display = Display::Flex;
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
        color.0.set_a(lerp(
            info_state.timer.elapsed_secs(),
            0.,
            info_state.timer.duration().as_secs_f32(),
            1.,
            0.,
        ));
    }
}

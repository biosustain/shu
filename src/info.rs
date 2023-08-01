//! Information to show in the UI.
use bevy::prelude::*;

pub struct InfoPlugin;
impl Plugin for InfoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Info { msg: None })
            .add_startup_system(spawn_info_box)
            .add_system(display_information);
    }
}

#[derive(Resource)]
/// information about IO.
pub struct Info {
    msg: Option<&'static str>,
}

impl Info {
    pub fn notify(&mut self, msg: &'static str) {
        info!(msg);
        self.msg = Some(msg);
    }
    pub fn close(&mut self) {
        self.msg = None;
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
                    bottom: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            focus_policy: bevy::ui::FocusPolicy::Block,
            z_index: ZIndex::Global(10),
            ..Default::default()
        })
        .insert(InfoBox)
        .with_children(|p| {
            p.spawn(TextBundle {
                focus_policy: bevy::ui::FocusPolicy::Block,
                z_index: ZIndex::Global(10),
                ..Default::default()
            });
        });
}

fn display_information(
    info_state: Res<Info>,
    asset_server: Res<AssetServer>,
    mut info_query: Query<&Children, With<InfoBox>>,
    mut text_query: Query<&mut Text>,
) {
    if info_state.is_changed() {
        let children = info_query.single_mut();
        for child in children.iter() {
            if let Ok(mut info_box) = text_query.get_mut(*child) {
                let font = asset_server.load("fonts/Assistant-Regular.ttf");
                let msg = info_state.msg.unwrap_or_default();
                *info_box = Text::from_section(
                    msg.to_string(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 15.,
                        color: Color::hex("E49596").unwrap(),
                    },
                );
            }
        }
    }
}

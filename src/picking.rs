//! Picking systems driving interactions with the mouses and entitys
//! on the screen (draggin, rotating, scaling).

use crate::escher::{ArrowTag, Hover, NodeToText, ARROW_COLOR};
use crate::geom::{AnyTag, Drag, HistTag, VisCondition, Xaxis};
use crate::gui::UiState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::fmt::Debug;

const HIGH_COLOR: Color = Color::srgb(183. / 255., 210. / 255., 255.);

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, show_hover)
            .add_systems(Update, follow_mouse_on_drag)
            .add_systems(Update, rotate_or_scale_on_right_drag)
            .add_systems(Update, mouse_hover_highlight)
            .add_systems(Update, mouse_click_system);
    }
}

/// Cursor to mouse position. Adapted from bevy cheatbook.
pub fn get_pos(win: &Window, camera: &Camera, camera_transform: &GlobalTransform) -> Option<Vec2> {
    win.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
}

/// Register an non-UI entity (histogram) as being dragged by center or right button.
///
/// This is a custom picking system implemented for `Xaxis`. The built-in bevy picking
/// mechanism does not seem to work for non-UI nodes for some reason.
pub fn mouse_click_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut drag_query: Query<(&Transform, &mut Drag), (Without<Node>, With<Xaxis>)>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    let Ok((_, win)) = windows.get_single() else {
        return;
    };
    let middle_click = mouse_button_input.just_pressed(MouseButton::Middle);
    let right_click = mouse_button_input.just_pressed(MouseButton::Right);
    if middle_click | right_click {
        let scaling =
            key_input.pressed(KeyCode::ShiftLeft) | key_input.pressed(KeyCode::ShiftRight);
        if let Some(world_pos) = get_pos(win, camera, camera_transform) {
            for (trans, mut drag) in drag_query.iter_mut() {
                if (world_pos - Vec2::new(trans.translation.x, trans.translation.y))
                    .length_squared()
                    < 5000.
                {
                    if middle_click {
                        drag.dragged = true;
                    // do not move more than one component at the same time
                    } else {
                        drag.scaling = scaling;
                        drag.rotating = !scaling;
                    }

                    break;
                }
            }
        }
    }

    if mouse_button_input.just_released(MouseButton::Middle) {
        for (_, mut drag) in drag_query.iter_mut() {
            drag.dragged = false;
        }
    }

    if mouse_button_input.just_released(MouseButton::Right) {
        for (_, mut drag) in drag_query.iter_mut() {
            drag.scaling = false;
            drag.rotating = false;
        }
    }
}

pub fn mouse_hover_highlight(
    node_to_text: Res<NodeToText>,
    mut drag_query: Query<(&Transform, &mut Drag, &Xaxis, &mut Visibility), Without<Node>>,
    mut text_query: Query<&mut TextColor, With<ArrowTag>>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    let Ok((_, win)) = windows.get_single() else {
        return;
    };
    if let Some(world_pos) = get_pos(win, camera, camera_transform) {
        for (trans, drag, axis, mut vis) in drag_query.iter_mut() {
            let already_interacting = drag.scaling | drag.rotating | drag.dragged;
            if ((world_pos - Vec2::new(trans.translation.x, trans.translation.y)).length_squared()
                < 5000.)
                | already_interacting
            {
                // on hover: show axis line and highlight reaction name
                node_to_text.inner.get(&axis.node_id).map(|e| {
                    text_query.get_mut(*e).map(|mut color| {
                        color.0 = HIGH_COLOR;
                    })
                });
                *vis = Visibility::Visible;
                break;
            } else {
                node_to_text.inner.get(&axis.node_id).map(|e| {
                    text_query.get_mut(*e).map(|mut color| {
                        color.0 = ARROW_COLOR;
                    })
                });
                *vis = Visibility::Hidden;
            }
        }
    }
}

/// Move the center-dragged interactable non-UI entities (histograms).
pub fn follow_mouse_on_drag(
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut drag_query: Query<(&mut Transform, &Drag), Without<Node>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    for (mut trans, drag) in drag_query.iter_mut() {
        if drag.dragged {
            let (camera, camera_transform) = q_camera.single();
            let Ok((_, win)) = windows.get_single() else {
                return;
            };
            if let Some(world_pos) = get_pos(win, camera, camera_transform) {
                trans.translation = Vec3::new(world_pos.x, world_pos.y, trans.translation.z);
            }
        }
    }
}

/// Observer: move UI nodes on drag with the middle mouse button.
pub fn move_ui_on_drag(
    drag: Trigger<Pointer<bevy::prelude::Drag>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut nodes: Query<(&mut Node, &Drag)>,
    ui_scale: Res<UiScale>,
) {
    let (mut node, _drag) = nodes.get_mut(drag.entity()).unwrap();
    if !mouse_button_input.pressed(MouseButton::Middle) {
        return;
    }
    let Ok((_, win)) = windows.get_single() else {
        return;
    };
    if let Some(screen_pos) = win.cursor_position() {
        let base_offset_x = 80.;
        let base_offset_y = 50.;
        node.left = Val::Px(screen_pos.x / ui_scale.0 - base_offset_x);
        node.top = Val::Px(screen_pos.y / ui_scale.0 - base_offset_y);
    }
}

/// Observer: change the target entity's background color.
pub fn recolor_background_on<E: Debug + Clone + Reflect>(
    color: Color,
) -> impl Fn(Trigger<E>, Query<&mut BackgroundColor>) {
    move |ev, mut bgs| {
        let Ok(mut background_color) = bgs.get_mut(ev.entity()) else {
            return;
        };
        *background_color = BackgroundColor(color);
    }
}

/// Scale the right-dragged interactable (histograms and legend) entities on AxisMode::Show.
pub fn rotate_or_scale_on_right_drag(
    mut drag_query: Query<(&mut Transform, &Drag)>,
    mut mouse_motion_events: EventReader<bevy::input::mouse::MouseMotion>,
) {
    for ev in mouse_motion_events.read() {
        for (mut trans, drag) in drag_query.iter_mut() {
            if drag.scaling {
                const FACTOR: f32 = 0.01;
                let scale = ev.delta.x * FACTOR;
                trans.scale.x += scale;
            } else if drag.rotating {
                let pos = trans.translation;
                trans.rotate_around(pos, Quat::from_axis_angle(Vec3::Z, -ev.delta.y * 0.05));
                // clamping of angle to rect angles
                let (_, angle) = trans.rotation.to_axis_angle();
                const TOL: f32 = 0.06;
                if f32::abs(angle) < TOL {
                    trans.rotation = Quat::from_axis_angle(Vec3::Z, 0.);
                } else if f32::abs(angle - std::f32::consts::PI) < TOL {
                    trans.rotation = Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI);
                } else if f32::abs(angle - std::f32::consts::PI / 2.) < TOL {
                    trans.rotation = Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 2.);
                } else if f32::abs(angle - 3. * std::f32::consts::PI / 2.) < TOL {
                    trans.rotation = Quat::from_axis_angle(Vec3::Z, 3. * std::f32::consts::PI / 2.);
                }
            }
        }
    }
}

/// Show hovered data on cursor enter.
fn show_hover(
    ui_state: Res<UiState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    hover_query: Query<(&Transform, &Hover)>,
    mut popup_query: Query<(&mut Visibility, &AnyTag, &VisCondition), With<HistTag>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    let Ok(win) = windows.get_single() else {
        return;
    };
    if let Some(world_pos) = get_pos(win, camera, camera_transform) {
        for (trans, hover) in hover_query.iter() {
            if (world_pos - Vec2::new(trans.translation.x, trans.translation.y)).length_squared()
                < 5000.
            {
                for (mut vis, tag, hist) in popup_query.iter_mut() {
                    let cond_if = hist
                        .condition
                        .as_ref()
                        .map(|c| (c == &ui_state.condition) || (ui_state.condition == "ALL"))
                        .unwrap_or(true);
                    if (hover.node_id == tag.id) & cond_if {
                        *vis = Visibility::Visible;
                    }
                }
            } else {
                for (mut vis, tag, hist) in popup_query.iter_mut() {
                    let cond_if = hist
                        .condition
                        .as_ref()
                        .map(|c| (c != &ui_state.condition) & (ui_state.condition != "ALL"))
                        .unwrap_or(false);
                    if (hover.node_id == tag.id) || cond_if {
                        *vis = Visibility::Hidden;
                    }
                }
            }
        }
    }
}

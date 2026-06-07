use bevy::prelude::*;

use crate::app::markers::MenuBackground;
use crate::app::state::AppState;
use crate::ui::theme::{MENU_BG_TINT, MENU_PANEL_BORDER, MENU_PANEL_COLOR, REF_HEIGHT, REF_WIDTH};

pub fn setup_menu_background(mut commands: Commands) {
    commands.spawn((
        Sprite {
            color: MENU_BG_TINT,
            custom_size: Some(Vec2::new(REF_WIDTH * 1.2, REF_HEIGHT * 1.2)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        MenuBackground,
        Visibility::Hidden,
    ));
    commands.spawn((
        Sprite {
            color: MENU_PANEL_COLOR,
            custom_size: Some(Vec2::new(REF_WIDTH * 0.92, REF_HEIGHT * 0.88)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        MenuBackground,
        Visibility::Hidden,
    ));
    commands.spawn((
        Sprite {
            color: MENU_PANEL_BORDER,
            custom_size: Some(Vec2::new(REF_WIDTH * 0.92 + 4.0, REF_HEIGHT * 0.88 + 4.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.5),
        MenuBackground,
        Visibility::Hidden,
    ));
}

pub(crate) fn sync_menu_background_visibility(
    state: Res<State<AppState>>,
    mut query: Query<&mut Visibility, With<MenuBackground>>,
) {
    let visible = matches!(state.get(), AppState::MainMenu | AppState::SongSelect);
    let visibility = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut vis in &mut query {
        *vis = visibility;
    }
}

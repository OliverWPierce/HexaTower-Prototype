use bevy::prelude::*;

use crate::{
    backend::game_parameters::SetUpBoard,
    frontend::cameras::{LEFT_PANEL_WIDTH, LOWER_PANEL_HEIGHT, RIGHT_PANEL_WIDTH},
};

pub struct InGameUI;

impl Plugin for InGameUI {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, create_panels);
    }
}

const BACKGROUND_COLOR: Color = Color::srgb(0.057805, 0.068478, 0.093059);
const BORDER_COLOR: Color = Color::srgb(0.032044, 0.038248, 0.047155);

fn create_panels(mut commands: Commands) {
    // Lower
    commands.spawn((
        Node {
            left: Val::Percent(LEFT_PANEL_WIDTH),
            right: Val::Percent(100.0 - RIGHT_PANEL_WIDTH),
            width: Val::Percent(100.0 - RIGHT_PANEL_WIDTH - LEFT_PANEL_WIDTH),
            top: Val::Percent(100.0 - LOWER_PANEL_HEIGHT),
            height: Val::Percent(LOWER_PANEL_HEIGHT),
            border: UiRect::all(Val::Px(5.0)),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            justify_content: JustifyContent::SpaceEvenly,
            ..default()
        },
        BackgroundColor(BACKGROUND_COLOR),
        BorderColor::all(BORDER_COLOR),
    ));

    // Left
    commands.spawn((
        Node {
            left: Val::Percent(0.0),
            right: Val::Percent(LEFT_PANEL_WIDTH),
            width: Val::Percent(LEFT_PANEL_WIDTH),
            top: Val::Percent(0.0),
            height: Val::Percent(100.0),
            border: UiRect::all(Val::Px(5.0)),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            justify_content: JustifyContent::SpaceEvenly,
            ..default()
        },
        BackgroundColor(BACKGROUND_COLOR),
        BorderColor::all(BORDER_COLOR),
    ));

    // Right
    commands.spawn((
        Node {
            left: Val::Percent(100.0 - RIGHT_PANEL_WIDTH),
            right: Val::Percent(100.0),
            width: Val::Percent(RIGHT_PANEL_WIDTH),
            top: Val::Percent(0.0),
            height: Val::Percent(100.0),
            border: UiRect::all(Val::Px(5.0)),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            justify_content: JustifyContent::SpaceEvenly,
            ..default()
        },
        BackgroundColor(BACKGROUND_COLOR),
        BorderColor::all(BORDER_COLOR),
    ));
}

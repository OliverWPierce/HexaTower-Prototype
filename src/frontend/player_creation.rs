use std::{f32, panic};

use bevy::{color::palettes::tailwind, prelude::*};

use crate::{
    backend::{ActivePlayer, AppState, AvailibleThemeColors, PlayerID, set_p1_as_active},
    frontend::{ColorTools, Theme},
};

pub struct PlayerCreation;

impl Plugin for PlayerCreation {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::PlayerCreation),
            (spawn_framework, player_name, color_options)
                .chain()
                .after(set_p1_as_active),
        );
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct NamePanel;

#[derive(Component, Clone, Copy, Debug)]
struct ClassPanel;

#[derive(Component, Clone, Copy, Debug)]
struct ColorPanel;

fn spawn_framework(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceAround,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    height: Val::Percent(20.0),
                    width: Val::Percent(80.0),
                    ..Default::default()
                },
                NamePanel,
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    height: Val::Percent(20.0),
                    width: Val::Percent(80.0),
                    ..Default::default()
                },
                children![
                    (
                        Node {
                            height: Val::Px(10.0),
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        children![(
                            Text::new("Choose your banner color!"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            }
                        )]
                    ),
                    (
                        Node {
                            width: Val::Percent(40.0),
                            height: Val::Percent(80.0),
                            flex_wrap: FlexWrap::Wrap,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceAround,
                            ..default()
                        },
                        ColorPanel,
                    )
                ]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    height: Val::Percent(20.0),
                    width: Val::Percent(80.0),
                    ..Default::default()
                },
                ClassPanel,
                BackgroundColor(tailwind::ZINC_400.into()),
            )
        ],
    ));
}

fn player_name(
    mut commands: Commands,
    panel: Single<Entity, With<NamePanel>>,
    active: Res<ActivePlayer>,
    players: Query<&PlayerID>,
) {
    let id = if let Ok(id) = players.get(active.0) {
        id.id() + 1
    } else {
        panic!("The active player had no id")
    };

    commands.spawn((
        ChildOf(panel.into_inner()),
        Text::new(format!("Player{id}")),
        TextFont {
            font_size: 48.0,
            ..default()
        },
    ));
}

fn color_options(
    panel: Single<Entity, With<ColorPanel>>,
    mut commands: Commands,
    colors: Res<AvailibleThemeColors>,
    theme: Res<Theme>,
) {
    let panel_ent = panel.into_inner();
    let theme = theme.into_inner();

    for color in colors.into_inner().0.iter() {
        commands.spawn((
            ChildOf(panel_ent),
            Node {
                width: Val::Percent(17.0),
                height: Val::Percent(23.0),
                border: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(color.color(theme)),
        ));
    }
}

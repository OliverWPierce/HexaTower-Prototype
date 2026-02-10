use bevy::{
    color::palettes::tailwind::{SLATE_600, SLATE_800},
    prelude::*,
};

use crate::{
    backend::{
        game_parameters::SetUpBoard,
        players::{ActivePlayer, StartTurn, SwitchPlayerRequest},
    },
    frontend::{
        in_game_ui::{RightPanelEnt, inspector},
        visual_player_data::{DataForPlayer, DisplayName},
    },
};

pub struct EndTurnButtonPlugin;

impl Plugin for EndTurnButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            SetUpBoard,
            add_player_dispalys.after(inspector::create_inspector_panel),
        );

        app.add_systems(StartTurn, update_displayed_name);
        app.add_observer(request_turn_change);
    }
}
#[derive(Debug, Component)]
struct EndTurnButton;

#[derive(Debug, Component)]
struct PlayerNameDisplay;

fn add_player_dispalys(right_panel: Res<RightPanelEnt>, mut commands: Commands) {
    commands.spawn((
        ChildOf(right_panel.0),
        Node {
            width: Val::Percent(95.0),
            height: Val::Px(48.0),
            border: UiRect::all(Val::Percent(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        BackgroundColor(SLATE_600.into()),
        BorderColor::all(SLATE_800),
        children![(Text::new("NAME HERE"), PlayerNameDisplay)],
    ));

    commands.spawn((
        ChildOf(right_panel.0),
        EndTurnButton,
        Node {
            width: Val::Percent(95.0),
            height: Val::Px(48.0),
            border: UiRect::all(Val::Percent(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        BackgroundColor(SLATE_600.into()),
        BorderColor::all(SLATE_800),
        children![(Text::new("End Turn"),)],
    ));
}

fn request_turn_change(
    click: On<Pointer<Click>>,
    button: Single<Entity, With<EndTurnButton>>,
    mut commands: Commands,
) {
    if click.entity == button.entity() {
        commands.trigger(SwitchPlayerRequest);
    }
}

fn update_displayed_name(
    active_player: Res<ActivePlayer>,
    player_names: Query<(&DataForPlayer, &DisplayName)>,
    mut text: Single<&mut Text, With<PlayerNameDisplay>>,
) {
    for (player, name) in player_names {
        if active_player.0 != player.0 {
            continue;
        }

        text.0 = name.0.clone();
        break;
    }
}

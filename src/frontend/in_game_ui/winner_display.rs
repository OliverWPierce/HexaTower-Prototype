use bevy::prelude::*;

use crate::{
    backend::players::GameOver,
    frontend::{
        in_game_ui::{LeftPanelEnt, LowerPanelEnt, RightPanelEnt},
        visual_player_data::{DataForPlayer, DisplayName},
    },
};

pub struct WinScreenPlugin;

impl Plugin for WinScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_screen);
    }
}

fn spawn_screen(
    game_over: On<GameOver>,
    left: Res<LeftPanelEnt>,
    right: Res<RightPanelEnt>,
    lower: Res<LowerPanelEnt>,
    player_name: Query<(&DataForPlayer, &DisplayName)>,
    mut commands: Commands,
) {
    commands.entity(lower.0).despawn_children();
    commands.entity(right.0).despawn_children();
    commands.entity(left.0).despawn_children();

    if let Some(winner) = game_over.winner {
        for (log_player, name) in player_name {
            if log_player.0 == winner {
                commands.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(20.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                    children![(
                        Text::new(format!("{} wins!!", name.0)),
                        TextFont {
                            font_size: 100.0,
                            ..Default::default()
                        }
                    )],
                ));
                break;
            }
        }
    } else {
        commands.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(20.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            children![(
                Text::new("Tie Game... do better"),
                TextFont {
                    font_size: 100.0,
                    ..Default::default()
                }
            )],
        ));
    }
}

use bevy::{
    color::palettes::tailwind::{EMERALD_500, SLATE_600, SLATE_800, SLATE_900},
    prelude::*,
};

use crate::{
    backend::{
        cards::Card,
        game_parameters::SetUpBoard,
        pieces::{Health, LogPieceOwnedByPlayer},
    },
    frontend::{
        in_game_ui::{RightPanelEnt, execute_action_button},
        visual_pieces::{PieceName, VisPieceOf},
        visual_player_data::{DataForPlayer, DisplayName},
    },
};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            SetUpBoard,
            create_inspector_panel.after(execute_action_button::add_button),
        );
        app.add_observer(clear_inspector_panel);
        app.add_observer(inspect_piece);
    }
}

#[derive(Component, Debug)]
pub struct InspectorPanel;

pub fn create_inspector_panel(mut commands: Commands, right_panel: Res<RightPanelEnt>) {
    commands.spawn((
        ChildOf(right_panel.0),
        Node {
            width: Val::Percent(95.0),
            height: Val::Percent(60.0),
            border: UiRect::all(Val::Percent(2.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            padding: UiRect::top(Val::Percent(2.0)).with_bottom(Val::Percent(2.0)),
            ..default()
        },
        BackgroundColor(SLATE_600.into()),
        BorderColor::all(SLATE_800),
        InspectorPanel,
    ));
}

fn clear_inspector_panel(
    _out: On<Pointer<Out>>,
    mut commands: Commands,
    inspector: Single<Entity, With<InspectorPanel>>,
) {
    commands.entity(inspector.entity()).despawn_children();
}

pub fn inspect_card(inspector: Entity, commands: &mut Commands, card_data: &Card, owned: bool) {
    commands.entity(inspector.entity()).despawn_children();

    commands.spawn((
        ChildOf(inspector),
        Node {
            width: Val::Percent(90.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Text::new(card_data.name.clone()),
            TextFont {
                font_size: 36.0,
                ..default()
            }
        )],
    ));

    commands.spawn((
        ChildOf(inspector),
        Node {
            width: Val::Percent(90.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Text::new(card_data.rarity.display_name()),
                TextColor(card_data.rarity.text_color()),
                TextFont {
                    font_size: 24.0,
                    ..default()
                }
            ),
            (
                Text::new(" Card"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
            )
        ],
    ));

    if owned {
        commands.spawn((
            ChildOf(inspector),
            Node {
                width: Val::Percent(90.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            children![(
                Text::new("Owned"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
            )],
        ));
    } else {
        commands.spawn((
            ChildOf(inspector),
            Node {
                width: Val::Percent(90.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                (
                    Text::new("Price:"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                ),
                (
                    Text::new(format!("{}", card_data.price)),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(EMERALD_500.into())
                )
            ],
        ));
    }

    commands.spawn((
        ChildOf(inspector),
        Node {
            aspect_ratio: Some(3.0 / 5.0),
            height: Val::Percent(50.0),
            border: UiRect::all(Val::Px(7.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderRadius::all(Val::Px(5.0)),
        BorderColor::all(Color::Srgba(SLATE_900)),
        children![(
            ImageNode {
                image: card_data.image.clone(),
                color: card_data.rarity.card_color(),
                image_mode: NodeImageMode::Auto,
                ..Default::default()
            },
            BorderRadius::all(Val::Px(5.0)),
            Node {
                aspect_ratio: Some(3.0 / 5.0),
                max_height: Val::Percent(100.0),
                ..default()
            }
        )],
    ));

    commands.spawn((
        ChildOf(inspector),
        Node {
            width: Val::Percent(90.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::top(Val::Px(12.0)),
            ..default()
        },
        children![(
            Text::new(card_data.description.clone()),
            TextFont {
                font_size: 16.0,
                ..default()
            },
        )],
    ));
}

fn inspect_piece(
    over: On<Pointer<Over>>,
    mut commands: Commands,
    inspector: Single<Entity, With<InspectorPanel>>,
    vis_pieces: Query<(&VisPieceOf, &PieceName)>,
    log_pieces: Query<(&Health, &LogPieceOwnedByPlayer)>,
    player_vis_data: Query<(&DisplayName, &DataForPlayer)>,
) {
    let Ok((log_piece_ent, name)) = vis_pieces.get(over.entity) else {
        return;
    };

    let Ok((health_data, commanding_player)) = log_pieces.get(log_piece_ent.0) else {
        error!("The visual piece did not point to a logical piece");
        return;
    };

    let inspector = inspector.entity();

    commands.entity(inspector).despawn_children();

    commands.spawn((
        ChildOf(inspector),
        Node {
            width: Val::Percent(90.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Text::new(name.0.clone()),
            TextFont {
                font_size: 36.0,
                ..default()
            }
        )],
    ));

    for (player_name, player) in player_vis_data {
        if commanding_player.0 != player.0 {
            continue;
        }

        commands.spawn((
            ChildOf(inspector),
            Node {
                width: Val::Percent(90.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            children![(
                Text::new(format!("Commander: {}", player_name.0.clone())),
                TextFont {
                    font_size: 24.0,
                    ..default()
                }
            )],
        ));
        break;
    }

    let health_panel = commands
        .spawn((
            ChildOf(inspector),
            Node {
                height: Val::Percent(20.0),
                width: Val::Percent(90.0),
                border: UiRect::all(Val::Percent(0.5)),
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            BorderRadius::all(Val::Px(15.0)),
            BorderColor::all(Color::Srgba(SLATE_800)),
        ))
        .id();

    commands.spawn((
        ChildOf(health_panel),
        Text::new(format!(
            "{}/{} Hp",
            health_data.current_health, health_data.max_health
        )),
        TextFont {
            font_size: 24.0,
            ..default()
        },
    ));

    commands.spawn((ChildOf(inspector),));
}

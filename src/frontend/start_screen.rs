use bevy::{
    color::{
        self,
        palettes::tailwind::{self, SLATE_700},
    },
    prelude::*,
};

use crate::backend::{AppState, BOARD_SIZES, BoardSize, PLAYER_COUNTS, PlayerCount};

pub struct StartScreen;

impl Plugin for StartScreen {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::GameParameters),
            (
                create_start_screen,
                display_player_count_choices,
                display_board_sizes,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (
                update_selected_player_count_node,
                update_selected_board_size_node,
            )
                .run_if(in_state(AppState::GameParameters)),
        );
    }
}

// (
//     Button,
//     StartGameButton,
//     Node {
//         width: Val::Px(200.0),
//         height: Val::Px(70.0),
//         align_items: AlignItems::Center,
//         justify_content: JustifyContent::Center,
//         border: UiRect::all(Val::Px(3.0)),
//         ..default()
//     },
//     BackgroundColor(tailwind::ZINC_600.into()),
//     BorderColor(tailwind::ZINC_300.into()),
//     BorderRadius::all(Val::Px(1000.0)),
//     children![(
//         Text::new("Start"),
//         TextFont {
//             font_size: 60.0,
//             ..default()
//         },
//         TextColor(tailwind::ZINC_300.into())
//     )]
// )

#[derive(Debug, Component)]
struct BoardSizePanel;

#[derive(Debug, Component)]
struct PlayerCountPanel;

#[derive(Debug, Component)]
struct StartGameButton;

#[derive(Debug, Component)]
struct RootNode;

fn create_start_screen(mut commands: Commands) {
    commands.spawn((
        Node {
            height: Val::Percent(100.0),
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceAround,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        RootNode,
        children![
            (
                Node {
                    height: Val::Percent(15.0),
                    top: Val::Percent(5.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![(
                    Text::new("New Game"),
                    TextFont {
                        font_size: 80.0,
                        ..default()
                    }
                )],
            ),
            (
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Px(140.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![
                    (
                        Text::new("Player Count"),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(tailwind::ZINC_300.into())
                    ),
                    (
                        Node {
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        PlayerCountPanel
                    )
                ]
            ),
            (
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Px(200.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![
                    (
                        Text::new("Arena Size"),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(tailwind::ZINC_300.into())
                    ),
                    (
                        Node {
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BoardSizePanel
                    )
                ]
            )
        ],
    ));
}

#[derive(Debug, Resource)]
struct SelectedPlayerCountNode(Entity);

#[derive(Debug, Resource)]
struct SelectedBoardSizeNode(Entity);

fn display_player_count_choices(
    panel: Single<Entity, With<PlayerCountPanel>>,
    mut commands: Commands,
) {
    let panel_entity = panel.into_inner();

    for count in PLAYER_COUNTS.iter() {
        commands
            .spawn((
                *count,
                Button,
                ChildOf(panel_entity),
                Node {
                    min_height: Val::Px(60.0),
                    width: Val::Percent(17.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(tailwind::SLATE_800.into()),
                BorderColor(tailwind::SLATE_500.into()),
                BorderRadius::all(Val::Px(100.0)),
                children![(
                    Text::new(match count {
                        PlayerCount::Two => "2 Players",
                        PlayerCount::Three => "3 Players",
                        PlayerCount::Four => "4 Players",
                        PlayerCount::Five => "5 Players",
                        PlayerCount::Six => "6 Players",
                    }),
                    TextFont {
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(tailwind::SLATE_500.into())
                )],
            ))
            .observe(player_count_selector);
    }
}

fn display_board_sizes(panel: Single<Entity, With<BoardSizePanel>>, mut commands: Commands) {
    let panel_entity = panel.into_inner();

    for size in BOARD_SIZES.iter() {
        commands
            .spawn((
                *size,
                Button,
                ChildOf(panel_entity),
                Node {
                    min_height: Val::Px(80.0),
                    width: Val::Percent(23.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(tailwind::SLATE_800.into()),
                BorderColor(tailwind::SLATE_500.into()),
                BorderRadius::all(Val::Px(100.0)),
                children![
                    (
                        Text::new(match size {
                            BoardSize::Small => "Small",
                            BoardSize::Medium => "Medium",
                            BoardSize::Large => "Large",
                            BoardSize::ExtraLarge => "Extra Large",
                        }),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(tailwind::SLATE_500.into())
                    ),
                    (
                        Text::new(match size {
                            BoardSize::Small => "recommended 2 players",
                            BoardSize::Medium => "recomended 3 players",
                            BoardSize::Large => "recomended 4-6 players",
                            BoardSize::ExtraLarge => "simulate the earth",
                        }),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(tailwind::SLATE_400.into())
                    )
                ],
            ))
            .observe(board_size_selector);
    }
}

fn player_count_selector(
    mut trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    selected_count: Option<ResMut<SelectedPlayerCountNode>>,
) {
    trigger.propagate(false);

    if selected_count.is_none() {
        commands.insert_resource(SelectedPlayerCountNode(trigger.target()));
    } else {
        selected_count.unwrap().0 = trigger.target()
    }
}

fn board_size_selector(
    mut trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    selected_size: Option<ResMut<SelectedBoardSizeNode>>,
) {
    trigger.propagate(false);

    if selected_size.is_none() {
        commands.insert_resource(SelectedBoardSizeNode(trigger.target()));
    } else {
        selected_size.unwrap().0 = trigger.target()
    }
}

const SELECTED_BORDER: Color = Color::Srgba(tailwind::EMERALD_400);
const SELECTED_FILL: Color = Color::Srgba(tailwind::EMERALD_950);

fn update_selected_player_count_node(
    node_res: Option<Res<SelectedPlayerCountNode>>,
    mut nodes: Query<(&mut BackgroundColor, &mut BorderColor), With<PlayerCount>>,
) {
    let Some(res) = node_res else { return };
    if !res.is_changed() {
        return;
    }

    for (mut background, mut border) in nodes.iter_mut() {
        background.0 = tailwind::SLATE_800.into();
        border.0 = tailwind::SLATE_500.into();
    }

    if let Ok((mut background, mut border)) = nodes.get_mut(res.0) {
        background.0 = SELECTED_FILL;
        border.0 = SELECTED_BORDER;
    }
}

fn update_selected_board_size_node(
    node_res: Option<Res<SelectedBoardSizeNode>>,
    mut nodes: Query<(&mut BackgroundColor, &mut BorderColor), With<BoardSize>>,
) {
    if node_res.is_none() {
    } else {
        let res = node_res.unwrap();

        if res.is_changed() {
            for (mut background, mut border) in nodes.iter_mut() {
                background.0 = tailwind::SLATE_800.into();
                border.0 = tailwind::SLATE_500.into();
            }

            if let Ok((mut background, mut border)) = nodes.get_mut(res.0) {
                background.0 = SELECTED_FILL;
                border.0 = SELECTED_BORDER;
            }
        }
    }
}

use std::panic;

use bevy::{color::palettes::tailwind, prelude::*};

use crate::{
    backend::{
        ActivePlayer, AppState, AvailibleThemeColors, Class, PlayerID, SetNextPlayerAsActive,
        set_p1_as_active,
    },
    frontend::{ColorTools, DisplayName, Theme},
};

pub struct PlayerCreation;

impl Plugin for PlayerCreation {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::PlayerCreation),
            (
                spawn_framework,
                player_name,
                color_options,
                class_choices,
                start_selection_timer,
            )
                .chain()
                .after(set_p1_as_active),
        );
        app.add_systems(
            Update,
            (tick_timer, shrink_timer_bar).run_if(in_state(AppState::PlayerCreation)),
        );
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct NamePanel;

#[derive(Component, Clone, Copy, Debug)]
struct ClassPanel;

#[derive(Component, Clone, Copy, Debug)]
struct ColorPanel;

#[derive(Component, Clone, Copy, Debug)]
struct TimerPanel;

fn spawn_framework(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        children![
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    top: Val::Percent(5.0),
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
                BackgroundColor(tailwind::ZINC_400.into()),
                children![
                    (
                        Node {
                            height: Val::Px(10.0),
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        children![(
                            Text::new("Choose your class!"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            }
                        )]
                    ),
                    (
                        Node {
                            width: Val::Percent(65.0),
                            height: Val::Percent(80.0),
                            flex_wrap: FlexWrap::Wrap,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceAround,
                            ..default()
                        },
                        ClassPanel,
                    )
                ]
            ),
            (
                Node {
                    height: Val::Percent(2.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                TimerPanel,
                BackgroundColor(Color::WHITE),
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

const CLASS_CHOICES: [Class; 3] = [Class::Class1, Class::Class2, Class::Class3];

fn class_choices(
    mut commands: Commands,
    panel: Single<Entity, With<ClassPanel>>,
    theme: Res<Theme>,
) {
    let panel_ent = panel.into_inner();

    for class in CLASS_CHOICES {
        commands.spawn((
            ChildOf(panel_ent),
            Node {
                width: Val::Percent(30.0),
                height: Val::Percent(25.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(tailwind::SLATE_700.into()),
            children![(
                Text::new(class.display_text()),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(class.color(&theme).with_luminance(0.9)),
            )],
        ));
    }
}

// Timer stuff

#[derive(Debug, Resource)]
struct SelectionTimer(Timer);

const TIMER_DURATION: f32 = 10.0;

fn start_selection_timer(mut commands: Commands) {
    commands.insert_resource(SelectionTimer(Timer::from_seconds(
        TIMER_DURATION,
        TimerMode::Once,
    )));
}

fn tick_timer(mut timer: ResMut<SelectionTimer>, mut commands: Commands, time: Res<Time>) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        commands.trigger(SetNextPlayerAsActive);
    }
}

fn shrink_timer_bar(mut panel: Single<&mut Node, With<TimerPanel>>, timer: Res<SelectionTimer>) {
    let percent = (-100.0 / TIMER_DURATION) * timer.0.elapsed_secs() + 100.0;

    panel.width = Val::Percent(percent);

    println!("{}", timer.0.elapsed_secs());
}

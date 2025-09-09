use std::panic;

use bevy::{
    color::palettes::{css::RED, tailwind},
    prelude::*,
};

use crate::{
    backend::{
        ActivePlayer, AppState, AvailibleThemeColors, Class, PlayerCreated,
        PlayerCreationInstructions, PlayerID, SetNextPlayerAsActive, ThemeColorId,
        set_p1_as_active,
    },
    frontend::{ColorTools, DisplayName, Theme},
};

pub struct PlayerCreation;

impl Plugin for PlayerCreation {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::PlayerCreation),
            (spawn_framework, class_choices, start_selection_timer)
                .chain()
                .after(set_p1_as_active),
        );
        app.add_systems(
            Update,
            (tick_timer, shrink_timer_bar).run_if(in_state(AppState::PlayerCreation)),
        );
        app.add_event::<SubmitPlayerData>();

        app.init_resource::<SelectedClassButton>();
        app.init_resource::<SelectedColorButton>();

        app.add_systems(
            Update,
            (start_selection_timer, player_name)
                .run_if(in_state(AppState::PlayerCreation))
                .run_if(resource_exists_and_changed::<ActivePlayer>),
        );
        app.add_systems(
            Update,
            color_options
                .run_if(in_state(AppState::PlayerCreation))
                .run_if(resource_changed::<AvailibleThemeColors>),
        );
        app.add_observer(assign_clicked_to_res);

        app.add_systems(
            Update,
            highlight_selected.run_if(in_state(AppState::PlayerCreation)),
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

    let ent = panel.into_inner();

    commands.entity(ent).despawn_related::<Children>();

    commands.spawn((
        ChildOf(ent),
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

    commands.entity(panel_ent).despawn_related::<Children>();

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
            Button,
            *color,
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
                border: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(tailwind::SLATE_700.into()),
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            Button,
            class,
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

const TIMER_DURATION: f32 = 1.0;

fn start_selection_timer(mut commands: Commands) {
    commands.insert_resource(SelectionTimer(Timer::from_seconds(
        TIMER_DURATION,
        TimerMode::Once,
    )));
}

fn tick_timer(mut timer: ResMut<SelectionTimer>, mut commands: Commands, time: Res<Time>) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        commands.run_system_cached(end_selection_turn_and_go_next_player);
    }
}

fn shrink_timer_bar(
    mut panel: Single<(&mut Node, &mut BackgroundColor), With<TimerPanel>>,
    timer: Res<SelectionTimer>,
) {
    let percent = (-100.0 / TIMER_DURATION) * timer.0.elapsed_secs() + 100.0;

    panel.0.width = Val::Percent(percent);
    panel.1.0 = Color::srgb(1.0, 0.0, 0.0).with_saturation((100.0 - percent) / 100.0);
}

// Select Characteristics

#[derive(Debug, Event)]
struct SubmitPlayerData;

#[derive(Debug, Resource, Default)]
struct SelectedClassButton(Option<Entity>);

#[derive(Debug, Resource, Default)]
struct SelectedColorButton(Option<Entity>);

use rand::{rng, seq::IndexedRandom};

fn end_selection_turn_and_go_next_player(
    mut commands: Commands,
    mut selected_class: ResMut<SelectedClassButton>,
    mut selected_color: ResMut<SelectedColorButton>,
    classes: Query<&Class>,
    colors: Query<&ThemeColorId>,
    availible_colors: Res<AvailibleThemeColors>,
    active: Res<ActivePlayer>,
) {
    // if there is a selected color, then use it. Else, pick a color from the list.
    let color = {
        if let Some(ent) = selected_color.0 {
            colors
                .get(ent)
                .expect("The entity had no ThemeColorId component")
        } else {
            availible_colors
                .0
                .choose(&mut rng())
                .expect("There were no more theme colors remaining.")
        }
    };

    let class = {
        if let Some(ent) = selected_class.0 {
            classes
                .get(ent)
                .expect("The entity had no ThemeColorId component")
        } else {
            CLASS_CHOICES
                .choose(&mut rng())
                .expect("There were no more theme colors remaining.")
        }
    };

    commands.trigger(PlayerCreationInstructions {
        entity: active.0,
        class: *class,
        color: *color,
    });

    selected_class.0 = None;
    selected_color.0 = None;

    commands.trigger(SetNextPlayerAsActive);
}

#[derive(Debug, Component)]
struct Highlight;

fn assign_clicked_to_res(
    trigger: Trigger<Pointer<Click>>,
    colors: Query<Entity, With<ThemeColorId>>,
    classes: Query<Entity, With<Class>>,
    mut selected_color: ResMut<SelectedColorButton>,
    mut selected_class: ResMut<SelectedClassButton>,
) {
    let clicked = trigger.target();

    if let Ok(color_button) = colors.get(clicked) {
        selected_color.0 = Some(color_button);
    } else if let Ok(class_button) = classes.get(clicked) {
        selected_class.0 = Some(class_button);
    }
}

fn highlight_selected(
    color_button: Res<SelectedColorButton>,
    class_button: Res<SelectedClassButton>,
    mut buttons: Query<(Entity, &mut BorderColor)>,
) {
    for (ent, mut color) in buttons.iter_mut() {
        color.0 = Color::BLACK;

        if color_button.0.is_some() && ent == color_button.0.unwrap() {
            color.0 = Color::WHITE
        }

        if class_button.0.is_some() && ent == class_button.0.unwrap() {
            // this is safe because we already checked that the selected button was some.
            color.0 = Color::WHITE
        }
    }
}

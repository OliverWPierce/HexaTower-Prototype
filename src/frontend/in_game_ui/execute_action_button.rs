use crate::{
    backend::{
        game_actions::{
            ActionOrSelectionChanged, CurrentAction, ExecuteActionRequest,
            dangerous_selection_mechanics::SelectedLogTiles,
        },
        game_parameters::SetUpBoard,
    },
    frontend::{
        FrontEndSystems,
        in_game_ui::{RightPanelEnt, create_panels},
    },
};
use bevy::{color::palettes::tailwind, prelude::*};

pub struct ExecuteActionButtonPlugin;

impl Plugin for ExecuteActionButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(send_execute_actions);

        app.add_systems(SetUpBoard, add_button.after(create_panels));

        app.add_systems(
            ActionOrSelectionChanged,
            update_button.in_set(FrontEndSystems),
        );
    }
}

const NO_ACTION_BACKGROUND: Color = Color::Srgba(tailwind::SLATE_600);
const NO_ACTION_BORDER: Color = Color::Srgba(tailwind::SLATE_800);

const ACTION_READY_BACKGROUND: Color = Color::Srgba(tailwind::RED_600);
const ACTION_READY_BORDER: Color = Color::Srgba(tailwind::RED_800);

const ACTION_NOT_READY_BACKGROUND: Color = Color::Srgba(tailwind::RED_900);
const ACTION_NOT_READY_BORDER: Color = Color::Srgba(tailwind::RED_950);

#[derive(Debug, Component)]
struct ExecuteActionButton;

fn add_button(mut commands: Commands, parent: Res<RightPanelEnt>) {
    commands.spawn((
        ChildOf(parent.0),
        Node {
            height: Val::Percent(15.0),
            width: Val::Percent(95.0),
            border: UiRect::all(Val::Percent(2.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,

            ..default()
        },
        BackgroundColor(NO_ACTION_BACKGROUND),
        BorderColor::all(NO_ACTION_BORDER),
        ExecuteActionButton,
        Button,
        children![(
            Text::new("No Previewed Action"),
            TextLayout::new_with_justify(Justify::Center)
        )],
    ));
}

fn update_button(
    action: Res<CurrentAction>,
    currently_selected: Res<SelectedLogTiles>,
    button: Single<(Entity, &mut BackgroundColor, &mut BorderColor), With<ExecuteActionButton>>,
    mut commands: Commands,
) {
    if let Some(action) = action.0 {
        let (button_ent, mut background, mut border) = button.into_inner();

        if currently_selected.as_read_only_list().len() < action.read_min_and_max_tiles().0 {
            *background = BackgroundColor(ACTION_NOT_READY_BACKGROUND);
            *border = BorderColor::all(ACTION_NOT_READY_BORDER);
        } else {
            *background = BackgroundColor(ACTION_READY_BACKGROUND);
            *border = BorderColor::all(ACTION_READY_BORDER);
        }

        commands.entity(button_ent).despawn_children();
        commands.spawn((
            ChildOf(button_ent),
            Text::new("ORDER"),
            TextLayout::new_with_justify(Justify::Center),
            TextFont {
                font_size: 36.0,
                ..default()
            },
        ));
        commands.spawn((
            ChildOf(button_ent),
            Text::new(format!(
                "Selected {} / {}",
                currently_selected.as_read_only_list().len(),
                action.read_min_and_max_tiles().1
            )),
            TextLayout::new_with_justify(Justify::Center),
            TextFont {
                font_size: 24.0,
                ..default()
            },
        ));
        commands.spawn((
            ChildOf(button_ent),
            Text::new(format!(
                "Select at least {}",
                action.read_min_and_max_tiles().0
            )),
            TextLayout::new_with_justify(Justify::Center),
            TextFont {
                font_size: 16.0,
                ..default()
            },
        ));
    } else {
        let (button_ent, mut background, mut border) = button.into_inner();

        *background = BackgroundColor(NO_ACTION_BACKGROUND);
        *border = BorderColor::all(NO_ACTION_BORDER);

        commands.entity(button_ent).despawn_children();
        commands.spawn((
            ChildOf(button_ent),
            Text::new("No Previewed Action"),
            TextLayout::new_with_justify(Justify::Center),
        ));
    }
}

fn send_execute_actions(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    execute_button: Single<Entity, With<ExecuteActionButton>>,
) {
    if click.entity == execute_button.into_inner() {
        commands.trigger(ExecuteActionRequest);
    }
}

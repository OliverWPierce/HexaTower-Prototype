use crate::{
    backend::{
        game_actions::{
            ActionOrSelectionChanged, CurrentAction, CurrentActiveElement, ExecuteActionRequest,
            SelectionBounds, dangerous_selection_mechanics::SelectedLogTiles,
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
    action_soruce: Res<CurrentActiveElement>,
    mut commands: Commands,
) {
    if let Some(action) = action.0 {
        let (button_ent, mut background, mut border) = button.into_inner();

        let display_action_name = match action_soruce.0 {
            Some(source) => match source {
                crate::backend::game_actions::ActiveElement::LogCard(_) => "Play Card",
                crate::backend::game_actions::ActiveElement::LogPiece(_) => "Order",
            },
            None => "???",
        };

        let ready_background_color = match action_soruce.0 {
            Some(source) => match source {
                crate::backend::game_actions::ActiveElement::LogCard(_) => {
                    Color::Srgba(tailwind::BLUE_600)
                }
                crate::backend::game_actions::ActiveElement::LogPiece(_) => {
                    Color::Srgba(tailwind::RED_600)
                }
            },
            None => Color::Srgba(tailwind::YELLOW_600),
        };

        let ready_border_color = match action_soruce.0 {
            Some(source) => match source {
                crate::backend::game_actions::ActiveElement::LogCard(_) => {
                    Color::Srgba(tailwind::BLUE_800)
                }
                crate::backend::game_actions::ActiveElement::LogPiece(_) => {
                    Color::Srgba(tailwind::RED_800)
                }
            },
            None => Color::Srgba(tailwind::YELLOW_800),
        };

        let unready_background_color = match action_soruce.0 {
            Some(source) => match source {
                crate::backend::game_actions::ActiveElement::LogCard(_) => {
                    Color::Srgba(tailwind::BLUE_900)
                }
                crate::backend::game_actions::ActiveElement::LogPiece(_) => {
                    Color::Srgba(tailwind::RED_900)
                }
            },
            None => Color::Srgba(tailwind::YELLOW_900),
        };

        let unready_border_color = match action_soruce.0 {
            Some(source) => match source {
                crate::backend::game_actions::ActiveElement::LogCard(_) => {
                    Color::Srgba(tailwind::BLUE_950)
                }
                crate::backend::game_actions::ActiveElement::LogPiece(_) => {
                    Color::Srgba(tailwind::RED_950)
                }
            },
            None => Color::Srgba(tailwind::YELLOW_950),
        };

        if currently_selected.as_read_only_list().len() < action.bounds().min_tiles {
            *background = BackgroundColor(unready_background_color);
            *border = BorderColor::all(unready_border_color);
        } else {
            *background = BackgroundColor(ready_background_color);
            *border = BorderColor::all(ready_border_color);
        }

        commands.entity(button_ent).despawn_children();
        commands.spawn((
            ChildOf(button_ent),
            Text::new(display_action_name),
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
                action.bounds().max_tiles
            )),
            TextLayout::new_with_justify(Justify::Center),
            TextFont {
                font_size: 24.0,
                ..default()
            },
        ));
        commands.spawn((
            ChildOf(button_ent),
            Text::new(format!("Select at least {}", action.bounds().min_tiles)),
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

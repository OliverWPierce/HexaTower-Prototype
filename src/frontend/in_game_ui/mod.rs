mod execute_action_button;
mod inspector;
mod inventory;
mod order_display;
mod player_info_displays;
mod shop_visuals;

use bevy::{color::palettes::tailwind, prelude::*};

use crate::{
    backend::{cards::CardRarity, game_parameters::SetUpBoard},
    frontend::{
        cameras::{LEFT_PANEL_WIDTH, LOWER_PANEL_HEIGHT, RIGHT_PANEL_WIDTH},
        in_game_ui::{
            execute_action_button::ExecuteActionButtonPlugin, inspector::InspectorPlugin,
            inventory::InventoryPlugin, order_display::OrderDisplayPlugin,
            player_info_displays::EndTurnButtonPlugin, shop_visuals::ShopVisualPlugin,
        },
    },
};

pub struct InGameUI;

impl Plugin for InGameUI {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, create_panels);

        app.add_plugins((
            ExecuteActionButtonPlugin,
            InventoryPlugin,
            ShopVisualPlugin,
            EndTurnButtonPlugin,
            InspectorPlugin,
            OrderDisplayPlugin,
        ));
    }
}

pub const BACKGROUND_COLOR: Color = Color::srgb(0.057805, 0.068478, 0.093059);
pub const BORDER_COLOR: Color = Color::srgb(0.032044, 0.038248, 0.047155);
pub const BORDER_WIDTH: Val = Val::Percent(0.2);

impl CardRarity {
    fn card_color(&self) -> Color {
        match self {
            CardRarity::Legendary => tailwind::ROSE_600.into(),
            CardRarity::Epic => tailwind::PURPLE_600.into(),
            CardRarity::Rare => tailwind::TEAL_600.into(),
            CardRarity::Common => tailwind::STONE_600.into(),
        }
    }

    fn text_color(&self) -> Color {
        self.card_color().with_saturation(1.0)
    }

    fn display_name(&self) -> String {
        match self {
            CardRarity::Legendary => String::from("Legendary"),
            CardRarity::Epic => String::from("Epic"),
            CardRarity::Rare => String::from("Rare"),
            CardRarity::Common => String::from("Common"),
        }
    }
}

#[derive(Debug, Resource)]
pub struct LeftPanelEnt(Entity);
#[derive(Debug, Resource)]
pub struct LowerPanelEnt(Entity);
#[derive(Debug, Resource)]
pub struct RightPanelEnt(Entity);

fn create_panels(mut commands: Commands) {
    // Lower
    let lower = commands
        .spawn((
            Node {
                left: Val::Percent(LEFT_PANEL_WIDTH),
                right: Val::Percent(100.0 - RIGHT_PANEL_WIDTH),
                width: Val::Percent(100.0 - RIGHT_PANEL_WIDTH - LEFT_PANEL_WIDTH),
                top: Val::Percent(100.0 - LOWER_PANEL_HEIGHT),
                height: Val::Percent(LOWER_PANEL_HEIGHT),
                border: UiRect::top(BORDER_WIDTH).with_bottom(BORDER_WIDTH),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            BorderColor::all(BORDER_COLOR),
        ))
        .id();

    // Left
    let left = commands
        .spawn((
            Node {
                left: Val::Percent(0.0),
                right: Val::Percent(LEFT_PANEL_WIDTH),
                width: Val::Percent(LEFT_PANEL_WIDTH),
                top: Val::Percent(0.0),
                height: Val::Percent(100.0),
                border: UiRect::all(BORDER_WIDTH),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            BorderColor::all(BORDER_COLOR),
        ))
        .id();

    // Right
    let right = commands
        .spawn((
            Node {
                left: Val::Percent(100.0 - RIGHT_PANEL_WIDTH),
                right: Val::Percent(100.0),
                width: Val::Percent(RIGHT_PANEL_WIDTH),
                top: Val::Percent(0.0),
                height: Val::Percent(100.0),
                border: UiRect::all(BORDER_WIDTH),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            BorderColor::all(BORDER_COLOR),
        ))
        .id();

    commands.insert_resource(LowerPanelEnt(lower));
    commands.insert_resource(RightPanelEnt(right));
    commands.insert_resource(LeftPanelEnt(left));
}

use crate::{
    backend::{
        cards::{CardAsset, LogCard, TmpLogCardInventory},
        game_parameters::SetUpBoard,
    },
    frontend::{
        FrontEndSystems,
        cameras::{LEFT_PANEL_WIDTH, LOWER_PANEL_HEIGHT, RIGHT_PANEL_WIDTH},
        in_game_ui::{BACKGROUND_COLOR, BORDER_COLOR, LowerPanelEnt, create_panels},
    },
};

use bevy::{color::palettes::tailwind, prelude::*};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, create_inventory_panel.after(create_panels));

        app.add_systems(Update, tmp_load_cards_into_ui.in_set(FrontEndSystems));
    }
}
#[derive(Debug, Component)]
struct CardHolderPanel;

#[derive(Debug, Component)]
struct InventoryInfoNode;

fn create_inventory_panel(mut commands: Commands, lower_panel: Res<LowerPanelEnt>) {
    let parent = commands
        .spawn((
            ChildOf(lower_panel.0),
            Node {
                height: Val::Percent(95.0),
                width: Val::Percent(65.0),
                border: UiRect::all(Val::Percent(0.5)),
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            BorderRadius::all(Val::Px(15.0)),
            BorderColor::all(Color::Srgba(tailwind::SLATE_800)),
            BackgroundColor(BACKGROUND_COLOR),
        ))
        .id();

    commands.spawn((
        ChildOf(parent),
        Node {
            width: Val::Percent(98.0),
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::bottom(Val::Percent(0.5)),
            ..Default::default()
        },
        BorderColor::all(Color::Srgba(tailwind::SLATE_700)),
        children![
            (
                Text::from("Inventory - You own and can play these cards."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                }
            ),
            (
                Text::from("0 / 8 cards"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                InventoryInfoNode,
            )
        ],
    ));

    commands.spawn((
        ChildOf(parent),
        CardHolderPanel,
        Node {
            flex_shrink: 0.0,
            width: Val::Percent(98.0),
            height: Val::Percent(80.0),
            flex_wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::SpaceAround,
            ..default()
        },
    ));
}

fn tmp_load_cards_into_ui(
    log_inventory: Res<TmpLogCardInventory>,
    cards: Query<&LogCard>,
    card_assets: Res<Assets<CardAsset>>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
    panel: Single<Entity, With<CardHolderPanel>>,
    inputs: Res<ButtonInput<KeyCode>>,
) {
    if !inputs.just_pressed(KeyCode::KeyR) {
        return;
    }

    commands.entity(panel.entity()).despawn_children();

    for log_card in log_inventory.0.iter() {
        let Ok(handle) = cards.get(*log_card) else {
            warn!("the logical inventory contained an entity that was not a logical card.");
            continue;
        };

        let Some(card_data) = card_assets.get(handle.0.id()) else {
            warn!("A logical card's handle to the card asset failed to retrieve the asset.");
            continue;
        };

        println!("Added card {} to the visual inventory.", card_data.name);

        commands.spawn((
            ChildOf(panel.entity()),
            Node {
                aspect_ratio: Some(3.0 / 5.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BorderRadius::all(Val::Px(5.0)),
            ImageNode {
                image: asset_server.load(card_data.image_path.clone()),
                image_mode: NodeImageMode::Auto,
                ..Default::default()
            },
        ));
    }
}

use crate::{
    backend::{
        cards::{CardAsset, InventoryUpdated, PlayerCardInventory},
        game_actions::{ActionOrSelectionChanged, ActionSource, CurrentSource, SetActionTo},
        game_parameters::SetUpBoard,
        players::{ActivePlayer, StartTurn},
    },
    frontend::in_game_ui::{LowerPanelEnt, create_panels},
};

use bevy::{color::palettes::tailwind, prelude::*};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, create_inventory_panel.after(create_panels));

        app.add_systems(
            InventoryUpdated,
            (load_cards_into_ui, update_selection).chain(),
        );
        app.add_systems(StartTurn, (load_cards_into_ui, update_selection).chain());
        app.add_systems(ActionOrSelectionChanged, update_selection);

        app.add_observer(load_card_action);
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

#[derive(Debug, Component)]
struct CorrespondingInventoryIndex(usize);

fn load_cards_into_ui(
    active_player: Res<ActivePlayer>,
    inventories: Query<&PlayerCardInventory>,
    card_assets: Res<Assets<CardAsset>>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
    card_parent_panel: Single<Entity, With<CardHolderPanel>>,
    info_panel: Single<Entity, With<InventoryInfoNode>>,
) {
    commands
        .entity(card_parent_panel.entity())
        .despawn_children();

    let Ok(log_inventory) = inventories.get(active_player.0) else {
        error!("The player had no inventory");
        return;
    };

    commands
        .entity(info_panel.entity())
        .insert(Text::from(format!(
            "{} / {} cards",
            log_inventory.cards.len(),
            log_inventory.max_size,
        )));

    for (index, handle) in log_inventory.cards.iter().enumerate() {
        let Some(card_data) = card_assets.get(handle.id()) else {
            warn!("A handle to the card asset failed to retrieve the asset.");
            continue;
        };

        commands.spawn((
            ChildOf(card_parent_panel.entity()),
            CorrespondingInventoryIndex(index),
            Node {
                aspect_ratio: Some(3.0 / 5.0),
                height: Val::Percent(100.0),
                border: UiRect::all(Val::Px(3.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::all(Val::Px(5.0)),
            BorderColor::all(Color::Srgba(tailwind::SLATE_700)),
            children![(
                ImageNode {
                    image: asset_server.load(card_data.image_path.clone()),
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
    }
}

fn load_card_action(
    click: On<Pointer<Click>>,
    vis_cards: Query<&CorrespondingInventoryIndex>,
    active_player: Res<ActivePlayer>,
    inventories: Query<&PlayerCardInventory>,
    card_assets: Res<Assets<CardAsset>>,
    source: Res<CurrentSource>,
    mut commands: Commands,
) {
    let Ok(CorrespondingInventoryIndex(index)) = vis_cards.get(click.entity) else {
        return;
    };

    let Ok(log_inventory) = inventories.get(active_player.0) else {
        error!("The player had no inventory");
        return;
    };

    let Some(card_handle) = log_inventory.cards.get(*index) else {
        error!("A vis card pointed to an index that was out of the inventory's bounds");
        return;
    };

    let Some(card_data) = card_assets.get(card_handle.id()) else {
        warn!("A handle to the card asset failed to retrieve the asset.");
        return;
    };

    if let Some(ActionSource::Card { inventory_index }) = source.0
        && inventory_index == *index
    {
        commands.trigger(SetActionTo::None);
    } else {
        commands.trigger(SetActionTo::Action {
            action: card_data.action,
            source: ActionSource::Card {
                inventory_index: *index,
            },
        });
    }
}

fn update_selection(
    current_source: Res<CurrentSource>,
    mut vis_inventory_cards: Query<(&CorrespondingInventoryIndex, &mut BorderColor)>,
) {
    let Some(source) = current_source.0 else {
        for (_, mut color) in vis_inventory_cards.iter_mut() {
            color.set_all(Color::Srgba(tailwind::SLATE_700));
        }
        return;
    };

    match source {
        ActionSource::Card { inventory_index } => {
            for (vis_represents_index, mut color) in vis_inventory_cards.iter_mut() {
                if inventory_index == vis_represents_index.0 {
                    color.set_all(Color::WHITE);
                } else {
                    color.set_all(Color::Srgba(tailwind::SLATE_700));
                }
            }
        }
        _ => {
            for (_, mut color) in vis_inventory_cards.iter_mut() {
                color.set_all(Color::Srgba(tailwind::SLATE_700));
            }
        }
    }
}

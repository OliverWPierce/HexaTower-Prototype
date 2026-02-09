use bevy::{color::palettes::tailwind, picking::hover::Hovered, prelude::*};

use crate::{
    backend::{
        cards::{CardAsset, CardRarity},
        game_parameters::SetUpBoard,
        players::{ActivePlayer, StartTurn},
        shop::{
            CoinBag, PlayerShopLuckStats, PlayerShopSetsInfo, ShopDataChanged, TryPurchaseCard,
        },
    },
    frontend::{
        FrontEndSystems,
        in_game_ui::{
            LeftPanelEnt, create_panels,
            inspector::{InspectorPanel, inspect_card},
        },
    },
};

pub struct ShopVisualPlugin;

impl Plugin for ShopVisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, basic_shop_orgnanization.after(create_panels));
        app.add_systems(
            Update,
            (update_coin_count, update_shop_stats_node).in_set(FrontEndSystems),
        );
        app.add_systems(StartTurn, render_all_panels.in_set(FrontEndSystems));
        app.add_systems(ShopDataChanged, render_all_panels.in_set(FrontEndSystems));

        app.add_observer(attempt_purchase);
        app.add_observer(inspect_shop_card);
    }
}

#[derive(Debug, Clone, Copy, Component)]
struct CoinDisplay;

#[derive(Component)]
struct ShopPanelParent;

fn basic_shop_orgnanization(mut commands: Commands, left_panel: Res<LeftPanelEnt>) {
    commands.spawn((
        ChildOf(left_panel.0),
        Node {
            width: Val::Percent(95.0),
            min_height: Val::Px(40.0),
            height: Val::Percent(7.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Text::new("Card Shop"),
            TextLayout::new_with_justify(Justify::Center),
        )],
    ));

    let shop_info_bar = commands
        .spawn((
            ChildOf(left_panel.0),
            BackgroundColor(tailwind::SLATE_700.into()),
            Node {
                width: Val::Percent(95.0),
                min_height: Val::Px(30.0),
                height: Val::Percent(5.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        ChildOf(shop_info_bar),
        Node {
            width: Val::Percent(45.0),
            height: Val::Percent(95.0),
            border: UiRect::all(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(tailwind::EMERALD_700.into()),
        BorderColor::all(tailwind::EMERALD_900),
        children![(
            CoinDisplay,
            Text::new("$777"),
            TextFont {
                font_size: 24.0,
                ..default()
            }
        )],
    ));

    commands.spawn((
        ChildOf(shop_info_bar),
        Node {
            width: Val::Percent(45.0),
            height: Val::Percent(95.0),
            border: UiRect::all(Val::Px(2.0)),
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderRadius::all(Val::Px(2.0)),
        BorderColor::all(Color::Srgba(tailwind::SLATE_800)),
        children![
            (
                Text::new("1"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                ShopStatNodeMarker::Legendary,
                TextColor(CardRarity::Legendary.text_color())
            ),
            (
                Text::new("1"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                ShopStatNodeMarker::Epic,
                TextColor(CardRarity::Epic.text_color())
            ),
            (
                Text::new("1"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                ShopStatNodeMarker::Rare,
                TextColor(CardRarity::Rare.text_color())
            ),
            (
                Text::new("1"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                ShopStatNodeMarker::Common,
                TextColor(CardRarity::Common.text_color())
            ),
            (
                Text::new("1"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                ShopStatNodeMarker::All,
            )
        ],
    ));

    commands.spawn((
        ChildOf(left_panel.0),
        Node {
            width: Val::Percent(95.0),
            height: Val::Percent(85.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceAround,
            ..Default::default()
        },
        ShopPanelParent,
    ));
}

#[derive(Debug, Component)]
enum ShopStatNodeMarker {
    Legendary,
    Epic,
    Rare,
    Common,
    All,
}

fn update_shop_stats_node(
    active_player: Res<ActivePlayer>,
    player_luck: Query<&PlayerShopLuckStats>,
    displays: Query<(&mut Text, &ShopStatNodeMarker)>,
) {
    let Ok(luck_stats) = player_luck.get(active_player.0) else {
        warn!("The player had no shop luck stats");
        return;
    };

    for (mut display_count, data_represented) in displays {
        match data_represented {
            ShopStatNodeMarker::Legendary => {
                *display_count = Text::new(format!("{}", luck_stats.legendary))
            }
            ShopStatNodeMarker::Epic => *display_count = Text::new(format!("{}", luck_stats.epic)),
            ShopStatNodeMarker::Rare => *display_count = Text::new(format!("{}", luck_stats.rare)),
            ShopStatNodeMarker::Common => {
                *display_count = Text::new(format!("{}", luck_stats.common))
            }
            ShopStatNodeMarker::All => {
                let total_points =
                    luck_stats.legendary + luck_stats.epic + luck_stats.rare + luck_stats.common;
                *display_count = Text::new(format!("{}", total_points))
            }
        }
    }
}

fn update_coin_count(
    active_player: Res<ActivePlayer>,
    log_coins: Query<&CoinBag>,
    mut text: Single<&mut Text, With<CoinDisplay>>,
) {
    let Ok(coin_count) = log_coins.get(active_player.0) else {
        warn!("The player had no coin component");
        return;
    };

    text.0 = format!("${}", coin_count.coins);
}

#[derive(Debug, Component)]
struct RepresentsLogCardOffered {
    panel: usize,
    card_slot: usize,
}

fn render_all_panels(
    parent: Single<Entity, With<ShopPanelParent>>,
    active_player: Res<ActivePlayer>,
    player_shop_contents: Query<&PlayerShopSetsInfo>,
    card_assets: Res<Assets<CardAsset>>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
) {
    commands.entity(parent.entity()).despawn_children();

    let Ok(shop_contents) = player_shop_contents.get(active_player.0) else {
        warn!("The active player had no shop info when the shop rendered.");
        return;
    };

    for (set_index, possible_set) in shop_contents.shop_sets.iter().enumerate() {
        if let Some(set) = possible_set {
            let panel_ent = commands
                .spawn((
                    ChildOf(parent.entity()),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(23.0),
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceAround,
                        ..Default::default()
                    },
                    BackgroundColor(tailwind::SLATE_700.into()),
                ))
                .id();

            commands.spawn((
                ChildOf(panel_ent),
                Node {
                    width: Val::Percent(95.0),
                    height: Val::Px(30.0),
                    border: UiRect::bottom(Val::Px(2.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(tailwind::SLATE_500),
                children![
                    (
                        Text::new("Special"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        }
                    ),
                    (
                        Text::new(format!("refreshes in: {}", set.turns_until_auto_restock)),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        }
                    )
                ],
            ));

            let card_dock = commands
                .spawn((
                    ChildOf(panel_ent),
                    Node {
                        height: Val::Percent(80.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceEvenly,
                        align_content: AlignContent::SpaceAround,
                        ..default()
                    },
                ))
                .id();

            for (card_index, card_slot) in set.cards_offered.iter().enumerate() {
                let Some(card_handle) = card_slot else {
                    println!("The card slot was empty");
                    continue;
                };

                let Some(card_data) = card_assets.get(card_handle.id()) else {
                    warn!("A handle to the card asset failed to retrieve the asset.");
                    continue;
                };

                let card_vis = commands
                    .spawn((
                        ChildOf(card_dock),
                        RepresentsLogCardOffered {
                            panel: set_index,
                            card_slot: card_index,
                        },
                        Node {
                            height: Val::Percent(90.0),
                            aspect_ratio: Some(3.0 / 5.0),
                            flex_direction: FlexDirection::ColumnReverse,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::End,
                            ..Default::default()
                        },
                        ImageNode {
                            image: asset_server.load(card_data.image_path.clone()),
                            color: card_data.rarity.card_color(),
                            ..default()
                        },
                    ))
                    .id();

                commands.spawn((
                    ChildOf(card_vis),
                    Node {
                        width: Val::Percent(70.0),
                        height: Val::Px(30.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(tailwind::SLATE_700.into()),
                    BorderColor::all(tailwind::SLATE_900),
                    children![(
                        Text::new(format!("${}", card_data.price)),
                        TextFont {
                            font_size: 24.0,
                            ..Default::default()
                        },
                        TextColor(tailwind::EMERALD_500.into()),
                    )],
                ));
            }
        } else {
            //draw an empty rectangle where a panel could go.
        }
    }
}

fn attempt_purchase(
    click: On<Pointer<Click>>,
    cards: Query<&RepresentsLogCardOffered>,
    mut commands: Commands,
) {
    let Ok(log_card_cords) = cards.get(click.entity) else {
        return;
    };

    commands.trigger(TryPurchaseCard {
        slot: log_card_cords.card_slot,
        set: log_card_cords.panel,
    });
}
#[allow(clippy::too_many_arguments)]
fn inspect_shop_card(
    hover: On<Pointer<Over>>,
    vis_shop_cards: Query<&RepresentsLogCardOffered>,
    active_player: Res<ActivePlayer>,
    player_shop_contents: Query<&PlayerShopSetsInfo>,
    card_assets: Res<Assets<CardAsset>>,
    mut asset_server: ResMut<AssetServer>,
    mut commands: Commands,
    inspector: Single<Entity, With<InspectorPanel>>,
) {
    let Ok(log_card_cords) = vis_shop_cards.get(hover.entity) else {
        return;
    };

    let Ok(log_shop) = player_shop_contents.get(active_player.0) else {
        warn!("The player had no shop contents");
        return;
    };

    let Some(log_set) = &log_shop.shop_sets[log_card_cords.panel] else {
        warn!("A visual card in the shop pointed to a panel that doesn't exist.");
        return;
    };

    let Some(card_handle) = &log_set.cards_offered[log_card_cords.card_slot] else {
        warn!("A visual card in the shop pointed to a logical card that doesn't exist.");
        return;
    };

    let Some(card_data) = card_assets.get(card_handle.id()) else {
        warn!("A handle to the card asset failed to retrieve the asset.");
        return;
    };

    inspect_card(
        inspector.entity(),
        &mut commands,
        card_data,
        &mut asset_server,
        false,
    );
}

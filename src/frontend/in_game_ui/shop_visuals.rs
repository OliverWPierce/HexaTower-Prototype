use bevy::{
    color::palettes::{
        css::BLACK,
        tailwind::{self, SLATE_500, SLATE_900},
    },
    log::{self, tracing_subscriber::fmt::format},
    prelude::*,
};

use crate::{
    backend::{
        cards::CardAsset,
        game_parameters::SetUpBoard,
        players::{ActivePlayer, StartTurn},
        shop::{CoinBag, PlayerShopInfo},
    },
    frontend::{
        FrontEndSystems,
        in_game_ui::{LeftPanelEnt, create_panels},
    },
};

pub struct ShopVisualPlugin;

impl Plugin for ShopVisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, basic_shop_orgnanization.after(create_panels));
        app.add_systems(Update, update_coin_count.in_set(FrontEndSystems));
        app.add_systems(StartTurn, render_all_panels.in_set(FrontEndSystems));
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
            ..default()
        },
        BorderRadius::all(Val::Px(2.0)),
        BorderColor::all(Color::Srgba(tailwind::SLATE_800)),
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

fn render_all_panels(
    parent: Single<Entity, With<ShopPanelParent>>,
    active_player: Res<ActivePlayer>,
    player_shop_contents: Query<&PlayerShopInfo>,
    card_assets: Res<Assets<CardAsset>>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
) {
    commands.entity(parent.entity()).despawn_children();

    let Ok(shop_contents) = player_shop_contents.get(active_player.0) else {
        warn!("The active player had no shop info when the shop rendered.");
        return;
    };

    for possible_set in shop_contents.shop_sets.iter() {
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
                        max_width: Val::Percent(95.0),
                        justify_content: JustifyContent::SpaceAround,
                        ..default()
                    },
                ))
                .id();

            for (index, card_slot) in set.cards_offered.iter().enumerate() {
                // let Some(card_handle) = card_slot else {
                //     continue;
                // };

                // let Some(card_data) = card_assets.get(card_handle.id()) else {
                //     warn!("A handle to the card asset failed to retrieve the asset.");
                //     continue;
                // };

                commands.spawn((
                    ChildOf(card_dock),
                    Node {
                        height: Val::Percent(100.0),
                        width: Val::Percent(31.5),
                        flex_direction: FlexDirection::Column,
                        align_content: AlignContent::Center,
                        justify_content: JustifyContent::SpaceAround,
                        ..Default::default()
                    },
                    BackgroundColor(SLATE_900.into()),
                    // add children with the card and its price.
                ));
            }
        } else {
            //draw an empty rectangle where a panel could go.
        }
    }
}

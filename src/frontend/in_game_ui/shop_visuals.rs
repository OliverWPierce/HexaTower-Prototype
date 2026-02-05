use bevy::{color::palettes::tailwind, log, prelude::*};

use crate::{
    backend::{game_parameters::SetUpBoard, players::ActivePlayer, shop::CoinBag},
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
    }
}

#[derive(Debug, Clone, Copy, Component)]
struct CoinDisplay;

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

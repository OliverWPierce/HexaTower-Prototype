use bevy::{
    color::palettes::tailwind::{
        RED_300, RED_500, RED_950, SLATE_300, SLATE_600, SLATE_800, SLATE_900, SLATE_950,
    },
    prelude::*,
};

use crate::{
    backend::{
        game_actions::{ActionOrSelectionChanged, ActionSource, CurrentSource, SetActionTo},
        game_parameters::SetUpBoard,
        pieces::{ActiveLogPiece, OccupiesTile, Order, PieceOrders},
        tiles::TileType,
    },
    frontend::{
        FrontEndSystems,
        in_game_ui::{LowerPanelEnt, inventory::create_inventory_panel},
        inputs::ClickCounter,
    },
};

pub struct OrderDisplayPlugin;

impl Plugin for OrderDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            SetUpBoard,
            create_order_display_panel.after(create_inventory_panel),
        );

        app.add_systems(
            Update,
            update_panel.run_if(resource_changed::<ActiveLogPiece>),
        );

        app.add_observer(select_peice_order);
        app.add_observer(load_tile_order);

        app.add_systems(
            ActionOrSelectionChanged,
            (highlight_active_peice_order_icon, highlight_order_from_tile).in_set(FrontEndSystems),
        );
    }
}
#[derive(Debug, Component)]
struct OrderPanel;

fn create_order_display_panel(mut commands: Commands, lower_panel: Res<LowerPanelEnt>) {
    commands.spawn((
        ChildOf(lower_panel.0),
        Node {
            width: Val::Percent(34.0),
            height: Val::Percent(95.0),
            border: UiRect::all(Val::Px(3.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderRadius::all(Val::Px(15.0)),
        BackgroundColor(SLATE_600.into()),
        BorderColor::all(SLATE_800),
        OrderPanel,
    ));
}
#[derive(Debug, Component)]
pub struct ButtonForOrderAtIndex(pub usize);

#[derive(Debug, Component)]
struct TileOrderIcon;

fn update_panel(
    active_piece: Res<ActiveLogPiece>,
    mut commands: Commands,
    parent: Single<Entity, With<OrderPanel>>,
    pieces: Query<(&PieceOrders, &OccupiesTile)>,
    all_orders: Res<Assets<Order>>,
    tile_type: Query<&TileType>,
) -> Result<(), BevyError> {
    commands.entity(parent.entity()).despawn_children();

    if let Some(piece) = active_piece.0 {
        commands.spawn((
            ChildOf(parent.entity()),
            Node {
                width: Val::Percent(95.0),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(SLATE_950),
            children![(
                Text::new("Availible Orders"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                }
            )],
        ));

        let order_icon_panel = commands
            .spawn((
                ChildOf(parent.entity()),
                Node {
                    max_width: Val::Percent(95.0),
                    flex_grow: 1.0,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::SpaceAround,
                    align_content: AlignContent::SpaceAround,
                    align_items: AlignItems::Start,
                    ..default()
                },
            ))
            .id();

        let (orders, OccupiesTile { log_tile }) = pieces.get(piece)?;

        for (index, order) in orders.0.iter().enumerate() {
            if let Some(order_handle) = order {
                let Some(order_data) = all_orders.get(order_handle) else {
                    error!("An order handle was unable to retrieve that order's data.");
                    continue;
                };

                commands.spawn((
                    ChildOf(order_icon_panel),
                    Node {
                        aspect_ratio: Some(1.0),
                        max_height: Val::Percent(47.0),
                        min_height: Val::Percent(47.0),
                        border: UiRect::all(Val::Px(5.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    BorderColor::all(SLATE_900),
                    BackgroundColor(SLATE_800.into()),
                    BorderRadius::all(Val::Percent(100.0)),
                    ButtonForOrderAtIndex(index),
                    children![(
                        ImageNode {
                            image: order_data.icon.clone(),
                            image_mode: NodeImageMode::Stretch,
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        }
                    )],
                ));
            } else {
                commands.spawn((
                    ChildOf(order_icon_panel),
                    Node {
                        aspect_ratio: Some(1.0),
                        border: UiRect::all(Val::Px(5.0)),
                        max_height: Val::Percent(47.0),
                        min_height: Val::Percent(47.0),
                        ..Default::default()
                    },
                    BorderColor::all(SLATE_900),
                    BackgroundColor(SLATE_800.into()),
                    BorderRadius::all(Val::Percent(100.0)),
                ));
            }
        }

        match tile_type.get(*log_tile)? {
            TileType::Basic => (),
            TileType::PassiveGold => (),
            TileType::Portal => {
                commands.spawn((
                    ChildOf(order_icon_panel),
                    Node {
                        aspect_ratio: Some(1.0),
                        border: UiRect::all(Val::Px(5.0)),
                        max_height: Val::Percent(47.0),
                        min_height: Val::Percent(47.0),
                        ..Default::default()
                    },
                    BorderColor::all(RED_950),
                    BackgroundColor(RED_500.into()),
                    TileOrderIcon,
                ));
            }
        }
    } else {
        commands.spawn((
            ChildOf(parent.entity()),
            Text::new("Activate a piece to see its orders."),
            TextFont {
                font_size: 24.0,
                ..default()
            },
        ));
    }

    Ok(())
}

fn select_peice_order(
    click: On<Pointer<Click>>,
    mut declare_meaningful: ResMut<ClickCounter>,
    selectable_icons: Query<&ButtonForOrderAtIndex>,
    maybe_active: Res<ActiveLogPiece>,
    pieces: Query<&PieceOrders>,
    order_assets: Res<Assets<Order>>,
    mut commands: Commands,
) {
    let Ok(index) = selectable_icons.get(click.entity) else {
        return;
    };

    declare_meaningful.0 += 1;

    let Some(active_piece) = maybe_active.0 else {
        error!("The player selected an order while there was no active piece.");
        return;
    };

    let Ok(orders) = pieces.get(active_piece) else {
        error!("Active piece had no orders.");
        return;
    };

    let Some(order_handle) = &orders.0[index.0] else {
        return;
    };

    let Some(order_data) = order_assets.get(order_handle) else {
        return;
    };

    commands.trigger(SetActionTo::Action {
        action: order_data.action.clone(),
        source: crate::backend::game_actions::ActionSource::Order {
            index_in_piece_orders: index.0,
        },
    });
}

fn highlight_active_peice_order_icon(
    icons: Query<(&mut BorderColor, &ButtonForOrderAtIndex)>,
    action_source: Res<CurrentSource>,
) {
    let Some(ActionSource::Order {
        index_in_piece_orders,
    }) = action_source.0
    else {
        for (mut color, _) in icons {
            color.set_all(SLATE_900);
        }
        return;
    };

    for (mut color, index) in icons {
        if index.0 == index_in_piece_orders {
            color.set_all(SLATE_300);
        } else {
            color.set_all(SLATE_900);
        }
    }
}

fn highlight_order_from_tile(
    mut icon: Single<&mut BorderColor, With<TileOrderIcon>>,
    action_source: Res<CurrentSource>,
) {
    if let Some(source) = action_source.0
        && source == ActionSource::TileOrder
    {
        icon.set_all(RED_300);
    } else {
        icon.set_all(RED_950);
    };
}

fn load_tile_order(
    click: On<Pointer<Click>>,
    mut declare_meaningful: ResMut<ClickCounter>,
    icon: Single<Entity, With<TileOrderIcon>>,
    active_piece: Res<ActiveLogPiece>,
    peices: Query<&OccupiesTile>,
    tile_type: Query<&TileType>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    if click.entity != icon.entity() {
        return Ok(());
    }

    declare_meaningful.0 += 1;

    let action = tile_type
        .get(
            peices
                .get(active_piece.0.ok_or("Expected a peice to be active.")?)?
                .log_tile,
        )?
        .to_action()
        .ok_or("expected tile to have action")?;

    commands.trigger(SetActionTo::Action {
        action,
        source: ActionSource::TileOrder,
    });

    Ok(())
}

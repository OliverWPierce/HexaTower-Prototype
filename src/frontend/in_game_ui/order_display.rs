use bevy::{
    color::palettes::tailwind::{SLATE_300, SLATE_600, SLATE_800, SLATE_900, SLATE_950},
    prelude::*,
};

use crate::{
    backend::{
        game_parameters::SetUpBoard,
        pieces::{ActiveLogPiece, Order, PieceOrders},
    },
    frontend::in_game_ui::{LowerPanelEnt, inventory::create_inventory_panel},
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

fn update_panel(
    active_piece: Res<ActiveLogPiece>,
    mut commands: Commands,
    parent: Single<Entity, With<OrderPanel>>,
    pieces: Query<&PieceOrders>,
    all_orders: Res<Assets<Order>>,
) {
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
                Text::new("Availible Orders for Active Piece"),
                TextFont {
                    font_size: 24.0,
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

        let Ok(orders) = pieces.get(piece) else {
            error!("A piece had no orders.");
            return;
        };

        for order in orders.0.iter() {
            if let Some(order_handle) = order {
                let Some(order_data) = all_orders.get(order_handle) else {
                    error!("An order handle was unable to retrieve that order's data.");
                    return;
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
                    BorderColor::all(SLATE_300),
                    BackgroundColor(SLATE_800.into()),
                    BorderRadius::all(Val::Percent(100.0)),
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
}

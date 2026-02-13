use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use rand::seq::{IndexedRandom, IteratorRandom};

use crate::backend::{
    BackEndSystems,
    cards::{Card, InventoryUpdated, PlayerCardInventory, SortedCardHandles},
    game_parameters::SetUpBoard,
    players::{ActivePlayer, PlayerMarker, StartTurn, create_basic_players},
};

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            SetUpBoard,
            initialize_player_shop_data
                .in_set(BackEndSystems)
                .after(create_basic_players),
        );

        app.add_observer(change_coins);
        app.add_observer(manage_purchase_requests);

        app.add_systems(StartTurn, refresh_shop_on_new_turn.in_set(BackEndSystems));
    }
}
#[derive(Component, Debug, Clone)]
pub struct PlayerShopSetsInfo {
    pub shop_sets: [Option<ShopOfferSet>; 4],
}
#[derive(Debug, Default, Clone)]
pub struct ShopOfferSet {
    pub cards_offered: [Option<Handle<Card>>; 3],
    pub turns_until_auto_restock: u32,
    auto_restock_cooldown: u32,
}

impl ShopOfferSet {
    fn with_restock_data(
        mut self,
        initial_turns_to_restock: u32,
        ordinary_turns_to_restock: u32,
    ) -> Self {
        self.turns_until_auto_restock = initial_turns_to_restock;
        self.auto_restock_cooldown = ordinary_turns_to_restock;
        self
    }
}

#[derive(Debug, Component)]
pub struct PlayerShopLuckStats {
    pub legendary: u32,
    pub epic: u32,
    pub rare: u32,
    pub common: u32,
}

impl Default for PlayerShopLuckStats {
    fn default() -> Self {
        Self {
            legendary: 1,
            epic: 2,
            rare: 5,
            common: 10,
        }
    }
}

impl ShopOfferSet {
    fn restock(&mut self, sorted_cards: &Res<SortedCardHandles>, stats: &PlayerShopLuckStats) {
        let raffle_range = 0..(stats.common + stats.rare + stats.epic + stats.legendary);
        let mut rng = rand::rng();

        self.turns_until_auto_restock = self.auto_restock_cooldown;

        for slot in self.cards_offered.iter_mut() {
            let rolled_num = raffle_range
                .clone()
                .choose(&mut rng)
                .expect("The raffle range had no magnitude");

            if rolled_num < stats.common {
                *slot = if let Some(handle) = sorted_cards.common_cards.iter().choose(&mut rng) {
                    Some(handle.clone())
                } else {
                    warn!(
                        "A player was supposed to be offered a card with a specific rarity, but no cards of that rarity existed, so they were offered a random card instead."
                    );

                    Some(
                        sorted_cards
                            .all_cards
                            .choose(&mut rng)
                            .expect("There were no cards")
                            .clone(),
                    )
                };
            } else if rolled_num < (stats.rare + stats.common) {
                *slot = if let Some(handle) = sorted_cards.rare_cards.iter().choose(&mut rng) {
                    Some(handle.clone())
                } else {
                    warn!(
                        "A player was supposed to be offered a card with a specific rarity, but no cards of that rarity existed, so they were offered a random card instead."
                    );

                    Some(
                        sorted_cards
                            .all_cards
                            .choose(&mut rng)
                            .expect("There were no cards")
                            .clone(),
                    )
                };
            } else if rolled_num < (stats.epic + stats.rare + stats.common) {
                *slot = if let Some(handle) = sorted_cards.epic_cards.iter().choose(&mut rng) {
                    Some(handle.clone())
                } else {
                    warn!(
                        "A player was supposed to be offered a card with a specific rarity, but no cards of that rarity existed, so they were offered a random card instead."
                    );

                    Some(
                        sorted_cards
                            .all_cards
                            .choose(&mut rng)
                            .expect("There were no cards")
                            .clone(),
                    )
                };
            } else {
                *slot = if let Some(handle) = sorted_cards.legendary_cards.iter().choose(&mut rng) {
                    Some(handle.clone())
                } else {
                    warn!(
                        "A player was supposed to be offered a card with a specific rarity, but no cards of that rarity existed, so they were offered a random card instead."
                    );

                    Some(
                        sorted_cards
                            .all_cards
                            .choose(&mut rng)
                            .expect("There were no cards")
                            .clone(),
                    )
                };
            }
        }
    }
}

#[derive(Debug, Component)]
pub struct CoinBag {
    pub coins: i32,
}
#[derive(Debug, Event)]
pub struct ChangeActivePlayerCoinsBy(pub i32);

fn change_coins(
    change: On<ChangeActivePlayerCoinsBy>,
    active_player: Res<ActivePlayer>,
    mut coins: Query<&mut CoinBag>,
) {
    let Ok(mut player_coins) = coins.get_mut(active_player.0) else {
        warn!("The player had no coinbag.");
        return;
    };

    player_coins.coins += change.0;
}

fn initialize_player_shop_data(players: Query<Entity, With<PlayerMarker>>, mut commands: Commands) {
    for player in players {
        commands.entity(player).insert((
            CoinBag { coins: 35 },
            PlayerShopSetsInfo {
                shop_sets: [
                    Some(ShopOfferSet::default().with_restock_data(0, 1)),
                    Some(ShopOfferSet::default().with_restock_data(0, 1)),
                    Some(ShopOfferSet::default().with_restock_data(0, 1)),
                    Some(ShopOfferSet::default().with_restock_data(0, 1)),
                ],
            },
            PlayerShopLuckStats::default(),
        ));
    }
}

fn refresh_shop_on_new_turn(
    active_player: Res<ActivePlayer>,
    mut shop_data: Query<(&mut PlayerShopSetsInfo, &PlayerShopLuckStats)>,
    sorted_cards: Res<SortedCardHandles>,
) {
    let Ok((mut shop, stats)) = shop_data.get_mut(active_player.0) else {
        error!("The active player had no shop.");
        return;
    };

    for possible_offer_set in shop.shop_sets.iter_mut() {
        let Some(set) = possible_offer_set else {
            continue;
        };

        if set.turns_until_auto_restock > 0 {
            set.turns_until_auto_restock -= 1
        } else {
            set.restock(&sorted_cards, stats);
        }
    }
}
#[derive(Debug, Event)]
pub struct TryPurchaseCard {
    pub set: usize,
    pub slot: usize,
}

/// This schedule is used when shop related data has changed that does not change frequently.
#[derive(Debug, ScheduleLabel, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShopDataChanged;

fn manage_purchase_requests(
    request: On<TryPurchaseCard>,
    mut commands: Commands,
    mut players: Query<(
        &mut CoinBag,
        &mut PlayerShopSetsInfo,
        &mut PlayerCardInventory,
    )>,
    active_player: Res<ActivePlayer>,
    card_assets: Res<Assets<Card>>,
) {
    let Ok((mut coin_bag, mut shop_offers_info, mut player_inventory)) =
        players.get_mut(active_player.0)
    else {
        warn!("The active player had no associated shop data");
        return;
    };

    let Some(set) = &mut shop_offers_info.shop_sets[request.set] else {
        return;
    };

    let slot = &mut set.cards_offered[request.slot];

    let Some(card_handle) = slot else { return };

    let Some(card_data) = card_assets.get(card_handle.id()) else {
        warn!("a handle failed to get a card asset.");
        return;
    };

    if coin_bag.coins >= card_data.price as i32
        && player_inventory.add_card_succeeds(card_handle.clone())
    {
        coin_bag.coins -= card_data.price as i32;
        *slot = None;
        commands.run_schedule(ShopDataChanged);
        commands.run_schedule(InventoryUpdated);
    }
}

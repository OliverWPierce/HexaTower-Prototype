use bevy::prelude::*;
use rand::seq::{IndexedRandom, IteratorRandom};

use crate::backend::{
    BackEndSystems,
    cards::{CardAsset, SortedCardHandles},
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

        app.add_systems(StartTurn, refresh_shop_on_new_turn.in_set(BackEndSystems));
    }
}
#[derive(Component)]
pub struct PlayerShopInfo {
    pub shop_sets: [Option<ShopOfferSet>; 4],
}
#[derive(Debug, Default)]
pub struct ShopOfferSet {
    pub cards_offered: [Option<Handle<CardAsset>>; 3],
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
struct PlayerShopLuckStats {
    legendary: u32,
    epic: u32,
    rare: u32,
    common: u32,
}

impl Default for PlayerShopLuckStats {
    fn default() -> Self {
        Self {
            legendary: 1,
            epic: 2,
            rare: 3,
            common: 4,
        }
    }
}

impl ShopOfferSet {
    fn restock(&mut self, sorted_cards: &Res<SortedCardHandles>, stats: &PlayerShopLuckStats) {
        let raffle_range = 1..(stats.common + stats.rare + stats.epic + stats.legendary);
        let mut rng = rand::rng();

        self.turns_until_auto_restock = self.auto_restock_cooldown;

        for slot in self.cards_offered.iter_mut() {
            let rolled_num = raffle_range
                .clone()
                .choose(&mut rng)
                .expect("The raffle range had no magnitude");

            if rolled_num <= stats.common {
                // choose a common card
            } else if rolled_num <= (stats.rare + stats.common) {
                //choose a rare card
            } else if rolled_num <= (stats.epic + stats.rare + stats.common) {
                // choose an epic card
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
            PlayerShopInfo {
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
    mut shop_data: Query<(&mut PlayerShopInfo, &PlayerShopLuckStats)>,
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
            info!("Restocked a card set for the newly active player.")
        }
    }
}

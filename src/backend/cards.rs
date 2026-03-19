use bevy::{
    asset::{AssetLoader, LoadedFolder},
    ecs::schedule::ScheduleLabel,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::{
    BackEndSystems,
    game_actions::{CurrentSource, ExecuteSelectedAction, GameAction, ProxyAction},
    game_parameters::SetUpBoard,
    players::{ActivePlayer, PlayerMarker, create_basic_players},
};

use super::game_actions::ActionSource;

#[derive(Debug, Asset, Clone, TypePath)]
pub struct Card {
    pub name: String,
    pub price: u32,
    pub action: GameAction,
    pub image: Handle<Image>,
    pub rarity: CardRarity,
    pub description: String,
}
#[derive(Debug, Deserialize, Serialize, Reflect, Clone)]
struct ProxyCard {
    pub name: String,
    pub price: u32,
    pub action: ProxyAction,
    pub image_path: String,
    pub rarity: CardRarity,
    pub description: String,
}

#[derive(Debug, Reflect, Serialize, Deserialize, Clone, Copy)]
pub enum CardRarity {
    Legendary,
    Epic,
    Rare,
    Common,
    Never,
}

#[derive(Debug, Default, TypePath)]
pub struct CardAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CardAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for CardAssetLoader {
    type Asset = Card;
    type Settings = ();
    type Error = CardAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let proxy = ron::de::from_bytes::<ProxyCard>(&bytes)?;

        let image_handle: Handle<Image> = load_context.load(proxy.image_path);

        Ok(Card {
            name: proxy.name,
            price: proxy.price,
            action: GameAction::from_proxy(proxy.action, load_context),
            image: image_handle,
            rarity: proxy.rarity,
            description: proxy.description.clone(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["card.ron"]
    }
}

pub struct CardsPlugin;

impl Plugin for CardsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SortedCardHandles>();
        app.init_resource::<CardFolderAsset>();

        app.init_asset::<Card>();
        app.init_asset_loader::<CardAssetLoader>();

        app.add_systems(Startup, open_card_folder);
        app.add_systems(
            SetUpBoard,
            initialize_player_inventories
                .in_set(BackEndSystems)
                .after(create_basic_players),
        );
        app.add_systems(Update, validate_and_sort_newly_loaded_cards);

        app.add_systems(
            SetUpBoard,
            validate_and_sort_newly_loaded_cards.after(open_card_folder),
        );

        app.add_systems(ExecuteSelectedAction, consume_card.in_set(BackEndSystems));
    }
}

#[derive(Debug, Resource, Default)]
pub struct SortedCardHandles {
    pub all_cards: Vec<Handle<Card>>,
    pub legendary_cards: Vec<Handle<Card>>,
    pub epic_cards: Vec<Handle<Card>>,
    pub rare_cards: Vec<Handle<Card>>,
    pub common_cards: Vec<Handle<Card>>,
    pub never_shop_cards: Vec<Handle<Card>>,
}

#[derive(Resource, Debug, Default)]
struct CardFolderAsset(Option<Handle<LoadedFolder>>);

fn open_card_folder(
    asset_server: ResMut<AssetServer>,
    mut folder_resource: ResMut<CardFolderAsset>,
) {
    folder_resource.0 = Some(asset_server.load_folder("cards/card_parameters/"));
}

fn validate_and_sort_newly_loaded_cards(
    mut sorted_cards: ResMut<SortedCardHandles>,
    mut asset_events: MessageReader<AssetEvent<Card>>,
    mut cards: ResMut<Assets<Card>>,
) {
    for asset_event in asset_events.read() {
        match asset_event {
            AssetEvent::LoadedWithDependencies { id } => {
                let Some(card) = cards.get(*id) else {
                    continue;
                };

                if !card.action.is_valid_for_card() {
                    error!(
                        "The action associated with card {} is in contradiction with the code's capabilities.",
                        card.name
                    );
                    continue;
                }

                let card_rarity = card.rarity;

                if let Some(handle) = cards.get_strong_handle(*id) {
                    sorted_cards.all_cards.push(handle.clone());

                    match card_rarity {
                        CardRarity::Legendary => sorted_cards.legendary_cards.push(handle),
                        CardRarity::Epic => sorted_cards.epic_cards.push(handle),
                        CardRarity::Rare => sorted_cards.rare_cards.push(handle),
                        CardRarity::Common => sorted_cards.common_cards.push(handle),
                        CardRarity::Never => sorted_cards.never_shop_cards.push(handle),
                    }
                }
                // consider making it also remove the matching handle from the folder asset in order to save memory.
            }
            _ => continue,
        }
    }
}
#[derive(Debug, Component)]
pub struct PlayerCardInventory {
    pub max_size: usize,
    pub cards: Vec<Handle<Card>>,
}

impl PlayerCardInventory {
    pub fn add_card_succeeds(&mut self, card: Handle<Card>) -> bool {
        if self.cards.len() < self.max_size {
            self.cards.push(card);
            true
        } else {
            false
        }
    }
}

fn initialize_player_inventories(
    players: Query<Entity, With<PlayerMarker>>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    for player in players {
        commands.entity(player).insert(PlayerCardInventory {
            // code handles a max of 7 on mac laptop screen
            max_size: 4,
            cards: vec![
                asset_server.load("cards/card_parameters/spawn_tower.card.ron"),
                asset_server.load("cards/card_parameters/spawn_drill.card.ron"),
            ],
        });
    }
}
#[derive(Debug, ScheduleLabel, Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
pub struct InventoryUpdated;

fn consume_card(
    player: Res<ActivePlayer>,
    mut inventories: Query<&mut PlayerCardInventory>,
    action_source: Res<CurrentSource>,
    mut commands: Commands,
) {
    let Some(source) = action_source.0 else {
        warn!(
            "There was no action source when deciding whether to consume a card after action execution."
        );
        return;
    };

    let inventory_index = match source {
        ActionSource::Card { inventory_index } => inventory_index,
        _ => return,
    };

    let Ok(mut inventory) = inventories.get_mut(player.0) else {
        error!("The active player had no inventory.");
        return;
    };

    if inventory_index > inventory.cards.len() {
        error!(
            "The inventory index listed as sourcing the game action was out of bounds of the inventory length."
        );
        return;
    }

    inventory.cards.remove(inventory_index);
    commands.run_schedule(InventoryUpdated);
}

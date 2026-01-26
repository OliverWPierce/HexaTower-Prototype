use bevy::{
    asset::{AssetLoader, LoadedFolder},
    ecs::schedule::ScheduleLabel,
    prelude::*,
};
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    backend::{
        BackEndSystems,
        game_actions::ActionInfo,
        game_parameters::SetUpBoard,
        players::{ActivePlayer, PlayerMarker, create_basic_players},
    },
    frontend::FrontEndSystems,
};

#[derive(Debug, Asset, Reflect, Serialize, Deserialize, Clone)]
pub struct CardAsset {
    pub name: String,
    pub action: ActionInfo,
    pub image_path: String,
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
    type Asset = CardAsset;
    type Settings = ();
    type Error = CardAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let card = ron::de::from_bytes::<CardAsset>(&bytes)?;
        Ok(card)
    }

    fn extensions(&self) -> &[&str] {
        &["card.ron"]
    }
}

pub struct CardsPlugin;

impl Plugin for CardsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CardHandles>();
        app.init_resource::<CardFolderAsset>();

        app.init_asset::<CardAsset>();
        app.init_asset_loader::<CardAssetLoader>();

        app.add_systems(SetUpBoard, open_card_folder);
        app.add_systems(
            SetUpBoard,
            initialize_player_inventories
                .in_set(BackEndSystems)
                .after(create_basic_players),
        );
        app.add_systems(Update, validate_and_sort_newly_loaded_cards);

        //temporary testing systems
        app.add_systems(Update, tmp_add_card_to_inventory.in_set(FrontEndSystems));
    }
}

#[derive(Debug, Resource, Default)]
pub struct CardHandles(pub Vec<Handle<CardAsset>>);

#[derive(Resource, Debug, Default)]
struct CardFolderAsset(Option<Handle<LoadedFolder>>);

fn open_card_folder(
    asset_server: ResMut<AssetServer>,
    mut folder_resource: ResMut<CardFolderAsset>,
) {
    folder_resource.0 = Some(asset_server.load_folder("cards/card_parameters/"));
}

fn validate_and_sort_newly_loaded_cards(
    mut sorted_cards: ResMut<CardHandles>,
    mut asset_events: MessageReader<AssetEvent<CardAsset>>,
    mut cards: ResMut<Assets<CardAsset>>,
) {
    for asset_event in asset_events.read() {
        match asset_event {
            AssetEvent::LoadedWithDependencies { id } => {
                let Some(card) = cards.get(*id) else {
                    continue;
                };

                if !card.action.is_valid() {
                    error!(
                        "The action associated with card {} fails validation, as it is in contradiction with the code's capabilities.",
                        card.name
                    );
                    continue;
                }

                if let Some(handle) = cards.get_strong_handle(*id) {
                    sorted_cards.0.push(handle);
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
    pub cards: Vec<Handle<CardAsset>>,
}

impl PlayerCardInventory {
    pub fn add_card_succeeds(&mut self, card: Handle<CardAsset>) -> bool {
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
    let tower_spawn_card_handle: Handle<CardAsset> =
        asset_server.load("cards/card_parameters/tower_genesis.card.ron");

    for player in players {
        commands.entity(player).insert(PlayerCardInventory {
            max_size: 5,
            cards: vec![tower_spawn_card_handle.clone()],
        });
    }
}
#[derive(Debug, ScheduleLabel, Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
pub struct InventoryUpdated;

/// Later, simply change how this system is triggered.
fn tmp_add_card_to_inventory(
    inputs: Res<ButtonInput<KeyCode>>,
    all_cards: Res<CardHandles>,
    player: Res<ActivePlayer>,
    mut inventories: Query<&mut PlayerCardInventory>,
    mut commands: Commands,
) {
    if !inputs.just_pressed(KeyCode::KeyA) {
        return;
    }

    let Ok(mut inventory) = inventories.get_mut(player.0) else {
        error!("The active player had no inventory.");
        return;
    };

    let mut rng = rand::rng();

    if inventory.add_card_succeeds(
        all_cards
            .0
            .choose(&mut rng)
            .expect("There were no cards to choose from")
            .clone(),
    ) {
        commands.run_schedule(InventoryUpdated);
    } else {
        info!("The players inventory was full, so the card was not added.");
    }
}

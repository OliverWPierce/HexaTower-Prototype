use bevy::{
    asset::{AssetLoader, LoadedFolder},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::{cards, game_actions::ActionInfo, game_parameters::SetUpBoard};

#[derive(Debug, Asset, Reflect, Serialize, Deserialize)]
pub struct CardAsset {
    pub name: String,
    pub action: ActionInfo,
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
        settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
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
        app.add_systems(Update, validate_and_sort_newly_loaded_cards);
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

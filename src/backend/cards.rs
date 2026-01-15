use bevy::{asset::AssetLoader, prelude::*};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::game_actions::ActionInfo;

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
        &["card"]
    }
}

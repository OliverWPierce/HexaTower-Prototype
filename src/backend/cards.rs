use bevy::{asset::AssetLoader, prelude::*};

use crate::backend::game_actions::ActionInfo;

#[derive(Debug, Asset, Reflect)]
struct CardAsset {
    price: u32,
    action: ActionInfo,
    name: String,
    image_path: String,
}

#[derive(Debug, Component)]
struct BasicCardData(Handle<CardAsset>);

#[derive(Debug, Resource)]
struct Tmp_Inventory(Vec<Entity>);

fn data_to_ent(mut commands: Commands, card: Handle<CardAsset>) -> Entity {
    commands.spawn(BasicCardData(card.clone())).id()
}

impl AssetLoader for CardAsset {
    type Asset = CardAsset;

    type Settings = ();

    type Error = std::io::Error;

    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = std::result::Result<Self::Asset, Self::Error>>
    {
        todo!()
    }

    fn extensions(&self) -> &[&str] {
        &["card"]
    }
}

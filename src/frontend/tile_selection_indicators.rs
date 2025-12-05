use bevy::{prelude::*, time::Stopwatch};
use rand::{Rng, rngs::ThreadRng, seq::IndexedRandom};

use crate::{
    backend::{
        game_actions::EligibileTile, game_parameters::SetUpBoard, tiles::LogicalTileLocation,
    },
    frontend::FrontEndUpdateSystems,
};

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, initialize_indicator_model_handles);

        app.add_systems(
            Update,
            (modify_or_create_indicators_of_new_or_changed_tiles).in_set(FrontEndUpdateSystems),
        );
    }
}

#[derive(Debug, Resource, Clone)]
struct IndicatorHandles {
    selected: Handle<Scene>,
    unselected: Handle<Scene>,
}

fn initialize_indicator_model_handles(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(IndicatorHandles {
        selected: assets.load(GltfAssetLabel::Scene(0).from_asset("3d Selected Indicator.glb")),
        unselected: assets.load(GltfAssetLabel::Scene(0).from_asset("3d Selectable Indicator.glb")),
    });
}

#[derive(Debug, Clone, Copy, Component)]
struct IndicatorWatches(Entity);

#[derive(Debug, Clone, Copy, Component)]
enum ScaleMode {
    In,
    Out,
    Pop,
}

#[derive(Component, Clone)]
struct AnimationInstructions {
    t: Stopwatch,
    mode: ScaleMode,
    to_selected: bool,
    offset: i32,
}

fn modify_or_create_indicators_of_new_or_changed_tiles(
    eligible_tiles_that_changed: Query<
        (Entity, &EligibileTile, &LogicalTileLocation),
        Changed<EligibileTile>,
    >,
    indicators: Query<&IndicatorWatches>,
    mut commands: Commands,
    models: Res<IndicatorHandles>,
) {
    let offset_options = [1, 2, 3, 4, 5, 6];
    let mut rng = rand::rng();

    for (tile, EligibileTile { selected }, location) in eligible_tiles_that_changed {
        let mut indicator_already_existed = false;

        for tile_watched in indicators.iter() {
            if tile_watched.0 == tile {
                indicator_already_existed = true;
                break;
            }
        }

        if indicator_already_existed {
            commands.entity(tile).insert(AnimationInstructions {
                t: Stopwatch::new(),
                mode: ScaleMode::Pop,
                to_selected: *selected,
                offset: *offset_options.choose(&mut rng).unwrap(),
            });
        } else {
            commands.spawn((
                IndicatorWatches(tile),
                Transform::default().with_translation(Vec3 {
                    x: location.read().x,
                    y: 0.0,
                    z: location.read().y,
                }),
                SceneRoot(match selected {
                    true => models.selected.clone(),
                    false => models.unselected.clone(),
                }),
                AnimationInstructions {
                    t: Stopwatch::new(),
                    mode: ScaleMode::Pop,
                    to_selected: *selected,
                    offset: *offset_options.choose(&mut rng).unwrap(),
                },
            ));
        }
    }
}

fn update_animations(
    models: Res<IndicatorHandles>,
    to_update: Query<(&mut Transform, &mut AnimationInstructions)>,
    time: Res<Time>,
) {
    for (transform, mut instructions) in to_update {
        instructions.t.tick(time.delta());

        match instructions.mode {
            ScaleMode::In => todo!(),
            ScaleMode::Out => todo!(),
            ScaleMode::Pop => todo!(),
        }
    }
}

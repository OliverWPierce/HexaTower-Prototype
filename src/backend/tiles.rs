use bevy::prelude::*;
pub use hex_grid_tools::ADJACENTS;
use serde::{Deserialize, Serialize};

use crate::backend::BackEndSystems;
use crate::backend::game_parameters::{BoardSize, SetUpBoard};
use crate::backend::pieces::{OccupiedByPiece, OwnsLogPieces};
use crate::backend::players::{ActivePlayer, EndTurn};
use crate::backend::shop::CoinBag;
use crate::backend::tiles::hex_grid_tools::{GenerationMode, hex_cords};
pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LogicalTileCreated>();
        app.add_message::<LogTileDeleted>();
        app.add_message::<DeleteLogTileRequest>();
        app.init_resource::<ActiveTile>();

        app.add_systems(
            SetUpBoard,
            (spawn_tiles, find_adjacenents)
                .chain()
                .in_set(EssentialTileCreationSystems),
        );
        app.add_systems(Update, delete_tiles.in_set(BackEndSystems));

        app.add_systems(EndTurn, gold_tile_passive);
    }
}

#[derive(Debug, SystemSet, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct EssentialTileCreationSystems;

// tiles go counter clockwise, starting from two o'clock.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Component, Clone, Copy)]
pub struct AdjacentTiles(pub [Option<Entity>; 6]);

#[derive(Debug, Resource, PartialEq, Eq, PartialOrd, Ord, Default)]
struct ActiveTile(Option<Entity>);

#[derive(Component, Debug, Clone, Copy)]
pub struct LogicalTileLocation(Vec2);

impl From<LogicalTileLocation> for Vec3 {
    fn from(value: LogicalTileLocation) -> Self {
        Vec3::new(value.0.x, 0.0, value.0.y)
    }
}

impl LogicalTileLocation {
    pub fn read(&self) -> Vec2 {
        self.0
    }
}

#[derive(Debug, Event)]
struct BasicSpawningDone;

/// BUG CAUSER: If this message is sent after the set-up cycle runs, be sure to re-evaluate selections too.
/// Otherwise, there could be an edge case where the new tile is not considered for selection eligibility until the next evaluation.
#[derive(Debug, Message)]
pub struct LogicalTileCreated(pub Entity);

fn spawn_tiles(
    board_size: Res<BoardSize>,
    mut writer: MessageWriter<LogicalTileCreated>,
    mut commands: Commands,
) {
    for cords in hex_cords(GenerationMode::TrueHex {
        depth: match *board_size {
            BoardSize::Small => 4,
            BoardSize::Medium => 5,
            BoardSize::Large => 6,
            BoardSize::ExtraLarge => 10,
        },
    })
    .iter()
    {
        let ent = commands
            .spawn((
                LogicalTileLocation(*cords),
                AdjacentTiles([None, None, None, None, None, None]),
                TileType::Basic,
            ))
            .id();

        writer.write(LogicalTileCreated(ent));
    }
    commands.trigger(BasicSpawningDone);
}

fn find_adjacenents(mut tiles: Query<(&LogicalTileLocation, &mut AdjacentTiles, Entity)>) {
    let mut combinations = tiles.iter_combinations_mut();

    while let Some([(t1_pos, mut t1_adj, t1), (t2_pos, mut t2_adj, t2)]) = combinations.fetch_next()
    {
        if Vec2::distance_squared(t1_pos.0, t2_pos.0) < 4.0001 {
            let v1 = (t1_pos.0 - t2_pos.0).normalize();

            for (id, dir) in ADJACENTS.iter().enumerate() {
                if dir.normalize().dot(v1) >= 0.9 {
                    t1_adj.0[id] = Some(t2);
                }
            }

            let v2 = (t2_pos.0 - t1_pos.0).normalize();

            for (id, dir) in ADJACENTS.iter().enumerate() {
                if dir.normalize().dot(v2) >= 0.9 {
                    t2_adj.0[id] = Some(t1);
                }
            }
        }
    }
}

#[derive(Debug, Message, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DeleteLogTileRequest(pub Entity);

#[derive(Debug, Message, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct LogTileDeleted(pub Entity);

fn delete_tiles(
    mut requests: MessageReader<DeleteLogTileRequest>,
    mut log_tiles: Query<(Entity, &mut AdjacentTiles)>,
    mut commands: Commands,
    mut deleted_writer: MessageWriter<LogTileDeleted>,
) {
    for request in requests.read() {
        if let Ok((entity_to_delete, adjacencies_of_tile_to_delete_ref)) = log_tiles.get(request.0)
        {
            let adjacencies_of_tile_to_delete = *adjacencies_of_tile_to_delete_ref;
            for (index, adjacent_tile) in adjacencies_of_tile_to_delete
                .0
                .iter()
                .enumerate()
                .filter(|(_, option_of_adjacent)| option_of_adjacent.is_some())
            {
                log_tiles
                    .get_mut(adjacent_tile.unwrap())
                    .expect("A tile had an adjacent entity that was not a tile.")
                    .1
                    .0[(index + 3) % 6] = None
            }
            deleted_writer.write(LogTileDeleted(entity_to_delete));
            commands.entity(entity_to_delete).despawn();
        } else {
            warn!(
                "A tile deletion request was made for an entity that had no adjacent tiles and therefore was not a tile."
            )
        }
    }
}

pub mod hex_grid_tools {
    use bevy::prelude::*;
    pub(super) const SQRT3: f32 = 1.7320508;

    pub enum GenerationMode {
        TrueHex { depth: u32 },
    }

    // this function acts as a kind of "routing" function that allows an interface for other files to use.
    // note that it assumes hexagons have a radius of 1.0.
    pub fn hex_cords(mode: GenerationMode) -> Vec<Vec2> {
        match mode {
            GenerationMode::TrueHex { depth } => true_hex_generation(depth),
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    struct ExaminedLocation {
        iteration: u32,
        cords: Vec2,
    }

    pub const ADJACENTS: [Vec2; 6] = [
        vec2(1.5, 0.5 * SQRT3),
        vec2(0.0, SQRT3),
        vec2(-1.5, 0.5 * SQRT3),
        vec2(-1.5, -0.5 * SQRT3),
        vec2(0.0, -SQRT3),
        vec2(1.5, -0.5 * SQRT3),
    ];

    fn true_hex_generation(depth: u32) -> Vec<Vec2> {
        let mut to_examine = vec![ExaminedLocation {
            iteration: 1,
            cords: vec2(0.0, 0.0), // this is the root hexagon
        }];
        let mut generated = vec![vec2(0.0, 0.0)];

        for iteration in 1..depth {
            to_examine.retain(|&x| iteration - 1 <= x.iteration);

            for examined in to_examine.clone() {
                for pre_computed in &ADJACENTS {
                    let new_vec = { pre_computed + examined.cords };
                    let mut is_in_list = false;
                    for already_generated in generated.iter() {
                        if (already_generated - new_vec).abs().element_sum() <= 0.00001 {
                            is_in_list = true;
                            break;
                        }
                    }
                    if !is_in_list {
                        generated.push(new_vec);
                        to_examine.push(ExaminedLocation {
                            iteration,
                            cords: new_vec,
                        });
                    }
                }
            }
        }

        generated
    }
}

#[derive(
    Component, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Deserialize, Serialize, Reflect,
)]
pub enum TileType {
    Basic,
    PassiveGold,
    Portal,
}

fn gold_tile_passive(
    tiles: Query<(&OccupiedByPiece, &TileType)>,
    active_player: Res<ActivePlayer>,
    mut player_data: Query<(&OwnsLogPieces, &mut CoinBag)>,
) -> Result<(), BevyError> {
    let player_peices = player_data.get(active_player.0)?.0.list();

    player_data.get_mut(active_player.0)?.1.coins += tiles
        .iter()
        .filter(|(piece, tile_type)| {
            **tile_type == TileType::PassiveGold && player_peices.contains(&piece.log_piece())
        })
        .count() as i32;

    Ok(())
}

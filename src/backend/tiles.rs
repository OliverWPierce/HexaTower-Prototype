use bevy::prelude::*;
use hex_grid_tools::ADJACENTS;

use crate::backend::game_parameters::{BoardSize, SetUpBoard};
use crate::backend::tiles::hex_grid_tools::{GenerationMode, hex_cords};
pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LogicalTileCreated>();
        app.init_resource::<ActiveTile>();

        app.add_systems(SetUpBoard, (spawn_tiles, find_adjacenents).chain());
    }
}

// tiles go counter clockwise, starting from two o'clock.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Component)]
pub struct AdjacentTiles([Option<Entity>; 6]);

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Component)]
pub struct ValidMove;

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

#[derive(Debug, Event)]
pub struct LogicalTileCreated(pub Entity);

fn spawn_tiles(
    board_size: Res<BoardSize>,
    mut writer: EventWriter<LogicalTileCreated>,
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

mod hex_grid_tools {
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

    pub(super) const ADJACENTS: [Vec2; 6] = [
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

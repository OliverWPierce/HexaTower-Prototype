use std::process::id;

use bevy::{ecs::relationship, prelude::*, transform::commands};
use hex_grid_tools::ADJACENTS;
use rand::{rng, seq::IteratorRandom};

use crate::backend::{
    AppState, BoardSize, VisualOf,
    tiles::hex_grid_tools::{GenerationMode, hex_cords},
};

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (test_move, random_active).run_if(in_state(AppState::InGame)),
        );
        app.add_event::<BasicSpawningDone>();
        app.add_event::<TileReadyForVisual>();
        app.add_observer(spawn_tiles);
        app.add_observer(find_adjacenents);

        app.add_systems(OnEnter(AppState::InGame), create_visual_entities);
    }
}

// determines if the tile can be used for player movement.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Component)]
struct Blocked;

// tiles go counter clockwise, starting from two o'clock.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Component)]
pub struct AdjacentTiles([Option<Entity>; 6]);

impl AdjacentTiles {
    pub fn get_ent(&self, direction: usize) -> Option<Entity> {
        if direction >= 6 {
            panic!("A tile id was asked for that was not a number within 0-5.")
        } else {
            self.0[direction]
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Component)]
pub struct ValidMove;

#[derive(Debug, Resource, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActiveTile(Entity);

impl ActiveTile {
    pub fn read(&self) -> Entity {
        self.0
    }

    pub fn set(&mut self, new_ent: Entity) {
        self.0 = new_ent;
        println!("set the entity {new_ent} to be the active tile.");
    }
}

// This should highlight the four tiles north of the active one.
fn test_move(tiles: Query<&AdjacentTiles>, active: Res<ActiveTile>, mut commands: Commands) {
    if let Ok(adjacencies) = tiles.get(active.0) {
        for entity in adjacencies.0.into_iter().flatten() {
            commands.entity(entity).insert(ValidMove);
        }
    }
}

#[derive(Component, Debug)]
struct RootTile;

#[derive(Component, Debug, Clone, Copy)]
pub struct TrueTileLocation(Vec2);

impl From<TrueTileLocation> for Vec3 {
    fn from(value: TrueTileLocation) -> Self {
        Vec3::new(value.0.x, 0.0, value.0.y)
    }
}

impl TrueTileLocation {
    pub fn read(&self) -> Vec2 {
        self.0
    }
}

#[derive(Debug, Event)]
struct BasicSpawningDone;

fn spawn_tiles(trigger: Trigger<BoardSize>, mut commands: Commands) {
    let mut is_root = true;

    for cords in hex_cords(GenerationMode::TrueHex {
        depth: match *trigger {
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
                TrueTileLocation(*cords),
                AdjacentTiles([None, None, None, None, None, None]),
            ))
            .id();

        if is_root {
            commands.entity(ent).insert(RootTile);
            commands.insert_resource(ActiveTile(ent));
            is_root = false;
            println!("root is at {cords}");
        }
    }
    commands.trigger(BasicSpawningDone);
}

fn find_adjacenents(
    trigger: Trigger<BasicSpawningDone>,
    mut tiles: Query<(&TrueTileLocation, &mut AdjacentTiles, Entity)>,
) {
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

fn random_active(
    inputs: Res<ButtonInput<KeyCode>>,
    mut active: ResMut<ActiveTile>,
    tiles: Query<Entity, With<AdjacentTiles>>,
) {
    if inputs.just_pressed(KeyCode::Space) {
        let new = tiles
            .iter()
            .choose(&mut rand::rng())
            .expect("no tiles existed.");

        active.set(new);
    }
}

#[derive(Debug, Event)]
pub struct TileReadyForVisual(pub Entity);

fn create_visual_entities(
    mut commands: Commands,
    tiles: Query<Entity, With<TrueTileLocation>>,
    mut writer: EventWriter<TileReadyForVisual>,
) {
    let mut visuals: Vec<TileReadyForVisual> = Vec::new();

    for entity in tiles {
        visuals.push(TileReadyForVisual((commands.spawn(VisualOf(entity))).id()));
    }

    writer.write_batch(visuals);
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

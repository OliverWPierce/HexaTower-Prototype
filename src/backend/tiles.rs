use std::process::id;

use bevy::prelude::*;

use crate::backend::{
    BoardSize,
    tiles::HexGridTools::{GenerationMode, hex_cords},
};

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            test_move.run_if(resource_exists_and_changed::<ActiveTile>),
        );
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
struct Highlight;

#[derive(Debug, Resource, PartialEq, Eq, PartialOrd, Ord)]
struct ActiveTile(Entity);

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
    if let Ok(adjacentcies) = tiles.get(active.read()) {
        if let Some(ent) = adjacentcies.get_ent(0) {
            commands.entity(ent).insert(Highlight);
            println!("highlighted {ent}");
            if let Ok(adjacentcies) = tiles.get(ent) {
                if let Some(ent) = adjacentcies.get_ent(0) {
                    commands.entity(ent).insert(Highlight);
                    println!("highlighted {ent}");
                    if let Ok(adjacentcies) = tiles.get(ent) {
                        if let Some(ent) = adjacentcies.get_ent(0) {
                            commands.entity(ent).insert(Highlight);
                            println!("highlighted {ent}");
                            if let Ok(adjacentcies) = tiles.get(ent) {
                                if let Some(ent) = adjacentcies.get_ent(0) {
                                    commands.entity(ent).insert(Highlight);
                                    println!("highlighted {ent}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
#[derive(Component, Debug)]
struct RootTile;

#[derive(Component, Debug)]
struct TrueTileLocation(Vec2);

fn spawn_tiles(trigger: Trigger<BoardSize>, mut commands: Commands) {
    let mut spawned_tiles: Vec<Entity> = Vec::new();

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
        spawned_tiles.push(commands.spawn(TrueTileLocation(*cords)).id());
    }
}

mod HexGridTools {
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
            cords: vec2(0.0, 0.0),
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

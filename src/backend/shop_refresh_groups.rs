use bevy::prelude::*;

use crate::backend::PlayerCreated;

pub struct RefreshGroupPlugin;

impl Plugin for RefreshGroupPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_default_refresh_groups_to_player);
    }
}

#[derive(Debug, Component, Default)]
struct RefreshTimer(u8);

#[derive(Debug, Component)]
struct RefreshProperties {
    max_offer_count: u8,
    duration: u8,
    group_type: GroupType,
}

#[derive(Debug)]
enum GroupType {
    Standard,
    Class,
}

impl Default for RefreshProperties {
    fn default() -> Self {
        Self {
            max_offer_count: 3,
            duration: 1,
            group_type: GroupType::Standard,
        }
    }
}

#[derive(Component)]
#[relationship(relationship_target = RefreshGroups)]
pub struct RefreshGroupOf(pub Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship = RefreshGroupOf)]
pub struct RefreshGroups(Vec<Entity>);

impl RefreshGroups {
    pub fn groups(&self) -> Vec<Entity> {
        self.0.clone()
    }
}

fn add_default_refresh_groups_to_player(trigger: Trigger<PlayerCreated>, mut commands: Commands) {
    let entity = trigger.0;

    // first group
    commands.spawn((
        RefreshProperties {
            max_offer_count: 3,
            duration: 1,
            group_type: GroupType::Standard,
        },
        RefreshGroupOf(entity),
        RefreshTimer::default(),
    ));

    // second group
    commands.spawn((
        RefreshProperties {
            max_offer_count: 3,
            duration: 3,
            group_type: GroupType::Standard,
        },
        RefreshGroupOf(entity),
        RefreshTimer::default(),
    ));

    // third group
    commands.spawn((
        RefreshProperties {
            max_offer_count: 1,
            duration: 4,
            group_type: GroupType::Standard,
        },
        RefreshGroupOf(entity),
        RefreshTimer::default(),
    ));
}

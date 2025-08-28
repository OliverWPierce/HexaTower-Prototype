use bevy::prelude::*;

use crate::backend::PlayerAdded;

pub struct RefreshGroupPlugin;

impl Plugin for RefreshGroupPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(add_refresh_groups_to_player);
    }
}

#[derive(Debug, Component)]
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

#[derive(Component)]
#[relationship_target(relationship = RefreshGroupOf)]
pub struct RefreshGroups(Vec<Entity>);

fn add_refresh_groups_to_player(trigger: Trigger<PlayerAdded>, mut commands: Commands) {
    let entity = trigger.0;

    // first group
    commands.spawn((
        RefreshProperties {
            max_offer_count: 3,
            duration: 1,
            group_type: GroupType::Standard,
        },
        RefreshGroupOf(entity),
        RefreshTimer(1),
    ));
}

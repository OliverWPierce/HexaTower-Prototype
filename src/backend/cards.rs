use bevy::prelude::*;

#[derive(Component)]
#[relationship(relationship_target = GroupCards)]
struct CardOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = CardOf)]
struct GroupCards(Vec<Entity>);

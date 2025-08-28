use bevy::prelude::*;

pub struct PlayerSpawningPlugin;

impl Plugin for PlayerSpawningPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerAdded>();
        app.add_systems(Startup, spawn_players);
    }
}

const PLAYER_COUNT: u8 = 2;

#[derive(Debug, Component)]
pub struct PlayerID(u8);

impl PlayerID {
    pub fn id(&self) -> u8 {
        self.0
    }
}
#[derive(Debug, Component)]
struct CoinBag {
    coins: i32,
    minimum: i32,
}

impl CoinBag {
    pub fn balance(&self) -> i32 {
        self.coins
    }

    pub fn try_purchase(&mut self, coins_added: i32) -> bool {
        if self.coins + coins_added >= self.minimum {
            self.coins += coins_added;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Event)]
pub struct PlayerAdded(pub Entity);

fn spawn_players(mut commands: Commands) {
    for id in 1..PLAYER_COUNT {
        let entity = commands.spawn(PlayerID(id)).id();
        commands.trigger(PlayerAdded(entity));
    }
}

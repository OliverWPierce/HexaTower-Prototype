use bevy::{color::palettes::tailwind, prelude::*, ui::update};

use crate::backend::AppState;

pub struct InGameUiPlugin;

impl Plugin for InGameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_test);
        app.add_systems(Update, move_target);
    }
}

#[derive(Component)]
struct Target;

fn spawn_test(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        BackgroundColor(Color::BLACK),
        children![(
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(50.0),
                ..Default::default()
            },
            BackgroundColor(tailwind::ROSE_700.into()),
            children![(
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(50.0),
                    ..Default::default()
                },
                BackgroundColor(tailwind::BLUE_600.into()),
                Target,
                Transform::default(),
            )]
        )],
    ));
}

fn move_target(target: Single<&mut Node, With<Target>>, time: Res<Time>) {
    target.into_inner().left = Val::Percent(
        match target.clone().left {
            Val::Px(y) => y,
            _ => panic!("test failed"),
        } + time.delta_secs() * 10.0,
    );
}

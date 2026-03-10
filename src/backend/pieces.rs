use bevy::prelude::*;

use bevy::asset::AssetLoader;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::{
    BackEndSystems,
    game_actions::{
        ExecuteSelectedAction, GameAction, ProxyAction, SetActionTo, execute_action_functionality,
    },
    players::{CheckForWinner, StartTurn},
    shop::ChangePlayerCoinsBy,
    tiles::AdjacentTiles,
};

pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnLogPiece>();
        app.add_message::<SpawnedLogPieceInfo>();
        app.add_message::<LogPieceDespawned>();

        app.init_asset::<Piece>();
        app.init_asset::<Order>();
        app.init_asset_loader::<PieceAssetLoader>();
        app.init_asset_loader::<OrderAssetLoader>();

        app.init_resource::<ActiveLogPiece>();

        app.add_systems(
            Update,
            (spawn_logpiece, send_despawn_notifications).in_set(BackEndSystems),
        );

        app.add_systems(
            ExecuteSelectedAction,
            (damage_piece, clear_active).after(execute_action_functionality),
        );

        app.add_systems(StartTurn, clear_active.in_set(BackEndSystems));

        app.add_observer(set_active_piece);
        app.add_message::<DamagePiece>();
        app.add_message::<PieceMoved>();
        app.add_observer(rotate_piece);
        app.add_observer(move_piece);
    }
}

#[derive(Debug, Serialize, Reflect, Deserialize)]
struct ProxyPiece {
    name: String,
    model_path: String,
    health: u32,
    default_coin_value: u32,
    is_win_condition: bool,
    should_give_player_extra_command: bool,

    // It would be simpler to store these as a vector, but having separate feilds is clearer to modders and prevents inncorrect situations, since the code is only designed to handle five orders per peice.
    order1_path: Option<String>,
    order2_path: Option<String>,
    order3_path: Option<String>,
    order4_path: Option<String>,
    order5_path: Option<String>,
}

#[derive(Debug, Component)]
pub struct PieceOrders(pub [Option<Handle<Order>>; 5]);

#[derive(Asset, Debug, TypePath, Clone)]
pub struct Order {
    pub name: String,
    pub action: GameAction,
    pub description: String,
    pub icon: Handle<Image>,
}
#[derive(Debug, Deserialize, Reflect, Serialize)]
struct ProxyOrder {
    name: String,
    proxy_action: ProxyAction,
    description: String,
    icon_path: String,
}

#[derive(Debug, Default, TypePath)]
pub struct OrderAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum OrderAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for OrderAssetLoader {
    type Asset = Order;

    type Settings = ();

    type Error = OrderAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let proxy = ron::de::from_bytes::<ProxyOrder>(&bytes)?;

        let order = Order {
            name: proxy.name,
            action: GameAction::from_proxy(proxy.proxy_action, load_context),
            description: proxy.description.clone(),
            icon: load_context.load(proxy.icon_path),
        };

        Ok(order)
    }

    fn extensions(&self) -> &[&str] {
        &["order.ron"]
    }
}

#[derive(Debug, Asset, Clone, TypePath)]
pub struct Piece {
    pub name: String,
    pub model: Handle<Scene>,
    pub health: u32,
    pub default_coin_value: u32,
    pub default_orders: [Option<Handle<Order>>; 5],
    pub is_win_condtion: bool,
    pub should_give_player_extra_command: bool,
}

#[derive(Debug, Default, TypePath)]
pub struct PieceAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PieceAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for PieceAssetLoader {
    type Asset = Piece;
    type Settings = ();
    type Error = PieceAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let proxy = ron::de::from_bytes::<ProxyPiece>(&bytes)?;

        let piece = Piece {
            name: proxy.name.clone(),
            model: load_context.load(GltfAssetLabel::Scene(0).from_asset(proxy.model_path)),
            health: proxy.health,
            is_win_condtion: proxy.is_win_condition,
            default_coin_value: proxy.default_coin_value,
            should_give_player_extra_command: proxy.should_give_player_extra_command,
            default_orders: [
                proxy.order1_path.map(|path| load_context.load(path)),
                proxy.order2_path.map(|path| load_context.load(path)),
                proxy.order3_path.map(|path| load_context.load(path)),
                proxy.order4_path.map(|path| load_context.load(path)),
                proxy.order5_path.map(|path| load_context.load(path)),
            ],
        };

        Ok(piece)
    }

    fn extensions(&self) -> &[&str] {
        &["piece.ron"]
    }
}

#[derive(Debug, Message, Clone, PartialEq, PartialOrd)]
pub struct SpawnLogPiece {
    pub piece: Handle<Piece>,
    pub player: Entity,
    pub log_tile: Entity,
}

#[derive(Debug, Message, Clone, PartialEq, PartialOrd)]
pub struct SpawnedLogPieceInfo {
    pub log_piece_entity: Entity,
    pub from_asset: Handle<Piece>,
}

#[derive(Clone, Component, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[relationship(relationship_target = OccupiedByPiece)]
pub struct OccupiesTile {
    pub log_tile: Entity,
}

#[derive(Clone, Component, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[relationship_target(relationship = OccupiesTile)]
pub struct OccupiedByPiece {
    log_piece: Entity,
}

impl OccupiedByPiece {
    pub fn log_piece(&self) -> Entity {
        self.log_piece
    }
}

#[derive(Component)]
#[relationship_target(relationship = LogPieceOwnedByPlayer, linked_spawn)]
pub struct OwnsLogPieces(Vec<Entity>);

impl OwnsLogPieces {
    pub fn list(&self) -> &Vec<Entity> {
        &self.0
    }
}

#[derive(Component)]
#[relationship(relationship_target = OwnsLogPieces)]
pub struct LogPieceOwnedByPlayer(pub Entity);

#[derive(Debug, Component)]
pub struct Health {
    pub max_health: u32,
    pub current_health: u32,
}

#[derive(Debug, Component)]
pub struct WinCondition;

#[derive(Debug, Component)]
struct MonataryValue(u32);

#[derive(Debug, Component)]
pub struct FacingDirection(pub u8);

impl FacingDirection {
    pub fn offset_index(&self, offset: i8) -> u8 {
        let current_index = self.0 as i8;

        let corrected_offset = offset.abs() % 6;

        (((current_index + 6) + corrected_offset * offset.signum()) % 6) as u8
    }
}

#[derive(Debug, Component)]
pub struct OrdersPerTurn {
    pub max: u32,
    pub current: u32,
}
#[derive(Debug, Component)]
pub struct CommandPoint;

fn spawn_logpiece(
    mut spawn_requests: MessageReader<SpawnLogPiece>,
    mut commands: Commands,
    occupied_tiles: Query<Entity, With<OccupiedByPiece>>,
    mut notify_of_spawns: MessageWriter<SpawnedLogPieceInfo>,
    pieces: Res<Assets<Piece>>,
) {
    for SpawnLogPiece {
        piece,
        player,
        log_tile,
    } in spawn_requests.read()
    {
        if occupied_tiles.contains(*log_tile) {
            warn!(
                "A request was sent to spawn a piece on a tile that was already occupied. The request was not fulfilled."
            )
        } else {
            let Some(piece_instructions) = pieces.get(piece) else {
                error!("a piece asset was not yet fully loaded.");
                continue;
            };

            let logpiece_ent = commands
                .spawn((
                    LogPieceOwnedByPlayer(*player),
                    OrdersPerTurn { max: 1, current: 1 },
                    OccupiesTile {
                        log_tile: *log_tile,
                    },
                    FacingDirection(0),
                    Health {
                        max_health: piece_instructions.health,
                        current_health: piece_instructions.health,
                    },
                    PieceOrders(piece_instructions.default_orders.clone()),
                    MonataryValue(piece_instructions.default_coin_value),
                ))
                .id();

            if piece_instructions.is_win_condtion {
                commands.entity(logpiece_ent).insert(WinCondition);
            }

            if piece_instructions.should_give_player_extra_command {
                commands.entity(logpiece_ent).insert(CommandPoint);
            }

            notify_of_spawns.write(SpawnedLogPieceInfo {
                log_piece_entity: logpiece_ent,
                from_asset: piece.clone(),
            });
        }
    }
}

#[derive(Debug, Message)]
pub struct LogPieceDespawned(pub Entity);

fn send_despawn_notifications(
    mut despawned_logical_pieces: RemovedComponents<OccupiesTile>,
    mut despawns: MessageWriter<LogPieceDespawned>,
) {
    for logical_piece in despawned_logical_pieces.read() {
        despawns.write(LogPieceDespawned(logical_piece));
    }
}

#[derive(Debug, Resource, Default)]
pub struct ActiveLogPiece(pub Option<Entity>);

#[derive(Event)]
pub struct SetPieceToActive(pub Option<Entity>);

fn clear_active(mut commands: Commands) {
    commands.trigger(SetPieceToActive(None));
}

fn set_active_piece(
    new_piece: On<SetPieceToActive>,
    mut current_piece: ResMut<ActiveLogPiece>,
    mut commands: Commands,
) {
    if new_piece.0.is_none() {
        current_piece.0 = None;
        return;
    }

    if let Some(piece) = current_piece.0
        && new_piece.0.unwrap() == piece
    {
        current_piece.0 = None;
    } else {
        current_piece.0 = new_piece.0;
    }

    commands.trigger(SetActionTo::None);
}

#[derive(Debug, Message)]
pub struct DamagePiece {
    pub log_piece: Entity,
    pub method: DamageType,
    pub source_player: Option<Entity>,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Reflect, Deserialize)]
pub enum DamageType {
    Constant(u32),
    FractionOfMissing(f32),
    FractionOfMax(f32),
}

fn damage_piece(
    mut reader: MessageReader<DamagePiece>,
    mut pieces: Query<(&mut Health, Has<WinCondition>, &MonataryValue)>,
    mut commands: Commands,
) {
    for DamagePiece {
        log_piece,
        method,
        source_player,
    } in reader.read()
    {
        let Ok((mut health, win_condition, value)) = pieces.get_mut(*log_piece) else {
            error!("Received a message to damage an entity which was not a piece");
            return;
        };

        let base_damage = match *method {
            DamageType::Constant(damage) => damage as f32,
            DamageType::FractionOfMissing(fraction) => {
                (health.max_health - health.current_health) as f32 * fraction
            }
            DamageType::FractionOfMax(fraction) => health.max_health as f32 * fraction,
        };

        let new_health = (health.current_health as f32 - base_damage).clamp(0.0, f32::MAX) as u32;

        if new_health == 0 {
            commands.entity(*log_piece).despawn();

            if let Some(player) = source_player {
                commands.trigger(ChangePlayerCoinsBy(value.0 as i32, *player));
            }

            if win_condition {
                commands.trigger(CheckForWinner);
            }
        } else {
            health.current_health = new_health;
        }
    }
}
#[derive(Debug, Event)]
pub struct MovePiece {
    pub log_piece: Entity,
    pub target_tile: Entity,
}
#[derive(Debug, Message)]
pub struct PieceMoved(pub Entity);

fn move_piece(
    instructions: On<MovePiece>,
    mut commands: Commands,
    mut notify_of_movement: MessageWriter<PieceMoved>,
) {
    commands
        .entity(instructions.log_piece)
        .insert(OccupiesTile {
            log_tile: instructions.target_tile,
        });
    notify_of_movement.write(PieceMoved(instructions.log_piece));
}

#[derive(Debug, Event)]
pub struct RotatePiece {
    pub log_piece: Entity,
    pub target_tile: Entity,
}

fn rotate_piece(
    event: On<RotatePiece>,
    adjacencies: Query<&AdjacentTiles>,
    mut piece_info: Query<(&OccupiesTile, &mut FacingDirection)>,
) -> Result<(), BevyError> {
    let (occupied_tile, mut current_direction) = piece_info.get_mut(event.log_piece)?;

    for (direction, maybe_tile) in adjacencies
        .get(occupied_tile.log_tile)?
        .0
        .iter()
        .enumerate()
    {
        if let Some(tile) = maybe_tile
            && *tile == event.target_tile
        {
            current_direction.0 = direction as u8;
            break;
        }
    }

    Ok(())
}

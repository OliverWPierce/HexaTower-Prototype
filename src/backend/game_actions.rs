use bevy::{asset::LoadContext, ecs::schedule::ScheduleLabel, prelude::*};
use serde::{Deserialize, Serialize};

use crate::backend::{
    BackEndSystems,
    game_actions::dangerous_selection_mechanics::{SelectedLogTiles, TileSelectionStatus},
    game_parameters::SetUpBoard,
    pieces::{
        ActiveLogPiece, AlterPieceHealth, DamageOrHealType, DamageUpgradePercent, FacingDirection,
        Health, LogPieceOwnedByPlayer, MonataryValue, MovePiece, OccupiedByPiece, OccupiesTile,
        OrdersPerTurn, OwnsLogPieces, Piece, PieceForSale, PiecesToSwap, RotatePiece,
        SpawnLogPiece, SpawnPoint, TransferPieceOwnership, WinCondition,
    },
    players::{ActivePlayer, PlayerOrdersRemaining, StartTurn},
    shop::{
        ChangePlayerCoinsBy, ChangePlayerPassiveCoinsBy, CoinBag, PlayerShopLuckStats,
        ShopUpgradeTable,
    },
    tiles::{
        AdjacentTiles, DeleteLogTileRequest, EssentialTileCreationSystems, LogicalTileCreated,
    },
};
/// Note that it is highly important that nothing about the backend changes between when an action is loaded and when it
/// is finished being executed or canceled. This could mess up "magic indexes" and other things. If something must change, make sure it calls all the needed schedules to update the backend.
pub struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentAction>();
        app.init_resource::<SelectedLogTiles>();
        app.init_resource::<CurrentSource>();

        app.add_observer(dangerous_selection_mechanics::handle_action_change);

        app.add_observer(dangerous_selection_mechanics::select_tile);

        app.add_systems(
            ExecuteSelectedAction,
            (execute_action_functionality, modify_orders_remaining).in_set(BackEndSystems),
        );

        app.add_systems(
            ActionOrSelectionChanged,
            evaluate_tiles.in_set(BackEndSystems),
        );
        app.add_systems(
            SetUpBoard,
            insert_selection_data
                .after(EssentialTileCreationSystems)
                .in_set(BackEndSystems),
        );

        app.add_systems(
            ExecuteSelectedAction,
            clear_action_related_data.in_set(ClearBackendData),
        );

        app.add_systems(StartTurn, clear_action_related_data.in_set(BackEndSystems));

        app.add_systems(Update, insert_selection_data.in_set(BackEndSystems));

        app.add_observer(validate_execution_request);
    }
}
#[derive(Debug, Deserialize, Serialize, Reflect, Clone)]
pub struct ProxyAction {
    functionality: ProxyActionFunctionality,
    eligibility_method: ProxyDeterminationMethod,
    selection_count_bounds: GameDesignBounds,
}
#[derive(Debug, Deserialize, Serialize, Reflect, Clone)]
enum ProxyActionFunctionality {
    DeleteTile,
    SpawnPiece {
        path_to_proxy_piece: String,
    },
    DoubleTakeTest {
        path_to_proxy_piece: String,
    },
    AlterActivePlayerCoinCount(i32),
    ChangePieceHealth {
        change_method: DamageOrHealType,
        is_heal: bool,
    },
    MoveSelf,
    RotateSelf,
    ChangeActivePlayerPassiveIncomeBy(i32),
    SwapSelf,
    UpgradeSelfShop(ShopUpgradeTable),
    UpgradeDamage {
        fraction: f32,
    },
    UpgradeMaxHealth(u32),
}

#[derive(Debug, Deserialize, Serialize, Reflect, Clone, Copy)]
enum ProxyDeterminationMethod {
    AllTiles,
    AllPieces,
    UnoccupiedTiles,
    PieceChain,
    None,
    Fan {
        width: FanWidth,
        depth: u32,
        occupied_or_not: OccupationStatus,
    },
    RotateSelf,
    GeneralSpawning {
        depth: u32,
    },
    TowerSpawn,
    FriendlyPieces,
}

impl GameAction {
    pub fn from_proxy(proxy: ProxyAction, loader: &mut LoadContext) -> Self {
        let converted_action_func = match proxy.functionality {
            ProxyActionFunctionality::DeleteTile => ActionFunctionality::DeleteTile,
            ProxyActionFunctionality::SpawnPiece {
                path_to_proxy_piece,
            } => ActionFunctionality::SpawnPiece(loader.load(path_to_proxy_piece)),
            ProxyActionFunctionality::DoubleTakeTest {
                path_to_proxy_piece,
            } => ActionFunctionality::DoubleTakeTest(loader.load(path_to_proxy_piece)),
            ProxyActionFunctionality::AlterActivePlayerCoinCount(change) => {
                ActionFunctionality::AlterActivePlayerCoinCount(change)
            }
            ProxyActionFunctionality::ChangePieceHealth {
                change_method,
                is_heal,
            } => ActionFunctionality::ChangePieceHealth {
                change_method,
                is_heal,
            },
            ProxyActionFunctionality::MoveSelf => ActionFunctionality::MoveSelfToTile,
            ProxyActionFunctionality::RotateSelf => ActionFunctionality::RotateSelf,
            ProxyActionFunctionality::ChangeActivePlayerPassiveIncomeBy(coins) => {
                ActionFunctionality::IncreaseActivePlayerPassiveIncomeBy(coins)
            }
            ProxyActionFunctionality::SwapSelf => ActionFunctionality::SwapSelfWithPiece,
            ProxyActionFunctionality::UpgradeSelfShop(shop_upgrade_table) => {
                ActionFunctionality::UpgradeSelfShop(shop_upgrade_table)
            }
            ProxyActionFunctionality::UpgradeDamage { fraction } => {
                ActionFunctionality::UpgradeDamage { fraction }
            }
            ProxyActionFunctionality::UpgradeMaxHealth(amount) => {
                ActionFunctionality::UpgradeHealth { amount }
            }
        };

        let converted_action_method = match proxy.eligibility_method {
            ProxyDeterminationMethod::AllTiles => EligibilityDeterminationMethod::AllTiles,
            ProxyDeterminationMethod::AllPieces => EligibilityDeterminationMethod::AllPieces,
            ProxyDeterminationMethod::UnoccupiedTiles => {
                EligibilityDeterminationMethod::UnoccupiedTiles
            }
            ProxyDeterminationMethod::PieceChain => EligibilityDeterminationMethod::PieceChain,
            ProxyDeterminationMethod::None => EligibilityDeterminationMethod::None,
            ProxyDeterminationMethod::Fan {
                width,
                depth,
                occupied_or_not,
            } => EligibilityDeterminationMethod::Fan {
                width,
                depth,
                occupied_or_not,
            },
            ProxyDeterminationMethod::RotateSelf => EligibilityDeterminationMethod::Fan {
                width: FanWidth::All,
                depth: 1,
                occupied_or_not: OccupationStatus::Either,
            },
            ProxyDeterminationMethod::GeneralSpawning { depth } => {
                EligibilityDeterminationMethod::GeneralSpawning { depth }
            }
            ProxyDeterminationMethod::TowerSpawn => EligibilityDeterminationMethod::TowerSpawning,
            ProxyDeterminationMethod::FriendlyPieces => {
                EligibilityDeterminationMethod::FriendlyPieces
            }
        };

        GameAction {
            functionality: converted_action_func,
            eligibility_method: converted_action_method,
            selection_count_bounds: proxy.selection_count_bounds,
        }
    }
}

/// This set is used to tell a system to run only after front end systems run. It ensures the backend doesn't delete data before the front end gets to look at it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClearBackendData;

#[derive(Debug, PartialEq, Clone)]
pub enum ActionFunctionality {
    DeleteTile,
    SpawnPiece(Handle<Piece>),
    DoubleTakeTest(Handle<Piece>),
    AlterActivePlayerCoinCount(i32),
    ChangePieceHealth {
        change_method: DamageOrHealType,
        is_heal: bool,
    },
    MoveSelfToTile,
    /// This function assumes that only adjacent tiles can be selected.
    RotateSelf,
    PurchasePiece,
    IncreaseActivePlayerPassiveIncomeBy(i32),
    SwapSelfWithPiece,
    UpgradeSelfShop(ShopUpgradeTable),
    UpgradeHealth {
        amount: u32,
    },
    UpgradeDamage {
        fraction: f32,
    },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EligibilityDeterminationMethod {
    AllTiles,
    AllPieces,
    UnoccupiedTiles,
    PieceChain,
    None,
    Fan {
        width: FanWidth,
        depth: u32,
        occupied_or_not: OccupationStatus,
    },
    GeneralSpawning {
        depth: u32,
    },
    PiecesForSale,
    TowerSpawning,
    FriendlyPieces,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, Reflect, Copy)]
pub enum OccupationStatus {
    Vacant,
    Occupied,
    Either,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, Reflect, Copy)]
pub enum FanWidth {
    One,
    Three,
    Five,
    All,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GameAction {
    pub functionality: ActionFunctionality,
    pub eligibility_method: EligibilityDeterminationMethod,
    pub selection_count_bounds: GameDesignBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionSource {
    Card { inventory_index: usize },
    Order { index_in_piece_orders: usize },
    OrphanPiecePurchasing,
    InitialPieceRotation,
}

#[derive(Debug, Resource, Default)]
pub struct CurrentSource(pub Option<ActionSource>);

#[derive(Debug, Event, Clone)]
pub enum SetActionTo {
    None,
    Action {
        action: GameAction,
        source: ActionSource,
    },
}

impl GameAction {
    fn is_valid(&self) -> bool {
        (self.bounds().min_tiles >= self.eligibility_method.bounds().min_tiles)
            && (self.bounds().min_tiles >= self.functionality.bounds().min_tiles)
            && self.bounds().max_tiles >= self.bounds().min_tiles
            && self.functionality.bounds().max_tiles >= self.bounds().max_tiles
            && self.eligibility_method.bounds().max_tiles >= self.bounds().max_tiles
    }

    pub fn is_valid_for_card(&self) -> bool {
        self.is_valid()
            && !self.eligibility_method.requires_active_piece()
            && !self.functionality.requires_active_piece()
    }
}

impl SelectionBounds for GameAction {
    fn bounds(&self) -> Bounds {
        self.selection_count_bounds.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct GameDesignBounds(pub Bounds);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct Bounds {
    pub min_tiles: usize,
    pub max_tiles: usize,
}

pub trait SelectionBounds {
    fn bounds(&self) -> Bounds;
}

impl SelectionBounds for ActionFunctionality {
    fn bounds(&self) -> Bounds {
        match self {
            ActionFunctionality::DeleteTile => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            ActionFunctionality::DoubleTakeTest(_) => Bounds {
                min_tiles: 2,
                max_tiles: 2,
            },
            ActionFunctionality::AlterActivePlayerCoinCount(_) => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            ActionFunctionality::SpawnPiece(_) => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            ActionFunctionality::ChangePieceHealth { .. } => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            ActionFunctionality::MoveSelfToTile => Bounds {
                min_tiles: 1,
                max_tiles: 1,
            },
            ActionFunctionality::RotateSelf => Bounds {
                min_tiles: 1,
                max_tiles: 1,
            },
            ActionFunctionality::PurchasePiece => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            ActionFunctionality::IncreaseActivePlayerPassiveIncomeBy(_) => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            ActionFunctionality::SwapSelfWithPiece => Bounds {
                min_tiles: 1,
                max_tiles: 1,
            },
            ActionFunctionality::UpgradeSelfShop(..) => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            ActionFunctionality::UpgradeHealth { .. } => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            ActionFunctionality::UpgradeDamage { .. } => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
        }
    }
}

impl SelectionBounds for EligibilityDeterminationMethod {
    fn bounds(&self) -> Bounds {
        match self {
            EligibilityDeterminationMethod::AllTiles => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            EligibilityDeterminationMethod::AllPieces => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            EligibilityDeterminationMethod::UnoccupiedTiles => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            EligibilityDeterminationMethod::PieceChain => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            EligibilityDeterminationMethod::None => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            EligibilityDeterminationMethod::Fan { .. } => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            EligibilityDeterminationMethod::GeneralSpawning { .. } => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            EligibilityDeterminationMethod::PiecesForSale => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            EligibilityDeterminationMethod::TowerSpawning => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
            EligibilityDeterminationMethod::FriendlyPieces => Bounds {
                min_tiles: 0,
                max_tiles: usize::MAX,
            },
        }
    }
}

pub trait RequiresActivePiece {
    fn requires_active_piece(&self) -> bool;
}

impl RequiresActivePiece for EligibilityDeterminationMethod {
    fn requires_active_piece(&self) -> bool {
        match self {
            EligibilityDeterminationMethod::AllTiles => false,
            EligibilityDeterminationMethod::AllPieces => false,
            EligibilityDeterminationMethod::UnoccupiedTiles => false,
            EligibilityDeterminationMethod::PieceChain => false,
            EligibilityDeterminationMethod::None => false,
            EligibilityDeterminationMethod::Fan { .. } => true,
            EligibilityDeterminationMethod::GeneralSpawning { .. } => false,
            EligibilityDeterminationMethod::PiecesForSale => false,
            EligibilityDeterminationMethod::TowerSpawning => false,
            EligibilityDeterminationMethod::FriendlyPieces => false,
        }
    }
}

impl RequiresActivePiece for ActionFunctionality {
    fn requires_active_piece(&self) -> bool {
        match self {
            ActionFunctionality::DeleteTile => false,
            ActionFunctionality::SpawnPiece(..) => false,
            ActionFunctionality::DoubleTakeTest(..) => false,
            ActionFunctionality::AlterActivePlayerCoinCount(_) => false,
            ActionFunctionality::ChangePieceHealth { .. } => false,
            ActionFunctionality::MoveSelfToTile => true,
            ActionFunctionality::RotateSelf => true,
            ActionFunctionality::PurchasePiece => false,
            ActionFunctionality::IncreaseActivePlayerPassiveIncomeBy(_) => false,
            ActionFunctionality::SwapSelfWithPiece => true,
            ActionFunctionality::UpgradeSelfShop(..) => false,
            ActionFunctionality::UpgradeHealth { .. } => false,
            ActionFunctionality::UpgradeDamage { .. } => false,
        }
    }
}

#[derive(Debug, Resource, PartialEq, Default)]
pub struct CurrentAction(pub Option<GameAction>);

#[derive(Debug, ScheduleLabel, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExecuteSelectedAction;

fn insert_selection_data(mut new_tiles: MessageReader<LogicalTileCreated>, mut commands: Commands) {
    for LogicalTileCreated(tile) in new_tiles.read() {
        commands.entity(*tile).insert(TileSelectionStatus::new());
    }
}
#[allow(clippy::too_many_arguments)]
pub fn execute_action_functionality(
    action: Res<CurrentAction>,
    selected_tiles: Res<SelectedLogTiles>,
    active_player: Res<ActivePlayer>,
    mut deletions: MessageWriter<DeleteLogTileRequest>,
    mut piece_spawns: MessageWriter<SpawnLogPiece>,
    mut damage_writer: MessageWriter<AlterPieceHealth>,
    mut commands: Commands,
    map_tile_to_piece: Query<&OccupiedByPiece>,
    active_piece: Res<ActiveLogPiece>,
    log_pieces: Query<(&MonataryValue, &DamageUpgradePercent)>,
) -> Result<(), BevyError> {
    let Some(GameAction { functionality, .. }) = action.0.clone() else {
        return Ok(());
    };

    match functionality {
        ActionFunctionality::DeleteTile => {
            for log_tile in selected_tiles.as_read_only_list() {
                deletions.write(DeleteLogTileRequest(*log_tile));
            }
        }
        ActionFunctionality::SpawnPiece(piece) => {
            for log_tile in selected_tiles.as_read_only_list() {
                piece_spawns.write(SpawnLogPiece {
                    player: active_player.0,
                    piece: piece.clone(),
                    log_tile: *log_tile,
                });
            }
        }
        ActionFunctionality::DoubleTakeTest(piece) => {
            deletions.write(DeleteLogTileRequest(
                *(selected_tiles.as_read_only_list().get(1).unwrap()),
            ));
            piece_spawns.write(SpawnLogPiece {
                player: active_player.0,
                piece: piece.clone(),
                log_tile: *selected_tiles.as_read_only_list().first().unwrap(),
            });
        }
        ActionFunctionality::AlterActivePlayerCoinCount(delta_coins) => {
            commands.trigger(ChangePlayerCoinsBy(delta_coins, active_player.0));
        }
        ActionFunctionality::ChangePieceHealth {
            change_method,
            is_heal,
        } => {
            for log_tile in selected_tiles.as_read_only_list() {
                let Ok(log_piece) = map_tile_to_piece.get(*log_tile) else {
                    warn!("tried to damage a piece on a tile which itself had no piece.");
                    continue;
                };

                let damge_multiplyer = if let Some(source_piece) = active_piece.0
                    && !is_heal
                {
                    log_pieces.get(source_piece)?.1.0
                } else {
                    1.0
                };

                damage_writer.write(AlterPieceHealth {
                    log_piece: log_piece.log_piece(),
                    method: change_method,
                    source_player: Some(active_player.0),
                    is_heal,
                    multiplyer: damge_multiplyer,
                });
            }
        }
        ActionFunctionality::MoveSelfToTile => {
            commands.trigger(MovePiece {
                log_piece: active_piece.0.unwrap(),
                target_tile: *selected_tiles.as_read_only_list().first().unwrap(),
            });
        }
        ActionFunctionality::RotateSelf => {
            commands.trigger(RotatePiece {
                log_piece: active_piece.0.unwrap(),
                target_tile: *selected_tiles.as_read_only_list().first().unwrap(),
            });
        }
        ActionFunctionality::PurchasePiece => {
            for log_tile in selected_tiles
                .as_read_only_list()
                .iter()
                .filter_map(|tile| map_tile_to_piece.get(*tile).ok())
            {
                commands.trigger(TransferPieceOwnership {
                    piece: log_tile.log_piece(),
                    to_player: active_player.0,
                });

                commands
                    .entity(log_tile.log_piece())
                    .remove::<PieceForSale>();

                commands.trigger(ChangePlayerCoinsBy(
                    -(log_pieces.get(log_tile.log_piece())?.0.0 as i32),
                    active_player.0,
                ));
            }
        }
        ActionFunctionality::IncreaseActivePlayerPassiveIncomeBy(coins) => {
            commands.trigger(ChangePlayerPassiveCoinsBy(coins, active_player.0));
        }
        ActionFunctionality::SwapSelfWithPiece => {
            let swap = PiecesToSwap(
                map_tile_to_piece
                    .get(*selected_tiles.as_read_only_list().first().unwrap())?
                    .log_piece(),
                active_piece.0.unwrap(),
            );
            commands.queue(|world: &mut World| swap.apply(world));
        }
        ActionFunctionality::UpgradeSelfShop(shop_upgrade_table) => {
            let active_player = active_player.0;

            commands.queue(move |world: &mut World| {
                world
                    .get_mut::<PlayerShopLuckStats>(active_player)
                    .unwrap()
                    .upgrade(shop_upgrade_table);
            });
        }
        ActionFunctionality::UpgradeHealth { amount } => {
            for piece in selected_tiles
                .as_read_only_list()
                .iter()
                .filter_map(|tile| Some(map_tile_to_piece.get(*tile).ok()?.log_piece()))
            {
                commands.queue(move |world: &mut World| {
                    world.get_mut::<Health>(piece).unwrap().max_health += amount
                });
            }
        }
        ActionFunctionality::UpgradeDamage { fraction } => {
            for piece in selected_tiles
                .as_read_only_list()
                .iter()
                .filter_map(|tile| Some(map_tile_to_piece.get(*tile).ok()?.log_piece()))
            {
                commands.queue(move |world: &mut World| {
                    world.get_mut::<DamageUpgradePercent>(piece).unwrap().0 += fraction;
                });
            }
        }
    }

    Ok(())
}

fn clear_action_related_data(mut commands: Commands) {
    commands.trigger(SetActionTo::None);
}

#[allow(clippy::type_complexity)]
fn evaluate_tiles(
    action: Res<CurrentAction>,
    mut tiles: Query<(
        &mut TileSelectionStatus,
        Has<OccupiedByPiece>,
        &AdjacentTiles,
        Entity,
    )>,
    selected_tiles: Res<SelectedLogTiles>,
    log_pieces: Query<(
        &OccupiesTile,
        &FacingDirection,
        Has<SpawnPoint>,
        Has<PieceForSale>,
        &LogPieceOwnedByPlayer,
        Has<WinCondition>,
    )>,
    active_piece: Res<ActiveLogPiece>,
    active_player: Res<ActivePlayer>,
    players: Query<&OwnsLogPieces>,
) -> Result<(), BevyError> {
    let Some(GameAction {
        eligibility_method,
        selection_count_bounds,
        ..
    }) = action.0.clone()
    else {
        return Ok(());
    };

    if selected_tiles.as_read_only_list().len() >= selection_count_bounds.0.max_tiles {
        for (mut selection_state, ..) in tiles.iter_mut() {
            selection_state.try_make_ineligble();
        }
        return Ok(());
    }

    match eligibility_method {
        EligibilityDeterminationMethod::AllTiles => {
            for (mut selection_state, ..) in tiles.iter_mut() {
                selection_state.try_make_eligible();
            }
        }
        EligibilityDeterminationMethod::AllPieces => {
            for (mut selection_state, is_occupied, ..) in tiles.iter_mut() {
                if is_occupied {
                    selection_state.try_make_eligible();
                } else {
                    selection_state.try_make_ineligble();
                }
            }
        }
        EligibilityDeterminationMethod::UnoccupiedTiles => {
            for (mut selection_state, is_occupied, ..) in tiles.iter_mut() {
                if !is_occupied {
                    selection_state.try_make_eligible();
                } else {
                    selection_state.try_make_ineligble();
                }
            }
        }
        EligibilityDeterminationMethod::PieceChain => {
            if selected_tiles.as_read_only_list().is_empty() {
                for (mut selection_state, is_occupied, ..) in tiles.iter_mut() {
                    if is_occupied {
                        selection_state.try_make_eligible();
                    } else {
                        selection_state.try_make_ineligble();
                    }
                }
            } else {
                for (mut selection_state, ..) in tiles.iter_mut() {
                    selection_state.try_make_ineligble();
                }

                for log_tile in selected_tiles.as_read_only_list() {
                    let Ok((_, _, adjacents, ..)) = tiles.get(*log_tile) else {
                        error!("A tile had no adjacent tiles component");
                        continue;
                    };

                    for possible_tile in adjacents.0 {
                        let Some(adjacent_tile) = possible_tile else {
                            continue;
                        };

                        let Ok((mut selection_state, is_occupied, ..)) =
                            tiles.get_mut(adjacent_tile)
                        else {
                            error!("An entity listed as adjacent to a tile was not a tile");
                            continue;
                        };
                        if is_occupied {
                            selection_state.try_make_eligible();
                        }
                    }
                }
            }
        }
        EligibilityDeterminationMethod::None => (),
        EligibilityDeterminationMethod::Fan {
            width,
            depth,
            occupied_or_not,
        } => {
            let (basis_tile, basis_direction, ..) = log_pieces.get(active_piece.0.unwrap())?;

            let mut directional_indices_of_fan = Vec::new();

            match width {
                FanWidth::One => directional_indices_of_fan.push(basis_direction.0),
                FanWidth::Three => {
                    directional_indices_of_fan.push(basis_direction.offset_index(-1));
                    directional_indices_of_fan.push(basis_direction.offset_index(1));
                    directional_indices_of_fan.push(basis_direction.0);
                }
                FanWidth::Five => {
                    directional_indices_of_fan.push(basis_direction.offset_index(-1));
                    directional_indices_of_fan.push(basis_direction.offset_index(1));
                    directional_indices_of_fan.push(basis_direction.offset_index(-2));
                    directional_indices_of_fan.push(basis_direction.offset_index(2));
                    directional_indices_of_fan.push(basis_direction.0);
                }
                FanWidth::All => {
                    directional_indices_of_fan = vec![0, 1, 2, 3, 4, 5];
                }
            }

            let mut eligible_tiles = vec![basis_tile.log_tile];

            for _ in 0..depth {
                for tile in eligible_tiles.clone() {
                    for direction in &directional_indices_of_fan {
                        let Ok((_, _, adjacents, ..)) = tiles.get(tile) else {
                            panic!()
                        };

                        let Some(adjacent_tile) = adjacents.0[*direction as usize] else {
                            continue;
                        };

                        let Ok((mut selection_state, is_occupied, ..)) =
                            tiles.get_mut(adjacent_tile)
                        else {
                            panic!()
                        };

                        eligible_tiles.push(adjacent_tile);

                        match occupied_or_not {
                            OccupationStatus::Vacant => {
                                if !is_occupied {
                                    selection_state.try_make_eligible();
                                }
                            }
                            OccupationStatus::Occupied => {
                                if is_occupied {
                                    selection_state.try_make_eligible();
                                }
                            }
                            OccupationStatus::Either => {
                                selection_state.try_make_eligible();
                            }
                        }
                    }
                }
            }
        }
        EligibilityDeterminationMethod::GeneralSpawning { depth } => {
            let tiles_close_enough_to_friendly_spawn_point = {
                let mut basis_tiles: Vec<Entity> = players
                    .get(active_player.0)
                    .ok()
                    .unwrap_or(&OwnsLogPieces::non_component_default())
                    .iter()
                    .filter_map(|piece| {
                        let (tile, _, is_spawn_point, ..) = log_pieces.get(piece).ok()?;
                        is_spawn_point.then_some(tile.log_tile)
                    })
                    .collect();

                for _ in 0..depth {
                    for tile in basis_tiles.clone() {
                        basis_tiles.extend(
                            tiles
                                .get(tile)?
                                .2
                                .0
                                .iter()
                                .filter_map(|maybe_adjacent| *maybe_adjacent),
                        );
                    }
                }
                basis_tiles
            };

            let tiles_around_unfriendly_spawn_points: Vec<Entity> = {
                let mut basis_tiles = Vec::new();

                for tile in log_pieces.iter().filter_map(
                    |(
                        OccupiesTile { log_tile },
                        _,
                        is_spawn_point,
                        _,
                        LogPieceOwnedByPlayer(owner),
                        ..,
                    )| {
                        (is_spawn_point && *owner != active_player.0).then_some(*log_tile)
                    },
                ) {
                    basis_tiles.extend(
                        tiles
                            .get(tile)?
                            .2
                            .0
                            .iter()
                            .filter_map(|maybe_adjacent| *maybe_adjacent),
                    );
                }

                basis_tiles
            };

            for tile in tiles_close_enough_to_friendly_spawn_point
                .iter()
                .filter(|tile| !tiles_around_unfriendly_spawn_points.contains(*tile))
            {
                let (mut selection_status, is_occupied, ..) = tiles.get_mut(*tile)?;
                match is_occupied {
                    true => selection_status.try_make_ineligble(),
                    false => selection_status.try_make_eligible(),
                }
            }
        }
        EligibilityDeterminationMethod::PiecesForSale => {
            for (OccupiesTile { log_tile }, _, _, for_sale, ..) in log_pieces.iter() {
                if for_sale {
                    tiles.get_mut(*log_tile)?.0.try_make_eligible();
                } else {
                    tiles.get_mut(*log_tile)?.0.try_make_ineligble();
                }
            }
        }
        EligibilityDeterminationMethod::TowerSpawning => {
            let tiles_adjacent_to_enemy_win_conditions = {
                let mut basis_tiles = Vec::new();

                for tile in log_pieces.iter().filter_map(
                    |(
                        OccupiesTile { log_tile },
                        ..,
                        LogPieceOwnedByPlayer(owner),
                        is_win_condit,
                    )| {
                        (is_win_condit && *owner != active_player.0).then_some(*log_tile)
                    },
                ) {
                    basis_tiles.extend(
                        tiles
                            .get(tile)?
                            .2
                            .0
                            .iter()
                            .filter_map(|maybe_adjacent| *maybe_adjacent),
                    );
                }

                basis_tiles
            };

            for (mut selection_state, is_occupied, ..) in tiles
                .iter_mut()
                .filter(|(.., tile)| !tiles_adjacent_to_enemy_win_conditions.contains(tile))
            {
                if is_occupied {
                    selection_state.try_make_ineligble();
                } else {
                    selection_state.try_make_eligible();
                }
            }
        }
        EligibilityDeterminationMethod::FriendlyPieces => {
            let friendly_player = match active_piece.0 {
                Some(piece) => log_pieces.get(piece)?.4.0,
                None => active_player.0,
            };

            for tile in players
                .get(friendly_player)?
                .list()
                .iter()
                .filter_map(|piece| Some(log_pieces.get(*piece).ok()?.0.log_tile))
            {
                tiles.get_mut(tile)?.0.try_make_eligible();
            }
        }
    }

    Ok(())
}

#[derive(Debug, ScheduleLabel, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActionOrSelectionChanged;

/// The idea of this module is to privatize the ability to mutate selection data so that future me doesn't break stuff.
/// Essentially, it ensures that once a tile is selected, it cannot be deselected unless the selection process resets. Also,
/// the only way to select a tile is through a special event.
pub mod dangerous_selection_mechanics {

    use crate::backend::{
        game_actions::{ActionOrSelectionChanged, CurrentAction, CurrentSource, SetActionTo},
        pieces::SetPieceToActive,
    };
    use bevy::prelude::*;

    #[derive(Debug, Default, Resource)]
    pub struct SelectedLogTiles(Vec<Entity>);

    impl SelectedLogTiles {
        pub fn as_read_only_list(&self) -> &Vec<Entity> {
            &self.0
        }
    }
    #[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
    pub enum SelectionState {
        Selected,
        Eligible,
        #[default]
        Neither,
    }

    #[derive(Debug, Component)]
    pub struct TileSelectionStatus(SelectionState);

    impl TileSelectionStatus {
        /// Unless the tile is already selected, this function will change a tile to "neither" eligible or selected.
        /// It is used to prevent accidental alterations to the data of selected tiles.
        pub fn try_make_ineligble(&mut self) {
            if self.0 != SelectionState::Selected {
                self.0 = SelectionState::Neither
            }
        }

        /// This function will make a tile eligible to be selected next unless it is already selected.
        /// It is used to prevent accidental alterations to the data of selected tiles.
        pub fn try_make_eligible(&mut self) {
            if self.0 != SelectionState::Selected {
                self.0 = SelectionState::Eligible
            }
        }

        pub fn read(&self) -> SelectionState {
            self.0
        }

        pub fn new() -> Self {
            TileSelectionStatus(SelectionState::default())
        }
    }

    pub fn handle_action_change(
        instructions: On<SetActionTo>,
        mut current_action: ResMut<CurrentAction>,
        mut current_source: ResMut<CurrentSource>,
        mut selection_list: ResMut<SelectedLogTiles>,
        mut log_tiles: Query<&mut TileSelectionStatus>,
        mut commands: Commands,
    ) {
        match instructions.clone() {
            SetActionTo::None => {
                current_action.0 = None;
                current_source.0 = None;
            }
            SetActionTo::Action { action, source } => {
                current_action.0 = Some(action);
                current_source.0 = Some(source);

                match source {
                    super::ActionSource::Card { .. } => commands.trigger(SetPieceToActive(None)),
                    super::ActionSource::Order { .. } => (),
                    super::ActionSource::OrphanPiecePurchasing => (),
                    super::ActionSource::InitialPieceRotation => (),
                }
            }
        }

        selection_list.0.clear();
        for mut tile_state in log_tiles.iter_mut() {
            tile_state.0 = SelectionState::Neither;
        }
        commands.run_schedule(ActionOrSelectionChanged);
    }

    #[derive(Debug, Event)]
    pub struct SelectLogTile(pub Entity);

    pub fn select_tile(
        tile: On<SelectLogTile>,
        mut selection_list: ResMut<SelectedLogTiles>,
        mut selection_data: Query<&mut TileSelectionStatus>,
        mut commands: Commands,
    ) {
        let Ok(mut status) = selection_data.get_mut(tile.0) else {
            warn!("A tile selection request was made for an entity with no tile selection status");
            return;
        };

        if status.0 == SelectionState::Eligible {
            status.0 = SelectionState::Selected;
            selection_list.0.push(tile.0);
            commands.run_schedule(ActionOrSelectionChanged);
        }
    }
}
#[derive(Debug, Event)]
pub struct ExecuteActionRequest;

#[allow(clippy::too_many_arguments)]
fn validate_execution_request(
    _request: On<ExecuteActionRequest>,
    mut commands: Commands,
    action: Res<CurrentAction>,
    source: Res<CurrentSource>,
    selected_tiles: Res<SelectedLogTiles>,
    active_plyer: Res<ActivePlayer>,
    active_piece: Res<ActiveLogPiece>,
    player_info: Query<(&PlayerOrdersRemaining, &CoinBag)>,
    piece_info: Query<(&OrdersPerTurn, &LogPieceOwnedByPlayer, &MonataryValue)>,
    map_tile_to_piece: Query<&OccupiedByPiece>,
) -> Result<(), BevyError> {
    let Some(action_info) = &action.0 else {
        return Ok(());
    };

    let Some(source) = &source.0 else {
        return Ok(());
    };

    let amount_selected = selected_tiles.as_read_only_list().len();

    let otherwise_allowed = match source {
        ActionSource::Order { .. } => {
            let Some(piece) = active_piece.0 else {
                warn!("tried to execute an order but there was no active piece.");
                //its not really "ok" but since this issue isn't worth panicking over I don't want to emit a bevy error. In the future, this could be fixed with my own error type.
                return Ok(());
            };

            let (
                OrdersPerTurn {
                    current: piece_orders_remaining,
                    ..
                },
                LogPieceOwnedByPlayer(owner),
                _,
            ) = piece_info.get(piece)?;

            player_info.get(active_plyer.0)?.0.0 > 0
                && *piece_orders_remaining > 0
                && *owner == active_plyer.0
        }
        ActionSource::OrphanPiecePurchasing => {
            player_info.get(active_plyer.0)?.1.coins
                >= selected_tiles
                    .as_read_only_list()
                    .iter()
                    .filter_map(|tile| {
                        Some(
                            piece_info
                                .get(map_tile_to_piece.get(*tile).ok()?.log_piece())
                                .ok()?
                                .2
                                .0,
                        )
                    })
                    .sum::<u32>() as i32
        }
        _ => true,
    };

    if amount_selected >= action_info.functionality.bounds().min_tiles
        && amount_selected <= action_info.functionality.bounds().max_tiles
        && otherwise_allowed
    {
        commands.run_schedule(ExecuteSelectedAction);
    }

    Ok(())
}

fn modify_orders_remaining(
    active_piece: Res<ActiveLogPiece>,
    current_source: Res<CurrentSource>,
    active_player: Res<ActivePlayer>,
    mut player_orders: Query<&mut PlayerOrdersRemaining>,
    mut piece_orders: Query<&mut OrdersPerTurn>,
) -> Result<(), BevyError> {
    if let Some(source) = current_source.0
        && let ActionSource::Order { .. } = source
        && let Some(piece) = active_piece.0
    {
        player_orders.get_mut(active_player.0)?.0 -= 1;
        piece_orders.get_mut(piece)?.current -= 1;
    }
    Ok(())
}

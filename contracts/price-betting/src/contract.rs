use soroban_sdk::{Address, BytesN, contract, contractimpl, Env, panic_with_error, Symbol, token};

use crate::errors::ContractErrors;
use crate::storage::core::{CoreData, CoreDataFunc, CoreUpdateAddress};
use crate::storage::predictions::{Game, GameResult, Prediction, PredictionsDataFunc};
use crate::utils::core::is_started;
use crate::utils::oracle::{get_latest_price, validate_asset};

pub trait PricePredictionContractTrait {
    fn init(
        e: Env,
        admin: Address,
        fee_taker: Address,
        fee: u128,
        paying_asset: Address,
        oracle: Address,
    );

    fn upgrade(e: Env, hash: BytesN<32>);

    fn update_address(e: Env, target: CoreUpdateAddress, address: Address);

    fn create_game(
        e: Env,
        id: BytesN<32>,
        host: Address,
        asset: Symbol,
        deadline: u64,
        target_date: u64,
        target_price: u128,
    );

    fn predict(e: &Env, game_id: BytesN<32>, caller: Address, result: GameResult, deposit: u128);

    fn execute(e: &Env, game_id: BytesN<32>);

    fn withdraw(e: &Env, game_id: BytesN<32>, caller: Address);
}

#[contract]
pub struct PricePredictionContract;

#[contractimpl]
impl PricePredictionContractTrait for PricePredictionContract {
    fn init(
        e: Env,
        admin: Address,
        fee_taker: Address,
        fee: u128,
        paying_asset: Address,
        oracle: Address,
    ) {
        if e._core().data().is_some() {
            panic_with_error!(&e, &ContractErrors::AlreadyInitiated);
        }

        e._core().set_data(&CoreData {
            admin,
            fee_taker,
            fee,
            paying_asset,
            oracle,
        });

        e._core().bump();
    }

    fn upgrade(e: Env, hash: BytesN<32>) {
        e._core().data().unwrap().admin.require_auth();
        e.deployer().update_current_contract_wasm(hash);
        e._core().bump();
    }

    fn update_address(e: Env, target: CoreUpdateAddress, address: Address) {
        let mut core_data: CoreData = e._core().data().unwrap();
        core_data.admin.require_auth();

        match target {
            CoreUpdateAddress::Admin => core_data.admin = address,
            CoreUpdateAddress::FeeTaker => core_data.fee_taker = address,
            CoreUpdateAddress::PayingAsset => core_data.paying_asset = address,
            CoreUpdateAddress::Oracle => core_data.oracle = address,
        }

        e._core().set_data(&core_data);
        e._core().bump();
    }

    fn create_game(
        e: Env,
        id: BytesN<32>,
        host: Address,
        asset: Symbol,
        deadline: u64,
        target_date: u64,
        target_price: u128,
    ) {
        is_started(&e);
        e._core().bump();

        let core_data: CoreData = e._core().data().unwrap();
        validate_asset(&e, &core_data, &asset);

        if e._predictions().game(&id).is_some() {
            panic_with_error!(&e, &ContractErrors::GameAlreadyExists);
        }

        // Deadline can not be lower than an hour
        // Deadline can not be more than the target date
        if deadline < (e.ledger().timestamp() + 3600) || deadline > target_date {
            panic_with_error!(&e, &ContractErrors::InvalidDeadline);
        }

        // Target date can not be lower than 24hrs
        if target_date < (e.ledger().timestamp() + 86400) {
            panic_with_error!(&e, &ContractErrors::InvalidTargetDate);
        }

        let new_game: Game = Game {
            id,
            host,
            asset,
            deadline,
            target_date,
            target_price,
            highs_deposit: 0,
            highs_participants: 0,
            lows_deposit: 0,
            lows_participants: 0,
            prize: 0,
            fee: 0,
            executed_at: 0,
            result: GameResult::None,
        };

        e._predictions().set_game(&new_game);
        e._predictions().bump_game(&new_game.id);
    }

    fn predict(e: &Env, game_id: BytesN<32>, caller: Address, result: GameResult, deposit: u128) {
        caller.require_auth();
        is_started(&e);
        e._core().bump();

        if deposit < 1_0000000 {
            panic_with_error!(&e, &ContractErrors::InvalidPredictionAmount);
        }

        let mut game: Game = e._predictions().game(&game_id).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::GameDoesntExist);
        });

        if game.deadline < e.ledger().timestamp() {
            panic_with_error!(&e, &ContractErrors::GameDeadlineReached);
        }

        if e._predictions().prediction(&game_id, &caller).is_some() {
            panic_with_error!(&e, &ContractErrors::AlreadyPredicted);
        }

        let new_prediction: Prediction = Prediction {
            game_id: game.id.clone(),
            player: caller.clone(),
            result,
            date: e.ledger().timestamp(),
            deposit,
            prize: 0,
            claimed: false,
        };

        e._predictions().set_prediction(&new_prediction);
        e._predictions().bump_prediction(&game_id, &caller);

        match new_prediction.result.clone() {
            GameResult::Higher => {
                game.highs_participants += 1;
                game.highs_deposit += deposit;
            }
            GameResult::Lower => {
                game.lows_participants += 1;
                game.lows_deposit += deposit;
            }
            GameResult::None => panic_with_error!(&e, &ContractErrors::InvalidPredictionResult),
            GameResult::Cancelled => {
                panic_with_error!(&e, &ContractErrors::InvalidPredictionResult)
            }
        };

        let xlm = token::Client::new(&e, &e._core().data().unwrap().paying_asset);
        let deposit_result =
            xlm.try_transfer(&caller, &e.current_contract_address(), &(deposit as i128));

        if deposit_result.is_err() {
            panic_with_error!(&e, &ContractErrors::FailedToDeposit);
        }

        e._predictions().set_game(&game);
        e._predictions().bump_game(&game_id);
    }

    fn execute(e: &Env, game_id: BytesN<32>) {
        is_started(&e);
        e._core().bump();

        let core_data: CoreData = e._core().data().unwrap();

        let mut game: Game = e._predictions().game(&game_id).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::GameDoesntExist);
        });

        if game.result != GameResult::None {
            panic_with_error!(&e, &ContractErrors::GameAlreadyExecuted);
        }

        if e.ledger().timestamp() < game.target_date {
            panic_with_error!(&e, &ContractErrors::GameCantBeExecuted);
        }
        // If there wasn't a participant on one of the sides, we cancel the game
        if game.highs_participants == 0 || game.lows_participants == 0 {
            game.result = GameResult::Cancelled;
        } else {
            let latest = get_latest_price(&e, &core_data, &game.asset);

            if latest.timestamp < game.target_date {
                panic_with_error!(&e, &ContractErrors::AssetPriceIsNotUpdated);
            }

            if (latest.price as u128) < game.target_price {
                game.result = GameResult::Lower;
                game.fee = (game.highs_deposit * core_data.fee).div_ceil(1_0000000);
                game.prize = game.highs_deposit - game.fee;
            } else {
                game.result = GameResult::Higher;
                game.fee = (game.lows_deposit * core_data.fee).div_ceil(1_0000000);
                game.prize = game.lows_deposit - game.fee;
            }

            let xlm = token::Client::new(&e, &core_data.paying_asset);

            let host_share: u128 = game.fee / 2;
            let host_share_result = xlm.try_transfer(
                &e.current_contract_address(),
                &game.host,
                &(host_share as i128),
            );

            if host_share_result.is_err() {
                panic_with_error!(&e, &ContractErrors::FailedToPayHostShare);
            }

            let protocol_share: u128 = game.fee - host_share;
            let protocol_share_result = xlm.try_transfer(
                &e.current_contract_address(),
                &core_data.fee_taker,
                &(protocol_share as i128),
            );

            if protocol_share_result.is_err() {
                panic_with_error!(&e, &ContractErrors::FailedToPayProtocolShare);
            }
        }

        game.executed_at = e.ledger().timestamp();

        e._predictions().set_game(&game);
        e._predictions().bump_game(&game_id);
    }

    fn withdraw(e: &Env, game_id: BytesN<32>, caller: Address) {
        caller.require_auth();
        is_started(&e);
        e._core().bump();

        let core_data: CoreData = e._core().data().unwrap();

        let game: Game = e._predictions().game(&game_id).unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::GameDoesntExist);
        });

        e._predictions().bump_game(&game_id);

        if game.result != GameResult::Higher && game.result != GameResult::Lower {
            panic_with_error!(&e, &ContractErrors::GameHasNotBeenExecuted);
        }

        let mut prediction: Prediction = e
            ._predictions()
            .prediction(&game_id, &caller)
            .unwrap_or_else(|| {
                panic_with_error!(&e, &ContractErrors::PredictionDoesntExist);
            });

        if prediction.result != game.result {
            panic_with_error!(&e, &ContractErrors::PredictionWasIncorrect);
        }

        if prediction.claimed {
            panic_with_error!(&e, &ContractErrors::PredictionAlreadyClaimed);
        }

        prediction.claimed = true;

        let participation: u128 = if prediction.result == GameResult::Higher {
            (prediction.deposit * 1_0000000) / game.highs_deposit
        } else {
            (prediction.deposit * 1_0000000) / game.lows_deposit
        };

        let reward: u128 = (game.prize * participation) / 1_0000000;
        prediction.prize = reward;
        e._predictions().set_prediction(&prediction);
        e._predictions().bump_prediction(&game_id, &caller);

        let withdraw_amount: u128 = prediction.deposit + reward;
        let xlm = token::Client::new(&e, &core_data.paying_asset);
        let withdraw_result = xlm.try_transfer(
            &e.current_contract_address(),
            &caller,
            &(withdraw_amount as i128),
        );

        if withdraw_result.is_err() {
            panic_with_error!(&e, &ContractErrors::FailedToWithdrawFunds);
        }
    }
}

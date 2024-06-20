use soroban_sdk::{Address, BytesN, contracttype, Env, Symbol};

use crate::DAY_LEDGER;

#[contracttype]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum GameResult {
    Higher,
    Lower,
    None,
    Cancelled,
}

#[contracttype]
#[derive(Debug, Eq, PartialEq)]
pub struct Game {
    pub id: BytesN<32>,

    // The host of the prediction and whom the fee is going to share with
    pub host: Address,

    // This asset must be a compatible asset with the Oracle options
    pub asset: Symbol,

    // The date users can no longer put participation
    pub deadline: u64,

    // The date the prediction closes and when its resolution can be executed
    pub target_date: u64,

    // The price target to predict if it's going to be higher or lower
    pub target_price: u128,

    pub highs_deposit: u128,
    pub highs_participants: u64,

    pub lows_deposit: u128,
    pub lows_participants: u64,

    // The amount to distributed to winners after fee
    pub prize: u128,

    // Fee distributed between host and platform
    pub fee: u128,

    pub executed_at: u64,
    pub result: GameResult,
}

#[contracttype]
pub struct Prediction {
    pub game_id: BytesN<32>,
    pub player: Address,
    pub result: GameResult,
    pub date: u64,
    pub deposit: u128,
    pub prize: u128,
    pub claimed: bool,
}

#[contracttype]
pub enum PredictionDataKeys {
    Game(BytesN<32>),
    Prediction((BytesN<32>, Address)),
}

pub struct Predictions {
    env: Env,
}

impl Predictions {
    pub fn new(e: &Env) -> Predictions {
        Predictions { env: e.clone() }
    }

    pub fn game(&self, id: &BytesN<32>) -> Option<Game> {
        self.env
            .storage()
            .persistent()
            .get(&PredictionDataKeys::Game(id.clone()))
    }

    pub fn set_game(&self, data: &Game) {
        self.env
            .storage()
            .persistent()
            .set(&PredictionDataKeys::Game(data.id.clone()), data);
    }

    pub fn bump_game(&self, id: &BytesN<32>) {
        self.env.storage().persistent().extend_ttl(
            &PredictionDataKeys::Game(id.clone()),
            DAY_LEDGER * 15,
            DAY_LEDGER * 30,
        );
    }

    pub fn prediction(&self, id: &BytesN<32>, player: &Address) -> Option<Prediction> {
        self.env
            .storage()
            .persistent()
            .get(&PredictionDataKeys::Prediction((
                id.clone(),
                player.clone(),
            )))
    }

    pub fn set_prediction(&self, data: &Prediction) {
        self.env.storage().persistent().set(
            &PredictionDataKeys::Prediction((data.game_id.clone(), data.player.clone())),
            data,
        );
    }

    pub fn bump_prediction(&self, id: &BytesN<32>, player: &Address) {
        self.env.storage().persistent().extend_ttl(
            &PredictionDataKeys::Prediction((id.clone(), player.clone())),
            DAY_LEDGER * 15,
            DAY_LEDGER * 30,
        );
    }
}

pub trait PredictionsDataFunc {
    fn _predictions(&self) -> Predictions;
}

impl PredictionsDataFunc for Env {
    fn _predictions(&self) -> Predictions {
        Predictions::new(self)
    }
}

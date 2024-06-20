use soroban_sdk::{Address, contracttype, Env};

use crate::DAY_LEDGER;

#[contracttype]
pub enum CoreUpdateAddress {
    Admin,
    FeeTaker,
    PayingAsset,
    Oracle,
}

#[contracttype]
pub struct CoreData {
    pub admin: Address,
    pub fee_taker: Address,
    pub fee: u128,
    pub paying_asset: Address,

    pub oracle: Address,
}

#[contracttype]
pub enum CoreDataKeys {
    CoreData,
}

pub struct Core {
    env: Env,
}

impl Core {
    pub fn new(e: &Env) -> Core {
        Core { env: e.clone() }
    }

    pub fn data(&self) -> Option<CoreData> {
        self.env.storage().instance().get(&CoreDataKeys::CoreData)
    }

    pub fn set_data(&self, core_data: &CoreData) {
        self.env
            .storage()
            .instance()
            .set(&CoreDataKeys::CoreData, core_data);
    }

    pub fn bump(&self) {
        self.env
            .storage()
            .instance()
            .extend_ttl(DAY_LEDGER, DAY_LEDGER * 15);
    }
}

pub trait CoreDataFunc {
    fn _core(&self) -> Core;
}

impl CoreDataFunc for Env {
    fn _core(&self) -> Core {
        Core::new(self)
    }
}

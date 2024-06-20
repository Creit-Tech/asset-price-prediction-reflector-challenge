use soroban_sdk::{Env, panic_with_error};

use crate::errors::ContractErrors;
use crate::storage::core::CoreDataFunc;

pub fn is_started(e: &Env) {
    if e._core().data().is_none() {
        panic_with_error!(&e, &ContractErrors::NotInitiated);
    }
}

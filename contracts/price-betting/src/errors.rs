use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractErrors {
    AlreadyInitiated = 0,
    NotInitiated = 1,
    InvalidAsset = 2,
    GameAlreadyExists = 3,
    InvalidDeadline = 4,
    InvalidTargetDate = 5,
    GameDoesntExist = 6,
    GameDeadlineReached = 7,
    AlreadyPredicted = 8,
    InvalidPredictionResult = 9,
    InvalidPredictionAmount = 10,
    GameHasNotBeenExecuted = 11,
    PredictionDoesntExist = 12,
    PredictionWasIncorrect = 13,
    PredictionAlreadyClaimed = 14,
    FailedToWithdrawFunds = 15,
    FailedToPayHostShare = 16,
    FailedToPayProtocolShare = 17,
    AssetPriceNotFound = 18,
    GameCantBeExecuted = 19,
    AssetPriceIsNotUpdated = 20,
    FailedToDeposit = 21,
    GameAlreadyExecuted = 22,
}

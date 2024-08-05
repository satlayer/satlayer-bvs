use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("RewardsCoordinator: Unauthorized")]
    Unauthorized {},

    #[error("RewardsCoordinator.instantiate: invalid genesis timestamp")]
    InvalidGenesisTimestamp {},

    #[error("RewardsCoordinator.instantiate: invalid calculation interval")]
    InvalidCalculationInterval {},

    #[error("RewardsCoordinator._only_rewards_updater: caller is not the rewardsUpdater")]
    NotRewardsUpdater {},

    #[error("RewardsCoordinator._only_rewards_for_all_submitter: caller is not a valid createRewardsForAllSubmission submitter")]
    ValidCreateRewardsForAllSubmission {},

    #[error("RewardsCoordinator.validate_rewards_submission: no strategies set")]
    NoStrategiesSet {},

    #[error("RewardsCoordinator.validate_rewards_submission: amount cannot be 0")]
    AmountCannotBeZero {},

    #[error("RewardsCoordinator.validate_rewards_submission: amount too large")]
    AmountTooLarge {},

    #[error("RewardsCoordinator.validate_rewards_submission: duration exceeds MAX_REWARDS_DURATION")]
    ExceedsMaxRewardsDuration {},

    #[error("RewardsCoordinator.validate_rewards_submission: duration must be a multiple of CALCULATION_INTERVAL_SECONDS")]
    DurationMustBeMultipleOfCalcIntervalSec {},

    #[error("RewardsCoordinator.validate_rewards_submission: startTimestamp must be a multiple of CALCULATION_INTERVAL_SECONDS")]
    TimeMustBeMultipleOfCalcIntervalSec {},

    #[error("RewardsCoordinator.validate_rewards_submission: startTimestamp too far in the past")]
    StartTimeStampTooFarInPase {},

    #[error("RewardsCoordinator.validate_rewards_submission: startTimestamp too far in the future")]
    StartTimeStampTooFarInFuture {},

    #[error("RewardsCoordinator.validate_rewards_submission: invalid strategy considered")]
    InvaildStrategyConsidered {},

    #[error("RewardsCoordinator.validate_rewards_submission: strategies must be in ascending order to handle duplicates")]
    StrategiesMuseBeHandleDuplicates {},
}
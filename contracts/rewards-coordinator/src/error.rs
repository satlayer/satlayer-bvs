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

    #[error("RewardsCoordinator.disable_root: invalid root index")]
    InvalidRootIndex {},

    #[error("RewardsCoordinator.disable_root: root does not exist")]
    RootNotExist {},

    #[error("RewardsCoordinator.disable_root: This root is already disabled")]
    AlreadyDisabled {},

    #[error("RewardsCoordinator.disable_root: This root is already activated and cannot be disabled")]
    AlreadyActivated {},

    #[error("RewardsCoordinator.submit_root: new root must be for newer calculated period")]
    InvalidTimestamp {},

    #[error("RewardsCoordinator.submit_root: timestamp in future")]
    TimestampInFuture {},

    #[error("RewardsCoordinator._verify_token_claim_proof: invalid tokenLeafIndex")]
    InvalidTokenLeafIndex {},

    #[error("RewardsCoordinator._verify_token_claim_proof: invalid token claim proof")]
    InvalidTokenClaimProof {},

    #[error("RewardsCoordinator._verify_earner_claim_proof: invalid earnerLeafIndex")]
    InvalidEarnerLeafIndex {},

    #[error("RewardsCoordinator._verify_earner_claim_proof: invalid earner claim proof")]
    InvalidEarnerClaimProof {},

    #[error("RewardsCoordinator._check_claim: root disabled")]
    RootDisabled {},

    #[error("RewardsCoordinator._check_claim: root not activated yet")]
    RootNotActivatedYet {},

    #[error("RewardsCoordinator._check_claim: tokenIndices and tokenProofs length mismatch")]
    TokenIndicesAndProofsMismatch {},

    #[error("RewardsCoordinator._check_claim: tokenTreeProofs and leaves length mismatch")]
    TokenProofsAndLeavesMismatch {},

    #[error("RewardsCoordinator.process_claim: caller is not valid claimer")]
    UnauthorizedClaimer {},

    #[error("RewardsCoordinator.process_claim: cumulativeEarnings must be gt than cumulativeClaimed")]
    CumulativeEarningsTooLow {},

    #[error("RewardsCoordinator.query_calculate_token_leaf_hash: invalid cumulative earnings")]
    InvalidCumulativeEarnings {},

    #[error("RewardsCoordinator.query_get_distribution_root_at_index: invalid index format")]
    InvalidIndexFormat {},

    #[error("RewardsCoordinator.query_get_current_distribution_root: no active root found")]
    NoActiveRootFound {},

    #[error("RewardsCoordinator.query_get_current_claimable_distribution_root: no claimable root found")]
    NoClaimableRootFound {},

    #[error("RewardsCoordinator.query_get_root_index_from_hash: root not found")]
    RootNotFound {},

    #[error("RewardsCoordinator.query_get_root_index_from_hash: invalid hex format")]
    InvalidHexFormat {},
}
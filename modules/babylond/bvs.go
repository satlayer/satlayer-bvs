package babylond

import (
	"encoding/json"
	"math/big"

	bvscw "github.com/satlayer/satlayer-bvs/bvs-cw"
	delegationmanager "github.com/satlayer/satlayer-bvs/bvs-cw/delegation-manager"
	"github.com/satlayer/satlayer-bvs/bvs-cw/directory"
	"github.com/satlayer/satlayer-bvs/bvs-cw/driver"
	rewardscoordinator "github.com/satlayer/satlayer-bvs/bvs-cw/rewards-coordinator"
	slashmanager "github.com/satlayer/satlayer-bvs/bvs-cw/slash-manager"
	statebank "github.com/satlayer/satlayer-bvs/bvs-cw/state-bank"
	strategybase "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-base"
	strategybasetvllimits "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-base-tvl-limits"
	strategyfactory "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-factory"
	strategymanager "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-manager"
)

type Deployed[T interface{}] struct {
	DeployedWasmContract
	InstantiateMsg T
}

func deployCrate[T interface{}](c *BabylonContainer, crate string, initMsg T, label string) *Deployed[T] {
	wasmByteCode, err := bvscw.ReadWasmFile(crate)
	if err != nil {
		panic(err)
	}

	initBytes, err := json.Marshal(initMsg)
	if err != nil {
		panic(err)
	}

	contract, err := c.StoreAndInitWasm(wasmByteCode, initBytes, label, "genesis")
	if err != nil {
		panic(err)
	}
	return &Deployed[T]{
		DeployedWasmContract: *contract,
		InstantiateMsg:       initMsg,
	}
}

func (c *BabylonContainer) DeployStateBank() *Deployed[statebank.InstantiateMsg] {
	initMsg := statebank.InstantiateMsg{
		InitialOwner: c.GenerateAddress("state-bank:initial_owner").String(),
	}
	return deployCrate(c, "bvs-state-bank", initMsg, "BVS State Bank")
}

func (c *BabylonContainer) DeployDriver() *Deployed[driver.InstantiateMsg] {
	initMsg := driver.InstantiateMsg{
		InitialOwner: c.GenerateAddress("driver:initial_owner").String(),
	}
	return deployCrate(c, "bvs-driver", initMsg, "BVS Driver")
}

// TODO: every contract on top is considered "pure" can be deployed without circular dependency
//   For contracts below, we need to do cleanup.

// TODO: Too much initialization. Some can be moved to `ExecuteMsg` instead of `InstantiateMsg`

func (c *BabylonContainer) DeploySlashManager(
	delegationManager string, strategyManager string,
) *Deployed[slashmanager.InstantiateMsg] {
	initMsg := slashmanager.InstantiateMsg{
		InitialPausedStatus: 0,
		InitialOwner:        c.GenerateAddress("slash-manager:initial_owner").String(),
		Pauser:              c.GenerateAddress("slash-manager:pauser").String(),
		Unpauser:            c.GenerateAddress("slash-manager:unpauser").String(),
		StrategyManager:     strategyManager,
		DelegationManager:   delegationManager,
	}

	return deployCrate(c, "bvs-slash-manager", initMsg, "BVS Slash Manager")
}

func (c *BabylonContainer) DeployStrategyManager(
	delegationManager string,
	slashManager string,
	strategyFactory string,
	initialStrategyWhitelister string,
) *Deployed[strategymanager.InstantiateMsg] {
	initMsg := strategymanager.InstantiateMsg{
		InitialPausedStatus:        0,
		InitialOwner:               c.GenerateAddress("strategy-manager:initial_owner").String(),
		Pauser:                     c.GenerateAddress("strategy-manager:pauser").String(),
		Unpauser:                   c.GenerateAddress("strategy-manager:unpauser").String(),
		DelegationManager:          delegationManager,
		SlashManager:               slashManager,
		StrategyFactory:            strategyFactory,
		InitialStrategyWhitelister: initialStrategyWhitelister,
	}

	return deployCrate(c, "bvs-strategy-manager", initMsg, "BVS Strategy Manager")
}

func (c *BabylonContainer) DeployDelegationManager(
	slashManager string,
	strategyManager string,
	minWithdrawalDelayBlocks int64,
	strategies []string,
	withdrawalDelayBlocks []int64,
) *Deployed[delegationmanager.InstantiateMsg] {
	initMsg := delegationmanager.InstantiateMsg{
		InitialPausedStatus:      0,
		InitialOwner:             c.GenerateAddress("delegation-manager:initial_owner").String(),
		Pauser:                   c.GenerateAddress("delegation-manager:pauser").String(),
		Unpauser:                 c.GenerateAddress("delegation-manager:unpauser").String(),
		SlashManager:             slashManager,
		StrategyManager:          strategyManager,
		MinWithdrawalDelayBlocks: minWithdrawalDelayBlocks,
		Strategies:               strategies,
		WithdrawalDelayBlocks:    withdrawalDelayBlocks,
	}

	return deployCrate(c, "bvs-delegation-manager", initMsg, "BVS Delegation Manager")
}

func (c *BabylonContainer) DeployDirectory(
	delegationManager string,
) *Deployed[directory.InstantiateMsg] {
	initMsg := directory.InstantiateMsg{
		InitialPausedStatus: 0,
		InitialOwner:        c.GenerateAddress("directory:initial_owner").String(),
		Pauser:              c.GenerateAddress("directory:pauser").String(),
		Unpauser:            c.GenerateAddress("directory:unpauser").String(),
		DelegationManager:   delegationManager,
	}

	return deployCrate(c, "bvs-directory", initMsg, "BVS Directory")
}

func (c *BabylonContainer) DeployStrategyBaseTvlLimits(
	strategyManager string,
	underlyingToken string,
	maxPerDeposit big.Int,
	maxTotalDeposits big.Int,
) *Deployed[strategybasetvllimits.InstantiateMsg] {
	initMsg := strategybasetvllimits.InstantiateMsg{
		InitialPausedStatus: 0,
		InitialOwner:        c.GenerateAddress("strategy-base-tvl-limits:initial_owner").String(),
		Pauser:              c.GenerateAddress("strategy-base-tvl-limits:pauser").String(),
		Unpauser:            c.GenerateAddress("strategy-base-tvl-limits:unpauser").String(),
		StrategyManager:     strategyManager,
		UnderlyingToken:     underlyingToken,
		MaxPerDeposit:       maxPerDeposit.String(),
		MaxTotalDeposits:    maxTotalDeposits.String(),
	}

	return deployCrate(c, "bvs-strategy-base-tvl-limits", initMsg, "BVS Strategy Base TVL Limits")
}

func (c *BabylonContainer) DeployRewardsCoordinator(
	delegationManager string,
	strategyManager string,
	activationDelay int64,
	calculationIntervalSeconds int64,
	genesisRewardsTimestamp int64,
	maxFutureLength int64,
	maxRetroactiveLength int64,
	maxRewardsDuration int64,
	rewardsUpdater string,
) *Deployed[rewardscoordinator.InstantiateMsg] {
	initMsg := rewardscoordinator.InstantiateMsg{
		InitialPausedStatus:        0,
		InitialOwner:               c.GenerateAddress("rewards-coordinator:initial_owner").String(),
		Pauser:                     c.GenerateAddress("rewards-coordinator:pauser").String(),
		Unpauser:                   c.GenerateAddress("rewards-coordinator:unpauser").String(),
		StrategyManager:            strategyManager,
		DelegationManager:          delegationManager,
		ActivationDelay:            activationDelay,
		CalculationIntervalSeconds: calculationIntervalSeconds,
		GenesisRewardsTimestamp:    genesisRewardsTimestamp,
		MaxFutureLength:            maxFutureLength,
		MaxRetroactiveLength:       maxRetroactiveLength,
		MaxRewardsDuration:         maxRewardsDuration,
		RewardsUpdater:             rewardsUpdater,
	}

	return deployCrate(c, "bvs-rewards-coordinator", initMsg, "BVS Rewards Coordinator")
}

func (c *BabylonContainer) DeployStrategyFactory(
	strategyManager string, strategyCodeId int64,
) *Deployed[strategyfactory.InstantiateMsg] {
	initMsg := strategyfactory.InstantiateMsg{
		InitialPausedStatus: 0,
		InitialOwner:        c.GenerateAddress("strategy-factory:initial_owner").String(),
		Pauser:              c.GenerateAddress("strategy-factory:pauser").String(),
		Unpauser:            c.GenerateAddress("strategy-factory:unpauser").String(),
		StrategyManager:     strategyManager,
		StrategyCodeID:      strategyCodeId,
	}

	return deployCrate(c, "bvs-strategy-factory", initMsg, "BVS Strategy Factory")
}

func (c *BabylonContainer) DeployStrategyBase(
	underlyingToken string, strategyManager string,
) *Deployed[strategybase.InstantiateMsg] {
	initMsg := strategybase.InstantiateMsg{
		InitialPausedStatus: 0,
		InitialOwner:        c.GenerateAddress("strategy-base:initial_owner").String(),
		Pauser:              c.GenerateAddress("strategy-base:pauser").String(),
		Unpauser:            c.GenerateAddress("strategy-base:unpauser").String(),
		StrategyManager:     strategyManager,
		UnderlyingToken:     underlyingToken,
	}

	return deployCrate(c, "bvs-strategy-base", initMsg, "BVS Strategy Base")
}

package bvs

import (
	"encoding/json"

	"github.com/satlayer/satlayer-bvs/babylond"

	delegationmanager "github.com/satlayer/satlayer-bvs/bvs-cw/delegation-manager"
	"github.com/satlayer/satlayer-bvs/bvs-cw/directory"
	"github.com/satlayer/satlayer-bvs/bvs-cw/registry"
	rewardscoordinator "github.com/satlayer/satlayer-bvs/bvs-cw/rewards-coordinator"
	slashmanager "github.com/satlayer/satlayer-bvs/bvs-cw/slash-manager"
	strategybase "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-base"
	strategyfactory "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-factory"
	strategymanager "github.com/satlayer/satlayer-bvs/bvs-cw/strategy-manager"
)

type Contract[T interface{}] struct {
	babylond.DeployedWasmContract
	InstantiateMsg T
}

type Deployer struct {
	*babylond.BabylonContainer
}

func deployCrate[T interface{}](deployer *Deployer, crate string, initMsg T, label string) *Contract[T] {
	wasmByteCode, err := ReadArtifact(crate)
	if err != nil {
		panic(err)
	}

	initBytes, err := json.Marshal(initMsg)
	if err != nil {
		panic(err)
	}

	contract, err := deployer.StoreAndInitWasm(wasmByteCode, initBytes, label, "genesis")
	if err != nil {
		panic(err)
	}
	return &Contract[T]{
		DeployedWasmContract: *contract,
		InstantiateMsg:       initMsg,
	}
}

// TODO(fuxingloh): implement Deployer.Deploy()

func (d *Deployer) DeployRegistry(
	initMsg *registry.InstantiateMsg,
) *Contract[registry.InstantiateMsg] {
	if initMsg == nil {
		initMsg = &registry.InstantiateMsg{
			InitialPaused: false,
			Owner:         d.GenerateAddress("registry:owner").String(),
		}
	}

	return deployCrate(d, "bvs-registry", *initMsg, "BVS Registry")
}

// TODO: Too much initialization. Some can be moved to `ExecuteMsg` instead of `InstantiateMsg`

func (d *Deployer) DeploySlashManager(
	delegationManager string, strategyManager string,
) *Contract[slashmanager.InstantiateMsg] {
	initMsg := slashmanager.InstantiateMsg{
		InitialPausedStatus: 0,
		InitialOwner:        d.GenerateAddress("slash-manager:initial_owner").String(),
		Pauser:              d.GenerateAddress("slash-manager:pauser").String(),
		Unpauser:            d.GenerateAddress("slash-manager:unpauser").String(),
		StrategyManager:     strategyManager,
		DelegationManager:   delegationManager,
	}

	return deployCrate(d, "bvs-slash-manager", initMsg, "BVS Slash Manager")
}

func (d *Deployer) DeployStrategyManager(
	delegationManager string,
	slashManager string,
	initialStrategyWhitelister string,
) *Contract[strategymanager.InstantiateMsg] {
	initMsg := strategymanager.InstantiateMsg{
		InitialPausedStatus:        0,
		InitialOwner:               d.GenerateAddress("strategy-manager:initial_owner").String(),
		Pauser:                     d.GenerateAddress("strategy-manager:pauser").String(),
		Unpauser:                   d.GenerateAddress("strategy-manager:unpauser").String(),
		DelegationManager:          delegationManager,
		SlashManager:               slashManager,
		InitialStrategyWhitelister: initialStrategyWhitelister,
	}

	return deployCrate(d, "bvs-strategy-manager", initMsg, "BVS Strategy Manager")
}

func (d *Deployer) DeployDelegationManager(
	slashManager string,
	strategyManager string,
	minWithdrawalDelayBlocks int64,
	strategies []string,
	withdrawalDelayBlocks []int64,
) *Contract[delegationmanager.InstantiateMsg] {
	initMsg := delegationmanager.InstantiateMsg{
		InitialPausedStatus:      0,
		InitialOwner:             d.GenerateAddress("delegation-manager:initial_owner").String(),
		Pauser:                   d.GenerateAddress("delegation-manager:pauser").String(),
		Unpauser:                 d.GenerateAddress("delegation-manager:unpauser").String(),
		SlashManager:             slashManager,
		StrategyManager:          strategyManager,
		MinWithdrawalDelayBlocks: minWithdrawalDelayBlocks,
		Strategies:               strategies,
		WithdrawalDelayBlocks:    withdrawalDelayBlocks,
	}

	return deployCrate(d, "bvs-delegation-manager", initMsg, "BVS Delegation Manager")
}

func (d *Deployer) DeployDirectory(
	delegationManager string,
	registry string,
) *Contract[directory.InstantiateMsg] {
	initMsg := directory.InstantiateMsg{
		InitialOwner:      d.GenerateAddress("directory:initial_owner").String(),
		DelegationManager: delegationManager,
		Registry:          registry,
	}

	return deployCrate(d, "bvs-directory", initMsg, "BVS Directory")
}

func (d *Deployer) DeployRewardsCoordinator(
	delegationManager string,
	strategyManager string,
	activationDelay int64,
	calculationIntervalSeconds int64,
	genesisRewardsTimestamp int64,
	maxFutureLength int64,
	maxRetroactiveLength int64,
	maxRewardsDuration int64,
	rewardsUpdater string,
) *Contract[rewardscoordinator.InstantiateMsg] {
	initMsg := rewardscoordinator.InstantiateMsg{
		InitialPausedStatus:        0,
		InitialOwner:               d.GenerateAddress("rewards-coordinator:initial_owner").String(),
		Pauser:                     d.GenerateAddress("rewards-coordinator:pauser").String(),
		Unpauser:                   d.GenerateAddress("rewards-coordinator:unpauser").String(),
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

	return deployCrate(d, "bvs-rewards-coordinator", initMsg, "BVS Rewards Coordinator")
}

func (d *Deployer) DeployStrategyFactory(
	strategyManager string, strategyCodeId int64,
) *Contract[strategyfactory.InstantiateMsg] {
	initMsg := strategyfactory.InstantiateMsg{
		InitialPausedStatus: 0,
		InitialOwner:        d.GenerateAddress("strategy-factory:initial_owner").String(),
		Pauser:              d.GenerateAddress("strategy-factory:pauser").String(),
		Unpauser:            d.GenerateAddress("strategy-factory:unpauser").String(),
		StrategyManager:     strategyManager,
		StrategyCodeID:      strategyCodeId,
	}

	return deployCrate(d, "bvs-strategy-factory", initMsg, "BVS Strategy Factory")
}

func (d *Deployer) DeployStrategyBase(
	underlyingToken string, strategyManager string,
) *Contract[strategybase.InstantiateMsg] {
	initMsg := strategybase.InstantiateMsg{
		InitialPausedStatus: 0,
		InitialOwner:        d.GenerateAddress("strategy-base:initial_owner").String(),
		Pauser:              d.GenerateAddress("strategy-base:pauser").String(),
		Unpauser:            d.GenerateAddress("strategy-base:unpauser").String(),
		StrategyManager:     strategyManager,
		UnderlyingToken:     underlyingToken,
	}

	return deployCrate(d, "bvs-strategy-base", initMsg, "BVS Strategy Base")
}

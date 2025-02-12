package core

type Config struct {
	Chain             Chain
	Account           Account
	Contract          Contract
	StakerOperatorMap []StakerOperatorMap
}

type Chain struct {
	ID  string `json:"id"`
	RPC string `json:"rpc"`
}

type Account struct {
	KeyDir                 string   `json:"keyDir"`
	KeyringBackend         string   `json:"keyringBackend"`
	Bech32Prefix           string   `json:"bech32Prefix"`
	CallerKeyName          string   `json:"callerKeyName"`
	OperatorsKeyName       []string `json:"operatorsKeyName"`
	UploaderKeyName        string   `json:"uploaderKeyName"`
	AggregatorKeyName      string   `json:"aggregatorKeyName"`
	StakersKeyName         []string `json:"stakersKeyName"`
	ApproverKeyName        string   `json:"approverKeyName"`
	StrategyManagerKeyName string   `json:"strategyManagerKeyName"`
}

type Contract struct {
	BVSContractAddr       string `json:"bvsContractAddr"`
	BVSDriverAddr         string `json:"bvsDriverAddr"`
	StateBankAddr         string `json:"stateBankAddr"`
	DirectoryAddr         string `json:"directoryAddr"`
	RewardCoordinatorAddr string `json:"rewardCoordinatorAddr"`
	DelegationManagerAddr string `json:"delegationManagerAddr"`
	StrategyMangerAddr    string `json:"strategyMangerAddr"`
	StrategyAddr          string `json:"strategyAddr"`
	Cw20TokenAddr         string `json:"cw20TokenAddr"`
}

type StakerOperatorMap struct {
	StakerKeyName   string `json:"stakerKeyName"`
	Amount          uint64 `json:"amount"`
	OperatorKeyName string `json:"operatorKeyNames"`
}

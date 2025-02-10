package conf

type Conf struct {
	LogLevel string `json:"logLevel"`
	Account  Account
	Chain    Chain
	Contract Contract
}

type Account struct {
	KeyDir         string `json:"keyDir"`
	KeyringBackend string `json:"keyringBackend"`
	Bech32Prefix   string `json:"bech32Prefix"`
	EVMKeyDir      string `json:"evmKeyDir"`
}

type Chain struct {
	ID        string `json:"id"`
	RPC       string `json:"rpc"`
	EVMRPC    string `json:"evmRPC"`
	RewardAPI string `json:"rewardAPI"`
}

type Contract struct {
	Directory         string `json:"directory"`
	DirectoryEVM      string `json:"directoryEVM"`
	Delegation        string `json:"delegation"`
	Strategy          string `json:"strategy"`
	StrategyBase      string `json:"strategyBase"`
	RewardCoordinator string `json:"RewardCoordinator"`
	BVSDriver         string `json:"BVSDriver"`
	BVSDriverEVM      string `json:"BVSDriverEVM"`
	StateBank         string `json:"stateBank"`
	StateBankEVM      string `json:"stateBankEVM"`
	Cw20              string `json:"cw20"`
	Slash             string `json:"slashManager"`
	StrategyBaseTVL   string `json:"strategyBaseTVL"`
	StrategyFactory   string `json:"strategyFactory"`
	SlashEVM          string `json:"slashEVM"`
}

var content = `
logLevel = "info"

[account]
keyDir = "{keyDir}"
keyringBackend = "test"
bech32Prefix = "bbn"
evmKeyDir = "~/.eth/keystore"

[chain]
id = "sat-bbn-testnet1" # babylon chain id
rpc = "https://rpc.sat-bbn-testnet1.satlayer.net" # babylon chain rpc url
evmRPC = "https://arbitrum-sepolia-rpc.publicnode.com"
rewardAPI = "https://dev-reward.satlayer.net"

[contract]
directory = "bbn1hr2m4e6jwlplhwq0r8khyqnvvsquvsd8u44hmn0k4v99rcrmgd0slhmxn7"
directoryEVM = "0x3f1Ef09e5de85dBdE2a75d6110D3AD4571A5A28c"
delegation = "bbn1e6wth7499p3y4y7h45nx4dkjlm62gn8shgjtrezymdepz58ne3tq58hkmv"
strategy = "bbn1txnzllunz2syce9kdg6zev4g2t90afz53xs944vcan9pl7flp2lqfkuq62"
strategyBase = "bbn14rruau4y52cqyag6d9pxa3rrwhhh9xu7egndpafu55ztd8dprj8s860s8w"
rewardCoordinator = "bbn1xwpk5mrrrm7zsl606mhdj5lmtmegcu9c72ve7hyd7kf7n3v2jnrq2wgyxf"
stateBank = "bbn1h9zjs2zr2xvnpngm9ck8ja7lz2qdt5mcw55ud7wkteycvn7aa4pqpghx2q"
stateBankEVM = "0xD39fE868cCa5732c8684326C7D7E4c5E5A372825"
BVSDriver = "bbn18x5lx5dda7896u074329fjk4sflpr65s036gva65m4phavsvs3rqk5e59c"
BVSDriverEVM = "0x7c2a52ec56b5b2Ef053Fdd20AF84475559d8C901"
cw20 = "bbn1mx295r0mph0xvetqqcapsj4xxreg9mek7nlzhcacu4y0r83hhxfqu9mn0v"
slash = "bbn1kyjqfdnnmlqe0xt78rg4wtpfyhznzc60887rfuf2kr079hga3htqgzjhxw"
slashEVM = "0x66E371A5dfF96fcC88864F292ecfC85a5B853176"
strategyBaseTVL = "bbn108l2c6l5aw0pv68mhq764kq9344h4prefft4uufelmweasfstfzsxv0w5p"
strategyFactory = "bbn1x7v4jf9ezmy9zy7yzjqv4njy0ef3q8np0dey6agj67wznsa90zdslzvyxs"
`

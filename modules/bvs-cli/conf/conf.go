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
}

type Chain struct {
	ID  string `json:"id"`
	RPC string `json:"rpc"`
}

type Contract struct {
	Directory         string `json:"directory"`
	Delegation        string `json:"delegation"`
	Strategy          string `json:"strategy"`
	StrategyBase      string `json:"strategyBase"`
	RewardCoordinator string `json:"RewardCoordinator"`
	Cw20              string `json:"cw20"`
	Slash             string `json:"slashManager"`
	StrategyBaseTVL   string `json:"strategyBaseTVL"`
	StrategyFactory   string `json:"strategyFactory"`
}

var content = `
logLevel = "info"

[account]
keyDir = "{keyDir}"
keyringBackend = "os"
bech32Prefix = "bbn"

[chain]
id = "sat-bbn-testnet1" # chain id
rpc = "https://rpc.sat-bbn-testnet1.satlayer.net" # chain rpc url

[contract]
directory = "bbn1f803xuwl6l7e8jm9ld0kynvvjfhfs5trax8hmrn4wtnztglpzw0sm72xua"
delegation = "bbn1q7v924jjct6xrc89n05473juncg3snjwuxdh62xs2ua044a7tp8sydugr4"
strategy = "bbn1mju0w4qagjcgtrgepr796zmg083qurq9sngy0eyxm8wzf78cjt3qzfq7qy"
strategyBase = "bbn14x6qg6aus8jn6je8zq7fhpvaq8uz4c75dfh3zwcf8736ukc076rse9w8jy"
rewardCoordinator = "bbn1v9gyy4nzegj8z2w63gdkrtathenkqvght3yaa72edkp0rs5aks3sfkyg0t"
cw20 = "bbn1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sp4gequ"
slash = "bbn1z52hmh7ht0364lzcs8700sgrnns84sa3wr9c8upd80es5n5x65mq2dedfp"
strategyBaseTVL = "bbn108l2c6l5aw0pv68mhq764kq9344h4prefft4uufelmweasfstfzsxv0w5p"
strategyFactory = "bbn1x7v4jf9ezmy9zy7yzjqv4njy0ef3q8np0dey6agj67wznsa90zdslzvyxs"
`

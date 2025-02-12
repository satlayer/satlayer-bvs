package core

type Config struct {
	Chain      Chain
	Owner      Owner
	Aggregator Aggregator
}

type Chain struct {
	ID           string `json:"id"`
	RPC          string `json:"rpc"`
	BVSHash      string `json:"bvsHash"`
	BVSDirectory string `json:"bvsDirectory"`
	BVSDriver    string `json:"bvsDriver"`
	StateBank    string `json:"stateBank"`
}

type Owner struct {
	KeyDir         string `json:"keyDir"`
	KeyName        string `json:"keyName"`
	KeyringBackend string `json:"keyringBackend"`
	Bech32Prefix   string `json:"bech32Prefix"`
}

type Aggregator struct {
	URL string `json:"url"`
}

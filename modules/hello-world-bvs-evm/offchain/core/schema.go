package core

type Config struct {
	Chain      Chain
	Owner      Owner
	Aggregator Aggregator
}

type Chain struct {
	RPC          string `json:"rpc"`
	BVSHash      string `json:"bvsHash"`
	BVSDirectory string `json:"bvsDirectory"`
	BVSDriver    string `json:"bvsDriver"`
	StateBank    string `json:"stateBank"`
}

type Owner struct {
	KeyDir   string `json:"keyDir"`
	UserAddr string `json:"keyName"`
	Password string `json:"password"`
}

type Aggregator struct {
	URL string `json:"url"`
}

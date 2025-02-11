package core

type Config struct {
	Chain Chain
	Owner Owner
}

type Chain struct {
	RPC          string `json:"rpc"`
	BVSHash      string `json:"bvsHash"`
	BVSDirectory string `json:"bvsDirectory"`
}

type Owner struct {
	KeyDir   string `json:"keyDir"`
	UserAddr string `json:"keyName"`
	Password string `json:"password"`
}

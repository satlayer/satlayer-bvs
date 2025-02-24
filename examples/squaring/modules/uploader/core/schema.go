package core

import "github.com/go-redis/redis/v8"

type Config struct {
	Chain    Chain
	Owner    Owner
	Database Database
	Reward   Reward
}

type Chain struct {
	ID                string `json:"id"`
	RPC               string `json:"rpc"`
	InitBlockNum      uint64 `json:"initBlockNum"`
	RewardCoordinator string `json:"rewardCoordinator"`
	BVSHash           string `json:"bvsHash"`
	BVSDirectory      string `json:"bvsDirectory"`
	DelegationManager string `json:"delegationManager"`
}

type Owner struct {
	KeyDir         string `json:"keyDir"`
	KeyName        string `json:"keyName"`
	KeyringBackend string `json:"keyringBackend"`
	Bech32Prefix   string `json:"bech32Prefix"`
}

type Reward struct {
	Amount           float64 `json:"amount"`
	OperatorRatio    float64 `json:"operatorRatio"`
	OperatorStrategy string  `json:"operatorStrategy"`
}

type Database struct {
	RedisHost     string `json:"redisHost"`
	RedisPassword string `json:"redisPassword"`
	RedisDB       int    `json:"redisDB"`
}

type Store struct {
	RedisConn *redis.Client
}

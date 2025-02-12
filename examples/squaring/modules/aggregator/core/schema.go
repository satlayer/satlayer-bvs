package core

import "github.com/go-redis/redis/v8"

type Task struct {
	TaskID     uint64     `json:"taskID"`
	TaskResult TaskResult `json:"taskResult"`
}

type TaskResult struct {
	Operator string `json:"operator"`
	Result   int64  `json:"result"`
}

type Config struct {
	App      App
	Database Database
	Chain    Chain
	Owner    Owner
}
type App struct {
	Env       string
	Host      string
	Threshold uint
}

type Database struct {
	RedisHost     string `json:"redisHost"`
	RedisPassword string `json:"redisPassword"`
	RedisDB       int    `json:"redisDB"`
}

type Chain struct {
	ID           string `json:"id"`
	RPC          string `json:"rpc"`
	BvsHash      string `json:"bvsHash"`
	BvsDirectory string `json:"bvsDirectory"`
}

type Owner struct {
	KeyDir         string `json:"keyDir"`
	KeyName        string `json:"keyName"`
	KeyringBackend string `json:"keyringBackend"`
	Bech32Prefix   string `json:"bech32Prefix"`
}

type Store struct {
	RedisConn *redis.Client
}

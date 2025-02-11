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
	RPC          string `json:"rpc"`
	BVSHash      string `json:"bvsHash"`
	BVSDirectory string `json:"bvsDirectory"`
}

type Owner struct {
	KeyDir   string `json:"keyDir"`
	UserAddr string `json:"keyName"`
	Password string `json:"password"`
}

type Store struct {
	RedisConn *redis.Client
}

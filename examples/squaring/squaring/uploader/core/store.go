package core

import (
	"context"
	"fmt"
	"time"

	"github.com/go-redis/redis/v8"
)

// initStore Initializes the store with a Redis connection.
//
// db is a pointer to the Database struct containing Redis connection details.
// No return values.
func initStore(db *Database) {
	rdb := initRedis(db)
	S = Store{
		RedisConn: rdb,
	}
}

// initRedis Initializes a Redis client connection.
//
// db is a pointer to the Database struct containing Redis connection details.
// Returns a pointer to a redis.Client.
func initRedis(db *Database) *redis.Client {
	rdb := redis.NewClient(&redis.Options{
		Addr:     db.RedisHost,
		Password: db.RedisPassword,
		DB:       db.RedisDB,
	})
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	// test connection
	err := rdb.Ping(ctx).Err()
	if err != nil {
		panic(fmt.Sprintf("Could not connect to Redis: %v", err))
	}
	return rdb
}

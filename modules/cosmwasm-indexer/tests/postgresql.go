package tests

import (
	"context"
	"fmt"
	"log"
	"path/filepath"
	"time"

	"github.com/docker/go-connections/nat"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/modules/postgres"
	"github.com/testcontainers/testcontainers-go/wait"
)

const (
	dbName     = "indexer"
	dbUser     = "user"
	dbPassword = "password"
	dbPort     = "5432"
	image      = "postgres:16-alpine3.20"
)

type PostgreSQLContainer struct {
	testcontainers.Container
	URL string
}

func getHost(ctx context.Context, container testcontainers.Container, port nat.Port) string {
	host, err := container.Host(ctx)
	if err != nil {
		panic(err)
	}
	port, err = container.MappedPort(ctx, port)
	if err != nil {
		panic(err)
	}
	return fmt.Sprintf("%s:%s", host, port.Port())
}

func Run(ctx context.Context) *PostgreSQLContainer {
	container, err := postgres.Run(ctx,
		image,
		postgres.WithInitScripts(filepath.Join("testdata", "init-db.sh")),
		postgres.WithConfigFile(filepath.Join("testdata", "postgres.conf")),
		postgres.WithDatabase(dbName),
		postgres.WithUsername(dbUser),
		postgres.WithPassword(dbPassword),
		testcontainers.WithWaitStrategy(
			wait.ForLog("Database system is ready to accept connections").
				WithOccurrence(2).
				WithStartupTimeout(5*time.Second)),
	)
	if err != nil {
		log.Printf("Failed to start PostgreSQL container: %s", err)
		panic(err)
	}

	uri := fmt.Sprintf("%s", getHost(ctx, container, dbPort))
	return &PostgreSQLContainer{
		Container: container,
		URL:       fmt.Sprintf("postgresql://postgres:%s@%s/%s?sslmode=disable&search_path=public", dbPassword, uri, dbName),
	}
}

// func ExampleRun() {
// 	// runPostgresContainer {
// 	ctx := context.Background()

// 	postgresContainer, err := postgres.Run(ctx,
// 		image,
// 		postgres.WithInitScripts(filepath.Join("testdata", "init-user-db.sh")),
// 		postgres.WithConfigFile(filepath.Join("testdata", "my-postgres.conf")),
// 		postgres.WithDatabase(dbName),
// 		postgres.WithUsername(dbUser),
// 		postgres.WithPassword(dbPassword),
// 		testcontainers.WithWaitStrategy(
// 			wait.ForLog("database system is ready to accept connections").
// 				WithOccurrence(2).
// 				WithStartupTimeout(5*time.Second)),
// 	)

// 	defer func() {
// 		if err := testcontainers.TerminateContainer(postgresContainer); err != nil {
// 			log.Printf("failed to terminate container: %s", err)
// 		}
// 	}()
// 	if err != nil {
// 		log.Printf("failed to start container: %s", err)
// 		return
// 	}
// 	// }

// 	state, err := postgresContainer.State(ctx)
// 	if err != nil {
// 		log.Printf("failed to get container state: %s", err)
// 		return
// 	}

// 	fmt.Println(state.Running)

// 	// Output:
// 	// true
// }

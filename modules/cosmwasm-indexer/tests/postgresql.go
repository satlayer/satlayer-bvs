package tests

import (
	"context"
	"fmt"
	"path/filepath"
	"time"

	"github.com/docker/go-connections/nat"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/modules/postgres"
	"github.com/testcontainers/testcontainers-go/wait"
)

const (
	dbName     = "indexer"
	dbUser     = "docker"
	dbPassword = "password"
	dbPort     = "5432"
	image      = "postgres:16-alpine3.20"
)

type PostgreSQLContainer struct {
	testcontainers.Container
	URL string
}

func getHost(ctx context.Context, container testcontainers.Container, port nat.Port) string {
	host, err := container.ContainerIP(ctx)
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
			wait.ForLog("database system is ready to accept connections").
				WithStartupTimeout(30*time.Second).
				WithPollInterval(1*time.Second)),
	)
	if err != nil {
		panic(fmt.Errorf("failed to start PostgreSQL container: %s", err))
	}

	uri := fmt.Sprintf("%s", getHost(ctx, container, dbPort))
	return &PostgreSQLContainer{
		Container: container,
		URL: fmt.Sprintf("postgresql://%s:%s@%s/%s?sslmode=disable&search_path=public",
			dbUser, dbPassword, uri, dbName),
	}
}

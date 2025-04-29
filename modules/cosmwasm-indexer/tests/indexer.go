package tests

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/wait"
)

const (
	indexerImage = "satlayer/cosmwasm-indexer:latest"
	indexerPort  = "8080"
)

type IndexerContainer struct {
	testcontainers.Container
	URL string
}

func RunIndexer(ctx context.Context, postgresURL string) *IndexerContainer {
	// 定义容器请求
	req := testcontainers.ContainerRequest{
		Image: indexerImage,
		Env: map[string]string{
			"POSTGRES_URL": postgresURL,
			// 添加其他必要的环境变量
		},
		ExposedPorts: []string{fmt.Sprintf("%s/tcp", indexerPort)},
		WaitingFor: wait.ForLog("Starting indexer").
			WithStartupTimeout(30 * time.Second),
	}

	// 启动容器
	container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: req,
		Started:          true,
	})
	if err != nil {
		log.Printf("Failed to start indexer container: %s", err)
		panic(err)
	}

	// 获取容器URL
	host, err := container.Host(ctx)
	if err != nil {
		panic(err)
	}
	port, err := container.MappedPort(ctx, indexerPort)
	if err != nil {
		panic(err)
	}

	return &IndexerContainer{
		Container: container,
		URL:       fmt.Sprintf("http://%s:%s", host, port.Port()),
	}
}

func (c *IndexerContainer) Cleanup(ctx context.Context) {
	if err := c.Terminate(ctx); err != nil {
		log.Printf("Failed to terminate indexer container: %s", err)
	}
}

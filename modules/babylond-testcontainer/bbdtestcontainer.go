package bbdtestcontainer

import (
	"context"
	"fmt"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	"github.com/testcontainers/testcontainers-go"
	"time"
)

type BabylondContainer struct {
	Container testcontainers.Container
	Host      string
	ChainIO   io.ChainIO
	// TODO: add http client + grpc client
}

const (
	apiPort  = "1317"
	grpcPort = "9090"
	rpcPort  = "26657"

	BabylonHomePath   = "/home/babylon/.babylond"
	BabylonOutputPath = ".localnet"
	DockerVolume      = "babylon-node"
	ChainId           = "localnet" // TODO: update chain-id to be unique
)

func (bc *BabylondContainer) getAPIURL() string {
	port, err := bc.Container.MappedPort(context.Background(), apiPort)
	if err != nil {
		panic(err)
	}
	return fmt.Sprintf("http://%s:%s", bc.Host, port.Port())
}

func (bc *BabylondContainer) getGRPCURL() string {
	port, err := bc.Container.MappedPort(context.Background(), grpcPort)
	if err != nil {
		panic(err)
	}
	return fmt.Sprintf("grcp://%s:%s", bc.Host, port.Port())
}

func (bc *BabylondContainer) getRPCURL() string {
	port, err := bc.Container.MappedPort(context.Background(), rpcPort)
	if err != nil {
		panic(err)
	}
	return fmt.Sprintf("http://%s:%s", bc.Host, port.Port())
}

func CreateBabylondContainer(ctx context.Context) (*BabylondContainer, error) {

	babylondInitCommand := fmt.Sprintf("babylond testnet --output-dir %s --keyring-backend test --chain-id %s --home %s ", BabylonOutputPath, ChainId, BabylonHomePath)

	babylondStartCommand := fmt.Sprintf("babylond start --home %s/node0/babylond", BabylonOutputPath)

	// API port is 1317
	// GRPC port is 9090
	// RPC port is 26657
	container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: testcontainers.ContainerRequest{
			Image: "babylonlabs/babylond:v1.0.0-rc.5",
			Entrypoint: []string{
				"sh",
				"-c",
			},
			Cmd: []string{
				babylondInitCommand +
					"&&" + babylondStartCommand,
			},
			ExposedPorts: []string{apiPort, grpcPort, rpcPort},
			Mounts: testcontainers.ContainerMounts{
				{
					Source: testcontainers.GenericVolumeMountSource{
						Name: DockerVolume,
					},
					Target: "/home/babylon",
				},
			},
		},
		Started: true,
	})
	if err != nil {
		return nil, err
	}

	var bc *BabylondContainer

	if container != nil {
		bc = &BabylondContainer{Container: container}
	}

	host, err := container.Host(context.Background())
	if err != nil {
		return nil, err
	}
	bc.Host = host

	homeDir := "../.babylon"

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "io")
	chainIO, err := io.NewChainIO(ChainId, bc.getAPIURL(), homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		return nil, err
	}
	bc.ChainIO = chainIO

	return bc, nil
}

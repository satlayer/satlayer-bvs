package babylond

import (
	"context"
	"fmt"
	"github.com/docker/go-connections/nat"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/wait"
	"strings"
	"time"
)

const (
	apiPort  = "1317"
	grpcPort = "9090"
	rpcPort  = "26657"

	BabylonHomePath   = "/home/babylon/.babylond"
	BabylonOutputPath = "sat-bbn-localnet"
	ChainId           = "sat-bbn-localnet"
)

type BabylonContainer struct {
	Ctx       context.Context
	Container testcontainers.Container
	// TODO: add http client + grpc client
}

func (d *BabylonContainer) getHost(port nat.Port) string {
	// Technically, Container.Host should be Container.Hostname
	host, err := d.Container.Host(d.Ctx)
	if err != nil {
		panic(err)
	}
	port, err = d.Container.MappedPort(d.Ctx, port)
	if err != nil {
		panic(err)
	}
	return fmt.Sprintf("%s:%s", host, port.Port())
}

func (d *BabylonContainer) GetApiEndpoint() string {
	return fmt.Sprintf("http://%s", d.getHost(apiPort))
}

func (d *BabylonContainer) GetRpcUrl() string {
	return fmt.Sprintf("http://%s", d.getHost(rpcPort))
}

func (d *BabylonContainer) GetGrpcEndpoint() string {
	return fmt.Sprintf("grcp://%s", d.getHost(grpcPort))
}

func (d *BabylonContainer) GetChainIO() (io.ChainIO, error) {
	homeDir := "../.babylon"
	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "io")
	return io.NewChainIO(ChainId, d.GetRpcUrl(), homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          2 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
}

func Run(ctx context.Context) (*BabylonContainer, error) {
	// Setup Testnet
	initCmd := []string{
		"babylond",
		"testnet",
		"--v",
		"1",
		"--output-dir",
		BabylonOutputPath,
		"--keyring-backend",
		"test",
		"--chain-id",
		ChainId,
		"--home",
		BabylonHomePath,
	}

	startCmd := []string{
		"babylond",
		"start",
		"--home",
		fmt.Sprintf("%s/node0/babylond", BabylonOutputPath),
	}

	container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: testcontainers.ContainerRequest{
			Image: "babylonlabs/babylond:v1.0.0-rc.5",
			// TODO(fuxingloh): change entrypoint to babylond
			Entrypoint: []string{
				"sh",
				"-c",
			},
			Cmd: []string{
				strings.Join(initCmd[:], " ") + " && " + strings.Join(startCmd[:], " "),
			},
			ExposedPorts: []string{apiPort, grpcPort, rpcPort},
			WaitingFor:   wait.ForHTTP("/status").WithPort(rpcPort),
		},
		Started: true,
	})

	if err != nil {
		return nil, err
	}

	return &BabylonContainer{
		Ctx:       ctx,
		Container: container,
	}, nil
}

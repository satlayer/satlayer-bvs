package babylond

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/cometbft/cometbft/rpc/client/http"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/docker/go-connections/nat"
	"github.com/prometheus/client_golang/prometheus"
	rio "io"
	// TODO(fuxingloh): should remove internal dependencies from this package
	//  this package should act as a standalone package for running babylond
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
	ChainId  = "sat-bbn-localnet"
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

func (d *BabylonContainer) GetRpcClient() (*http.HTTP, error) {
	endpoint := d.GetRpcUrl()
	return client.NewClientFromNode(endpoint)
}

// Deprecated: GetChainIO is deprecated, use GetRpcClient instead
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
	// Setup Testnet Config
	initCmd := []string{
		"babylond",
		"testnet",
		"--v",
		"1",
		"--blocks-per-year",
		"31536000", // one block per second
		"--output-dir",
		".localnet",
		"--keyring-backend",
		"test",
		"--chain-id",
		ChainId,
	}

	startCmd := []string{
		"babylond",
		"start",
		"--home",
		".localnet/node0/babylond",
	}

	container, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: testcontainers.ContainerRequest{
			Image: "babylonlabs/babylond:v1.0.0-rc.5",
			Entrypoint: []string{
				"sh",
				"-c",
			},
			Cmd: []string{
				strings.Join(initCmd[:], " ") + " && " + strings.Join(startCmd[:], " "),
			},
			ExposedPorts: []string{apiPort, grpcPort, rpcPort},
			WaitingFor: wait.ForHTTP("/status").
				WithPort(rpcPort).WithResponseMatcher(func(body rio.Reader) bool {
				var data map[string]interface{}
				if err := json.NewDecoder(body).Decode(&data); err != nil {
					return false
				}
				data = data["result"].(map[string]interface{})
				data = data["sync_info"].(map[string]interface{})
				height := data["latest_block_height"].(string)
				return height != "0"
			}),
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

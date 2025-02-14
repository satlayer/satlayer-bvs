package babylond

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/docker/go-connections/nat"
	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/wait"
	"io"
	"strings"
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

func (d *BabylonContainer) GetApiUri() string {
	return fmt.Sprintf("http://%s", d.getHost(apiPort))
}

func (d *BabylonContainer) GetRpcUri() string {
	return fmt.Sprintf("http://%s", d.getHost(rpcPort))
}

func (d *BabylonContainer) GetGrpcUri() string {
	return fmt.Sprintf("grcp://%s", d.getHost(grpcPort))
}

func (d *BabylonContainer) ClientCtx() client.Context {
	nodeUri := d.GetRpcUri()
	rpcClient, err := client.NewClientFromNode(nodeUri)
	if err != nil {
		panic(err)
	}

	return client.Context{}.
		WithChainID(ChainId).
		WithClient(rpcClient).
		WithOutputFormat("json").
		WithBroadcastMode(flags.BroadcastSync)
}

func Run(ctx context.Context) (*BabylonContainer, error) {
	cmd := []string{
		// Setup Testnet
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
		"&&",
		// Setup Keyring
		"babylond",
		"keys",
		"import-hex",
		"genesis",
		"230FAE50A4FFB19125F89D8F321996A46F805E7BCF0CDAC5D102E7A42741887A",
		"--keyring-backend",
		"test",
		"--home",
		".localnet/node0/babylond",
		"&&",
		//// Setup Genesis Account
		"babylond",
		"add-genesis-account",
		"bbn1lmnc4gcvcu5dexa8p6vv2e6qkas5lu2r2nwlnv",
		"1000000ubbn",
		"--home",
		".localnet/node0/babylond",
		"&&",
		// Start babylond
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
				strings.Join(cmd[:], " "),
			},
			ExposedPorts: []string{apiPort, grpcPort, rpcPort},
			WaitingFor: wait.ForHTTP("/status").
				WithPort(rpcPort).WithResponseMatcher(func(body io.Reader) bool {
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

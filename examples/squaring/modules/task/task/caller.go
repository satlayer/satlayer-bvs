package task

import (
	"context"
	"fmt"
	"math/rand"
	"time"

	squaringcontract "github.com/satlayer/satlayer-bvs/examples/squaring/squaring-contract"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	"github.com/satlayer/satlayer-bvs/examples/squaring/task/core"
)

type Caller struct {
	bvsContract string
	chainIO     io.ChainIO
}

// RunCaller runs the caller by creating a new caller and executing its Run method.
//
// No parameters.
// No return.
func RunCaller() {
	c := NewCaller()
	c.Run()
}

// NewCaller creates a new Caller instance.
//
// Returns a pointer to Caller.
func NewCaller() *Caller {
	chainIO, err := io.NewChainIO(core.C.Chain.ID, core.C.Chain.RPC, core.C.Owner.KeyDir, core.C.Owner.Bech32Prefix, types.TxManagerParams{
		MaxRetries:             5,
		RetryInterval:          3 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	if err != nil {
		panic(err)
	}

	client, err := chainIO.SetupKeyring(core.C.Owner.KeyName, core.C.Owner.KeyringBackend)
	if err != nil {
		panic(err)
	}

	return &Caller{
		bvsContract: core.C.Chain.BVSContract,
		chainIO:     client,
	}
}

// Run runs the caller in an infinite loop, creating a new task with a random number every second.
//
// No parameters.
// No return.
func (c *Caller) Run() {
	bvsSquaring := squaringcontract.New(c.chainIO)
	for {
		bvsSquaring.BindClient(c.bvsContract)
		randomNumber := rand.Int63n(100)
		resp, err := bvsSquaring.CreateNewTask(context.Background(), randomNumber)
		if err != nil {
			panic(err)
		}
		fmt.Printf("resp: %s\n", resp.Hash.String())
	}
}

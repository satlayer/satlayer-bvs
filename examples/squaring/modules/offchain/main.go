package main

import (
	"context"

	"github.com/satlayer/hello-world-bvs/offchain/core"
	"github.com/satlayer/hello-world-bvs/offchain/node"
)

// main is the entry point of the program.
//
// It initializes a new offchain node and runs it.
func main() {
	core.InitConfig()
	ctx := context.Background()
	n := node.NewNode()
	n.Run(ctx)
}

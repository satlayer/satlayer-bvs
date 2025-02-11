package main

import (
	"context"
	"fmt"

	"github.com/gin-gonic/gin"

	"github.com/satlayer/satlayer-bvs/hello-world-bvs-evm/aggregator/api"
	"github.com/satlayer/satlayer-bvs/hello-world-bvs-evm/aggregator/core"
	"github.com/satlayer/satlayer-bvs/hello-world-bvs-evm/aggregator/svc"
)

// main is the entry point of the program.
//
// It initializes a background context and starts two goroutines:
// - startMonitor: checks the task queue and verifies the task result.
// - startHttp: starts an HTTP server to receive operator task results.
func main() {
	core.InitConfig()
	ctx := context.Background()
	// start to check task queue and verify the task result
	go startMonitor(ctx)
	// start http server to receive operator task result
	startHTTP()
}

// startHttp starts an HTTP server to receive operator task results.
//
// It sets up routes and starts the server at the specified host.
// Returns no value.
func startHTTP() {
	router := gin.Default()
	// setup routes
	api.SetupRoutes(router)
	// start server
	core.L.Info(fmt.Sprintf("Start server at {%s}", core.C.App.Host))
	if err := router.Run(core.C.App.Host); err != nil {
		core.L.Error(fmt.Sprintf("Failed to start server due to {%s}", err))
	}
}

// startMonitor starts the task queue monitor.
//
// It initializes a new monitor and runs it with the provided context.
// No return value.
func startMonitor(ctx context.Context) {
	svc.InitMonitor()
	svc.MonitorInstance.Run(ctx)
}

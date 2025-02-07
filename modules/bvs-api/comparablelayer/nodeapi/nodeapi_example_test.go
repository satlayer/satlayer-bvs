package nodeapi_test

import (
	"github.com/satlayer/satlayer-bvs/bvs-api/comparablelayer/logging"
	"github.com/satlayer/satlayer-bvs/bvs-api/comparablelayer/nodeapi"
)

func ExampleNodeApi() {
	logger, err := logging.NewMockZapLogger("development")
	if err != nil {
		panic(err)
	}

	nodeApi := nodeapi.NewNodeApi("testBVS", "v0.0.1", "localhost:8080", logger)
	// register a service with the nodeApi. This could be a db, a cache, a queue, etc.
	nodeApi.RegisterNewService(
		"testServiceId",
		"testServiceName",
		"testServiceDescription",
		nodeapi.ServiceStatusInitializing,
	)

	// this starts the nodeApi server in a goroutine, so no need to wrap it in a go func
	nodeApi.Start()

	// ... do other stuff

	// Whenever needed, update the health of the nodeApi or of its backing services
	nodeApi.UpdateHealth(nodeapi.PartiallyHealthy)
	_ = nodeApi.UpdateServiceStatus("testServiceId", nodeapi.ServiceStatusDown)
	_ = nodeApi.DeregisterService("testServiceId")

}

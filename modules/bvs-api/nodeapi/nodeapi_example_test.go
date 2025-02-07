package nodeapi_test

import (
	elklogger "github.com/satlayer/satlayer-api/logger"
	"github.com/satlayer/satlayer-api/nodeapi"
)

func ExampleNodeApi() {
	logger := elklogger.NewMockELKLogger()

	nodeApi := nodeapi.NewNodeApi("testBVS", "v0.0.1", "localhost:8080", logger)
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

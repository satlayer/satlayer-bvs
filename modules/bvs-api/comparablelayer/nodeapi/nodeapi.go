package nodeapi

import (
	"github.com/satlayer/satlayer-api/comparablelayer/logging"
	"github.com/satlayer/satlayer-api/nodeapi"
)

type NodeHealth int

const (
	Healthy          NodeHealth = iota
	PartiallyHealthy            // either initializing or some backing services are not healthy
	Unhealthy
)

type ServiceStatus string

const (
	ServiceStatusUp           ServiceStatus = "Up"
	ServiceStatusDown         ServiceStatus = "Down"
	ServiceStatusInitializing ServiceStatus = "Initializing"
)

type nodeService struct {
	Id          string        `json:"id"`
	Name        string        `json:"name"`
	Description string        `json:"description"`
	Status      ServiceStatus `json:"status"`
}

type NodeApi struct {
	bvsNodeName     string
	bvsNodeSemVer   string
	health          NodeHealth
	nodeServices    []nodeService
	ipPortAddr      string
	logger          logging.Logger
	satlayerNodeApi *nodeapi.NodeApi
}

func NewNodeApi(bvsNodeName, bvsNodeSemVer, IpPortAddr string, logger logging.Logger) *NodeApi {
	satlayerNodeApi := nodeapi.NewNodeApi(bvsNodeName, bvsNodeSemVer, IpPortAddr, logger.GetElkLogger())
	return &NodeApi{
		bvsNodeName:     bvsNodeName,
		bvsNodeSemVer:   bvsNodeSemVer,
		health:          Healthy,
		nodeServices:    []nodeService{},
		ipPortAddr:      IpPortAddr,
		logger:          logger,
		satlayerNodeApi: satlayerNodeApi,
	}
}

func (api *NodeApi) UpdateHealth(health NodeHealth) {
	api.satlayerNodeApi.UpdateHealth(nodeapi.NodeHealth(health))
}

func (api *NodeApi) RegisterNewService(serviceId, serviceName, serviceDescription string, serviceStatus ServiceStatus) {
	api.satlayerNodeApi.RegisterNewService(serviceId, serviceName, serviceDescription, nodeapi.ServiceStatus(serviceStatus))
}

func (api *NodeApi) UpdateServiceStatus(serviceId string, serviceStatus ServiceStatus) error {
	return api.satlayerNodeApi.UpdateServiceStatus(serviceId, nodeapi.ServiceStatus(serviceStatus))
}

func (api *NodeApi) DeregisterService(serviceId string) error {
	return api.satlayerNodeApi.DeregisterService(serviceId)
}

func (api *NodeApi) Start() <-chan error {
	return api.satlayerNodeApi.Start()
}

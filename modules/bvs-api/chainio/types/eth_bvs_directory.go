package types

import "github.com/ethereum/go-ethereum/common"

type GetETHBVSInfoResp struct {
	BVSHash     string         `json:"bvsHash"`
	BVSContract common.Address `json:"bvsContract"`
}

package types

type SetReq struct {
	Set Set `json:"set"`
}

type Set struct {
	Key   string `json:"key"`
	Value int64  `json:"value"`
}

type AddRegisteredBVSContractReq struct {
	AddRegisteredBVSContract AddRegisteredBVSContract `json:"add_registered_bvs_contract"`
}

type AddRegisteredBVSContract struct {
	Address string `json:"address"`
}

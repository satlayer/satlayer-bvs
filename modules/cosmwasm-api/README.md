# CosmWasm Api

This module contains the `api` types used to interact with the SatLayer CosmWasm smart contracts.

## Query

```go
package main

import (
	"context"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/satlayer/satlayer-bvs/cosmwasm-schema/vaultbank"
)

func GetShares(clientCtx client.Context, staker string) {
	queryMsg := vaultbank.QueryMsg{
		Shares: &vaultbank.Shares{
			Staker: staker,
		},
	}

	res, err := Query[vaultbank.SharesResponse](
		clientCtx,
		context.Background(),
		"contract",
		queryMsg,
	)
}
```

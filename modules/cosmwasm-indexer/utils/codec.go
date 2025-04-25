package utils

import (
	"sync"

	"github.com/CosmWasm/wasmd/x/wasm"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/std"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/x/auth"
	"github.com/cosmos/cosmos-sdk/x/bank"
	distr "github.com/cosmos/cosmos-sdk/x/distribution"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
	"github.com/cosmos/cosmos-sdk/x/mint"
	"github.com/cosmos/cosmos-sdk/x/slashing"
	"github.com/cosmos/cosmos-sdk/x/staking"
	"github.com/cosmos/gogoproto/proto"
)

var (
	once sync.Once
	cdc  *codec.ProtoCodec
)

func GetCodec() codec.Codec {
	once.Do(func() {
		interfaceRegistry := codectypes.NewInterfaceRegistry()
		getBasicManagers().RegisterInterfaces(interfaceRegistry)
		std.RegisterInterfaces(interfaceRegistry)
		cdc = codec.NewProtoCodec(interfaceRegistry)
	})
	return cdc
}

// getBasicManagers returns the various basic managers that are used to register the encoding to
// support custom messages.
func getBasicManagers() module.BasicManager {
	return module.NewBasicManager(
		auth.AppModuleBasic{},
		genutil.NewAppModuleBasic(genutiltypes.DefaultMessageValidator),
		bank.AppModuleBasic{},
		staking.AppModuleBasic{},
		mint.AppModuleBasic{},
		distr.AppModuleBasic{},
		slashing.AppModuleBasic{},
		wasm.AppModuleBasic{},
	)
}

// UnpackMessage unpacks a message from a byte slice
func UnpackMessage[T proto.Message](cdc codec.Codec, bz []byte, ptr T) T {
	var cdcAny codectypes.Any
	cdc.MustUnmarshalJSON(bz, &cdcAny)

	var cosmosMsg sdk.Msg
	if err := cdc.UnpackAny(&cdcAny, &cosmosMsg); err != nil {
		panic(err)
	}
	return cosmosMsg.(T)
}

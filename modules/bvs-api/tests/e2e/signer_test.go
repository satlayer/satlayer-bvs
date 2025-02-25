package e2e

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/binary"
	"encoding/json"
	"testing"
	"time"

	"github.com/satlayer/satlayer-bvs/babylond"
	"github.com/satlayer/satlayer-bvs/babylond/bvs"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/signer"
	"github.com/satlayer/satlayer-bvs/bvs-cw/directory"
)

const (
	OPERATOR_BVS_REGISTRATION_TYPE_HASH = "OperatorBVSRegistration(address operator,address bvs,bytes32 salt,uint256 expiry)"
	DOMAIN_TYPE_HASH                    = "EIP712Domain(string name,uint256 chainId,address verifyingContract)"
	DOMAIN_NAME                         = "EigenLayer"
)

type signerTestSuite struct {
	suite.Suite
	chainIO         io.ChainIO
	chaiID          string
	bvsDirContrAddr string
	container       *babylond.BabylonContainer
}

func (suite *signerTestSuite) SetupSuite() {
	container := babylond.Run(context.Background())
	suite.chainIO = container.NewChainIO("../.babylon")
	suite.container = container

	// Import And Fund Caller
	container.ImportPrivKey("directory:initial_owner", "E5DBC50CB04311A2A5C3C0E0258D396E962F64C6C2F758458FFB677D7F0C0E94")
	container.FundAddressUbbn("bbn1dcpzdejnywqc4x8j5tyafv7y4pdmj7p9fmredf", 1e8)

	tAddr := container.GenerateAddress("test-address").String()
	deployer := bvs.Deployer{BabylonContainer: container}
	registry := deployer.DeployRegistry(nil)

	suite.chaiID = container.ChainId
	suite.bvsDirContrAddr = deployer.DeployDirectory(tAddr, registry.Address).Address
}

func (suite *signerTestSuite) TearDownSuite() {
	suite.Require().NoError(suite.container.Terminate(context.Background()))
}

func (suite *signerTestSuite) Test_BuildAndSignTx() {
	t := suite.T()
	keyName := "caller"
	executeMsg := directory.ExecuteMsg{
		UpdateBvsMetadataURI: &directory.UpdateBvsMetadataURI{
			MetadataURI: "http://leek.test.uri",
		},
	}
	executeMsgBytes, err := json.Marshal(executeMsg)
	assert.NoError(t, err)
	amount, err := sdktypes.ParseCoinsNormalized("")
	assert.NoError(t, err)
	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err)
	contractMsg := &wasmtypes.MsgExecuteContract{
		Sender:   account.GetAddress().String(),
		Contract: suite.bvsDirContrAddr,
		Msg:      executeMsgBytes,
		Funds:    amount,
	}

	sig := signer.NewSigner(chainIO.GetClientCtx())
	resp, err := sig.BuildAndSignTx(1.2, sdktypes.NewInt64DecCoin("ubbn", 1), 200000, "test tx", false, contractMsg)
	assert.NoError(t, err)
	t.Log(resp)
}

func (suite *signerTestSuite) Test_Sign() {
	t := suite.T()
	keyName := "caller" // Please refer to the readme to obtain
	client, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	ac, err := client.GetCurrentAccount()
	assert.NoError(t, err)
	t.Log(ac.GetAddress().String())
	signStr, err := client.GetSigner().SignByKeyName([]byte{12}, keyName)
	assert.NoError(t, err)
	t.Log(signStr)
}

func (suite *signerTestSuite) Test_VerifySignature() {
	t := suite.T()
	keyName := "caller" // Please refer to the readme to obtain

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	key, err := chainIO.GetClientCtx().Keyring.Key("caller")
	assert.NoError(t, err)
	pubKey, err := key.GetPubKey()
	assert.NoError(t, err)
	address, err := key.GetAddress()
	assert.NoError(t, err)

	salt := "salt"
	expiry := uint64(time.Now().Unix()) + 10000
	msgHash := CalculateDigestHash(pubKey.Bytes(), address.String(), []byte(salt), expiry, suite.chaiID, suite.bvsDirContrAddr)

	sig := signer.NewSigner(chainIO.GetClientCtx())
	sigStr, err := sig.Sign(msgHash)
	assert.NoError(t, err)
	v, err := signer.VerifySignature(pubKey, msgHash, sigStr)
	assert.NoError(t, err)
	t.Logf("%+v\n", v)
}

func CalculateDigestHash(operatorPublicKey []byte, bvsAddr string, salt []byte, expiry uint64, chainID, contrAddr string) []byte {
	operatorBVSRegistrationTypeHash := Sha256([]byte(OPERATOR_BVS_REGISTRATION_TYPE_HASH))
	domainTypeHash := Sha256([]byte(DOMAIN_TYPE_HASH))
	domainNameHash := Sha256([]byte(DOMAIN_NAME))

	structHashInput := bytes.Join([][]byte{
		operatorBVSRegistrationTypeHash,
		operatorPublicKey,
		[]byte(bvsAddr),
		salt,
		binary.LittleEndian.AppendUint64(nil, expiry),
	}, nil)
	structHash := Sha256(structHashInput)

	domainSeparatorInput := bytes.Join([][]byte{
		domainTypeHash,
		domainNameHash,
		[]byte(chainID),
		[]byte(contrAddr),
	}, nil)
	domainSeparator := Sha256(domainSeparatorInput)

	digestHashInput := bytes.Join([][]byte{
		[]byte{0x19, 0x01},
		domainSeparator,
		structHash,
	}, nil)

	return Sha256(digestHashInput)
}

func Sha256(data []byte) []byte {
	hasher := sha256.New()
	hasher.Write(data)
	return hasher.Sum(nil)
}

func TestSignerTestSuite(t *testing.T) {
	suite.Run(t, new(signerTestSuite))
}

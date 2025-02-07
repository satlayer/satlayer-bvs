package e2e

import (
	"bytes"
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"testing"
	"time"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/io"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"
	apilogger "github.com/satlayer/satlayer-bvs/bvs-api/logger"
	transactionprocess "github.com/satlayer/satlayer-bvs/bvs-api/metrics/indicators/transaction_process"
	"github.com/satlayer/satlayer-bvs/bvs-api/signer"
)

const (
	OPERATOR_BVS_REGISTRATION_TYPEHASH = "OperatorBVSRegistration(address operator,address bvs,bytes32 salt,uint256 expiry)"
	DOMAIN_TYPEHASH                    = "EIP712Domain(string name,uint256 chainId,address verifyingContract)"
	DOMAIN_NAME                        = "EigenLayer"
)

type TestUpdateBVSMetadataURIReq struct {
	UpdateBVSMetadataURI TestUpdateBVSMetadataURI `json:"update_b_v_s_metadata_u_r_i"`
}
type TestUpdateBVSMetadataURI struct {
	MetadataURI string `json:"metadata_uri"`
}

type signerTestSuite struct {
	suite.Suite
	chainIO         io.ChainIO
	chaiID          string
	bvsDirContrAddr string
}

func (suite *signerTestSuite) SetupTest() {
	chainID := "sat-bbn-testnet1"
	rpcURI := "https://rpc.sat-bbn-testnet1.satlayer.net"
	homeDir := "../.babylon" // Please refer to the readme to obtain

	logger := apilogger.NewMockELKLogger()
	metricsIndicators := transactionprocess.NewPromIndicators(prometheus.NewRegistry(), "signer")
	chainIO, err := io.NewChainIO(chainID, rpcURI, homeDir, "bbn", logger, metricsIndicators, types.TxManagerParams{
		MaxRetries:             3,
		RetryInterval:          1 * time.Second,
		ConfirmationTimeout:    60 * time.Second,
		GasPriceAdjustmentRate: "1.1",
	})
	suite.Require().NoError(err)
	suite.chainIO = chainIO
	suite.chaiID = chainID
	suite.bvsDirContrAddr = "bbn1f803xuwl6l7e8jm9ld0kynvvjfhfs5trax8hmrn4wtnztglpzw0sm72xua"
}

func (suite *signerTestSuite) Test_BuildAndSignTx() {
	t := suite.T()
	keyName := "caller" // Please refer to the readme to obtain
	executeMsg := TestUpdateBVSMetadataURIReq{
		UpdateBVSMetadataURI: TestUpdateBVSMetadataURI{
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
	keyName := "caller2" // Please refer to the readme to obtain
	client, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	ac, err := client.GetCurrentAccount()
	assert.NoError(t, err)
	t.Log(ac.GetAddress().String())
	ct, err := hex.DecodeString("638789a8cd83d13edab39fcf89b7044e693a5e96ee56348328f2405442ec6d09")
	assert.NoError(t, err)
	signStr, err := client.GetSigner().SignByKeyName(ct, keyName)
	assert.NoError(t, err)
	t.Log(signStr)
}

func (suite *signerTestSuite) Test_VerifySignature() {
	t := suite.T()
	keyName := "caller" // Please refer to the readme to obtain

	chainIO, err := suite.chainIO.SetupKeyring(keyName, "test")
	assert.NoError(t, err)
	account, err := chainIO.GetCurrentAccount()
	assert.NoError(t, err)

	salt := "salt"
	expiry := uint64(time.Now().Unix()) + 10000
	msgHash := CalculateDigestHash(account.GetPubKey().Bytes(), account.GetAddress().String(), []byte(salt), expiry, suite.chaiID, suite.bvsDirContrAddr)

	sig := signer.NewSigner(chainIO.GetClientCtx())
	sigStr, err := sig.Sign(msgHash)
	assert.NoError(t, err)
	v, err := signer.VerifySignature(account.GetPubKey(), msgHash, sigStr)
	assert.NoError(t, err)
	t.Logf("%+v\n", v)
}

func CalculateDigestHash(operatorPublicKey []byte, bvsAddr string, salt []byte, expiry uint64, chainID, contrAddr string) []byte {
	operatorBVSRegistrationTypeHash := Sha256([]byte(OPERATOR_BVS_REGISTRATION_TYPEHASH))
	domainTypeHash := Sha256([]byte(DOMAIN_TYPEHASH))
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

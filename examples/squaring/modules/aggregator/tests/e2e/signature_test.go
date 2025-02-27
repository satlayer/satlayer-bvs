package e2e

import (
	"testing"

	"github.com/satlayer/satlayer-bvs/bvs-api/signer"
	"github.com/satlayer/satlayer-bvs/examples/squaring/internal/tests"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
)

type signatureTestSuite struct {
	tests.TestSuite
}

func (suite *signatureTestSuite) SetupSuite() {
	suite.TestSuite.SetupSuite(keyDir, "operator1", "8c6a223a07a0c8aa8c7f66319c89ec92c2aef76c116e7d8ee9e0fffc8c48bf3d")
}

// entrypoint for the test suite
func TestSignature(t *testing.T) {
	suite.Run(t, new(signatureTestSuite))
}

func (suite *signatureTestSuite) TestSign() {
	t := suite.T()

	msg := "hello world"
	msgByte := []byte(msg)

	// generate signature
	signature, err := suite.ChainIO.GetSigner().Sign(msgByte)
	assert.NoError(t, err)

	// verify signature
	pubKey := suite.ChainIO.GetCurrentAccountPubKey()
	verifyResult, err := signer.VerifySignature(pubKey, msgByte, signature)
	assert.NoError(t, err)
	assert.True(t, verifyResult, "signature verification failed")
}

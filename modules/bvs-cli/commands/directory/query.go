package directory

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/satlayer/satlayer-bvs/bvs-cli/conf"
)

func GetOperator(operatorAddress string) {
	s := NewService()
	resp, err := s.Directory.QueryOperator(operatorAddress, operatorAddress)
	if err != nil {
		fmt.Printf("Get operator error! %+v\n", err)
		return
	}
	result := resp.Status
	fmt.Printf("%s\n", result)
}

func CalcDigestHash(userKeyName, salt string, expire int64) {
	s := NewService()
	newChainIO, err := s.ChainIO.SetupKeyring(userKeyName, conf.C.Account.KeyringBackend)
	if err != nil {
		fmt.Printf("Setup keyring error! %+v\n", err)
		return
	}
	pubKey := newChainIO.GetCurrentAccountPubKey()
	address := sdk.AccAddress(pubKey.Address()).String()
	resp, err := s.Directory.CalculateDigestHash(pubKey, address, salt, expire)
	if err != nil {
		fmt.Printf("Calculate digest hash error! %+v\n", err)
		return
	}
	fmt.Printf("%s\n", resp.DigestHash)
}

func IsSaltSpent(operatorAddress, salt string) {
	s := NewService()
	resp, err := s.Directory.IsSaltSpent(operatorAddress, salt)
	if err != nil {
		fmt.Printf("Is salt spent error! %+v\n", err)
		return
	}
	result := resp.IsSaltSpent
	fmt.Printf("%t\n", result)
}

func DelegationManager() {
	s := NewService()
	resp, err := s.Directory.DelegationManager()
	if err != nil {
		fmt.Printf("Get delegation manager error! %+v\n", err)
		return
	}
	result := resp.DelegationAddr
	fmt.Printf("%s\n", result)
}

func Owner() {
	s := NewService()
	resp, err := s.Directory.Owner()
	if err != nil {
		fmt.Printf("Get owner error! %+v\n", err)
		return
	}
	fmt.Printf("%s\n", resp.OwnerAddr)
}

func OperatorBvsRegistrationTypeHash() {
	s := NewService()
	resp, err := s.Directory.OperatorBvsRegistrationTypeHash()
	if err != nil {
		fmt.Printf("Get operator BVS registration type hash error! %+v\n", err)
		return
	}
	result := resp.OperatorBvsRegistrationTypeHash
	fmt.Printf("%s\n", result)
}

func DomainTypeHash() {
	s := NewService()
	resp, err := s.Directory.DomainTypeHash()
	if err != nil {
		fmt.Printf("Get domain type hash error! %+v\n", err)
		return
	}
	result := resp.DomainTypeHash
	fmt.Printf("%s\n", result)
}

func DomainName() {
	s := NewService()
	resp, err := s.Directory.DomainName()
	if err != nil {
		fmt.Printf("Get domain name error! %+v\n", err)
		return
	}
	result := resp.DomainName
	fmt.Printf("%s\n", result)
}

func BvsInfo(BVSHash string) {
	s := NewService()
	resp, err := s.Directory.BvsInfo(BVSHash)
	if err != nil {
		fmt.Printf("Get BVS info error! %s\n", err)
		return
	}
	fmt.Printf("BVSContract: %s\n", resp.BvsContract)
}

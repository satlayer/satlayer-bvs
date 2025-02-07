package keys

import "fmt"

func EVMCreateAccount(password string) {
	s := NewService()
	account, err := s.ETHChainIO.CreateAccount(password)
	if err != nil {
		panic(err)
	}
	fmt.Printf("create new account: %s", account.Address)
}

func EVMImportKey(privateKey, password string) {
	s := NewService()
	account, err := s.ETHChainIO.ImportKey(privateKey, password)
	if err != nil {
		panic(err)
	}
	fmt.Printf("import new account: %s", account.Address)
}

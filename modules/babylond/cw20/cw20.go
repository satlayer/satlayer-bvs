package cw20

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"github.com/satlayer/satlayer-bvs/babylond"
)

// TODO(fuxingloh): Separate out Store and Init to allow for more flexibility/reusability

// DeployCw20 deploys the cw20 contract with the given initMsg
// The contract is in https://github.com/fuxingloh/cw20-test
func DeployCw20(c *babylond.BabylonContainer, initMsg InstantiateMsg) *babylond.DeployedWasmContract {
	_, currentFile, _, _ := runtime.Caller(0)
	baseDir := filepath.Dir(currentFile)
	targetFile := filepath.Join(baseDir, "cw20.wasm")
	wasmByteCode, err := os.ReadFile(targetFile)
	if err != nil {
		panic(err)
	}

	initBytes, err := initMsg.Marshal()
	if err != nil {
		panic(err)
	}

	label := fmt.Sprintf("CW20: %s", initMsg.Symbol)
	contract, err := c.StoreAndInitWasm(wasmByteCode, initBytes, label, "genesis")
	if err != nil {
		panic(err)
	}
	return contract
}

// TODO(fuxingloh): add utility to CW20 contract

func (r *InstantiateMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func (r *ExecuteMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func (r *QueryMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	Decimals        int64                     `json:"decimals"`
	InitialBalances []Cw20Coin                `json:"initial_balances"`
	Marketing       *InstantiateMarketingInfo `json:"marketing"`
	Mint            *MinterResponse           `json:"mint"`
	Name            string                    `json:"name"`
	Symbol          string                    `json:"symbol"`
}

type Cw20Coin struct {
	Address string `json:"address"`
	Amount  string `json:"amount"`
}

type InstantiateMarketingInfo struct {
	Description *string    `json:"description"`
	Logo        *LogoClass `json:"logo"`
	Marketing   *string    `json:"marketing"`
	Project     *string    `json:"project"`
}

type LogoClass struct {
	URL      *string           `json:"url,omitempty"`
	Embedded *LogoEmbeddedLogo `json:"embedded,omitempty"`
}

type LogoEmbeddedLogo struct {
	SVG *string `json:"svg,omitempty"`
	PNG *string `json:"png,omitempty"`
}

type MinterResponse struct {
	// cap is a hard cap on total supply that can be achieved by minting. Note that this refers
	// to total_supply. If None, there is unlimited cap.
	Cap    *string `json:"cap"`
	Minter string  `json:"minter"`
}

type ExecuteMsg struct {
	Transfer          *Transfer          `json:"transfer,omitempty"`
	Burn              *Burn              `json:"burn,omitempty"`
	Send              *Send              `json:"send,omitempty"`
	IncreaseAllowance *IncreaseAllowance `json:"increase_allowance,omitempty"`
	DecreaseAllowance *DecreaseAllowance `json:"decrease_allowance,omitempty"`
	TransferFrom      *TransferFrom      `json:"transfer_from,omitempty"`
	SendFrom          *SendFrom          `json:"send_from,omitempty"`
	BurnFrom          *BurnFrom          `json:"burn_from,omitempty"`
	Mint              *Mint              `json:"mint,omitempty"`
	UpdateMinter      *UpdateMinter      `json:"update_minter,omitempty"`
	UpdateMarketing   *UpdateMarketing   `json:"update_marketing,omitempty"`
	UploadLogo        *Logo              `json:"upload_logo,omitempty"`
}

type Burn struct {
	Amount string `json:"amount"`
}

type BurnFrom struct {
	Amount string `json:"amount"`
	Owner  string `json:"owner"`
}

type DecreaseAllowance struct {
	Amount  string      `json:"amount"`
	Expires *Expiration `json:"expires"`
	Spender string      `json:"spender"`
}

type Expiration struct {
	AtHeight *int64  `json:"at_height,omitempty"`
	AtTime   *string `json:"at_time,omitempty"`
	Never    *Never  `json:"never,omitempty"`
}

type Never struct {
}

type IncreaseAllowance struct {
	Amount  string      `json:"amount"`
	Expires *Expiration `json:"expires"`
	Spender string      `json:"spender"`
}

type Mint struct {
	Amount    string `json:"amount"`
	Recipient string `json:"recipient"`
}

type Send struct {
	Amount   string `json:"amount"`
	Contract string `json:"contract"`
	Msg      string `json:"msg"`
}

type SendFrom struct {
	Amount   string `json:"amount"`
	Contract string `json:"contract"`
	Msg      string `json:"msg"`
	Owner    string `json:"owner"`
}

type Transfer struct {
	Amount    string `json:"amount"`
	Recipient string `json:"recipient"`
}

type TransferFrom struct {
	Amount    string `json:"amount"`
	Owner     string `json:"owner"`
	Recipient string `json:"recipient"`
}

type UpdateMarketing struct {
	// A longer description of the token and it's utility. Designed for tooltips or such
	Description *string `json:"description"`
	// The address (if any) who can update this data structure
	Marketing *string `json:"marketing"`
	// A URL pointing to the project behind this token.
	Project *string `json:"project"`
}

type UpdateMinter struct {
	NewMinter *string `json:"new_minter"`
}

type Logo struct {
	URL      *string                `json:"url,omitempty"`
	Embedded *LogoEmbeddedLogoClass `json:"embedded,omitempty"`
}

type LogoEmbeddedLogoClass struct {
	SVG *string `json:"svg,omitempty"`
	PNG *string `json:"png,omitempty"`
}

type QueryMsg struct {
	Balance              *Balance              `json:"balance,omitempty"`
	TokenInfo            *TokenInfo            `json:"token_info,omitempty"`
	Minter               *Minter               `json:"minter,omitempty"`
	Allowance            *Allowance            `json:"allowance,omitempty"`
	AllAllowances        *AllAllowances        `json:"all_allowances,omitempty"`
	AllSpenderAllowances *AllSpenderAllowances `json:"all_spender_allowances,omitempty"`
	AllAccounts          *AllAccounts          `json:"all_accounts,omitempty"`
	MarketingInfo        *MarketingInfo        `json:"marketing_info,omitempty"`
	DownloadLogo         *DownloadLogo         `json:"download_logo,omitempty"`
}

type AllAccounts struct {
	Limit      *int64  `json:"limit"`
	StartAfter *string `json:"start_after"`
}

type AllAllowances struct {
	Limit      *int64  `json:"limit"`
	Owner      string  `json:"owner"`
	StartAfter *string `json:"start_after"`
}

type AllSpenderAllowances struct {
	Limit      *int64  `json:"limit"`
	Spender    string  `json:"spender"`
	StartAfter *string `json:"start_after"`
}

type Allowance struct {
	Owner   string `json:"owner"`
	Spender string `json:"spender"`
}

type Balance struct {
	Address string `json:"address"`
}

type DownloadLogo struct {
}

type MarketingInfo struct {
}

type Minter struct {
}

type TokenInfo struct {
}

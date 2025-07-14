// This file was automatically generated from vault-bank-tokenized/schema.json.
// DO NOT MODIFY IT BY HAND.

package vaultbanktokenized

type AssetsResponse string

type ConvertToAssetsResponse string

type ConvertToSharesResponse string

type SharesResponse string

type TotalAssetsResponse string

type TotalSharesResponse string

type InstantiateMsg struct {
	// The decimals of the receipt token. Must be the same as the denom's decimals.
	Decimals int64 `json:"decimals"`
	// The denom supported by this vault.
	Denom string `json:"denom"`
	// The name of the receipt token.
	Name string `json:"name"`
	// The address of the `operator`. Each vault is delegated to an `operator`.
	Operator string `json:"operator"`
	// The address of the `pauser` contract.
	Pauser string `json:"pauser"`
	// The address of the `router` contract.
	Router string `json:"router"`
	// The symbol for the receipt token.
	Symbol string `json:"symbol"`
}

// ExecuteMsg Transfer is a base message to move tokens to another account without
// triggering actions
//
// ExecuteMsg Send is a base message to transfer tokens to a contract and trigger an action
// on the receiving contract.
//
// ExecuteMsg IncreaseAllowance allows spender to access an additional amount tokens from
// the owner's (env.sender) account. If expires is Some(), overwrites current allowance
// expiration with this one.
//
// ExecuteMsg DecreaseAllowance Lowers the spender's access of tokens from the owner's
// (env.sender) account by amount. If expires is Some(), overwrites current allowance
// expiration with this one.
//
// ExecuteMsg TransferFrom tansfers amount tokens from owner -> recipient if `env.sender`
// has sufficient pre-approval.
//
// ExecuteMsg SendFrom Sends amount tokens from owner -> contract if `env.sender` has
// sufficient pre-approval.
//
// ExecuteMsg DepositFor assets into the vault. Sender must transfer the assets to the vault
// contract (this is implementation agnostic). The vault contract must mint shares to the
// `recipient`. Vault must be whitelisted in the `vault-router` to accept deposits.
//
// ExecuteMsg QueueWithdrawalTo assets from the vault. Sender must have enough shares to
// queue the requested amount to the `recipient`. Once the withdrawal is queued, the
// `recipient` can redeem the withdrawal after the lock period. Once the withdrawal is
// locked, the `sender` cannot cancel the withdrawal. The time-lock is enforced by the vault
// and cannot be changed retroactively.
//
// ### Lock Period Extension New withdrawals will extend the lock period of any existing
// withdrawals. You can queue the withdrawal to a different `recipient` than the `sender` to
// avoid this.
//
// ExecuteMsg RedeemWithdrawalTo all queued shares into assets from the vault for
// withdrawal. After the lock period, the `sender` (must be the `recipient` of the original
// withdrawal) can redeem the withdrawal.
//
// ExecuteMsg SlashLocked moves the assets from the vault to the `vault-router` contract for
// custody. Part of the [https://build.satlayer.xyz/getting-started/slashing](Programmable
// Slashing) lifecycle. This function can only be called by `vault-router`, and takes an
// absolute `amount` of assets to be moved. The amount is calculated and enforced by the
// router. Further utility of the assets, post-locked, is implemented and enforced on the
// router level.
//
// ExecuteMsg ApproveProxy allows the `proxy` to queue withdrawal and redeem withdrawal on
// behalf of the `owner`.
type ExecuteMsg struct {
	Transfer           *Transfer                 `json:"transfer,omitempty"`
	Send               *Send                     `json:"send,omitempty"`
	IncreaseAllowance  *IncreaseAllowance        `json:"increase_allowance,omitempty"`
	DecreaseAllowance  *DecreaseAllowance        `json:"decrease_allowance,omitempty"`
	TransferFrom       *TransferFrom             `json:"transfer_from,omitempty"`
	SendFrom           *SendFrom                 `json:"send_from,omitempty"`
	DepositFor         *RecipientAmount          `json:"deposit_for,omitempty"`
	QueueWithdrawalTo  *QueueWithdrawalToParams  `json:"queue_withdrawal_to,omitempty"`
	RedeemWithdrawalTo *RedeemWithdrawalToParams `json:"redeem_withdrawal_to,omitempty"`
	SlashLocked        *string                   `json:"slash_locked,omitempty"`
	SetApproveProxy    *SetApproveProxyParams    `json:"set_approve_proxy,omitempty"`
}

type DecreaseAllowance struct {
	Amount  string      `json:"amount"`
	Expires *Expiration `json:"expires"`
	Spender string      `json:"spender"`
}

// AtHeight will expire when `env.block.height` >= height
//
// AtTime will expire when `env.block.time` >= time
//
// Never will never expire. Used to express the empty variant
type Expiration struct {
	AtHeight *int64        `json:"at_height,omitempty"`
	AtTime   *string       `json:"at_time,omitempty"`
	Never    *ExpiresNever `json:"never,omitempty"`
}

type ExpiresNever struct {
}

// This struct is used to represent the recipient and amount fields together.
type RecipientAmount struct {
	Amount    string `json:"amount"`
	Recipient string `json:"recipient"`
}

type IncreaseAllowance struct {
	Amount  string      `json:"amount"`
	Expires *Expiration `json:"expires"`
	Spender string      `json:"spender"`
}

// This struct is used to represent the controller and amount fields together.
type QueueWithdrawalToParams struct {
	// the amount is the amount of shares to be withdrawn
	Amount string `json:"amount"`
	// the controller is the address that can redeem the withdrawal after the lock period
	Controller string `json:"controller"`
	// the owner is the address that owns the shares being withdrawn
	Owner string `json:"owner"`
}

// This struct is used to represent a recipient for RedeemWithdrawalTo.
type RedeemWithdrawalToParams struct {
	Controller string `json:"controller"`
	Recipient  string `json:"recipient"`
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

type SetApproveProxyParams struct {
	// whether the proxy is approved or not.
	Approve bool `json:"approve"`
	// The proxy address that is being approved.
	Proxy string `json:"proxy"`
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

// QueryMsg Balance: get the balance of a given address. Returns the current balance of the
// given address, 0 if unset.
//
// QueryMsg TokenInfo: get the token info of the contract. Returns metadata on the contract
// - name, decimals, supply, etc.
//
// QueryMsg Allowance: get the allowance of a given address. Returns how much spender can
// use from owner account, 0 if unset.
//
// QueryMsg AllAllowances: get all allowances of a given address. Returns all allowances
// this owner has approved. Supports pagination.
//
// QueryMsg AllSpenderAllowances: get all allowances of a given address. Returns all
// allowances this spender has been granted. Supports pagination.
//
// QueryMsg AllAccounts: get all accounts of the contract. Returns all accounts that have
// balances. Supports pagination.
//
// QueryMsg Shares: get the shares of a staker. Shares in this tokenized vault are CW20
// receipt tokens. The interface is kept the same as the original vault. to avoid breaking
// and minimize changes in vault consumer/frontend code.
//
// QueryMsg Assets: get the assets of a staker, converted from shares.
//
// QueryMsg ConvertToAssets: convert shares to assets.
//
// QueryMsg ConvertToShares: convert assets to shares.
//
// QueryMsg TotalShares: get the total shares in circulation.
//
// QueryMsg TotalAssets: get the total assets under vault.
//
// QueryMsg QueuedWithdrawal: get the queued withdrawal and unlock timestamp under vault.
//
// QueryMsg VaultInfo: get the vault information.
type QueryMsg struct {
	Balance              *Balance              `json:"balance,omitempty"`
	TokenInfo            *TokenInfo            `json:"token_info,omitempty"`
	Allowance            *Allowance            `json:"allowance,omitempty"`
	AllAllowances        *AllAllowances        `json:"all_allowances,omitempty"`
	AllSpenderAllowances *AllSpenderAllowances `json:"all_spender_allowances,omitempty"`
	AllAccounts          *AllAccounts          `json:"all_accounts,omitempty"`
	Shares               *Shares               `json:"shares,omitempty"`
	Assets               *Assets               `json:"assets,omitempty"`
	ConvertToAssets      *ConvertToAssets      `json:"convert_to_assets,omitempty"`
	ConvertToShares      *ConvertToShares      `json:"convert_to_shares,omitempty"`
	TotalShares          *TotalShares          `json:"total_shares,omitempty"`
	TotalAssets          *TotalAssets          `json:"total_assets,omitempty"`
	QueuedWithdrawal     *QueuedWithdrawal     `json:"queued_withdrawal,omitempty"`
	VaultInfo            *VaultInfo            `json:"vault_info,omitempty"`
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

type Assets struct {
	Staker string `json:"staker"`
}

type Balance struct {
	Address string `json:"address"`
}

type ConvertToAssets struct {
	Shares string `json:"shares"`
}

type ConvertToShares struct {
	Assets string `json:"assets"`
}

type QueuedWithdrawal struct {
	Controller string `json:"controller"`
}

type Shares struct {
	Staker string `json:"staker"`
}

type TokenInfo struct {
}

type TotalAssets struct {
}

type TotalShares struct {
}

type VaultInfo struct {
}

type AllAccountsResponse struct {
	Accounts []string `json:"accounts"`
}

type AllAllowancesResponse struct {
	Allowances []AllowanceInfo `json:"allowances"`
}

type AllowanceInfo struct {
	Allowance string           `json:"allowance"`
	Expires   PurpleExpiration `json:"expires"`
	Spender   string           `json:"spender"`
}

// Expiration represents a point in time when some event happens. It can compare with a
// BlockInfo and will return is_expired() == true once the condition is hit (and for every
// block in the future)
//
// AtHeight will expire when `env.block.height` >= height
//
// AtTime will expire when `env.block.time` >= time
//
// Never will never expire. Used to express the empty variant
type PurpleExpiration struct {
	AtHeight *int64       `json:"at_height,omitempty"`
	AtTime   *string      `json:"at_time,omitempty"`
	Never    *PurpleNever `json:"never,omitempty"`
}

type PurpleNever struct {
}

type AllSpenderAllowancesResponse struct {
	Allowances []SpenderAllowanceInfo `json:"allowances"`
}

type SpenderAllowanceInfo struct {
	Allowance string           `json:"allowance"`
	Expires   FluffyExpiration `json:"expires"`
	Owner     string           `json:"owner"`
}

// Expiration represents a point in time when some event happens. It can compare with a
// BlockInfo and will return is_expired() == true once the condition is hit (and for every
// block in the future)
//
// AtHeight will expire when `env.block.height` >= height
//
// AtTime will expire when `env.block.time` >= time
//
// Never will never expire. Used to express the empty variant
type FluffyExpiration struct {
	AtHeight *int64       `json:"at_height,omitempty"`
	AtTime   *string      `json:"at_time,omitempty"`
	Never    *FluffyNever `json:"never,omitempty"`
}

type FluffyNever struct {
}

type AllowanceResponse struct {
	Allowance string                      `json:"allowance"`
	Expires   AllowanceResponseExpiration `json:"expires"`
}

// Expiration represents a point in time when some event happens. It can compare with a
// BlockInfo and will return is_expired() == true once the condition is hit (and for every
// block in the future)
//
// AtHeight will expire when `env.block.height` >= height
//
// AtTime will expire when `env.block.time` >= time
//
// Never will never expire. Used to express the empty variant
type AllowanceResponseExpiration struct {
	AtHeight *int64          `json:"at_height,omitempty"`
	AtTime   *string         `json:"at_time,omitempty"`
	Never    *TentacledNever `json:"never,omitempty"`
}

type TentacledNever struct {
}

type BalanceResponse struct {
	Balance string `json:"balance"`
}

// The response to the `QueuedWithdrawal` query. This is just a wrapper around
// `QueuedWithdrawalInfo`, so that the schema can be generated.
type QueuedWithdrawalResponse struct {
	QueuedShares    string `json:"queued_shares"`
	UnlockTimestamp string `json:"unlock_timestamp"`
}

type TokenInfoResponse struct {
	Decimals    int64  `json:"decimals"`
	Name        string `json:"name"`
	Symbol      string `json:"symbol"`
	TotalSupply string `json:"total_supply"`
}

type VaultInfoResponse struct {
	// Asset identifier, using the CAIP-19 format.
	AssetID string `json:"asset_id"`
	// The asset reference stores the cw20 contract address or the bank denom.
	AssetReference string `json:"asset_reference"`
	// The asset type, either `AssetType::Cw20` or `AssetType::Bank`.
	AssetType AssetType `json:"asset_type"`
	// The name of the vault contract, see [`cw2::set_contract_version`] for more information.
	Contract string `json:"contract"`
	// The `operator` that this vault is delegated to
	Operator string `json:"operator"`
	// The `pauser` contract address
	Pauser string `json:"pauser"`
	// The `vault-router` contract address
	Router string `json:"router"`
	// The total assets under management
	TotalAssets string `json:"total_assets"`
	// The total shares in circulation
	TotalShares string `json:"total_shares"`
	// The version of the vault contract, see [`cw2::set_contract_version`] for more information.
	Version string `json:"version"`
}

// The asset type, either `AssetType::Cw20` or `AssetType::Bank`.
type AssetType string

const (
	Bank AssetType = "bank"
	Cw20 AssetType = "cw20"
)

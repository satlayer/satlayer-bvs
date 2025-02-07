package types

// AddStrategiesToWhitelistReq execute messages
type AddStrategiesToWhitelistReq struct {
	AddStrategiesToWhitelist AddStrategiesToWhitelist `json:"add_strategies_to_whitelist"`
}

type AddStrategiesToWhitelist struct {
	Strategies                         []string `json:"strategies"`
	ThirdPartyTransfersForbiddenValues []bool   `json:"third_party_transfers_forbidden_values"`
}

type RemoveStrategiesFromWhitelistReq struct {
	RemoveStrategiesFromWhitelist RemoveStrategiesFromWhitelist `json:"remove_strategies_from_whitelist"`
}

type RemoveStrategiesFromWhitelist struct {
	Strategies []string `json:"strategies"`
}

type SetStrategyWhitelisterReq struct {
	SetStrategyWhitelister SetStrategyWhitelister `json:"set_strategy_whitelister"`
}

type SetStrategyWhitelister struct {
	NewStrategyWhitelister string `json:"new_strategy_whitelister"`
}

type DepositIntoStrategyReq struct {
	DepositIntoStrategy DepositIntoStrategy `json:"deposit_into_strategy"`
}

type DepositIntoStrategy struct {
	Strategy string `json:"strategy"`
	Token    string `json:"token"`
	Amount   string `json:"amount"`
}

type SetThirdPartyTransfersForbiddenReq struct {
	SetThirdPartyTransfersForbidden SetThirdPartyTransfersForbidden `json:"set_third_party_transfers_forbidden"`
}

type SetThirdPartyTransfersForbidden struct {
	Strategy string `json:"strategy"`
	Value    bool   `json:"value"`
}

type DepositIntoStrategyWithSignatureReq struct {
	DepositIntoStrategyWithSignature DepositIntoStrategyWithSignature `json:"deposit_into_strategy_with_signature"`
}

type DepositIntoStrategyWithSignature struct {
	Strategy  string `json:"strategy"`
	Token     string `json:"token"`
	Amount    string `json:"amount"`
	Staker    string `json:"staker"`
	PublicKey string `json:"public_key"`
	Expiry    uint64 `json:"expiry"`
	Signature string `json:"signature"`
}

type RemoveSharesReq struct {
	RemoveShares RemoveShares `json:"remove_shares"`
}

type RemoveShares struct {
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
	Shares   string `json:"shares"`
}

type WithdrawSharesAsTokensReq struct {
	WithdrawSharesAsTokens WithdrawSharesAsTokens `json:"withdraw_shares_as_tokens"`
}

type WithdrawSharesAsTokens struct {
	Recipient string `json:"recipient"`
	Strategy  string `json:"strategy"`
	Shares    string `json:"shares"`
	Token     string `json:"token"`
}

type AddSharesReq struct {
	AddShares AddShares `json:"add_shares"`
}

type AddShares struct {
	Staker   string `json:"staker"`
	Token    string `json:"token"`
	Strategy string `json:"strategy"`
	Shares   string `json:"shares"`
}

type SetDelegationManagerReq struct {
	SetDelegationManager SetDelegationManager `json:"set_delegation_manager"`
}

type SetDelegationManager struct {
	NewDelegationManager string `json:"new_delegation_manager"`
}

type SetSlashManagerReq struct {
	SetSlashManager SetSlashManager `json:"set_slash_manager"`
}

type SetSlashManager struct {
	NewSlashManager string `json:"new_slash_manager"`
}

type SetStrategyFactoryReq struct {
	SetStrategyFactory SetStrategyFactory `json:"set_strategy_factory"`
}

type SetStrategyFactory struct {
	NewStrategyFactory string `json:"new_strategy_factory"`
}

type TransferStrategyManagerOwnershipReq struct {
	TransferOwnership TransferStrategyManagerOwnership `json:"transfer_ownership"`
}

type TransferStrategyManagerOwnership struct {
	NewOwner string `json:"new_owner"`
}

// query messages
type GetDepositsReq struct {
	GetDeposits GetDeposits `json:"get_deposits"`
}

type GetDeposits struct {
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type StakerStrategyListLengthReq struct {
	StakerStrategyListLength StakerStrategyListLength `json:"staker_strategy_list_length"`
}

type StakerStrategyListLength struct {
	Staker string `json:"staker"`
}

type GetStakerStrategySharesReq struct {
	GetStakerStrategyShares GetStakerStrategyShares `json:"get_staker_strategy_shares"`
}

type GetStakerStrategyShares struct {
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type IsThirdPartyTransfersForbiddenReq struct {
	IsThirdPartyTransfersForbidden IsThirdPartyTransfersForbidden `json:"is_third_party_transfers_forbidden"`
}

type IsThirdPartyTransfersForbidden struct {
	Strategy string `json:"strategy"`
}

type GetNonceReq struct {
	GetNonce GetNonce `json:"get_nonce"`
}

type GetNonce struct {
	Staker string `json:"staker"`
}

type GetStakerStrategyListReq struct {
	GetStakerStrategyList GetStakerStrategyList `json:"get_staker_strategy_list"`
}

type GetStakerStrategyList struct {
	Staker string `json:"staker"`
}

type GetStrategyManagerOwnerReq struct {
	GetOwner GetOwner `json:"get_owner"`
}

type GetOwner struct {
}

type IsStrategyWhitelistedReq struct {
	IsStrategyWhitelisted IsStrategyWhitelisted `json:"is_strategy_whitelisted"`
}

type IsStrategyWhitelisted struct {
	Strategy string `json:"strategy"`
}

type CalculateStrategyManagerDigestHashReq struct {
	CalculateDigestHash CalculateStrategyManagerDigestHash `json:"calculate_digest_hash"`
}

type CalculateStrategyManagerDigestHash struct {
	DigestHashParams DigestHashParams `json:"digst_hash_params"`
}

type DigestHashParams struct {
	Staker       string `json:"staker"`
	PublicKey    string `json:"public_key"`
	Strategy     string `json:"strategy"`
	Token        string `json:"token"`
	Amount       string `json:"amount"`
	Nonce        uint64 `json:"nonce"`
	Expiry       uint64 `json:"expiry"`
	ChainId      string `json:"chain_id"`
	ContractAddr string `json:"contract_addr"`
}

type GetStrategyWhitelisterReq struct {
	GetStrategyWhitelister GetStrategyWhitelister `json:"get_strategy_whitelister"`
}

type GetStrategyWhitelister struct {
}

type GetStrategyManagerStateReq struct {
	GetStrategyManagerState GetStrategyManagerState `json:"get_strategy_manager_state"`
}

type GetStrategyManagerState struct {
}

type GetDepositTypehashReq struct {
	GetDepositTypehash GetDepositTypehash `json:"get_deposit_typehash"`
}

type GetDepositTypehash struct {
}

type GetDomainTypehashReq struct {
	GetDomainTypehash GetDomainTypehash `json:"get_domain_typehash"`
}

type GetDomainTypehash struct {
}

type GetStrategyManagerDomainNameReq struct {
	GetDomainName GetDomainName `json:"get_domain_name"`
}

type GetDomainName struct {
}

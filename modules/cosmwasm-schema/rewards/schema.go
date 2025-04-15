// This file was automatically generated from rewards/schema.json.
// DO NOT MODIFY IT BY HAND.

package rewards

type DistributionRootResponse string

type InstantiateMsg struct {
	// Owner of this contract
	Owner string `json:"owner"`
}

type ExecuteMsg struct {
	DistributeRewards *DistributeRewards `json:"distribute_rewards,omitempty"`
	ClaimRewards      *ClaimRewards      `json:"claim_rewards,omitempty"`
}

type ClaimRewards struct {
	// amount refers to the total amount of rewards accrued to the user
	Amount            string            `json:"amount"`
	ClaimRewardsProof ClaimRewardsProof `json:"claim_rewards_proof"`
	Recipient         string            `json:"recipient"`
	RewardType        RewardsType       `json:"reward_type"`
	Service           string            `json:"service"`
	// token refers to the address of the token contract (CW20) or denom string (Bank)
	Token string `json:"token"`
}

type ClaimRewardsProof struct {
	// leaf_index is the index of the user leaf in the Merkle tree
	LeafIndex string `json:"leaf_index"`
	// proof is the Merkle proof of the user leaf in the Merkle tree
	Proof []string `json:"proof"`
	// root refers to the Merkle root of the Merkle tree
	Root string `json:"root"`
	// total_leaves_count is the total number of leaves in the Merkle tree
	TotalLeavesCount string `json:"total_leaves_count"`
}

type DistributeRewards struct {
	MerkleRoot         string             `json:"merkle_root"`
	RewardDistribution RewardDistribution `json:"reward_distribution"`
	RewardType         RewardsType        `json:"reward_type"`
}

type RewardDistribution struct {
	// amount refers to the additional rewards to be transferred to the contract and distributed
	Amount string `json:"amount"`
	// token refers to the address of the token contract (CW20) or denom string (Bank)
	Token string `json:"token"`
}

type QueryMsg struct {
	DistributionRoot DistributionRoot `json:"distribution_root"`
}

type DistributionRoot struct {
	Service string `json:"service"`
	Token   string `json:"token"`
}

type RewardsType string

const (
	Bank RewardsType = "bank"
	CW20 RewardsType = "c_w20"
)

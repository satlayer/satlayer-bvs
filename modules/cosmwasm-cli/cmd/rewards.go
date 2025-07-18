package cmd

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/spf13/cobra"
	"github.com/wealdtech/go-merkletree/v2"
	"github.com/wealdtech/go-merkletree/v2/keccak256"
)

var filePath string

// MerkleProofRes is partly based on `bvs-rewards::ExecuteMsg:ClaimRewards`
type MerkleProofRes struct {
	ClaimRewardsProof `json:"claim_rewards_proof"`
	Token             string `json:"token"`
	Amount            string `json:"amount"`
}

// ClaimRewardsProof based on `bvs-rewards::msg:ClaimRewardsProof`
type ClaimRewardsProof struct {
	Root             string   `json:"root"`
	Proof            []string `json:"proof"`
	LeafIndex        uint64   `json:"leaf_index"`
	TotalLeavesCount uint64   `json:"total_leaves_count"`
}

type RewardsMerkleTree struct {
	Tree  *merkletree.MerkleTree
	Token string `json:"token"`
}

func (t *RewardsMerkleTree) MarshalJSON() ([]byte, error) {
	type ExportTree merkletree.MerkleTree

	return json.Marshal(&struct {
		Token    string `json:"token"`
		HashType string `json:"hash_type"`
		*ExportTree
	}{
		Token:      t.Token,
		HashType:   t.Tree.Hash.HashName(),
		ExportTree: (*ExportTree)(t.Tree),
	})
}

func (t *RewardsMerkleTree) UnmarshalJSON(data []byte) error {
	// unmarshal the JSON to get Token
	aux := &struct {
		Token string `json:"token"`
	}{}
	if err := json.Unmarshal(data, &aux); err != nil {
		return fmt.Errorf("failed to unmarshal JSON: %w", err)
	}

	// Unmarshal the MerkleTree using its own UnmarshalJSON method
	var tree merkletree.MerkleTree
	if err := tree.UnmarshalJSON(data); err != nil {
		return fmt.Errorf("failed to unmarshal MerkleTree: %w", err)
	}

	// Assign the unmarshaled values to the RewardsMerkleTree
	t.Tree = &tree
	t.Token = aux.Token

	return nil
}

func RewardsCreateCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "create",
		Short: "Create a new Merkle tree rewards from distribution file and save it to a file",
		Run: func(cmd *cobra.Command, args []string) {
			// load distribution file from config
			distributionFilePath, err := loadDistributionFilePath(cmd)
			if err != nil {
				fmt.Println("Error getting distribution file path:", err)
				return
			}

			// Add logic to create a new Merkle tree
			dist, err := readMerkleInput(distributionFilePath)
			if err != nil {
				fmt.Println("Error reading distribution file:", err)
				return
			}

			// Create the Merkle tree from the distribution data
			rewardsMerkleTree, err := createMerkleTreeFromDistribution(dist)
			if err != nil {
				fmt.Println("Error creating Merkle tree:", err)
				return
			}

			// convert the tree to JSON bytes
			bytes, err := rewardsMerkleTree.MarshalJSON()
			if err != nil {
				fmt.Println("Error marshaling tree to JSON:", err)
				return
			}

			// Save the tree in json format
			merkleTreeFilePath, err := loadMerkleFilePath(cmd)
			if err != nil {
				fmt.Println("Error getting merkle file path:", err)
				return
			}
			err = os.WriteFile(merkleTreeFilePath, bytes, 0644)
			if err != nil {
				fmt.Println("Error writing tree to file:", err)
				return
			}

			// print merkle root
			fmt.Println("Merkle root:", rewardsMerkleTree.Tree.String())
			fmt.Println("Merkle tree saved to:", merkleTreeFilePath)
		},
	}
}

func RewardsLoadCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "load",
		Short: "Load a Merkle tree from a file",
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Println("Loading a Merkle tree from file...")
			// Add logic to load a Merkle tree from a file
			merkleFilePath, err := loadMerkleFilePath(cmd)
			if err != nil {
				fmt.Println("Error getting file path:", err)
				return
			}
			rewardsTree, err := loadMerkleTreeFromFile(merkleFilePath)

			// print merkle root
			fmt.Println("Merkle root:", rewardsTree.Tree.String())
		},
	}
}

func RewardsProofCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "proof",
		Short: "Generate a Merkle proof for a given earner and rewards",
		Args: func(cmd *cobra.Command, args []string) error {
			if len(args) != 2 {
				return fmt.Errorf("requires 2 arguments: merkle proof <earner> <reward>")
			}
			// TODO: validate earner and reward
			return nil
		},
		Run: func(cmd *cobra.Command, args []string) {
			merkleFilePath, err := loadMerkleFilePath(cmd)
			if err != nil {
				fmt.Println("Error getting merkle file path:", err)
				return
			}

			rewardsTree, err := loadMerkleTreeFromFile(merkleFilePath)
			if err != nil {
				fmt.Println("Error loading Merkle tree:", err)
				return
			}

			// Generate the proof for the given earner and reward
			earner := args[0]
			reward := args[1]
			leaf := leafHash(earner, reward)

			proof, err := rewardsTree.Tree.GenerateProof(leaf, 0)
			if err != nil {
				fmt.Println("Error generating proof:", err)
				return
			}

			// convert proof to hex strings
			hexProof := make([]string, len(proof.Hashes))
			for i, hash := range proof.Hashes {
				hexProof[i] = hex.EncodeToString(hash)
			}

			merkleProofRes := MerkleProofRes{
				ClaimRewardsProof: ClaimRewardsProof{
					Root:             rewardsTree.Tree.String(),
					Proof:            hexProof,
					LeafIndex:        proof.Index,
					TotalLeavesCount: uint64(len(rewardsTree.Tree.Data)),
				},
				Token:  rewardsTree.Token,
				Amount: reward,
			}

			merkleProofResJSON, err := json.MarshalIndent(merkleProofRes, "", "  ")
			if err != nil {
				fmt.Println("Error marshaling proof to JSON:", err)
				return
			}

			fmt.Println(string(merkleProofResJSON))
		},
	}
}

// Distribution represents the top-level JSON structure of distribution.json
type Distribution struct {
	Token   string   `json:"token"`
	Earners []Earner `json:"earners"`
}

// Earner represents each earner-reward pair entry
type Earner struct {
	Earner string `json:"earner"`
	Reward string `json:"reward"`
}

func (d *Distribution) String() string {
	jsonData, err := json.MarshalIndent(d, "", "  ")
	if err != nil {
		return fmt.Sprintf("error marshaling Distribution: %v", err)
	}
	return string(jsonData)
}

func leafHash(earner string, reward string) []byte {
	// Create a leaf hash by concatenating the earner and reward and hashing them
	hasher := keccak256.New()
	return hasher.Hash([]byte(earner), []byte(reward))
}

// loadDistributionFilePath forms the distribution file path from the command line flags
func loadDistributionFilePath(cmd *cobra.Command) (string, error) {
	// Get the data root path from the command line flags
	filePath, err := cmd.PersistentFlags().GetString("file-path")
	if err != nil {
		return "", err
	}

	// Check if file exists
	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		return "", fmt.Errorf("distribution file does not exist at path: %s", filePath)
	}

	// Verify file is a json file
	if !strings.HasSuffix(filePath, ".json") {
		return "", fmt.Errorf("distribution file must be a JSON file. found: %s", filePath)
	}

	return filePath, nil
}

// loadMerkleFilePath forms the Merkle file path from the command line flags
func loadMerkleFilePath(cmd *cobra.Command) (string, error) {
	// Get the data root path from the command line flags
	filePath, err := cmd.PersistentFlags().GetString("file-path")
	if err != nil {
		return "", err
	}

	fileDir := filepath.Dir(filePath)
	merkleFilePath := filepath.Join(fileDir, "merkle.json")
	return merkleFilePath, nil
}

// readMerkleInput reads the distribution data from a JSON file from the given path
func readMerkleInput(path string) (Distribution, error) {
	var distribution Distribution

	// sanity check on path
	if path == "" {
		return distribution, fmt.Errorf("path cannot be empty")
	}

	// Check if file exists and is a regular file
	fileInfo, err := os.Stat(path)
	if err != nil {
		if os.IsNotExist(err) {
			return distribution, fmt.Errorf("file does not exist: %s", path)
		}
		return distribution, fmt.Errorf("error accessing file: %w", err)
	}

	if !fileInfo.Mode().IsRegular() {
		return distribution, fmt.Errorf("%s is not a regular file", path)
	}

	// Read file from relative path
	data, err := os.ReadFile(path)
	if err != nil {
		return distribution, fmt.Errorf("error reading file: %w", err)
	}

	if err := json.Unmarshal(data, &distribution); err != nil {
		return distribution, fmt.Errorf("error unmarshaling JSON: %w", err)
	}

	return distribution, nil
}

// createMerkleTreeFromDistribution creates a Merkle tree from the given distribution data
func createMerkleTreeFromDistribution(dist Distribution) (*RewardsMerkleTree, error) {
	// Create data slice for merkle tree
	var data [][]byte

	for _, earner := range dist.Earners {
		// Add each earner-reward pair as a leaf hash (first hash)
		leaf := leafHash(earner.Earner, earner.Reward)
		data = append(data, leaf)
	}

	// Create new merkle tree with default settings - leaves are double-hashed
	tree, err := merkletree.NewTree(
		merkletree.WithData(data),
		merkletree.WithHashType(keccak256.New()),
		merkletree.WithSorted(false),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create merkle tree: %w", err)
	}

	// Create a new RewardsMerkleTree instance
	rewardsMerkleTree := &RewardsMerkleTree{
		Tree:  tree,
		Token: dist.Token,
	}

	return rewardsMerkleTree, nil
}

// loadMerkleTreeFromFile loads a Merkle tree from a file
func loadMerkleTreeFromFile(filePath string) (*RewardsMerkleTree, error) {
	var rewardTree *RewardsMerkleTree
	// Add logic to load a Merkle tree from a file
	data, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Println("Error reading file:", err)
		return rewardTree, err
	}
	if err := json.Unmarshal(data, &rewardTree); err != nil {
		fmt.Println("Error unmarshaling JSON:", err)
		return rewardTree, err
	}

	return rewardTree, nil
}

func RewardsCommand() *cobra.Command {
	command := &cobra.Command{
		Use:   "rewards",
		Short: "Rewards related commands",
	}

	rewardsCreateCmd := RewardsCreateCmd()
	rewardsLoadCmd := RewardsLoadCmd()
	rewardsProofCmd := RewardsProofCmd()

	command.AddCommand(rewardsCreateCmd) // add "rewards create -f <file-path>" command
	command.AddCommand(rewardsLoadCmd)   // add "rewards load -f <file-path>" command
	command.AddCommand(rewardsProofCmd)  // add "rewards proof <earner> <reward> -f <file-path>" command

	// merkle create
	rewardsCreateCmd.PersistentFlags().StringVarP(&filePath, "file-path", "f", "./data", "Path to the distribution json file")
	err := rewardsCreateCmd.MarkPersistentFlagRequired("file-path")
	if err != nil {
		fmt.Println("Error marking file-path flag as required:", err)
	}

	// merkle load
	rewardsLoadCmd.PersistentFlags().StringVarP(&filePath, "file-path", "f", "./data", "Path to the distribution json file")
	err = rewardsLoadCmd.MarkPersistentFlagRequired("file-path")
	if err != nil {
		fmt.Println("Error marking flag as required:", err)
	}

	// merkle proof
	rewardsProofCmd.PersistentFlags().StringVarP(&filePath, "file-path", "f", "./data", "Path to the distribution json file")
	err = rewardsProofCmd.MarkPersistentFlagRequired("file-path")
	if err != nil {
		fmt.Println("Error marking flag as required:", err)
	}

	return command
}

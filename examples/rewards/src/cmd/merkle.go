package cmd

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"github.com/wealdtech/go-merkletree/v2"
	"github.com/wealdtech/go-merkletree/v2/sha3"
)

var merkleTreeFile string
var distributionFile string

var merkleCmd = &cobra.Command{
	Use:   "merkle",
	Short: "Merkle related commands",
}

var merkleCreateCmd = &cobra.Command{
	Use:   "create",
	Short: "Create a new Merkle tree from distribution file",
	Run: func(cmd *cobra.Command, args []string) {
		file, err := cmd.PersistentFlags().GetString("distribution-file")
		if err != nil {
			fmt.Println("Error getting file path:", err)
			return
		}
		// Add logic to create a new Merkle tree
		dist, err := readMerkleInput(file)
		if err != nil {
			fmt.Println("Error reading distribution file:", err)
			return
		}

		// Create the Merkle tree from the distribution data
		tree, err := createMerkleTreeFromDistribution(dist)
		if err != nil {
			fmt.Println("Error creating Merkle tree:", err)
			return
		}

		// convert the tree to JSON bytes
		bytes, err := tree.MarshalJSON()
		if err != nil {
			fmt.Println("Error marshaling tree to JSON:", err)
			return
		}

		// Save the tree in json format
		filePath := "../data/tree/merkle.json"
		err = os.WriteFile(filePath, bytes, 0644)
		if err != nil {
			fmt.Println("Error writing tree to file:", err)
			return
		}

		// Save tree file to config
		err = cmd.Flag("tree-file").Value.Set(filePath)
		if err != nil {
			fmt.Println("Error setting tree file path:", err)
			return
		}

		// print merkle root
		fmt.Println("Merkle root:", tree.String())
	},
}

var merkleLoadCmd = &cobra.Command{
	Use:   "load",
	Short: "Load a Merkle tree from a file",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Loading a Merkle tree from file...")
		// Add logic to load a Merkle tree from a file
		filePath, err := cmd.Flags().GetString("tree-file")
		if err != nil {
			fmt.Println("Error getting file path:", err)
			return
		}
		tree, err := loadMerkleTreeFromFile(filePath)

		// print merkle root
		fmt.Println("Merkle root:", tree.String())
	},
}

var merkleProofCmd = &cobra.Command{
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
		// load tree from config
		filePath, err := cmd.Flags().GetString("tree-file")
		if err != nil {
			fmt.Println("Error getting file path:", err)
			return
		}
		tree, err := loadMerkleTreeFromFile(filePath)
		if err != nil {
			fmt.Println("Error loading Merkle tree:", err)
			return
		}

		// Generate the proof for the given earner and reward
		earner := args[0]
		reward := args[1]
		leaf := leafHash(earner, reward)

		proof, err := tree.GenerateProof(leaf, 0)
		if err != nil {
			fmt.Println("Error generating proof:", err)
			return
		}
		// Print the proof
		proofJSON, err := json.MarshalIndent(proof, "", "  ")
		if err != nil {
			fmt.Println("Error marshaling proof to JSON:", err)
			return
		}

		fmt.Println("Merkle root:", tree.String())
		fmt.Println("Merkle proof:", string(proofJSON))
	},
}

// Distribution represents the top-level JSON structure
type Distribution struct {
	Token   string   `json:"token"`
	Earners []Earner `json:"earners"`
}

// Earner represents each operator entry
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
	return []byte(fmt.Sprintf("%s:%s", earner, reward))
}

func readMerkleInput(path string) ([]Distribution, error) {
	// sanity check on path
	if path == "" {
		return nil, fmt.Errorf("path cannot be empty")
	}

	// Check if file exists and is a regular file
	fileInfo, err := os.Stat(path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, fmt.Errorf("file does not exist: %s", path)
		}
		return nil, fmt.Errorf("error accessing file: %w", err)
	}

	if !fileInfo.Mode().IsRegular() {
		return nil, fmt.Errorf("%s is not a regular file", path)
	}

	// Read file from relative path
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("error reading file: %w", err)
	}

	var distributions []Distribution
	if err := json.Unmarshal(data, &distributions); err != nil {
		return nil, fmt.Errorf("error unmarshaling JSON: %w", err)
	}

	return distributions, nil
}

func createMerkleTreeFromDistribution(distList []Distribution) (*merkletree.MerkleTree, error) {
	// Create data slice for merkle tree
	var data [][]byte

	for _, dist := range distList {
		// Add each operator-reward pair as a leaf
		for _, op := range dist.Earners {
			// Combine operator and reward into a single byte slice
			leaf := leafHash(op.Earner, op.Reward)
			data = append(data, leaf)
		}
	}

	// Create new merkle tree with default settings
	tree, err := merkletree.NewTree(
		merkletree.WithData(data),
		merkletree.WithHashType(sha3.New256()),
		merkletree.WithSorted(true),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create merkle tree: %w", err)
	}

	return tree, nil
}

func loadMerkleTreeFromFile(filePath string) (*merkletree.MerkleTree, error) {
	var tree *merkletree.MerkleTree
	// Add logic to load a Merkle tree from a file
	data, err := os.ReadFile(filePath)
	if err != nil {
		fmt.Println("Error reading file:", err)
		return tree, err
	}
	if err := json.Unmarshal(data, &tree); err != nil {
		fmt.Println("Error unmarshaling JSON:", err)
		return tree, err
	}
	return tree, nil
}

func init() {
	fmt.Println()
	RootCmd.AddCommand(merkleCmd)
	merkleCmd.AddCommand(merkleCreateCmd) // add "merkle create --distribution-file <file>" command
	merkleCmd.AddCommand(merkleLoadCmd)   // add "merkle load -f <file>" command
	merkleCmd.AddCommand(merkleProofCmd)  // add "merkle proof <earner> <reward> -f <file>" command

	// merkle create
	merkleCreateCmd.PersistentFlags().StringVar(&distributionFile, "distribution-file", "../data/data/distribution.json", "Path to the distribution JSON file")
	merkleCreateCmd.PersistentFlags().StringVarP(&merkleTreeFile, "tree-file", "f", "../data/tree/merkle.json", "Path to the Merkle tree JSON file")
	// hide "tree-file" flag
	err := merkleCreateCmd.PersistentFlags().MarkHidden("tree-file")
	if err != nil {
		fmt.Println("Error hiding flag:", err)
	}
	err = merkleCreateCmd.MarkPersistentFlagRequired("distribution-file")
	if err != nil {
		fmt.Println("Error marking flag as required:", err)
	}

	// merkle load
	merkleLoadCmd.PersistentFlags().StringVarP(&merkleTreeFile, "tree-file", "f", "../data/tree/merkle.json", "Path to the Merkle tree JSON file")
	err = merkleLoadCmd.MarkPersistentFlagRequired("tree-file")
	if err != nil {
		fmt.Println("Error marking flag as required:", err)
	}

	// merkle proof
	merkleProofCmd.PersistentFlags().StringVarP(&merkleTreeFile, "tree-file", "f", "../data/tree/merkle.json", "Path to the Merkle tree JSON file")

}

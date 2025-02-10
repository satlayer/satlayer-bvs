package reward

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"

	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/api"
	"github.com/satlayer/satlayer-bvs/bvs-api/chainio/types"

	"github.com/satlayer/satlayer-bvs/satlayer-cli/conf"
)

func fetchReward(userAddress string) ([]Response, error) {
	// Construct the API URL
	url := fmt.Sprintf("%s/api/v1/rewards/%s", conf.C.Chain.RewardAPI, userAddress)

	// Make HTTP GET request
	resp, err := http.Get(url)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch reward data: %v", err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			panic(err)
		}
	}(resp.Body)

	// Read response body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %v", err)
	}

	// Check if the response status is not successful
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API request failed with status code: %d, body: %s", resp.StatusCode, string(body))
	}

	// Parse the response into Response struct
	var response *EarnerResponse
	if err := json.Unmarshal(body, &response); err != nil {
		return nil, fmt.Errorf("failed to parse response data: %v", err)
	}

	return response.Rewards, nil
}

func claimReward(ctx context.Context, s api.RewardsCoordinator, address string, reward Response) (string, []Token, error) {
	fmt.Printf("reward info: %+v", reward)
	rootIndex := uint32(reward.RootIndex)
	earnerIndex := uint32(0)

	var earnerLeaf *types.ExecuteEarnerTreeMerkleLeaf
	earnerTreeProof := make([]byte, 0)
	earnerLeafHash := make([]string, 0)
	tokens := make([]Token, 0)
	for i, r := range reward.Earners {
		if r.Earner == address {
			earnerIndex = uint32(i)
			earnerLeaf = &types.ExecuteEarnerTreeMerkleLeaf{
				Earner:          r.Earner,
				EarnerTokenRoot: r.TokenHash,
			}
			tokens = append(tokens, r.Tokens...)
		} else {
			earnerHash, err := rpcEarnerLeafHash(s, r.Earner, r.TokenHash)
			if err != nil {
				panic(err)
			}
			earnerLeafHash = append(earnerLeafHash, earnerHash)
		}
	}
	if earnerLeaf == nil {
		return "", tokens, fmt.Errorf("earner not found")
	}
	if len(tokens) == 0 {
		return "", tokens, fmt.Errorf("no tokens found")
	}
	if len(earnerLeafHash) != 0 {
		earnerTreeProof, err := rpcMerkleizeLeaves(s, earnerLeafHash)
		if err != nil {
			panic(err)
		}
		earnerTreeProof = append(earnerTreeProof, earnerTreeProof...)
	}
	earnerProofByte := byteToUint16(earnerTreeProof)

	//
	tokenIndices := make([]uint32, 0)
	tokenLeaves := make([]types.TokenTreeMerkleLeaf, 0)
	tokenTreeProofs := make([][]uint16, 0)

	for i, token := range tokens {
		tokenIndices = append(tokenIndices, uint32(i))
		tokenLeaf := types.TokenTreeMerkleLeaf{
			Token:              token.Token,
			CumulativeEarnings: token.Amount,
		}
		tokenLeaves = append(tokenLeaves, tokenLeaf)
	}
	if len(tokenLeaves) == 1 {
		tokenTreeProofs = append(tokenTreeProofs, []uint16{})
	} else {
		for _, token := range tokenLeaves {
			tokenLeaveHash := make([]string, 0)
			for _, t := range tokenLeaves {
				if t.Token == token.Token {
					continue
				}
				tokenHash, err := rpcTokenHash(s, token.Token, token.CumulativeEarnings)
				if err != nil {
					panic(err)
				}
				tokenLeaveHash = append(tokenLeaveHash, tokenHash)
			}
			proofs, err := rpcMerkleizeLeaves(s, tokenLeaveHash)
			if err != nil {
				panic(err)
			}
			tokenProofs := byteToUint16(proofs)
			tokenTreeProofs = append(tokenTreeProofs, tokenProofs)
		}
	}
	fmt.Println("tokenTreeProff len: ", len(tokenTreeProofs))

	claim := types.ExeuteRewardsMerkleClaim{
		RootIndex:       rootIndex,
		EarnerIndex:     earnerIndex,
		EarnerTreeProof: earnerProofByte,
		EarnerLeaf:      *earnerLeaf,
		TokenIndices:    tokenIndices,
		TokenTreeProofs: tokenTreeProofs,
		TokenLeaves:     tokenLeaves,
	}
	result, err := s.ProcessClaim(ctx, claim, address)
	if err != nil {
		fmt.Println("ProcessClaim err: ", err)
	}

	return result.Hash.String(), tokens, nil
}

func rpcTokenHash(s api.RewardsCoordinator, token, amount string) (string, error) {
	resp, err := s.CalculateTokenLeafHash(token, amount)
	if err != nil {
		fmt.Println("CalculateTokenLeafHash err: ", err)
		return "", err
	}
	var hashResponse HashResponse
	if err := json.Unmarshal(resp.Data, &hashResponse); err != nil {
		fmt.Println("unmarshal err: ", err)
		return "", err
	}
	hashStr := base64.StdEncoding.EncodeToString(hashResponse.HashBinary)
	return hashStr, err
}

func rpcMerkleizeLeaves(s api.RewardsCoordinator, leaves []string) ([]byte, error) {
	hash := make([]byte, 0)
	resp, err := s.MerkleizeLeaves(leaves)
	if err != nil {
		fmt.Println("merkleizeLeaves err: ", err)
		return hash, err
	}
	var rsp MerkleizeLeavesResponse
	if err := json.Unmarshal(resp.Data, &rsp); err != nil {
		fmt.Println("unmarshal err: ", err)
		return hash, err
	}
	//merkleRoot := base64.StdEncoding.EncodeToString(merkleizeLeavesResponse.RootHashBinary)
	return rsp.RootHashBinary, err
}

func rpcEarnerLeafHash(s api.RewardsCoordinator, earner, rootHash string) (string, error) {
	resp, err := s.CalculateEarnerLeafHash(earner, rootHash)
	if err != nil {
		fmt.Println("CalculateEarnerLeafHash err: ", err)
		return "", err
	}
	var earnerLeafHashResponse EarnerLeafHashResponse
	err = json.Unmarshal(resp.Data, &earnerLeafHashResponse)
	hashStr := base64.StdEncoding.EncodeToString(earnerLeafHashResponse.RootHashBinary)
	return hashStr, err
}

func byteToUint16(b []byte) []uint16 {
	u16 := make([]uint16, len(b))
	for i, v := range b {
		u16[i] = uint16(v)
	}
	return u16
}

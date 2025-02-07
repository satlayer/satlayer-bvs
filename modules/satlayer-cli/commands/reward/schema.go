package reward

type Earner struct {
	Earner    string  `json:"earner"`
	TokenHash string  `json:"token_hash"`
	Tokens    []Token `json:"tokens"`
}

type Token struct {
	Token  string `json:"token"`
	Amount string `json:"amount"`
}

// Response structures
type Response struct {
	RootIndex   int64    `json:"root_index"`
	RootHash    string   `json:"root_hash"`
	ActivatedTs int64    `json:"activated_ts"`
	Earners     []Earner `json:"earners"`
}

type EarnerResponse struct {
	Rewards []Response `json:"rewards"`
}

type HashResponse struct {
	HashBinary []byte `json:"hash_binary"`
}

type MerkleizeLeavesResponse struct {
	RootHashBinary []byte `json:"root_hash_binary"`
}

type EarnerLeafHashResponse struct {
	RootHashBinary []byte `json:"hash_binary"`
}

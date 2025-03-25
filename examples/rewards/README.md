# Rewards Example Implementation

## File Structure

```txt
rewards/
├── data                        - Stores the data files output
│   ├── distribution            - Stores the distribution files
│   │   └── distribution.json
│   └── tree                    - Stores the merkle tree files created from distribution file
│       └── merkle.json
├── src
│   └── cmd                     - Contains the main command for the rewards example
│       ├── merkle.go           - Contains "merkle [command]" commands
│       └── root.go
├── go.mod
└── README.md
```

## Commands

### Creates Merkle Tree

Creates a merkle tree from the distribution file.

```sh
bvs-rewards merkle create --distribution-file <file>
```

### Generates Merkle Proof

Generates a merkle proof given earner address and amount
Either load the tree from a file or create it from the distribution file.

```sh
# load the tree from a file
bvs-rewards merkle load -f <merkle-tree-file>
```

then

```sh
bvs-rewards merkle proof <address> <amount>
```

or pass the tree file directly

```sh
bvs-rewards merkle proof <address> <amount> -f <merkle-tree-file>
```

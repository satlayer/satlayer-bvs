package indexer

type Event struct {
	BlockHeight int64
	TxHash      string
	EventType   string
	AttrMap     map[string]interface{}
}

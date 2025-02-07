package indexer

import (
	"context"
	"fmt"
	"log"
	"sync"
	"time"

	"github.com/cometbft/cometbft/libs/bytes"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cosmos/cosmos-sdk/client"
	tmtypes "github.com/tendermint/tendermint/types"
	"go.uber.org/zap"
	"golang.org/x/time/rate"
)

type EventIndexer struct {
	mu                 sync.Mutex
	clientCtx          client.Context
	contractAddress    string
	startBlockHeight   int64
	currentBlockHeight int64
	IsUpToDate         bool
	eventTypes         []string
	limiter            *rate.Limiter
	maxRetries         int
	processingQueue    chan *Event
}

func NewEventIndexer(clientCtx client.Context, contractAddress string, startBlockHeight int64, eventTypes []string, rateLimit rate.Limit, maxRetries int) *EventIndexer {
	indexer := &EventIndexer{
		clientCtx:          clientCtx,
		contractAddress:    contractAddress,
		startBlockHeight:   startBlockHeight,
		currentBlockHeight: startBlockHeight,
		eventTypes:         eventTypes,
		limiter:            rate.NewLimiter(rateLimit, 1),
		maxRetries:         maxRetries,
		processingQueue:    make(chan *Event, 1000),
	}
	return indexer
}

func (ei *EventIndexer) Run(ctx context.Context) (chan *Event, error) {
	zap.L().Info("Indexer starting block height", zap.Int64("block_height", ei.startBlockHeight))
	Go(func() { ei.syncHistoryBlocks(ctx) })
	Go(func() { ei.pollNewBlocks(ctx) })
	return ei.processingQueue, nil
}

// syncEvents Synchronize historical blocks
func (ei *EventIndexer) syncHistoryBlocks(ctx context.Context) {
	zap.L().Info("Syncing historical blocks...")
	for {
		latestHeight, err := ei.getLatestBlockHeight(ctx)
		if err != nil {
			zap.L().Error("Error getting latest block height:", zap.Error(err))
			time.Sleep(time.Second * 5)
			continue
		}

		if ei.currentBlockHeight >= latestHeight {
			ei.IsUpToDate = true
			zap.L().Info("Caught up with the latest block")
			return
		}

		endHeight := ei.currentBlockHeight + 100 // Process 100 blocks at a time
		if endHeight > latestHeight {
			endHeight = latestHeight
		}

		err = ei.processBlockRange(ctx, ei.currentBlockHeight, endHeight)
		if err != nil {
			zap.L().Error("Error processing block range:", zap.Error(err))
			time.Sleep(time.Second * 5)
			continue
		}

		ei.currentBlockHeight = endHeight + 1
	}
}

// pollNewBlocks Poll for new blocks
func (ei *EventIndexer) pollNewBlocks(ctx context.Context) {
	zap.L().Info("Starting to poll for new blocks...")
	ticker := time.NewTicker(time.Second * 5)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if !ei.IsUpToDate {
				continue
			}

			latestHeight, err := ei.getLatestBlockHeight(ctx)
			if err != nil {
				zap.L().Error("Error getting latest block height:", zap.Error(err))
				continue
			}

			if latestHeight >= ei.currentBlockHeight {
				err = ei.processBlockRange(ctx, ei.currentBlockHeight, latestHeight)
				if err != nil {
					zap.L().Error("Error processing new blocks:", zap.Error(err))
					continue
				}
				ei.currentBlockHeight = latestHeight + 1
			}
		}
	}
}

// getLatestBlockHeight Get the latest block height
func (ei *EventIndexer) getLatestBlockHeight(ctx context.Context) (int64, error) {
	for retry := 0; retry < ei.maxRetries; retry++ {
		if err := ei.limiter.Wait(ctx); err != nil {
			return 0, fmt.Errorf("rate limit error: %w", err)
		}

		res, err := ei.clientCtx.Client.Status(ctx)
		if err == nil {
			return res.SyncInfo.LatestBlockHeight, nil
		}

		log.Printf("Error getting latest block height (attempt %d/%d): %v", retry+1, ei.maxRetries, err)
		time.Sleep(time.Second * time.Duration(retry+1))
	}

	return 0, fmt.Errorf("failed to get latest block height after %d attempts", ei.maxRetries)
}

// processBlockRange Process blocks within a specified range
func (ei *EventIndexer) processBlockRange(ctx context.Context, startHeight, endHeight int64) error {
	for height := startHeight; height <= endHeight; height++ {
		if err := ei.limiter.Wait(ctx); err != nil {
			return fmt.Errorf("rate limit error: %w", err)
		}

		block, err := ei.clientCtx.Client.Block(ctx, &height)
		if err != nil {
			return fmt.Errorf("error fetching block at height %d: %w", height, err)
		}

		for _, tx := range block.Block.Data.Txs {
			txHash := bytes.HexBytes(tmtypes.Tx(tx).Hash()).String()
			txResponse, err := ei.clientCtx.Client.Tx(ctx, tmtypes.Tx(tx).Hash(), false)
			if err != nil {
				return fmt.Errorf("error fetching transaction %s: %w", txHash, err)
			}

			ei.processTxEvents(ctx, txResponse, height)
		}

		// Update the currently processed block height
		ei.mu.Lock()
		ei.currentBlockHeight = height
		ei.mu.Unlock()
	}
	return nil
}

func (ei *EventIndexer) processTxEvents(ctx context.Context, txResponse *coretypes.ResultTx, height int64) {
	for _, event := range txResponse.TxResult.Events {
		if !ei.shouldIndexEvent(event.Type) {
			continue
		}
		attrMap := make(map[string]interface{})
		for _, attr := range event.Attributes {
			attrMap[string(attr.Key)] = attr.Value
		}
		// Filter contracts
		if contractAddress, ok := attrMap["_contract_address"]; ok && contractAddress == ei.contractAddress {
			newEvent := &Event{
				BlockHeight: height,
				TxHash:      txResponse.Hash.String(),
				EventType:   event.Type,
				AttrMap:     attrMap,
			}
			// Put the event into the processing queue
			ei.processingQueue <- newEvent
		}
	}
}

// shouldIndexEvent Check if this event type should be indexed
func (ei *EventIndexer) shouldIndexEvent(eventType string) bool {
	for _, t := range ei.eventTypes {
		if t == eventType {
			return true
		}
	}
	return false
}

package indexer

import (
	"context"
	"fmt"
	"log"
	"math/big"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	"go.uber.org/zap"
	"golang.org/x/time/rate"
)

type ETHIndexer struct {
	mu                 sync.Mutex
	ethClient          *ethclient.Client
	contractABI        *abi.ABI
	contractAddress    common.Address
	startBlockHeight   uint64
	currentBlockHeight uint64
	IsUpToDate         bool
	eventTypes         []common.Hash
	limiter            *rate.Limiter
	maxRetries         int
	processingQueue    chan *Event
}

func NewETHIndexer(ethClient *ethclient.Client, contractABI *abi.ABI, contractAddress common.Address, startBlockHeight uint64, eventTypes []common.Hash, rateLimit rate.Limit, maxRetries int) *ETHIndexer {
	indexer := &ETHIndexer{
		ethClient:          ethClient,
		contractABI:        contractABI,
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

func (ei *ETHIndexer) Run(ctx context.Context) (chan *Event, error) {
	zap.L().Info("Indexer starting block height", zap.Uint64("block_height", ei.startBlockHeight))
	Go(func() { ei.syncHistoryBlocks(ctx) })
	Go(func() { ei.pollNewBlocks(ctx) })
	return ei.processingQueue, nil
}

// syncEvents Synchronize historical blocks
func (ei *ETHIndexer) syncHistoryBlocks(ctx context.Context) {
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
func (ei *ETHIndexer) pollNewBlocks(ctx context.Context) {
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
func (ei *ETHIndexer) getLatestBlockHeight(ctx context.Context) (uint64, error) {
	for retry := 0; retry < ei.maxRetries; retry++ {
		if err := ei.limiter.Wait(ctx); err != nil {
			return 0, fmt.Errorf("rate limit error: %w", err)
		}

		latestBlockHeight, err := ei.ethClient.BlockNumber(ctx)
		if err == nil {
			return latestBlockHeight, nil
		}

		log.Printf("Error getting latest block height (attempt %d/%d): %v", retry+1, ei.maxRetries, err)
		time.Sleep(time.Second * time.Duration(retry+1))
	}

	return 0, fmt.Errorf("failed to get latest block height after %d attempts", ei.maxRetries)
}

// processBlockRange Process blocks within a specified range
func (ei *ETHIndexer) processBlockRange(ctx context.Context, startHeight, endHeight uint64) error {
	topics := [][]common.Hash{ei.eventTypes}
	query := ethereum.FilterQuery{
		FromBlock: big.NewInt(int64(startHeight)),
		ToBlock:   big.NewInt(int64(endHeight)),
		Addresses: []common.Address{ei.contractAddress},
		Topics:    topics,
	}
	logs, err := ei.ethClient.FilterLogs(ctx, query)
	if err != nil {
		return fmt.Errorf("error fetching logs at startHeight %d endHeight %d: %w", startHeight, endHeight, err)
	}

	for _, item := range logs {
		if err = ei.processTxEvents(item); err != nil {
			return fmt.Errorf("error process log at blockNumber %d TxHash %d Index%d: %w", item.BlockNumber, item.TxHash, item.Index, err)
		}
	}

	// Update the currently processed block height
	ei.mu.Lock()
	ei.currentBlockHeight = endHeight
	ei.mu.Unlock()
	return nil
}

func (ei *ETHIndexer) processTxEvents(log types.Log) error {
	eventName := ""
	for _, event := range ei.contractABI.Events {
		if event.ID.String() == log.Topics[0].String() {
			eventName = event.Name
			break
		}
	}
	// Skip if event name is empty or no parser registered
	if eventName == "" {
		return nil
	}
	result, err := ei.eventParser(eventName, log)
	if err != nil {
		return fmt.Errorf("failed to parse event %s: %w", eventName, err)
	}
	newEvent := &Event{
		BlockHeight: int64(log.BlockNumber),
		TxHash:      log.TxHash.Hex(),
		EventType:   eventName,
		AttrMap:     result,
	}
	// Put the event into the processing queue
	ei.processingQueue <- newEvent
	return nil
}

func (ei *ETHIndexer) eventParser(eventName string, log types.Log) (map[string]interface{}, error) {
	result := make(map[string]interface{})
	event := ei.contractABI.Events[eventName]

	if err := ei.contractABI.UnpackIntoMap(result, eventName, log.Data); err != nil {
		return nil, fmt.Errorf("failed to unpack log data: %w", err)
	}

	var indexed abi.Arguments
	for _, arg := range event.Inputs {
		if arg.Indexed {
			indexed = append(indexed, arg)
		}
	}
	if err := abi.ParseTopicsIntoMap(result, indexed, log.Topics[1:]); err != nil {
		return nil, fmt.Errorf("failed to parse topics in event %s: %w", eventName, err)
	}

	return result, nil
}

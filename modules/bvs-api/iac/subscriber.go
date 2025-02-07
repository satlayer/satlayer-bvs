package iac

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/segmentio/kafka-go"
	"go.uber.org/zap"
)

type Subscriber interface {
	subscriber(ctx context.Context, callback func(message string))
}

type kafkaSubscriber struct {
	reader *kafka.Reader
}

// NewSubscriber Subsequent adjustments can be made based on actual scenarios to support subscribing to multiple topics.
func NewSubscriber(brokers []string, topic string, groupID string) Subscriber {
	reader := kafka.NewReader(kafka.ReaderConfig{
		Brokers:        brokers,
		Topic:          topic,
		CommitInterval: 1 * time.Second,
		GroupID:        groupID,
		StartOffset:    kafka.FirstOffset,
	})
	return &kafkaSubscriber{reader: reader}
}

func (k *kafkaSubscriber) subscriber(ctx context.Context, callback func(message string)) {
	// Stop spending gracefully
	stopChan := make(chan struct{})
	// Start a goroutine to listen for system signals
	Go(func() {
		sigChan := make(chan os.Signal, 1)
		signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
		select {
		case sig := <-sigChan:
			fmt.Printf("Received a signal: %s\n", sig)
			close(stopChan)
		case <-ctx.Done():
			close(stopChan)
		}
	})

	for {
		select {
		case <-stopChan:
			// When a stop signal or ctx cancel signal is received, the loop exits.
			if k.reader != nil {
				k.reader.Close()
			}
			return
		default:
			msg, err := k.reader.ReadMessage(ctx)
			if err != nil {
				zap.L().Error("read message failed", zap.Error(err))
				continue
			}
			callback(string(msg.Value))
		}
	}
}

func Go(f func()) {
	go func(f func()) {
		defer func() {
			if e := recover(); e != nil {
				zap.L().DPanic("panic recover", zap.Any("Panic", e))
			}
		}()
		f()
	}(f)
}

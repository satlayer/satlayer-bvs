package iac

import (
	"context"
	"time"

	"github.com/segmentio/kafka-go"
)

type Publisher interface {
	publish(ctx context.Context, messages ...Msg) error
}
type kafkaPublisher struct {
	writer *kafka.Writer
}

type Msg struct {
	PartitionKey string
	Message      string
}

// NewPublisher When automatically creating a topic is allowed, if the topic does not exist,
// the topic will be created for the first time, but the message will fail to be sent. Just try again.
func NewPublisher(brokers []string, topic string) Publisher {
	writer := &kafka.Writer{
		Addr:                   kafka.TCP(brokers...),
		Topic:                  topic,
		Balancer:               &kafka.Hash{},
		WriteTimeout:           1 * time.Second,
		RequiredAcks:           kafka.RequireOne,
		MaxAttempts:            3,
		AllowAutoTopicCreation: true,
	}
	return &kafkaPublisher{writer: writer}
}

func (k *kafkaPublisher) publish(ctx context.Context, messages ...Msg) error {
	msgs := make([]kafka.Message, 0, len(messages))
	for _, msg := range messages {
		msgs = append(msgs, kafka.Message{
			Key:   []byte(msg.PartitionKey),
			Value: []byte(msg.Message),
			Time:  time.Now(),
		})
	}
	defer k.writer.Close()
	if err := k.writer.WriteMessages(ctx, msgs...); err != nil {
		return err
	}
	return nil
}

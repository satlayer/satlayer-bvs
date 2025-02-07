package iac

import "context"

type IAC interface {
	RegisterSubscriber(ctx context.Context, callback func(msg string))
	Publish(ctx context.Context, msg ...Msg) error
}

type Facade struct {
	publisher  Publisher
	subscriber Subscriber
}

func NewIACFacade(publisher Publisher, subscriber Subscriber) IAC {
	return &Facade{
		publisher:  publisher,
		subscriber: subscriber,
	}
}

func (f Facade) RegisterSubscriber(ctx context.Context, callback func(msg string)) {
	f.subscriber.subscriber(ctx, callback)
}

func (f Facade) Publish(ctx context.Context, msg ...Msg) error {
	return f.publisher.publish(ctx, msg...)
}

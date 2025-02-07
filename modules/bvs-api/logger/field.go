package logger

type Field struct {
	Key string
	Val interface{}
}

func WithField(key string, val interface{}) Field {
	return Field{Key: key, Val: val}
}

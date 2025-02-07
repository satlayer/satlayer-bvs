package indexer

import (
	"fmt"
	"reflect"

	"go.uber.org/zap"
)

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

// StructToMap convert to map[string]interface{}
func StructToMap(input interface{}) map[string]string {
	result := make(map[string]string)
	val := reflect.ValueOf(input)
	typ := reflect.TypeOf(input)

	if val.Kind() == reflect.Ptr {
		val = val.Elem()
		typ = typ.Elem()
	}

	if val.Kind() != reflect.Struct {
		panic("input must be a struct or pointer to struct")
	}

	for i := 0; i < val.NumField(); i++ {
		field := typ.Field(i)
		fieldValue := val.Field(i)
		if !fieldValue.CanInterface() {
			continue
		}
		result[field.Name] = fmt.Sprintf("%v", fieldValue.Interface())
	}

	return result
}

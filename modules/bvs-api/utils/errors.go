package utils

import (
	"errors"
	"fmt"
)

func TypedErr(e interface{}) error {
	switch t := e.(type) {
	case error:
		return t
	case string:
		if t == "" {
			return nil
		}
		return errors.New(t)
	default:
		return nil
	}
}

func WrapError(mainErr, subErr interface{}) error {
	main := TypedErr(mainErr)
	sub := TypedErr(subErr)

	switch {
	case main == nil && sub == nil:
		return nil
	case main == nil:
		return sub
	case sub == nil:
		return main
	default:
		return fmt.Errorf("%w: %v", main, sub)
	}
}

package utils

import (
	"errors"
	"fmt"
	"strings"
)

var (
	ErrInvalidUrl          = errors.New("invalid url")
	ErrInvalidGithubRawUrl = errors.New("invalid github raw url")
	ErrInvalidText         = fmt.Errorf("invalid text format, doesn't conform to regex %s", TextRegex)
	ErrTextTooLong         = func(limit int) error {
		return fmt.Errorf("text should be less than %d characters", limit)
	}
	ErrEmptyText             = errors.New("text is empty")
	ErrInvalidImageExtension = errors.New(
		"invalid image extension. only " + strings.Join(ImageExtensions, ",") + " is supported",
	)
	ErrInvalidImageMimeType     = errors.New("invalid image mime-type. only png is supported")
	ErrInvalidUrlLength         = errors.New("url length should be no larger than 1024 character")
	ErrUrlPointingToLocalServer = errors.New("url should not point to local server")
	ErrEmptyUrl                 = errors.New("url is empty")
	ErrInvalidTwitterUrlRegex   = errors.New(
		"invalid twitter url, it should be of the format https://twitter.com/<username> or https://x.com/<username>",
	)
	ErrResponseTooLarge = errors.New("response too large, allowed size is 1 MB")
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

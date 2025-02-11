package resp

import (
	"encoding/json"
)

var _ Error = (*err)(nil)

type Error interface {
	WithData(data interface{}) Error
	ToString() string
}

type err struct {
	Code int         `json:"code"`
	Msg  string      `json:"msg"`
	Data interface{} `json:"data"`
}

func NewError(code int, msg string) Error {
	return &err{
		Code: code,
		Msg:  msg,
		Data: nil,
	}
}

func (e *err) WithData(data interface{}) Error {
	e.Data = data
	return e
}

func (e *err) ToString() string {
	err := &struct {
		Code int         `json:"code"`
		Msg  string      `json:"msg"`
		Data interface{} `json:"data"`
	}{
		Code: e.Code,
		Msg:  e.Msg,
		Data: e.Data,
	}

	raw, _ := json.Marshal(err)
	return string(raw)
}

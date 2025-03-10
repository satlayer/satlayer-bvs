package logging

import (
	"fmt"

	"github.com/satlayer/satlayer-bvs/bvs-api/logger"
)

type LogLevel string

type level = string

const (
	Development LogLevel = "development" // prints debug and above
	Production  LogLevel = "production"  // prints info and above
	info        level    = "info"
	debug       level    = "debug"
	bvsName              = "comparablelayer"
)

type ZapLogger struct {
	logger logger.Logger
}

var _ Logger = (*ZapLogger)(nil)

func NewMockZapLogger(env LogLevel) (Logger, error) {
	elkLogger := logger.NewMockELKLogger()
	if env == Production {
		elkLogger.SetLogLevel(info)
	} else if env == Development {
		elkLogger.SetLogLevel(debug)
	} else {
		panic(fmt.Sprintf("Unknown environment. Expected %s or %s. Received %s.", Development, Production, env))
	}

	return &ZapLogger{logger: elkLogger}, nil
}

func (z *ZapLogger) Debug(msg string, tags ...any) {
	z.logger.Debug(msg, z.logger.SweetenFields(tags)...)
}

func (z *ZapLogger) Info(msg string, tags ...any) {
	z.logger.Info(msg, z.logger.SweetenFields(tags)...)
}

func (z *ZapLogger) Warn(msg string, tags ...any) {
	z.logger.Warn(msg, z.logger.SweetenFields(tags)...)
}

func (z *ZapLogger) Error(msg string, tags ...any) {
	z.logger.Error(msg, z.logger.SweetenFields(tags)...)
}

func (z *ZapLogger) Fatal(msg string, tags ...any) {
	z.logger.Fatal(msg, z.logger.SweetenFields(tags)...)
}

func (z *ZapLogger) Debugf(template string, args ...interface{}) {
	z.logger.Debugf(template, args...)
}

func (z *ZapLogger) Infof(template string, args ...interface{}) {
	z.logger.Infof(template, args...)
}

func (z *ZapLogger) Warnf(template string, args ...interface{}) {
	z.logger.Warnf(template, args...)
}

func (z *ZapLogger) Errorf(template string, args ...interface{}) {
	z.logger.Errorf(template, args...)
}

func (z *ZapLogger) Fatalf(template string, args ...interface{}) {
	z.logger.Fatalf(template, args...)
}

func (z *ZapLogger) GetElkLogger() logger.Logger {
	return z.logger
}

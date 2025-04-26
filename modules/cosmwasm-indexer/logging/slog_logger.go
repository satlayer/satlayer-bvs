package logging

import (
	"fmt"
	"log/slog"
	"os"
	"strings"

	tmctypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/forbole/juno/v6/logging"
	"github.com/forbole/juno/v6/modules"
	"github.com/forbole/juno/v6/types"
)

var (
	_ logging.Logger = &slogLogger{}
)

// slogLogger implements the logging.Logger interface for slog
type slogLogger struct {
	logger *slog.Logger
	format string
}

// NewSlogLogger creates a default slogLogger instance.
func NewSlogLogger() logging.Logger {
	return &slogLogger{
		logger: slog.Default(),
	}
}

// SetLogLevel sets the log level
func (l *slogLogger) SetLogLevel(level string) error {
	opts, err := parseLogLevel(level)
	if err != nil {
		return err
	}

	if err = l.parseLogFormat(l.format, opts); err != nil {
		return err
	}

	return nil
}

// SetLogFormat sets the log format
func (l *slogLogger) SetLogFormat(format string) error {
	l.format = strings.ToLower(format)
	return nil
}

// Info logs an info message
func (l *slogLogger) Info(msg string, keyvals ...interface{}) {
	l.logger.Info(msg, keyvals...)
}

// Debug logs a debug message
func (l *slogLogger) Debug(msg string, keyvals ...interface{}) {
	l.logger.Debug(msg, keyvals...)
}

// Error logs an error message
func (l *slogLogger) Error(msg string, keyvals ...interface{}) {
	l.logger.Error(msg, keyvals...)
}

// GenesisError logs a genesis error
func (l *slogLogger) GenesisError(module modules.Module, err error) {
	l.logger.Error("genesis error", logging.LogKeyModule, module.Name(), "error", err.Error())
}

// BlockError logs a block error
func (l *slogLogger) BlockError(module modules.Module, block *tmctypes.ResultBlock, err error) {
	l.logger.Error("block error", logging.LogKeyModule, module.Name(),
		logging.LogKeyHeight, block.Block.Height, "error", err.Error())
}

// EventsError logs an events error
func (l *slogLogger) EventsError(module modules.Module, results *tmctypes.ResultBlock, err error) {
	l.logger.Error("events error", logging.LogKeyModule, module.Name(),
		logging.LogKeyHeight, results.Block.Height, "error", err.Error())
}

// TxError logs a transaction error
func (l *slogLogger) TxError(module modules.Module, tx *types.Transaction, err error) {
	l.logger.Error("transaction error", logging.LogKeyModule, module.Name(),
		logging.LogKeyTxHash, tx.TxHash, "error", err.Error())
}

// MsgError logs a message error
func (l *slogLogger) MsgError(module modules.Module, tx *types.Transaction, msg types.Message, err error) {
	l.logger.Error("message error", logging.LogKeyModule, module.Name(),
		logging.LogKeyTxHash, tx.TxHash, logging.LogKeyMsgType, msg.GetType(), "error", err.Error())
}

func parseLogLevel(level string) (*slog.HandlerOptions, error) {
	var lvl slog.LevelVar
	switch strings.ToLower(level) {
	case "debug":
		lvl.Set(slog.LevelDebug)
	case "info":
		lvl.Set(slog.LevelInfo)
	case "warn":
		lvl.Set(slog.LevelWarn)
	case "error":
		lvl.Set(slog.LevelError)
	default:
		return nil, fmt.Errorf("invalid log level: %s", level)
	}

	return &slog.HandlerOptions{
		Level: &lvl,
	}, nil
}

func (l *slogLogger) parseLogFormat(format string, opts *slog.HandlerOptions) error {
	switch format {
	case "text":
		l.logger = slog.New(slog.NewTextHandler(os.Stdout, opts))
	case "json":
		l.logger = slog.New(slog.NewJSONHandler(os.Stdout, opts))
	default:
		return fmt.Errorf("invalid log format: %s", format)
	}

	slog.SetDefault(l.logger)
	return nil
}

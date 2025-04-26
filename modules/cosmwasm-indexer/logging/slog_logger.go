package logging

import (
	"fmt"
	"log/slog"
	"os"
	"strings"

	tmctypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/forbole/juno/v6/modules"
	"github.com/forbole/juno/v6/types"
)

// SlogLogger implements the logging.Logger interface using slog
type SlogLogger struct {
	logger *slog.Logger
}

// NewSlogLogger creates a new SlogLogger instance
func NewSlogLogger() *SlogLogger {
	return &SlogLogger{
		logger: slog.Default(),
	}
}

// SetLogLevel sets the log level
func (l *SlogLogger) SetLogLevel(level string) error {
	var slogLevel slog.Level
	switch strings.ToLower(level) {
	case "debug":
		slogLevel = slog.LevelDebug
	case "info":
		slogLevel = slog.LevelInfo
	case "warn":
		slogLevel = slog.LevelWarn
	case "error":
		slogLevel = slog.LevelError
	default:
		return fmt.Errorf("invalid log level: %s", level)
	}

	l.logger = slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		Level: slogLevel,
	}))
	return nil
}

// SetLogFormat sets the log format
func (l *SlogLogger) SetLogFormat(format string) error {
	// slog only supports text and JSON formats
	switch strings.ToLower(format) {
	case "text":
		l.logger = slog.New(slog.NewTextHandler(os.Stdout, nil))
	case "json":
		l.logger = slog.New(slog.NewJSONHandler(os.Stdout, nil))
	default:
		return fmt.Errorf("invalid log format: %s", format)
	}
	return nil
}

// Info logs an info message
func (l *SlogLogger) Info(msg string, keyvals ...interface{}) {
	l.logger.Info(msg, keyvals...)
}

// Debug logs a debug message
func (l *SlogLogger) Debug(msg string, keyvals ...interface{}) {
	l.logger.Debug(msg, keyvals...)
}

// Error logs an error message
func (l *SlogLogger) Error(msg string, keyvals ...interface{}) {
	l.logger.Error(msg, keyvals...)
}

// GenesisError logs a genesis error
func (l *SlogLogger) GenesisError(module modules.Module, err error) {
	l.logger.Error("genesis error",
		"module", module.Name(),
		"error", err.Error(),
	)
}

// BlockError logs a block error
func (l *SlogLogger) BlockError(module modules.Module, block *tmctypes.ResultBlock, err error) {
	l.logger.Error("block error",
		"module", module.Name(),
		"height", block.Block.Height,
		"error", err.Error(),
	)
}

// EventsError logs an events error
func (l *SlogLogger) EventsError(module modules.Module, results *tmctypes.ResultBlock, err error) {
	l.logger.Error("events error",
		"module", module.Name(),
		"height", results.Block.Height,
		"error", err.Error(),
	)
}

// TxError logs a transaction error
func (l *SlogLogger) TxError(module modules.Module, tx *types.Transaction, err error) {
	l.logger.Error("transaction error",
		"module", module.Name(),
		"hash", tx.TxHash,
		"error", err.Error(),
	)
}

// MsgError logs a message error
func (l *SlogLogger) MsgError(module modules.Module, tx *types.Transaction, msg types.Message, err error) {
	l.logger.Error("message error",
		"module", module.Name(),
		"tx_hash", tx.TxHash,
		"msg_type", msg.GetType(),
		"error", err.Error(),
	)
}

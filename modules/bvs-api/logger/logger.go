package logger

import (
	"net"
	"strings"

	logstash "github.com/bshuster-repo/logrus-logstash-hook"
	"github.com/sirupsen/logrus"
)

var (
	elkLogger *ELKLogger
)

type Logger interface {
	SetLogLevel(level string)

	Info(msg string, fields ...Field)
	Warn(msg string, fields ...Field)
	Error(msg string, fields ...Field)
	Fatal(msg string, fields ...Field)
	Debug(msg string, fields ...Field)

	Infof(format string, args ...interface{})
	Warnf(format string, args ...interface{})
	Errorf(format string, args ...interface{})
	Fatalf(format string, args ...interface{})
	Debugf(format string, args ...interface{})

	SweetenFields(args []interface{}) []Field
}

type ELKLogger struct {
	logger *logrus.Logger
}

var _ Logger = (*ELKLogger)(nil)

func NewELKLogger(bvsName string) Logger {
	if elkLogger == nil {
		logger := logrus.New()
		// todo There are security issues. The official version needs to protect the server address.
		conn, err := net.Dial("tcp", "3.107.101.202:5044")
		if err != nil {
			logger.Fatalf("Failed to connect to Logstash: %v", err)
		}
		hook := logstash.New(conn, logstash.DefaultFormatter(logrus.Fields{
			"bvs_name": bvsName,
		}))
		logger.Hooks.Add(hook)
		elkLogger = &ELKLogger{logger: logger}
	}

	return elkLogger
}

func (l *ELKLogger) SetLogLevel(level string) {
	switch strings.ToLower(level) {
	case "debug":
		l.logger.SetLevel(logrus.DebugLevel)
	case "info":
		l.logger.SetLevel(logrus.InfoLevel)
	case "warn":
		l.logger.SetLevel(logrus.WarnLevel)
	case "error":
		l.logger.SetLevel(logrus.ErrorLevel)
	case "fatal":
		l.logger.SetLevel(logrus.FatalLevel)
	default:
		l.logger.SetLevel(logrus.InfoLevel)
	}
}

func (l *ELKLogger) Info(msg string, fields ...Field) {
	l.logger.WithFields(l.fmtFields(fields...)).Info(msg)
}

func (l *ELKLogger) Warn(msg string, fields ...Field) {
	l.logger.WithFields(l.fmtFields(fields...)).Warn(msg)
}

func (l *ELKLogger) Error(msg string, fields ...Field) {
	l.logger.WithFields(l.fmtFields(fields...)).Error(msg)
}

func (l *ELKLogger) Fatal(msg string, fields ...Field) {
	l.logger.WithFields(l.fmtFields(fields...)).Fatal(msg)
}

func (l *ELKLogger) Debug(msg string, fields ...Field) {
	l.logger.WithFields(l.fmtFields(fields...)).Debug(msg)
}

func (l *ELKLogger) Infof(format string, args ...interface{}) {
	l.logger.Infof(format, args...)
}

func (l *ELKLogger) Warnf(format string, args ...interface{}) {
	l.logger.Warnf(format, args...)
}

func (l *ELKLogger) Errorf(format string, args ...interface{}) {
	l.logger.Errorf(format, args...)
}

func (l *ELKLogger) Fatalf(format string, args ...interface{}) {
	l.logger.Fatalf(format, args...)
}

func (l *ELKLogger) Debugf(format string, args ...interface{}) {
	l.logger.Debugf(format, args...)
}

func (l *ELKLogger) SweetenFields(args []interface{}) []Field {
	if len(args) == 0 {
		return []Field{}
	}

	var (
		fields    = make([]Field, 0, len(args))
		seenError bool
	)

	for i := 0; i < len(args); {
		if f, ok := args[i].(Field); ok {
			fields = append(fields, f)
			i++
			continue
		}

		if err, ok := args[i].(error); ok {
			if !seenError {
				seenError = true
				fields = append(fields, WithField("error", err))
			}
			i++
			continue
		}
		if i == len(args)-1 {
			break
		}

		key, val := args[i], args[i+1]
		if keyStr, ok := key.(string); ok {
			fields = append(fields, WithField(keyStr, val))
		}
		i += 2
	}
	return fields
}

func (l *ELKLogger) fmtFields(fields ...Field) map[string]interface{} {
	if len(fields) == 0 {
		return make(map[string]interface{})
	}
	fieldsMap := make(map[string]interface{}, len(fields))
	for _, field := range fields {
		fieldsMap[field.Key] = field.Val
	}
	return fieldsMap
}

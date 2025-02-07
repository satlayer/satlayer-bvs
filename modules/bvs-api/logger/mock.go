package logger

import "fmt"

var (
	mockELKLogger *MockELKLogger
)

type MockELKLogger struct {
}

var _ Logger = (*MockELKLogger)(nil)

func (m MockELKLogger) SetLogLevel(level string) {
	// mock logger
}

func (m MockELKLogger) Info(msg string, fields ...Field) {
	// mock logger
	fmt.Printf("%s %+v \n", msg, fields)
}

func (m MockELKLogger) Warn(msg string, fields ...Field) {
	// mock logger
	fmt.Printf("%s %+v \n", msg, fields)
}

func (m MockELKLogger) Error(msg string, fields ...Field) {
	// mock logger
	fmt.Printf("%s %+v \n", msg, fields)
}

func (m MockELKLogger) Fatal(msg string, fields ...Field) {
	// mock logger
	fmt.Printf("%s %+v \n", msg, fields)
}

func (m MockELKLogger) Debug(msg string, fields ...Field) {
	// mock logger
	fmt.Printf("%s %+v \n", msg, fields)
}

func (m MockELKLogger) Infof(format string, args ...interface{}) {
	// mock logger
	fmt.Printf(format, args...)
}

func (m MockELKLogger) Warnf(format string, args ...interface{}) {
	// mock logger
	fmt.Printf(format, args...)
}

func (m MockELKLogger) Errorf(format string, args ...interface{}) {
	// mock logger
	fmt.Printf(format, args...)
}

func (m MockELKLogger) Fatalf(format string, args ...interface{}) {
	// mock logger
	fmt.Printf(format, args...)
}

func (m MockELKLogger) Debugf(format string, args ...interface{}) {
	// mock logger
	fmt.Printf(format, args...)
}

func (m MockELKLogger) SweetenFields(args []interface{}) []Field {
	// mock logger
	return []Field{}
}

func NewMockELKLogger() Logger {
	once.Do(func() {
		mockELKLogger = &MockELKLogger{}
	})

	return mockELKLogger
}

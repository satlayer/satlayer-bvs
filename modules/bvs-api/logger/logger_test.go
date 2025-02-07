package logger

import (
	"testing"
)

func TestLogging(t *testing.T) {
	l := NewELKLogger("bvs100")
	l.SetLogLevel("debug")
	// info demo
	l.Info("this is a info log test")
	// warn demo
	l.Warn("this is a warn log test")
	// error demo
	l.Error("this is a error log test", WithField("age", 100), WithField("gender", "man"))
	// debug demo
	l.Debug("this is a debug log test")
	// fatal demo
	//l.Fatal("this is a fatal log test")
}

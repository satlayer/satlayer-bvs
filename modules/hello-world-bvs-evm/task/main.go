package main

import (
	"fmt"
	"os"

	"github.com/satlayer/satlayer-bvs/hello-world-bvs-evm/task/core"
	"github.com/satlayer/satlayer-bvs/hello-world-bvs-evm/task/task"
)

// main checks parameters and runs the appropriate task based on the provided command-line argument.
//
// Need a parameter: caller or monitor
// No return values.
func main() {
	// init config
	core.InitConfig()
	// check parameters
	if len(os.Args) < 2 {
		fmt.Println("please provide a parameter: caller or monitor")
		os.Exit(1)
	}

	switch os.Args[1] {
	case "caller":
		task.RunCaller()
	case "monitor":
		task.RunMonitor()
	default:
		fmt.Println("please input param: caller or monitor")
		os.Exit(1)
	}
}

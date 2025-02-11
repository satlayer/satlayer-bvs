package conf

import (
	"fmt"
	"runtime/debug"
)

const Version = "0.1.0"

func GetVersion() string {
	commitHash := ""
	if info, ok := debug.ReadBuildInfo(); ok {
		for _, setting := range info.Settings {
			if setting.Key == "vcs.revision" {
				commitHash = setting.Value
			}
		}
	}
	ver := fmt.Sprintf("%s (%s)", Version, commitHash)
	return ver
}

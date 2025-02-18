package bvs

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"
)

// ReadArtifact returns the wasm binary for the given crate.name
func ReadArtifact(crate string) ([]byte, error) {
	_, currentFile, _, _ := runtime.Caller(0)
	baseDir := filepath.Dir(currentFile)

	artifactName := strings.ReplaceAll(crate, "-", "_")
	targetFile := filepath.Join(baseDir, "../node_modules/@satlayer", crate, "artifacts", artifactName+".wasm")
	return os.ReadFile(targetFile)
}

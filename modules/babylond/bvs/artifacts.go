package bvs

import (
	"os"
	"path/filepath"
	"runtime"
)

// ReadArtifact returns the wasm binary for the given crate.name
func ReadArtifact(crate string) ([]byte, error) {
	_, currentFile, _, _ := runtime.Caller(0)
	baseDir := filepath.Dir(currentFile)
	targetFile := filepath.Join(baseDir, "../node_modules/@satlayer", crate, "dist/contract.wasm")
	return os.ReadFile(targetFile)
}

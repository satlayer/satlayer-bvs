package wasm

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"
)

// ReadWasmFile returns the wasm binary for the given crate.name
func ReadWasmFile(name string) ([]byte, error) {
	_, currentFile, _, _ := runtime.Caller(0)
	baseDir := filepath.Dir(currentFile)

	artifactName := strings.ReplaceAll(name, "-", "_")
	targetFile := filepath.Join(baseDir, "../node_modules/@satlayer", name, "artifacts", artifactName+".wasm")
	return os.ReadFile(targetFile)
}

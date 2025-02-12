package main

import (
	"github.com/satlayer/satlayer-bvs/examples/squaring/uploader/core"
	"github.com/satlayer/satlayer-bvs/examples/squaring/uploader/uploader"
)

func main() {
	core.InitConfig()

	up := uploader.NewUploader()
	up.Run()
}

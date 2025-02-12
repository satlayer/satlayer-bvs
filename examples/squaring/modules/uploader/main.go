package main

import (
	"github.com/satlayer/hello-world-bvs/uploader/core"
	"github.com/satlayer/hello-world-bvs/uploader/uploader"
)

func main() {
	core.InitConfig()
	
	up := uploader.NewUploader()
	up.Run()
}

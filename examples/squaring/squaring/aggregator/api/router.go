package api

import "github.com/gin-gonic/gin"

// SetupRoutes sets up routes for the aggregator API.
//
// router is the Gin Engine instance used to set up the routes.
// No return values.
func SetupRoutes(router *gin.Engine) {
	router.POST("api/aggregator", Aggregator)
}

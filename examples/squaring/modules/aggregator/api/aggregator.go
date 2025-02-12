package api

import (
	"encoding/json"
	"fmt"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/satlayer/satlayer-bvs/bvs-api/signer"

	"github.com/satlayer/hello-world-bvs/aggregator/core"
	"github.com/satlayer/hello-world-bvs/aggregator/svc"
	"github.com/satlayer/hello-world-bvs/aggregator/util"
	"github.com/satlayer/hello-world-bvs/aggregator/util/resp"
)

type Payload struct {
	TaskID    uint64 `json:"task_id" binding:"required"`
	Result    int64  `json:"result" binding:"required"`
	Timestamp int64  `json:"timestamp" binding:"required"`
	Signature string `json:"signature" binding:"required"`
	PubKey    string `json:"pub_key" binding:"required"`
}

// Aggregator handles the aggregator endpoint for the API.
//
// It parses the payload from the request body and verifies the signature.
// It checks if the timestamp is within the allowed range.
// It verifies if the task is finished and if the operator has already sent the task.
// If all checks pass, it saves the task to the queue.
// It returns an HTTP response with the status of the operation.
func Aggregator(c *gin.Context) {
	// parse payload
	var payload Payload
	if err := c.ShouldBindJSON(&payload); err != nil {
		c.JSON(http.StatusBadRequest, resp.ErrParam)
		return
	}
	fmt.Printf("payload: %+v\n", payload)

	// get current timestamp
	nowTs := time.Now().Unix()
	if payload.Timestamp > nowTs || payload.Timestamp < nowTs-60*2 {
		c.JSON(http.StatusBadRequest, resp.ErrTimestamp)
		return
	}

	// verify signature
	pubKey, address, err := util.PubKeyToAddress(payload.PubKey)
	if err != nil {
		c.JSON(http.StatusBadRequest, resp.ErrPubKeyToAddr)
		return
	}

	msgPayload := fmt.Sprintf("%s-%d-%d-%d", core.C.Chain.BvsHash, payload.Timestamp, payload.TaskID, payload.Result)
	msgBytes := []byte(msgPayload)
	if isValid, err := signer.VerifySignature(pubKey, msgBytes, payload.Signature); err != nil || !isValid {
		c.JSON(http.StatusBadRequest, resp.ErrSignature)
		return
	}

	// verify task is finished
	pkTaskFinished := fmt.Sprintf("%s%d", core.PKTaskFinished, payload.TaskID)
	if isExist, err := core.S.RedisConn.Exists(c, pkTaskFinished).Result(); err != nil || isExist == 1 {
		c.JSON(http.StatusBadRequest, resp.ErrFinished)
		return
	}

	if ok, err := svc.MonitorInstance.VerifyOperator(address); err != nil || !ok {
		c.JSON(http.StatusBadRequest, resp.ErrOperator)
		return
	}

	// verify operator is already send
	taskOperatorKey := fmt.Sprintf("%s%d", core.PKTaskOperator, payload.TaskID)
	if result, err := core.S.RedisConn.Eval(c, core.LuaScript, []string{taskOperatorKey}, address).Result(); err != nil || result.(int64) == 1 {
		c.JSON(http.StatusBadRequest, resp.ErrSend)
		return
	}

	// save task to queue
	task := core.Task{TaskID: payload.TaskID, TaskResult: core.TaskResult{Operator: address, Result: payload.Result}}
	taskStr, err := json.Marshal(task)
	if err != nil {
		c.JSON(http.StatusInternalServerError, resp.ErrJson)
		return
	}
	if _, err := core.S.RedisConn.LPush(c, core.PKTaskQueue, taskStr).Result(); err != nil {
		c.JSON(http.StatusInternalServerError, resp.ErrRedis)
		return
	}

	c.JSON(http.StatusOK, resp.OK)
}

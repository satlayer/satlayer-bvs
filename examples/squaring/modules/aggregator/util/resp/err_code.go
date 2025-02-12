package resp

var (
	// OK result
	OK = NewError(0, "success")

	// ErrParam param errors
	ErrParam        = NewError(10001, "Param parse failed")
	ErrTimestamp    = NewError(10002, "Timestamp out of range")
	ErrPubKeyToAddr = NewError(10003, "PubKey parse to address failed")
	ErrSignature    = NewError(10004, "Invalid Signature")

	// ErrFinished check logic errors
	ErrFinished = NewError(20001, "Task already finished")
	ErrOperator = NewError(20002, "Invalid operator")
	ErrSend     = NewError(20003, "Task already send")

	// ErrJson internal errors
	ErrJson  = NewError(50001, "Json error")
	ErrRedis = NewError(50002, "Redis error")
)

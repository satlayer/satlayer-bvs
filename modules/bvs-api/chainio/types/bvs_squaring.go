package types

type CreateNewTaskReq struct {
	CreateNewTask CreateNewTask `json:"create_new_task"`
}

type CreateNewTask struct {
	Input int64 `json:"input"`
}

type RespondToTaskReq struct {
	RespondToTask RespondToTask `json:"respond_to_task"`
}

type RespondToTask struct {
	TaskID    int64  `json:"task_id"`
	Result    int64  `json:"result"`
	Operators string `json:"operators"`
}

type GetTaskInputReq struct {
	GetTaskInput GetTaskInput `json:"get_task_input"`
}

type GetTaskInput struct {
	TaskID int64 `json:"task_id"`
}

type GetTaskResultReq struct {
	GetTaskResult GetTaskResult `json:"get_task_input"`
}

type GetTaskResult struct {
	TaskID int64 `json:"task_id"`
}

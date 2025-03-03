package directory

import (
	"fmt"
)

func Status(operator, service string) {
	s := NewService()
	resp, err := s.Directory.QueryStatus(operator, service)
	if err != nil {
		fmt.Printf("Get operator error! %+v\n", err)
		return
	}
	fmt.Printf("Operator status! %+v\n", resp)
}

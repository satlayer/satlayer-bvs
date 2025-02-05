package main

import "testing"

func TestSquare(t *testing.T) {
	result := square(3)
	expected := 9
	if result != expected {
		t.Errorf("Add(3) = %d; want %d", result, expected)
	}
}

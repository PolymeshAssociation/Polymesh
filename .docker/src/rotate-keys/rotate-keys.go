package main

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/AdamSLevy/jsonrpc2/v14"
)

type RotateKeysResult struct {
	Keys  string `json:"keys"`
	Proof string `json:"proof"`
}

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "Usage: rotate-keys <owner-hex>")
		os.Exit(1)
	}
	owner := os.Args[1]

	var c jsonrpc2.Client
	params := []string{owner}
	var r RotateKeysResult
	err := c.Request(nil, "http://localhost:9944", "author_rotateKeysWithOwner", params, &r)
	if _, ok := err.(jsonrpc2.Error); ok {
		fmt.Fprintf(os.Stderr, "Error checking jsonrpc port. %v\n", err)
		os.Exit(1)
	}
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error checking jsonrpc port! %v\n", err)
		os.Exit(1)
	}
	out, err := json.Marshal(r)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error encoding result: %v\n", err)
		os.Exit(1)
	}
	fmt.Println(string(out))
}

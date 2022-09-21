package main

import (
    "fmt"
    "os"
    "github.com/AdamSLevy/jsonrpc2/v14"
)

func main() {
    var c jsonrpc2.Client
    params := []float64{}
    var r string
    err := c.Request(nil, "http://localhost:9933", "author_rotateKeys", params, &r)
    if _, ok := err.(jsonrpc2.Error); ok {
        fmt.Printf("Error checking jsonrpc port. %v\n", err)
        os.Exit(1)
    }
    if err != nil {
        fmt.Printf("Error checking jsonrpc port! %v\n", err)
        os.Exit(1)
    }
    fmt.Println(r)
}


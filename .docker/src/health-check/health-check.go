package main

import (
    "fmt"
    "os"
    "github.com/AdamSLevy/jsonrpc2/v14"
    "net/http"
    "io/ioutil"
)

type systemHealth struct {
    IsSyncing       bool    `json:"isSyncing"`
    Peers           float64 `json:"peers"`
    ShouldHavePeers bool    `json:"shouldHavePeers"`
}


func testJSONRPC(readinessCheck bool) {
    var c jsonrpc2.Client
    params := []float64{}
    var r systemHealth
    err := c.Request(nil, "http://localhost:9933", "system_health", params, &r)
    if _, ok := err.(jsonrpc2.Error); ok {
        fmt.Printf("Error checking jsonrpc port. %v\n", err)
        os.Exit(1)
    }
    if err != nil {
        fmt.Printf("Error checking jsonrpc port! %v\n", err)
        os.Exit(1)
    }
    //fmt.Printf("Node syncing: %v\n", r.IsSyncing )
    //fmt.Printf("Should have peers: %v\n", r.ShouldHavePeers )
    //fmt.Printf("Number of peers: %v\n", r.Peers )
    if r.IsSyncing && readinessCheck {
        fmt.Printf("Node is syncing" )
        os.Exit(1)
    }
}

func testPrometheus() {
    resp, err := http.Get("http://localhost:9615/metrics")
    if err != nil {
        fmt.Printf( "Error checking prometheus exporter! %v\n", err )
        os.Exit(1)
    }
    defer resp.Body.Close()
    _, err2 := ioutil.ReadAll(resp.Body)
    if err2 != nil {
        fmt.Printf( "Error reading prometheus metrics! %v\n", err2 )
        os.Exit(1)
    }
    if resp.StatusCode > 299 {
        fmt.Printf("Response status is %v", resp.StatusCode )
        os.Exit(1)
    }
}

func printHelp(progname string) {
    fmt.Printf(`Use:
    %v [command]
    Commands:
    * help: print this help
    * liveness: return 0 if localhost polymesh has both the RPC (9933) and prometheus (9615) ports open and responsive, 1 otherwise
    * readiness: return 0 if liveness check passes and the node is not syncing, 1 otherwise
    `, progname)
}

func main() {
    args := os.Args
    if len(args) != 2 {
        printHelp(args[0])
        os.Exit(1)
    }
    if args[1] == "liveness" {
        testJSONRPC(false)
    } else if args[1] == "readiness" {
        testJSONRPC(true)
    } else if args[1] == "help" {
        printHelp(args[0])
        os.Exit(0)
    } else {
        printHelp(args[0])
        os.Exit(1)
    }

    testPrometheus()
}


package main

import (
	"encoding/json"
	"fmt"
	"golang.org/x/net/http2"
	"golang.org/x/net/http2/h2c"
	"net/http"
	"os"
)

func main() {
	h2s := &http2.Server{}

	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.URL.Path {
		case "/json":
			w.Header().Set("Content-Type", "application/json")
			response := Response{Message: "Hello, World!"}
			json.NewEncoder(w).Encode(response)
		default:
			http.Error(w, "Not Found", http.StatusNotFound)
		}
	})

	server := &http.Server{
		Addr:    "0.0.0.0:8090",
		Handler: h2c.NewHandler(handler, h2s),
	}

	checkErr(http2.ConfigureServer(server, h2s), "during call to ConfigureServer()")

	fmt.Printf("Listening :8090...\n")
	checkErr(server.ListenAndServe(), "while listening")
}

type Response struct {
	Message string `json:"message"`
}

func checkErr(err error, msg string) {
	if err == nil {
		return
	}
	fmt.Printf("ERROR: %s: %s\n", msg, err)
	os.Exit(1)
}

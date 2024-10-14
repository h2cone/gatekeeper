package main

import (
	"encoding/json"
	"fmt"
	"net/http"
)

type Response struct {
	Message string `json:"message"`
}

func upstreamHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	response := Response{Message: "Hello, World!"}
	json.NewEncoder(w).Encode(response)
}

func main() {
	http.HandleFunc("/json", upstreamHandler)
	fmt.Println("Listening on :8090")
	if err := http.ListenAndServe(":8090", nil); err != nil {
		panic(err)
	}
}

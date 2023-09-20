package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"

	"github.com/google/uuid"
)

func main() {
	http.HandleFunc("/", jsonHandler)

	port := 3000
	fmt.Printf("Started go server on port %d\n", port)
	err := http.ListenAndServe(fmt.Sprintf(":%d", port), nil)
	if err != nil {
		panic(err)
	}

}

type Query struct {
	QueryParam1 string `json:"queryParam1"`
	QueryParam2 string `json:"queryParam2"`
	QueryParam3 string `json:"queryParam3"`
	QueryParam4 string `json:"queryParam4"`
}

func jsonHandler(w http.ResponseWriter, r *http.Request) {
	q1 := r.URL.Query().Get("q1")
	q2 := r.URL.Query().Get("q2")
	q3 := r.URL.Query().Get("q3")
	q4 := r.URL.Query().Get("q4")
	queryStruct := Query{
		QueryParam1: q1,
		QueryParam2: q2,
		QueryParam3: q3,
		QueryParam4: q4,
	}
	body, err := json.Marshal(queryStruct)
	if err != nil {
		panic(err)
	}
	id := uuid.New().String()
	filePath := "./json/" + id
	f, err := os.Create(filePath)
	if err != nil {
		panic(err)
	}
	defer f.Close()
	_, err = f.Write(body)
	if err != nil {
		panic(err)
	}
	f2, err := os.ReadFile(filePath)
	if err != nil {
		panic(err)
	}
	err = os.Remove(filePath)
	if err != nil {
		panic(err)
	}
	w.Header().Set("Content-Type", "applicaton/json")
	fmt.Fprint(w, string(f2))

}

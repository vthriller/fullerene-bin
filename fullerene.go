package main

import (
	"bytes"
	"github.com/wcharczuk/go-chart"
	"net/http"
)

func test(w http.ResponseWriter, req *http.Request) {
	graph := chart.Chart{
		Series: []chart.Series{
			chart.ContinuousSeries{
				XValues: []float64{1, 2, 3, 4},
				YValues: []float64{3.14159, 2.71828, -1, 0},
			},
		},
	}

	buffer := bytes.NewBuffer([]byte{})
	err := graph.Render(chart.PNG, buffer)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}

	w.Header().Add("content-type", "image/png")
	_, _ = w.Write(buffer.Bytes())
	// err: ¯\_(ツ)_/¯
}

func main() {
	http.HandleFunc("/test", test)
	http.ListenAndServe(":12345", nil)
}

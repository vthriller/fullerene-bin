package main

import (
	"bytes"
	"github.com/wcharczuk/go-chart"
	"net/http"
	"strconv"
)

func int_param(req *http.Request, key string, implicit int) (int, error) {
	value := req.URL.Query().Get(key)
	if len(value) == 0 {
		return implicit, nil
	}
	return strconv.Atoi(value)
}

func test(w http.ResponseWriter, req *http.Request) {
	width, err := int_param(req, "w", 800)
	if err != nil {
		http.Error(w, "width: " + err.Error(), 500)
		return
	}
	height, err := int_param(req, "h", 480)
	if err != nil {
		http.Error(w, "height: " + err.Error(), 500)
		return
	}

	graph := chart.Chart{
		Series: []chart.Series{
			chart.ContinuousSeries{
				XValues: []float64{1, 2, 3, 4},
				YValues: []float64{3.14159, 2.71828, -1, 0},
			},
		},
		Width: width,
		Height: height,
	}

	buffer := bytes.NewBuffer([]byte{})
	err = graph.Render(chart.PNG, buffer)
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

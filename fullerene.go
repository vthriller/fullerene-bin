package main

import (
	"bytes"
	"io/ioutil"
	"github.com/wcharczuk/go-chart"
)

func main() {
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
		panic(err)
	}
	ioutil.WriteFile("test.png", buffer.Bytes(), 0644)
	if err != nil {
		panic(err)
	}
}

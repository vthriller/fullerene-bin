package main

import (
	"bytes"
	"github.com/wcharczuk/go-chart"
	"net/http"
	"net/url"
	"strconv"
	"time"
	"encoding/json"
	"fmt"
	"io/ioutil"
)

func int_param(req *http.Request, key string, implicit int) (int, error) {
	value := req.URL.Query().Get(key)
	if len(value) == 0 {
		return implicit, nil
	}
	return strconv.Atoi(value)
}

type prom_response struct {
	Status string `json:"status"`
	Data prom_data `json:"data"`
}
type prom_data struct {
	ResType string `json:"resultType"`
	Result []prom_metric `json:"result"`
}
type prom_metric struct {
	Metric map[string]string `json:"metric"`
	Values []interface{} `json:"values"` // should rather be (int, string)
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

	resp, err := http.Get(fmt.Sprintf(
		"http://127.0.0.1:9090/api/v1/query_range?query=%s&start=%d&end=%d&step=5",
		url.QueryEscape("sum(rate(node_cpu{instance=\"localhost:9100\"} [5m])) by (mode)"),
		time.Now().Add(-time.Hour).Unix(),
		time.Now().Unix(),
	))
	if err != nil {
		http.Error(w, "remote: " + err.Error(), 502)
		return
	}
	defer resp.Body.Close()
	if resp.StatusCode != 200 {
		http.Error(w, fmt.Sprintf("remote: code %d", resp.StatusCode), 502)
		return
	}
	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		http.Error(w, "remote: " + err.Error(), 502)
		return
	}

	var resp_json prom_response
	err = json.Unmarshal(body, &resp_json)

	if err != nil {
		http.Error(w, "remote: data: " + err.Error(), 502)
		return
	}
	if resp_json.Status != "success" {
		http.Error(w, fmt.Sprintf("remote: data: status %q", resp_json.Status), 502)
		return
	}
	if resp_json.Data.ResType != "matrix" {
		http.Error(w, fmt.Sprintf("remote: data: unexpected resultType %q", resp_json.Data.ResType), 502)
		return
	}

	series := make([]chart.Series, 0)
	for _, metric := range resp_json.Data.Result {
		name, exists := metric.Metric["__name__"]
		if !exists {
			name = ""
		}
		name += "{"
		for k, v := range metric.Metric {
			if k == "__name__" { continue }
			name += fmt.Sprintf("%s=%q", k, v)
		}
		name += "}"
		xvals := make([]time.Time, 0)
		yvals := make([]float64, 0)
		for _, xy := range metric.Values {
			// xy is [12345., "123"] in json
			xy := xy.([]interface{})
			x := xy[0].(float64)
			y := xy[1].(string)
			yf, err := strconv.ParseFloat(y, 64)
			if err != nil {
				// XXX skip metric? return 502?
				continue
			}
			// don't care about sub-second precision, sorry
			xvals = append(xvals, time.Unix(int64(x), 0))
			yvals = append(yvals, yf)
		}
		series = append(series, chart.TimeSeries {
			Name: name,
			XValues: xvals,
			YValues: yvals,
		})
	}

	graph := chart.Chart{
		Series: series,
		Width: width,
		Height: height,
	}
	graph.Elements = []chart.Renderable{
		chart.Legend(&graph),
	}

	buffer := bytes.NewBuffer([]byte{})
	err = graph.Render(chart.PNG, buffer)
	if err != nil {
		http.Error(w, err.Error(), 500)
		return
	}

	hdr := w.Header()
	hdr.Add("content-type", "image/png")
	hdr.Add("cache-control", "no-cache, no-store, must-revalidate")
	hdr.Add("pragma", "no-cache")
	hdr.Add("expires", "0")

	_, _ = w.Write(buffer.Bytes())
	// err: ¯\_(ツ)_/¯
}

func main() {
	http.HandleFunc("/test", test)
	http.ListenAndServe(":12345", nil)
}

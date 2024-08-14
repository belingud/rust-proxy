package main

import (
    "io"
    "log"
    "net/http"
    "net/url"
)

func handleRequestAndRedirect(w http.ResponseWriter, r *http.Request) {
    targetURL := "http://example.com" // Replace with your target URL

    // Parse the URL
    parsedURL, _ := url.Parse(targetURL)

    // Update the request to point to the target server
    r.URL.Scheme = parsedURL.Scheme
    r.URL.Host = parsedURL.Host
    r.RequestURI = ""

    // Forward the request to the target server
    resp, err := http.DefaultTransport.RoundTrip(r)
    if err != nil {
        http.Error(w, "Server Error", http.StatusInternalServerError)
        return
    }
    defer resp.Body.Close()

    // Copy the response headers
    for key, value := range resp.Header {
        for _, v := range value {
            w.Header().Add(key, v)
        }
    }

    // Copy the response body
    w.WriteHeader(resp.StatusCode)
    io.Copy(w, resp.Body)
}

func main() {
    // Define the server
    http.HandleFunc("/", handleRequestAndRedirect)
    log.Println("Starting proxy server on :8080")
    log.Fatal(http.ListenAndServe(":8080", nil))
}

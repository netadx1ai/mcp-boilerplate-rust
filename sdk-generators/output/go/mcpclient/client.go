// MCP Client SDK for Go
// Auto-generated from mcp-boilerplate-rust v0.4.0
// Do not edit manually

package mcpclient

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type Config struct {
	BaseURL   string
	Port      int
	Transport string
	Timeout   time.Duration
}

type Response struct {
	Success bool                   `json:"success"`
	Data    map[string]interface{} `json:"data,omitempty"`
	Error   string                 `json:"error,omitempty"`
}

type Client struct {
	config     Config
	baseURL    string
	httpClient *http.Client
}

func NewClient(config Config) *Client {
	if config.Timeout == 0 {
		config.Timeout = 30 * time.Second
	}

	port := config.Port
	if port == 0 {
		port = getDefaultPort(config.Transport)
	}

	baseURL := config.BaseURL
	if baseURL == "" {
		baseURL = fmt.Sprintf("http://127.0.0.1:%d", port)
	}

	return &Client{
		config:  config,
		baseURL: baseURL,
		httpClient: &http.Client{
			Timeout: config.Timeout,
		},
	}
}

func getDefaultPort(transport string) int {
	switch transport {
	case "sse":
		return 8025
	case "websocket":
		return 9001
	case "http":
		return 8080
	case "http_stream":
		return 8026
	case "grpc":
		return 50051
	default:
		return 8080
	}
}

// Echo - Echo a message with timestamp validation
func (c *Client) Echo(args map[string]interface{}) (*Response, error) {
	return c.callTool("echo", args)
}

// Ping - Health check endpoint
func (c *Client) Ping(args map[string]interface{}) (*Response, error) {
	return c.callTool("ping", args)
}

// Info - Get server metadata
func (c *Client) Info(args map[string]interface{}) (*Response, error) {
	return c.callTool("info", args)
}

// Calculate - Perform arithmetic operations
func (c *Client) Calculate(args map[string]interface{}) (*Response, error) {
	return c.callTool("calculate", args)
}

// Evaluate - Evaluate mathematical expression
func (c *Client) Evaluate(args map[string]interface{}) (*Response, error) {
	return c.callTool("evaluate", args)
}

// ProcessWithProgress - Process data with progress notifications
func (c *Client) ProcessWithProgress(args map[string]interface{}) (*Response, error) {
	return c.callTool("process_with_progress", args)
}

// BatchProcess - Batch process items with logging
func (c *Client) BatchProcess(args map[string]interface{}) (*Response, error) {
	return c.callTool("batch_process", args)
}

// TransformData - Transform array data
func (c *Client) TransformData(args map[string]interface{}) (*Response, error) {
	return c.callTool("transform_data", args)
}

// SimulateUpload - Simulate file upload with progress
func (c *Client) SimulateUpload(args map[string]interface{}) (*Response, error) {
	return c.callTool("simulate_upload", args)
}

// HealthCheck - Comprehensive system health check
func (c *Client) HealthCheck(args map[string]interface{}) (*Response, error) {
	return c.callTool("health_check", args)
}

// LongTask - Execute long-running task with progress updates
func (c *Client) LongTask(args map[string]interface{}) (*Response, error) {
	return c.callTool("long_task", args)
}

func (c *Client) callTool(name string, args map[string]interface{}) (*Response, error) {
	payload := map[string]interface{}{
		"jsonrpc": "2.0",
		"id":      1,
		"method":  "tools/call",
		"params": map[string]interface{}{
			"name":      name,
			"arguments": args,
		},
	}

	jsonData, err := json.Marshal(payload)
	if err != nil {
		return nil, fmt.Errorf("marshal error: %w", err)
	}

	resp, err := c.httpClient.Post(
		fmt.Sprintf("%s/tools/call", c.baseURL),
		"application/json",
		bytes.NewBuffer(jsonData),
	)
	if err != nil {
		return nil, fmt.Errorf("request error: %w", err)
	}
	defer resp.Body.Close()

	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("decode error: %w", err)
	}

	if errData, ok := result["error"].(map[string]interface{}); ok {
		return &Response{
			Success: false,
			Error:   errData["message"].(string),
		}, nil
	}

	return &Response{
		Success: true,
		Data:    result["result"].(map[string]interface{}),
	}, nil
}

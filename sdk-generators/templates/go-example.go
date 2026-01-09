// Go SDK Example
// Demonstrates how to use the generated MCP client

package main

import (
	"fmt"
	"log"
	"time"

	"github.com/netadx/mcp-boilerplate-rust/sdk-generators/output/go/mcpclient"
)

func main() {
	fmt.Println("MCP Go Client Examples")
	fmt.Println("=====================================================")

	// Run all examples
	if err := runBasicExamples(); err != nil {
		log.Fatalf("Basic examples failed: %v", err)
	}

	if err := runAdvancedExamples(); err != nil {
		log.Fatalf("Advanced examples failed: %v", err)
	}

	if err := runTransportExamples(); err != nil {
		log.Fatalf("Transport examples failed: %v", err)
	}

	if err := runErrorHandling(); err != nil {
		log.Fatalf("Error handling examples failed: %v", err)
	}

	performanceTest()

	fmt.Println("\n=====================================================")
	fmt.Println("All examples completed successfully!")
}

func runBasicExamples() error {
	fmt.Println("\nBasic Examples")
	fmt.Println("-----------------------------------------------------")

	config := mcpclient.Config{
		Transport: "http",
		Port:      8080,
		Timeout:   30 * time.Second,
	}

	client := mcpclient.NewClient(config)

	// 1. Ping test
	fmt.Println("\n1. Testing ping...")
	pingResult, err := client.Ping(map[string]interface{}{})
	if err != nil {
		return fmt.Errorf("ping failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", pingResult.Success)
	fmt.Printf("   Data: %v\n", pingResult.Data)

	// 2. Echo test
	fmt.Println("\n2. Testing echo...")
	echoResult, err := client.Echo(map[string]interface{}{
		"message": "Hello from Go!",
	})
	if err != nil {
		return fmt.Errorf("echo failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", echoResult.Success)
	fmt.Printf("   Data: %v\n", echoResult.Data)

	// 3. Server info
	fmt.Println("\n3. Getting server info...")
	infoResult, err := client.Info(map[string]interface{}{})
	if err != nil {
		return fmt.Errorf("info failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", infoResult.Success)
	if infoResult.Data != nil {
		fmt.Printf("   Version: %v\n", infoResult.Data["version"])
		fmt.Printf("   Description: %v\n", infoResult.Data["description"])
	}

	// 4. Calculator - Addition
	fmt.Println("\n4. Testing calculator (addition)...")
	calcResult, err := client.Calculate(map[string]interface{}{
		"operation": "add",
		"a":         10.0,
		"b":         5.0,
	})
	if err != nil {
		return fmt.Errorf("calculate failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", calcResult.Success)
	fmt.Printf("   Result: %v\n", calcResult.Data)

	// 5. Multiple calculations
	fmt.Println("\n5. Multiple calculations...")
	operations := []struct {
		op string
		a  float64
		b  float64
	}{
		{"subtract", 20, 8},
		{"multiply", 6, 7},
		{"divide", 100, 4},
	}

	for _, op := range operations {
		result, err := client.Calculate(map[string]interface{}{
			"operation": op.op,
			"a":         op.a,
			"b":         op.b,
		})
		if err != nil {
			return fmt.Errorf("calculate %s failed: %w", op.op, err)
		}
		if result.Success && result.Data != nil {
			fmt.Printf("   %.0f %s %.0f = %v\n", op.a, op.op, op.b, result.Data["result"])
		}
	}

	// 6. Evaluate expression
	fmt.Println("\n6. Evaluating expression...")
	evalResult, err := client.Evaluate(map[string]interface{}{
		"expression": "2 * (3 + 4)",
	})
	if err != nil {
		return fmt.Errorf("evaluate failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", evalResult.Success)
	fmt.Printf("   Result: %v\n", evalResult.Data)

	return nil
}

func runAdvancedExamples() error {
	fmt.Println("\nAdvanced Examples")
	fmt.Println("-----------------------------------------------------")

	config := mcpclient.Config{
		Transport: "http",
		Port:      8080,
		Timeout:   60 * time.Second,
	}

	client := mcpclient.NewClient(config)

	// 1. Transform data
	fmt.Println("\n1. Transforming data...")
	transformResult, err := client.TransformData(map[string]interface{}{
		"data":      []string{"hello", "world", "mcp", "go"},
		"operation": "uppercase",
	})
	if err != nil {
		return fmt.Errorf("transform failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", transformResult.Success)
	if transformResult.Data != nil {
		fmt.Printf("   Transformed: %v items\n", transformResult.Data["transformed_count"])
	}

	// 2. Process with progress
	fmt.Println("\n2. Processing with progress...")
	progressResult, err := client.ProcessWithProgress(map[string]interface{}{
		"data":     []string{"item1", "item2", "item3", "item4", "item5"},
		"delay_ms": 100,
	})
	if err != nil {
		return fmt.Errorf("process with progress failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", progressResult.Success)
	if progressResult.Data != nil {
		fmt.Printf("   Processed: %v items\n", progressResult.Data["processed_count"])
	}

	// 3. Simulate upload
	fmt.Println("\n3. Simulating upload...")
	uploadResult, err := client.SimulateUpload(map[string]interface{}{
		"filename":   "test-document.pdf",
		"size_bytes": 1024000,
	})
	if err != nil {
		return fmt.Errorf("simulate upload failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", uploadResult.Success)
	if uploadResult.Data != nil {
		fmt.Printf("   Chunks: %v\n", uploadResult.Data["chunks"])
		fmt.Printf("   Duration: %v ms\n", uploadResult.Data["duration_ms"])
	}

	// 4. Health check
	fmt.Println("\n4. Performing health check...")
	healthResult, err := client.HealthCheck(map[string]interface{}{})
	if err != nil {
		return fmt.Errorf("health check failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", healthResult.Success)
	if healthResult.Data != nil {
		fmt.Printf("   Status: %v\n", healthResult.Data["status"])
		fmt.Printf("   Version: %v\n", healthResult.Data["version"])
	}

	// 5. Batch processing
	fmt.Println("\n5. Batch processing...")
	batchResult, err := client.BatchProcess(map[string]interface{}{
		"items":     []string{"task1", "task2", "task3", "task4"},
		"operation": "process",
	})
	if err != nil {
		return fmt.Errorf("batch process failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", batchResult.Success)
	if batchResult.Data != nil {
		fmt.Printf("   Processed: %v items\n", batchResult.Data["processed_count"])
	}

	// 6. Long task
	fmt.Println("\n6. Long running task...")
	longTaskResult, err := client.LongTask(map[string]interface{}{
		"duration_seconds": 3,
	})
	if err != nil {
		return fmt.Errorf("long task failed: %w", err)
	}
	fmt.Printf("   Success: %v\n", longTaskResult.Success)
	if longTaskResult.Data != nil {
		fmt.Printf("   Completed: %v\n", longTaskResult.Data["completed"])
	}

	return nil
}

func runTransportExamples() error {
	fmt.Println("\nTransport Examples")
	fmt.Println("-----------------------------------------------------")

	// SSE Transport
	fmt.Println("\n1. SSE Transport...")
	sseConfig := mcpclient.Config{
		Transport: "sse",
		Port:      8025,
		Timeout:   30 * time.Second,
	}
	sseClient := mcpclient.NewClient(sseConfig)
	sseResult, err := sseClient.Ping(map[string]interface{}{})
	if err != nil {
		fmt.Printf("   SSE ping failed (server may not be running): %v\n", err)
	} else {
		fmt.Printf("   SSE Ping - Success: %v\n", sseResult.Success)
	}

	// WebSocket Transport
	fmt.Println("\n2. WebSocket Transport...")
	wsConfig := mcpclient.Config{
		Transport: "websocket",
		Port:      9001,
		Timeout:   30 * time.Second,
	}
	wsClient := mcpclient.NewClient(wsConfig)
	wsResult, err := wsClient.Echo(map[string]interface{}{
		"message": "WebSocket test",
	})
	if err != nil {
		fmt.Printf("   WebSocket echo failed (server may not be running): %v\n", err)
	} else {
		fmt.Printf("   WebSocket Echo - Success: %v\n", wsResult.Success)
	}

	// HTTP Stream Transport
	fmt.Println("\n3. HTTP Stream Transport...")
	streamConfig := mcpclient.Config{
		Transport: "http_stream",
		Port:      8026,
		Timeout:   30 * time.Second,
	}
	streamClient := mcpclient.NewClient(streamConfig)
	streamResult, err := streamClient.Ping(map[string]interface{}{})
	if err != nil {
		fmt.Printf("   HTTP Stream ping failed (server may not be running): %v\n", err)
	} else {
		fmt.Printf("   HTTP Stream Ping - Success: %v\n", streamResult.Success)
	}

	// gRPC Transport
	fmt.Println("\n4. gRPC Transport...")
	grpcConfig := mcpclient.Config{
		Transport: "grpc",
		Port:      50051,
		Timeout:   30 * time.Second,
	}
	grpcClient := mcpclient.NewClient(grpcConfig)
	grpcResult, err := grpcClient.Ping(map[string]interface{}{})
	if err != nil {
		fmt.Printf("   gRPC ping failed (server may not be running): %v\n", err)
	} else {
		fmt.Printf("   gRPC Ping - Success: %v\n", grpcResult.Success)
	}

	return nil
}

func runErrorHandling() error {
	fmt.Println("\nError Handling Examples")
	fmt.Println("-----------------------------------------------------")

	config := mcpclient.Config{
		Transport: "http",
		Port:      8080,
		Timeout:   30 * time.Second,
	}

	client := mcpclient.NewClient(config)

	// 1. Invalid operation
	fmt.Println("\n1. Invalid operation...")
	result1, err := client.Calculate(map[string]interface{}{
		"operation": "invalid",
		"a":         5.0,
		"b":         3.0,
	})
	if err != nil {
		fmt.Printf("   Expected error: %v\n", err)
	} else if !result1.Success {
		fmt.Printf("   Expected error from server: %s\n", result1.Error)
	}

	// 2. Empty message
	fmt.Println("\n2. Empty message...")
	result2, err := client.Echo(map[string]interface{}{
		"message": "",
	})
	if err != nil {
		fmt.Printf("   Expected error: %v\n", err)
	} else if !result2.Success {
		fmt.Printf("   Expected error from server: %s\n", result2.Error)
	}

	// 3. Division by zero
	fmt.Println("\n3. Division by zero...")
	result3, err := client.Calculate(map[string]interface{}{
		"operation": "divide",
		"a":         10.0,
		"b":         0.0,
	})
	if err != nil {
		fmt.Printf("   Expected error: %v\n", err)
	} else if !result3.Success {
		fmt.Printf("   Expected error from server: %s\n", result3.Error)
	}

	// 4. Too large array
	fmt.Println("\n4. Too large array (will be trimmed)...")
	largeArray := make([]string, 15000)
	for i := 0; i < 15000; i++ {
		largeArray[i] = fmt.Sprintf("item_%d", i)
	}
	result4, err := client.TransformData(map[string]interface{}{
		"data":      largeArray,
		"operation": "uppercase",
	})
	if err != nil {
		fmt.Printf("   Expected error: %v\n", err)
	} else if !result4.Success {
		fmt.Printf("   Expected error from server: %s\n", result4.Error)
	}

	return nil
}

func performanceTest() {
	fmt.Println("\nPerformance Test")
	fmt.Println("-----------------------------------------------------")

	config := mcpclient.Config{
		Transport: "http",
		Port:      8080,
		Timeout:   30 * time.Second,
	}

	client := mcpclient.NewClient(config)

	iterations := 10
	start := time.Now()

	successCount := 0
	failCount := 0

	for i := 0; i < iterations; i++ {
		result, err := client.Ping(map[string]interface{}{})
		if err != nil || !result.Success {
			failCount++
		} else {
			successCount++
		}
	}

	elapsed := time.Since(start)
	avgTime := elapsed.Milliseconds() / int64(iterations)

	fmt.Printf("\nCompleted %d requests in %v\n", iterations, elapsed)
	fmt.Printf("Average time per request: %d ms\n", avgTime)
	fmt.Printf("Success: %d, Failed: %d\n", successCount, failCount)
}
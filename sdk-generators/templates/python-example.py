#!/usr/bin/env python3
"""
Python SDK Example
Demonstrates how to use the generated MCP client
"""

from mcp_client import McpClient, McpClientConfig, McpResponse


def main():
    """Main example demonstrating all tools"""
    
    # Configure client for HTTP transport
    config = McpClientConfig(
        transport='http',
        port=8080,
        timeout=30
    )
    
    client = McpClient(config)
    
    print("MCP Python Client Examples")
    print("=" * 50)
    
    # Basic health check
    print("\n1. Testing ping...")
    result = client.ping()
    print(f"   Success: {result.success}")
    if result.data:
        print(f"   Data: {result.data}")
    
    # Echo message
    print("\n2. Testing echo...")
    result = client.echo(message="Hello from Python!")
    print(f"   Success: {result.success}")
    if result.data:
        print(f"   Data: {result.data}")
    
    # Get server info
    print("\n3. Getting server info...")
    result = client.info()
    print(f"   Success: {result.success}")
    if result.data:
        print(f"   Data: {result.data}")
    
    # Perform calculation
    print("\n4. Testing calculator...")
    result = client.calculate(operation='add', a=10, b=5)
    print(f"   Success: {result.success}")
    if result.data:
        print(f"   Data: {result.data}")
    
    # Multiple operations
    print("\n5. Multiple calculations...")
    operations = [
        {'operation': 'subtract', 'a': 20, 'b': 8},
        {'operation': 'multiply', 'a': 6, 'b': 7},
        {'operation': 'divide', 'a': 100, 'b': 4}
    ]
    
    for op in operations:
        result = client.calculate(**op)
        if result.success:
            print(f"   {op['a']} {op['operation']} {op['b']} = {result.data}")
    
    # Evaluate expression
    print("\n6. Evaluating expression...")
    result = client.evaluate(expression='2 * (3 + 4)')
    print(f"   Success: {result.success}")
    if result.data:
        print(f"   Data: {result.data}")
    
    # Transform data
    print("\n7. Transforming data...")
    result = client.transform_data(
        data=['hello', 'world', 'mcp', 'python'],
        operation='uppercase'
    )
    print(f"   Success: {result.success}")
    if result.data:
        print(f"   Data: {result.data}")
    
    # Process with progress
    print("\n8. Processing with progress...")
    result = client.process_with_progress(
        data=['item1', 'item2', 'item3', 'item4', 'item5'],
        delay_ms=100
    )
    print(f"   Success: {result.success}")
    if result.data:
        print(f"   Processed: {result.data.get('processed_count')} items")
    
    # Simulate upload
    print("\n9. Simulating upload...")
    result = client.simulate_upload(
        filename='test-file.pdf',
        size_bytes=1024000
    )
    print(f"   Success: {result.success}")
    if result.data:
        print(f"   Upload completed: {result.data}")
    
    # Health check
    print("\n10. Performing health check...")
    result = client.health_check()
    print(f"    Success: {result.success}")
    if result.data:
        print(f"    Status: {result.data.get('status')}")
        print(f"    Version: {result.data.get('version')}")
    
    # Batch processing
    print("\n11. Batch processing...")
    result = client.batch_process(
        items=['task1', 'task2', 'task3', 'task4'],
        operation='process'
    )
    print(f"    Success: {result.success}")
    if result.data:
        print(f"    Processed: {result.data.get('processed_count')} items")
    
    # Long task with progress
    print("\n12. Long running task...")
    result = client.long_task(duration_seconds=5)
    print(f"    Success: {result.success}")
    if result.data:
        print(f"    Completed: {result.data}")
    
    print("\n" + "=" * 50)
    print("All examples completed!")


def sse_example():
    """Example using SSE transport"""
    print("\nSSE Transport Example")
    print("-" * 50)
    
    config = McpClientConfig(
        transport='sse',
        port=8025
    )
    
    client = McpClient(config)
    result = client.ping()
    
    print(f"SSE Ping - Success: {result.success}")
    if result.data:
        print(f"Data: {result.data}")


def websocket_example():
    """Example using WebSocket transport"""
    print("\nWebSocket Transport Example")
    print("-" * 50)
    
    config = McpClientConfig(
        transport='websocket',
        port=9001
    )
    
    client = McpClient(config)
    result = client.echo(message='WebSocket test message')
    
    print(f"WebSocket Echo - Success: {result.success}")
    if result.data:
        print(f"Data: {result.data}")


def http_stream_example():
    """Example using HTTP Stream transport"""
    print("\nHTTP Stream Transport Example")
    print("-" * 50)
    
    config = McpClientConfig(
        transport='http_stream',
        port=8026
    )
    
    client = McpClient(config)
    
    # Test with large data
    large_data = [f'item_{i}' for i in range(100)]
    result = client.transform_data(
        data=large_data,
        operation='uppercase'
    )
    
    print(f"HTTP Stream - Success: {result.success}")
    if result.data:
        print(f"Processed {result.data.get('transformed_count')} items")


def error_handling_example():
    """Example demonstrating error handling"""
    print("\nError Handling Example")
    print("-" * 50)
    
    config = McpClientConfig(transport='http', port=8080)
    client = McpClient(config)
    
    # Test with invalid parameters
    print("\n1. Invalid operation:")
    result = client.calculate(operation='invalid', a=5, b=3)
    if not result.success:
        print(f"   Expected error: {result.error}")
    
    # Test with missing required field
    print("\n2. Empty message:")
    result = client.echo(message='')
    if not result.success:
        print(f"   Expected error: {result.error}")
    
    # Test with out of range value
    print("\n3. Invalid division:")
    result = client.calculate(operation='divide', a=10, b=0)
    if not result.success:
        print(f"   Expected error: {result.error}")


def performance_test():
    """Simple performance test"""
    import time
    
    print("\nPerformance Test")
    print("-" * 50)
    
    config = McpClientConfig(transport='http', port=8080)
    client = McpClient(config)
    
    iterations = 10
    start_time = time.time()
    
    for i in range(iterations):
        result = client.ping()
        if not result.success:
            print(f"Request {i+1} failed: {result.error}")
    
    elapsed = time.time() - start_time
    avg_time = (elapsed / iterations) * 1000
    
    print(f"Completed {iterations} requests in {elapsed:.2f}s")
    print(f"Average time per request: {avg_time:.2f}ms")


if __name__ == '__main__':
    try:
        main()
        print("\n")
        sse_example()
        print("\n")
        websocket_example()
        print("\n")
        http_stream_example()
        print("\n")
        error_handling_example()
        print("\n")
        performance_test()
        
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
    except Exception as e:
        print(f"\nError: {e}")
        import traceback
        traceback.print_exc()
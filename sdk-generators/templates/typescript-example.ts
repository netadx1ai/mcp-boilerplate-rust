// TypeScript SDK Example
// Demonstrates how to use the generated MCP client

import { McpClient, McpClientConfig } from './mcp-client';

async function main() {
  // Configure client for HTTP transport
  const config: McpClientConfig = {
    transport: 'http',
    port: 8080,
    timeout: 30000
  };

  const client = new McpClient(config);

  try {
    // Basic health check
    console.log('Testing ping...');
    const pingResult = await client.ping();
    console.log('Ping result:', pingResult);

    // Echo message
    console.log('\nTesting echo...');
    const echoResult = await client.echo({
      message: 'Hello from TypeScript!'
    });
    console.log('Echo result:', echoResult);

    // Get server info
    console.log('\nGetting server info...');
    const infoResult = await client.info();
    console.log('Server info:', infoResult);

    // Perform calculation
    console.log('\nTesting calculator...');
    const calcResult = await client.calculate({
      operation: 'add',
      a: 10,
      b: 5
    });
    console.log('Calculation result:', calcResult);

    // Evaluate expression
    console.log('\nEvaluating expression...');
    const evalResult = await client.evaluate({
      expression: '2 * (3 + 4)'
    });
    console.log('Expression result:', evalResult);

    // Transform data
    console.log('\nTransforming data...');
    const transformResult = await client.transform_data({
      data: ['hello', 'world', 'mcp'],
      operation: 'uppercase'
    });
    console.log('Transform result:', transformResult);

    // Process with progress
    console.log('\nProcessing with progress...');
    const progressResult = await client.process_with_progress({
      data: ['item1', 'item2', 'item3', 'item4', 'item5'],
      delay_ms: 100
    });
    console.log('Progress result:', progressResult);

    // Simulate upload
    console.log('\nSimulating upload...');
    const uploadResult = await client.simulate_upload({
      filename: 'test.txt',
      size_bytes: 1024000
    });
    console.log('Upload result:', uploadResult);

    // Health check
    console.log('\nPerforming health check...');
    const healthResult = await client.health_check();
    console.log('Health result:', healthResult);

    // Batch processing
    console.log('\nBatch processing...');
    const batchResult = await client.batch_process({
      items: ['task1', 'task2', 'task3'],
      operation: 'process'
    });
    console.log('Batch result:', batchResult);

  } catch (error) {
    console.error('Error:', error);
  }
}

// Example with SSE transport
async function sseExample() {
  const config: McpClientConfig = {
    transport: 'sse',
    port: 8025
  };

  const client = new McpClient(config);

  const result = await client.ping();
  console.log('SSE ping result:', result);
}

// Example with WebSocket transport
async function websocketExample() {
  const config: McpClientConfig = {
    transport: 'websocket',
    port: 9001
  };

  const client = new McpClient(config);

  const result = await client.echo({
    message: 'WebSocket test'
  });
  console.log('WebSocket result:', result);
}

// Run examples
if (require.main === module) {
  main().catch(console.error);
}

export { main, sseExample, websocketExample };
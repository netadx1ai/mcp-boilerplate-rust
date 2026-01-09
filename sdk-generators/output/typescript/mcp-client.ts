// MCP Client SDK for TypeScript
// Auto-generated from mcp-boilerplate-rust v0.4.0
// Do not edit manually

export interface McpClientConfig {
	baseUrl?: string;
	port?: number;
	transport: 'sse' | 'websocket' | 'http' | 'http-stream' | 'grpc';
	timeout?: number;
}

export interface McpResponse<T = any> {
	success: boolean;
	data?: T;
	error?: string;
}

export interface EchoRequest {
	/** Message to echo (1-10240 bytes) */
	message: string;
}

export interface PingRequest {
}

export interface InfoRequest {
}

export interface CalculateRequest {
	/** First operand */
	a: number;
	/** Second operand */
	b: number;
	/** Arithmetic operation to perform */
	operation: string;
}

export interface EvaluateRequest {
	/** Mathematical expression to evaluate (e.g., '2 * (3 + 4)') */
	expression: string;
}

export interface ProcessWithProgressRequest {
	/** Array of data items to process */
	data: string[];
	/** Delay between items in milliseconds */
	delay_ms?: number;
}

export interface BatchProcessRequest {
	/** Items to process */
	items: string[];
	/** Operation to perform on each item */
	operation: string;
}

export interface TransformDataRequest {
	/** Array of strings to transform (max 10000 items) */
	data: string[];
	/** Transformation operation */
	operation: string;
}

export interface SimulateUploadRequest {
	/** Filename to simulate uploading */
	filename: string;
	/** File size in bytes */
	size_bytes: number;
}

export interface HealthCheckRequest {
}

export interface LongTaskRequest {
	/** Duration in seconds (1-60) */
	duration_seconds?: number;
}

export class McpClient {
	private config: McpClientConfig;
	private baseUrl: string;

	constructor(config: McpClientConfig) {
		this.config = config;
		const port = config.port || this.getDefaultPort(config.transport);
		this.baseUrl = config.baseUrl || `http://127.0.0.1:${port}`;
	}

	private getDefaultPort(transport: string): number {
		if (transport === 'sse') return 8025;
		if (transport === 'websocket') return 9001;
		if (transport === 'http') return 8080;
		if (transport === 'http_stream') return 8026;
		if (transport === 'grpc') return 50051;
		return 8080;
	}

	/**
	 * Echo a message with timestamp validation
	 */
	async echo(params: EchoRequest): Promise<McpResponse> {
		const args = params;
		return this.callTool('echo', args);
	}

	/**
	 * Health check endpoint
	 */
	async ping(): Promise<McpResponse> {
		const args = {};
		return this.callTool('ping', args);
	}

	/**
	 * Get server metadata
	 */
	async info(): Promise<McpResponse> {
		const args = {};
		return this.callTool('info', args);
	}

	/**
	 * Perform arithmetic operations
	 */
	async calculate(params: CalculateRequest): Promise<McpResponse> {
		const args = params;
		return this.callTool('calculate', args);
	}

	/**
	 * Evaluate mathematical expression
	 */
	async evaluate(params: EvaluateRequest): Promise<McpResponse> {
		const args = params;
		return this.callTool('evaluate', args);
	}

	/**
	 * Process data with progress notifications
	 */
	async process_with_progress(params: ProcessWithProgressRequest): Promise<McpResponse> {
		const args = params;
		return this.callTool('process_with_progress', args);
	}

	/**
	 * Batch process items with logging
	 */
	async batch_process(params: BatchProcessRequest): Promise<McpResponse> {
		const args = params;
		return this.callTool('batch_process', args);
	}

	/**
	 * Transform array data
	 */
	async transform_data(params: TransformDataRequest): Promise<McpResponse> {
		const args = params;
		return this.callTool('transform_data', args);
	}

	/**
	 * Simulate file upload with progress
	 */
	async simulate_upload(params: SimulateUploadRequest): Promise<McpResponse> {
		const args = params;
		return this.callTool('simulate_upload', args);
	}

	/**
	 * Comprehensive system health check
	 */
	async health_check(): Promise<McpResponse> {
		const args = {};
		return this.callTool('health_check', args);
	}

	/**
	 * Execute long-running task with progress updates
	 */
	async long_task(params: LongTaskRequest): Promise<McpResponse> {
		const args = params;
		return this.callTool('long_task', args);
	}

	private async callTool(name: string, args: any): Promise<McpResponse> {
		const response = await fetch(`${this.baseUrl}/tools/call`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				jsonrpc: '2.0',
				id: Date.now(),
				method: 'tools/call',
				params: { name, arguments: args }
			})
		});

		if (!response.ok) {
			return { success: false, error: `HTTP ${response.status}` };
		}

		const data = await response.json();
		if (data.error) {
			return { success: false, error: data.error.message };
		}

		return { success: true, data: data.result };
	}
}

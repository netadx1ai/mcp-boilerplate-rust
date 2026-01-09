"""MCP Client SDK for Python
Auto-generated from mcp-boilerplate-rust v0.4.0
Do not edit manually
"""

from typing import Optional, Dict, Any, List
from dataclasses import dataclass
import requests
import json

@dataclass
class McpClientConfig:
	base_url: Optional[str] = None
	port: Optional[int] = None
	transport: str = 'http'
	timeout: int = 30


@dataclass
class McpResponse:
	success: bool
	data: Optional[Dict[str, Any]] = None
	error: Optional[str] = None


class McpClient:
	"""MCP Client for Python"""

	DEFAULT_PORTS = {
		'sse': 8025,
		'websocket': 9001,
		'http': 8080,
		'http_stream': 8026,
		'grpc': 50051,
	}

	def __init__(self, config: McpClientConfig):
		self.config = config
		port = config.port or self.DEFAULT_PORTS.get(config.transport, 8080)
		self.base_url = config.base_url or f'http://127.0.0.1:{port}'
		self.session = requests.Session()

	def echo(self, **kwargs) -> McpResponse:
		"""Echo a message with timestamp validation"""
		return self._call_tool('echo', kwargs)

	def ping(self) -> McpResponse:
		"""Health check endpoint"""
		return self._call_tool('ping', {})

	def info(self) -> McpResponse:
		"""Get server metadata"""
		return self._call_tool('info', {})

	def calculate(self, **kwargs) -> McpResponse:
		"""Perform arithmetic operations"""
		return self._call_tool('calculate', kwargs)

	def evaluate(self, **kwargs) -> McpResponse:
		"""Evaluate mathematical expression"""
		return self._call_tool('evaluate', kwargs)

	def process_with_progress(self, **kwargs) -> McpResponse:
		"""Process data with progress notifications"""
		return self._call_tool('process_with_progress', kwargs)

	def batch_process(self, **kwargs) -> McpResponse:
		"""Batch process items with logging"""
		return self._call_tool('batch_process', kwargs)

	def transform_data(self, **kwargs) -> McpResponse:
		"""Transform array data"""
		return self._call_tool('transform_data', kwargs)

	def simulate_upload(self, **kwargs) -> McpResponse:
		"""Simulate file upload with progress"""
		return self._call_tool('simulate_upload', kwargs)

	def health_check(self) -> McpResponse:
		"""Comprehensive system health check"""
		return self._call_tool('health_check', {})

	def long_task(self, **kwargs) -> McpResponse:
		"""Execute long-running task with progress updates"""
		return self._call_tool('long_task', kwargs)

	def _call_tool(self, name: str, args: Dict[str, Any]) -> McpResponse:
		"""Internal method to call a tool"""
		payload = {
			'jsonrpc': '2.0',
			'id': 1,
			'method': 'tools/call',
			'params': {'name': name, 'arguments': args}
		}

		try:
			response = self.session.post(
				f'{self.base_url}/tools/call',
				json=payload,
				timeout=self.config.timeout
			)
			response.raise_for_status()
			data = response.json()

			if 'error' in data:
				return McpResponse(success=False, error=data['error'].get('message'))

			return McpResponse(success=True, data=data.get('result'))

		except requests.RequestException as e:
			return McpResponse(success=False, error=str(e))

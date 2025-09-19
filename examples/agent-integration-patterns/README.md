# Agent Integration Patterns with MCP Servers

This directory contains comprehensive examples and patterns for integrating AI agents with our MCP server ecosystem. These patterns demonstrate how popular agent frameworks can leverage our production-ready MCP servers to build sophisticated AI applications.

## 🎯 Overview

Agent integration with MCP servers enables:
- **Tool-Rich Environments**: Agents can access diverse capabilities through MCP tools
- **Scalable Architecture**: Distribute functionality across specialized servers
- **Production Reliability**: Leverage battle-tested server infrastructure
- **Flexible Deployment**: Mix and match servers based on agent needs
- **Performance Optimization**: Efficient resource utilization and caching

## 🤖 Supported Agent Frameworks

### 1. LangChain Integration (`langchain/`)
**Use Case**: Build conversational agents with advanced reasoning capabilities
```python
from langchain.agents import create_mcp_agent
from langchain.tools import MCPToolkit

# Create MCP toolkit with our servers
mcp_toolkit = MCPToolkit([
    "http://localhost:3001",  # news-data-server
    "http://localhost:3002",  # template-server
    "http://localhost:3003",  # database-server
    "http://localhost:3004",  # analytics-server
])

agent = create_mcp_agent(
    llm=llm,
    toolkit=mcp_toolkit,
    verbose=True
)
```

### 2. AutoGen Integration (`autogen/`)
**Use Case**: Multi-agent conversations with specialized server access
```python
import autogen
from autogen.agentchat.contrib import MCPAgent

# Create specialized agents with MCP server access
news_agent = MCPAgent(
    name="NewsAnalyst",
    mcp_servers=["http://localhost:3001"],
    system_message="I analyze news trends and provide insights."
)

content_agent = MCPAgent(
    name="ContentCreator", 
    mcp_servers=["http://localhost:3002", "http://localhost:3003"],
    system_message="I create and store content using templates."
)
```

### 3. CrewAI Integration (`crewai/`)
**Use Case**: Collaborative agent crews with role-based server access
```python
from crewai import Agent, Task, Crew
from crewai.tools import MCPTool

# Define agents with specific MCP server capabilities
researcher = Agent(
    role='Research Analyst',
    goal='Find and analyze relevant news data',
    tools=[MCPTool(server_url="http://localhost:3001")]
)

writer = Agent(
    role='Content Writer', 
    goal='Create engaging articles from research',
    tools=[MCPTool(server_url="http://localhost:3002")]
)
```

### 4. OpenAI Assistants Integration (`openai-assistants/`)
**Use Case**: GPT-based assistants with custom MCP tool functions
```python
import openai
from openai_mcp_bridge import MCPFunctionBridge

# Bridge MCP servers to OpenAI function calling
mcp_bridge = MCPFunctionBridge([
    "http://localhost:3001",
    "http://localhost:3002", 
    "http://localhost:3003",
    "http://localhost:3004"
])

assistant = openai.beta.assistants.create(
    name="Content Generation Assistant",
    instructions="You help create content using news data and templates.",
    tools=mcp_bridge.get_function_definitions(),
    model="gpt-4-turbo"
)
```

### 5. Custom Agent Framework (`custom/`)
**Use Case**: Build your own agent framework with direct MCP integration
```rust
// Rust-based agent with native MCP integration
use mcp_client::{MCPClient, MCPToolCall};

pub struct CustomAgent {
    mcp_clients: HashMap<String, MCPClient>,
    llm_client: LLMClient,
}

impl CustomAgent {
    pub async fn execute_task(&self, task: &str) -> Result<String> {
        let plan = self.llm_client.create_plan(task).await?;
        
        for step in plan.steps {
            match step.tool_call {
                Some(call) => {
                    let result = self.execute_mcp_tool(call).await?;
                    step.result = Some(result);
                }
                None => continue,
            }
        }
        
        Ok(plan.synthesize_result())
    }
}
```

## 🏗️ Integration Patterns

### 1. Direct Server Integration
Connect agents directly to individual MCP servers for specialized capabilities:

```python
class DirectIntegrationAgent:
    def __init__(self):
        self.news_client = MCPClient("http://localhost:3001")
        self.template_client = MCPClient("http://localhost:3002")
        
    async def create_news_summary(self, query: str):
        # Step 1: Get news data
        news_data = await self.news_client.call_tool(
            "search_news", 
            {"query": query, "limit": 5}
        )
        
        # Step 2: Generate summary using template
        summary = await self.template_client.call_tool(
            "render_template",
            {
                "template_id": "news_summary",
                "parameters": {"news_items": news_data}
            }
        )
        
        return summary
```

### 2. Orchestrated Multi-Server Integration
Use orchestration patterns for complex workflows:

```python
class OrchestrationAgent:
    def __init__(self):
        self.coordinator = MCPCoordinator([
            "http://localhost:3001",  # news
            "http://localhost:3002",  # template  
            "http://localhost:3003",  # database
            "http://localhost:3004"   # analytics
        ])
        
    async def execute_content_pipeline(self, topic: str):
        workflow = MCPWorkflow([
            MCPStep("news", "search_news", {"query": topic}),
            MCPStep("template", "render_template", {
                "template_id": "article",
                "parameters": {"news_data": "{{previous.result}}"}
            }),
            MCPStep("database", "execute_query", {
                "query": "INSERT INTO articles...",
                "parameters": ["{{previous.result}}"]
            }),
            MCPStep("analytics", "track_content_creation", {
                "content_id": "{{previous.result.id}}"
            })
        ])
        
        return await self.coordinator.execute_workflow(workflow)
```

### 3. Tool Registry Pattern
Create a unified tool registry for agent frameworks:

```python
class MCPToolRegistry:
    def __init__(self):
        self.servers = {}
        self.tool_catalog = {}
        
    def register_server(self, name: str, url: str):
        client = MCPClient(url)
        self.servers[name] = client
        
        # Discover available tools
        tools = client.list_tools()
        for tool in tools:
            self.tool_catalog[f"{name}.{tool.name}"] = {
                "server": name,
                "tool": tool.name,
                "description": tool.description,
                "parameters": tool.parameters
            }
    
    def get_available_tools(self) -> List[Dict]:
        return list(self.tool_catalog.values())
    
    async def execute_tool(self, tool_name: str, parameters: Dict):
        if tool_name not in self.tool_catalog:
            raise ValueError(f"Tool {tool_name} not found")
            
        tool_info = self.tool_catalog[tool_name]
        server = self.servers[tool_info["server"]]
        
        return await server.call_tool(tool_info["tool"], parameters)

# Usage with any agent framework
registry = MCPToolRegistry()
registry.register_server("news", "http://localhost:3001")
registry.register_server("template", "http://localhost:3002")

# Agent can now discover and use all available tools
available_tools = registry.get_available_tools()
```

### 4. Streaming Integration Pattern
Handle real-time data streams from MCP servers:

```python
class StreamingAgent:
    def __init__(self):
        self.stream_clients = {}
        
    async def subscribe_to_news_stream(self, callback):
        async with MCPStreamClient("ws://localhost:3001/stream") as client:
            await client.subscribe("news_updates")
            
            async for update in client:
                processed_update = await self.process_news_update(update)
                await callback(processed_update)
                
    async def process_news_update(self, update):
        # Process incoming news with template server
        return await self.template_client.call_tool(
            "render_template",
            {
                "template_id": "news_alert",
                "parameters": {"news_item": update}
            }
        )
```

### 5. Error Handling and Resilience
Implement robust error handling for production deployments:

```python
class ResilientAgent:
    def __init__(self):
        self.circuit_breakers = {}
        self.retry_policies = {}
        
    async def call_mcp_tool_with_resilience(
        self, 
        server_url: str, 
        tool_name: str, 
        parameters: Dict
    ):
        # Circuit breaker pattern
        if self.circuit_breakers.get(server_url, {}).get('open', False):
            raise CircuitBreakerOpenError(f"Circuit breaker open for {server_url}")
        
        retry_policy = self.retry_policies.get(server_url, DEFAULT_RETRY_POLICY)
        
        for attempt in range(retry_policy.max_attempts):
            try:
                client = MCPClient(server_url)
                result = await client.call_tool(tool_name, parameters)
                
                # Reset circuit breaker on success
                self.circuit_breakers[server_url] = {'open': False, 'failures': 0}
                
                return result
                
            except Exception as e:
                self.handle_mcp_error(server_url, e)
                
                if attempt < retry_policy.max_attempts - 1:
                    await asyncio.sleep(retry_policy.get_delay(attempt))
                else:
                    raise
    
    def handle_mcp_error(self, server_url: str, error: Exception):
        breaker = self.circuit_breakers.setdefault(server_url, {'open': False, 'failures': 0})
        breaker['failures'] += 1
        
        if breaker['failures'] >= CIRCUIT_BREAKER_THRESHOLD:
            breaker['open'] = True
            logger.warning(f"Circuit breaker opened for {server_url}")
```

## 🚀 Getting Started

### 1. Setup MCP Servers
Ensure all required MCP servers are running:
```bash
# Start servers in separate terminals
cd servers/news-data-server && cargo run
cd servers/template-server && cargo run  
cd servers/database-server && cargo run
cd servers/analytics-server && cargo run
```

### 2. Install Agent Framework Dependencies
```bash
# Python frameworks
pip install langchain autogen crewai openai

# For custom Rust agents
cargo add tokio serde reqwest
```

### 3. Run Integration Examples
```bash
# LangChain example
python examples/agent-integration-patterns/langchain/news_agent.py

# AutoGen multi-agent example
python examples/agent-integration-patterns/autogen/content_crew.py

# CrewAI collaborative agents
python examples/agent-integration-patterns/crewai/research_team.py

# Custom Rust agent
cargo run --bin custom-agent
```

## 🔧 Configuration

### Environment Variables
```bash
# MCP Server URLs
export NEWS_SERVER_URL="http://localhost:3001"
export TEMPLATE_SERVER_URL="http://localhost:3002"
export DATABASE_SERVER_URL="http://localhost:3003"
export ANALYTICS_SERVER_URL="http://localhost:3004"

# Agent Configuration
export AGENT_TIMEOUT=30
export MAX_RETRIES=3
export ENABLE_CIRCUIT_BREAKERS=true
export LOG_LEVEL=INFO
```

### Configuration File (agent_config.yaml)
```yaml
mcp_servers:
  news:
    url: "http://localhost:3001"
    timeout: 30
    max_retries: 3
  template:
    url: "http://localhost:3002"
    timeout: 15
    max_retries: 2
  database:
    url: "http://localhost:3003"
    timeout: 45
    max_retries: 3
  analytics:
    url: "http://localhost:3004"
    timeout: 20
    max_retries: 2

agent_settings:
  enable_circuit_breakers: true
  circuit_breaker_threshold: 5
  circuit_breaker_timeout: 60
  default_timeout: 30
  enable_streaming: false
  enable_caching: true
  cache_ttl: 300
```

## 📊 Performance Optimization

### 1. Connection Pooling
```python
class PooledMCPClient:
    def __init__(self, server_urls: List[str], pool_size: int = 5):
        self.pools = {}
        for url in server_urls:
            self.pools[url] = ConnectionPool(url, pool_size)
    
    async def call_tool(self, server_url: str, tool_name: str, params: Dict):
        async with self.pools[server_url].get_connection() as client:
            return await client.call_tool(tool_name, params)
```

### 2. Parallel Execution
```python
async def parallel_mcp_calls(calls: List[MCPCall]) -> List[Any]:
    tasks = []
    for call in calls:
        task = asyncio.create_task(
            execute_mcp_call(call.server, call.tool, call.params)
        )
        tasks.append(task)
    
    return await asyncio.gather(*tasks, return_exceptions=True)
```

### 3. Intelligent Caching
```python
class MCPCache:
    def __init__(self, ttl: int = 300):
        self.cache = {}
        self.ttl = ttl
    
    def cache_key(self, server: str, tool: str, params: Dict) -> str:
        params_str = json.dumps(params, sort_keys=True)
        return f"{server}:{tool}:{hashlib.md5(params_str.encode()).hexdigest()}"
    
    async def get_or_call(self, server: str, tool: str, params: Dict):
        key = self.cache_key(server, tool, params)
        
        if key in self.cache:
            result, timestamp = self.cache[key]
            if time.time() - timestamp < self.ttl:
                return result
        
        result = await execute_mcp_call(server, tool, params)
        self.cache[key] = (result, time.time())
        return result
```

## 🔒 Security Best Practices

### 1. Authentication and Authorization
```python
class SecureMCPClient:
    def __init__(self, server_url: str, api_key: str):
        self.server_url = server_url
        self.api_key = api_key
        
    async def call_tool(self, tool_name: str, parameters: Dict):
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }
        
        # Validate parameters
        sanitized_params = self.sanitize_parameters(parameters)
        
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{self.server_url}/{tool_name}",
                json=sanitized_params,
                headers=headers
            )
            
        return response.json()
```

### 2. Input Validation
```python
def validate_mcp_parameters(tool_schema: Dict, parameters: Dict) -> Dict:
    """Validate and sanitize parameters according to tool schema"""
    validated = {}
    
    for param_name, param_value in parameters.items():
        if param_name not in tool_schema["parameters"]:
            continue  # Skip unknown parameters
            
        param_spec = tool_schema["parameters"][param_name]
        
        # Type validation
        if not isinstance(param_value, param_spec["type"]):
            raise ValueError(f"Parameter {param_name} must be {param_spec['type']}")
        
        # Length validation for strings
        if param_spec["type"] == str and "max_length" in param_spec:
            if len(param_value) > param_spec["max_length"]:
                param_value = param_value[:param_spec["max_length"]]
        
        validated[param_name] = param_value
    
    return validated
```

### 3. Rate Limiting
```python
class RateLimitedMCPClient:
    def __init__(self, server_url: str, rate_limit: int = 100):
        self.server_url = server_url
        self.rate_limiter = TokenBucket(rate_limit, rate_limit)
        
    async def call_tool(self, tool_name: str, parameters: Dict):
        # Wait for rate limit token
        await self.rate_limiter.acquire()
        
        # Make the actual call
        return await super().call_tool(tool_name, parameters)
```

## 🧪 Testing

### Unit Tests
```python
import pytest
from unittest.mock import AsyncMock

@pytest.mark.asyncio
async def test_agent_mcp_integration():
    # Mock MCP client
    mock_client = AsyncMock()
    mock_client.call_tool.return_value = {"result": "test_data"}
    
    agent = TestAgent(mcp_client=mock_client)
    result = await agent.process_request("test query")
    
    # Verify MCP tool was called correctly
    mock_client.call_tool.assert_called_once_with(
        "search_news", 
        {"query": "test query", "limit": 5}
    )
    
    assert result["success"] is True
```

### Integration Tests
```python
@pytest.mark.integration
async def test_full_agent_workflow():
    # Start test MCP servers
    async with TestMCPServers() as servers:
        agent = ProductionAgent(servers.get_urls())
        
        # Execute full workflow
        result = await agent.create_content_pipeline("AI technology")
        
        # Verify end-to-end functionality
        assert result["article_id"] is not None
        assert result["analytics_tracked"] is True
```

## 📈 Monitoring and Observability

### Metrics Collection
```python
from prometheus_client import Counter, Histogram, Gauge

# Define metrics
mcp_tool_calls = Counter(
    'agent_mcp_tool_calls_total',
    'Total MCP tool calls',
    ['server', 'tool', 'status']
)

mcp_response_time = Histogram(
    'agent_mcp_response_time_seconds',
    'MCP tool response time',
    ['server', 'tool']
)

active_agents = Gauge(
    'active_agents_count',
    'Number of active agents'
)

# Instrument MCP calls
async def instrumented_mcp_call(server, tool, params):
    with mcp_response_time.labels(server=server, tool=tool).time():
        try:
            result = await call_mcp_tool(server, tool, params)
            mcp_tool_calls.labels(server=server, tool=tool, status='success').inc()
            return result
        except Exception as e:
            mcp_tool_calls.labels(server=server, tool=tool, status='error').inc()
            raise
```

### Distributed Tracing
```python
from opentelemetry import trace
from opentelemetry.exporter.jaeger.thrift import JaegerExporter

tracer = trace.get_tracer(__name__)

async def traced_agent_execution(task: str):
    with tracer.start_as_current_span("agent_execution") as span:
        span.set_attribute("task", task)
        
        # Trace MCP calls
        with tracer.start_as_current_span("mcp_call") as mcp_span:
            mcp_span.set_attribute("server", "news-server")
            mcp_span.set_attribute("tool", "search_news")
            
            result = await call_mcp_tool("news-server", "search_news", params)
            
        span.set_attribute("result_count", len(result))
        return result
```

## 🤝 Contributing

When adding new agent integration patterns:

1. **Follow Established Patterns**: Use the existing integration patterns as templates
2. **Add Comprehensive Examples**: Include complete, runnable examples
3. **Document Configuration**: Clearly document all configuration options
4. **Include Error Handling**: Implement robust error handling and resilience
5. **Add Tests**: Include unit and integration tests
6. **Update Documentation**: Update this README with new patterns
7. **Performance Considerations**: Benchmark and optimize for production use

### Development Commands
```bash
# Format Python code
black examples/agent-integration-patterns/

# Lint Python code
flake8 examples/agent-integration-patterns/

# Run tests
pytest examples/agent-integration-patterns/tests/

# Type checking
mypy examples/agent-integration-patterns/
```

## 📚 Additional Resources

- [MCP Server Documentation](../../servers/)
- [Multi-Server Coordination](../multi-server-coordination/)
- [Performance Optimization](../performance-optimization/)
- [Security Hardening](../security-hardening/)

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**Next Steps**: Explore specific framework examples in the subdirectories or learn about [performance optimization patterns](../performance-optimization/).
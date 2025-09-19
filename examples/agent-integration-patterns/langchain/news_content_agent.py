#!/usr/bin/env python3
"""
LangChain Integration Example with MCP Servers

This example demonstrates how to integrate LangChain agents with our MCP server ecosystem
to create a sophisticated news content generation agent. The agent can:

1. Search for news using the news-data-server
2. Generate content using the template-server
3. Store articles in the database-server
4. Track analytics using the analytics-server

Features:
- Custom MCP tool integration with LangChain
- Error handling and fallback mechanisms
- Streaming responses for real-time interaction
- Memory management for conversation context
- Performance monitoring and metrics
"""

import asyncio
import json
import logging
import time
from typing import Any, Dict, List, Optional, Type, Union
from dataclasses import dataclass
from datetime import datetime, timezone

import httpx
from pydantic import BaseModel, Field

# LangChain imports
from langchain.agents import AgentExecutor, create_openai_functions_agent
from langchain.agents.format_scratchpad import format_to_openai_function_messages
from langchain.agents.output_parsers import OpenAIFunctionsAgentOutputParser
from langchain.chat_models import ChatOpenAI
from langchain.memory import ConversationBufferWindowMemory
from langchain.prompts import ChatPromptTemplate, MessagesPlaceholder
from langchain.schema import AgentAction, AgentFinish, BaseMessage
from langchain.tools import BaseTool, StructuredTool
from langchain.callbacks.base import BaseCallbackHandler
from langchain.schema.output import LLMResult

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class MCPServerConfig:
    """Configuration for MCP server connection"""
    name: str
    url: str
    timeout: int = 30
    max_retries: int = 3
    health_endpoint: str = "/health"


class MCPToolCallError(Exception):
    """Exception raised when MCP tool call fails"""
    pass


class MCPClient:
    """Async HTTP client for MCP server communication"""
    
    def __init__(self, config: MCPServerConfig):
        self.config = config
        self.client = httpx.AsyncClient(timeout=config.timeout)
        self._healthy = None
        self._last_health_check = 0
        
    async def __aenter__(self):
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.client.aclose()
    
    async def health_check(self) -> bool:
        """Check if the MCP server is healthy"""
        current_time = time.time()
        
        # Cache health check for 30 seconds
        if self._healthy is not None and (current_time - self._last_health_check) < 30:
            return self._healthy
        
        try:
            response = await self.client.get(f"{self.config.url}{self.config.health_endpoint}")
            self._healthy = response.status_code == 200
            self._last_health_check = current_time
            return self._healthy
        except Exception as e:
            logger.warning(f"Health check failed for {self.config.name}: {e}")
            self._healthy = False
            self._last_health_check = current_time
            return False
    
    async def call_tool(self, tool_name: str, parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Call an MCP tool with retry logic"""
        if not await self.health_check():
            raise MCPToolCallError(f"Server {self.config.name} is not healthy")
        
        url = f"{self.config.url}/{tool_name}"
        
        for attempt in range(self.config.max_retries):
            try:
                response = await self.client.post(
                    url,
                    json=parameters,
                    headers={"Content-Type": "application/json"}
                )
                
                if response.status_code == 200:
                    return response.json()
                else:
                    error_msg = f"HTTP {response.status_code}: {response.text}"
                    if attempt == self.config.max_retries - 1:
                        raise MCPToolCallError(error_msg)
                    logger.warning(f"Attempt {attempt + 1} failed: {error_msg}")
                    
            except httpx.RequestError as e:
                if attempt == self.config.max_retries - 1:
                    raise MCPToolCallError(f"Request failed: {e}")
                logger.warning(f"Attempt {attempt + 1} failed: {e}")
                
            # Exponential backoff
            await asyncio.sleep(2 ** attempt)
        
        raise MCPToolCallError(f"All {self.config.max_retries} attempts failed")


class MCPCallbackHandler(BaseCallbackHandler):
    """Callback handler to monitor MCP tool usage"""
    
    def __init__(self):
        self.tool_calls = []
        self.start_times = {}
        
    def on_tool_start(self, serialized: Dict[str, Any], input_str: str, **kwargs) -> None:
        tool_name = serialized.get("name", "unknown")
        self.start_times[tool_name] = time.time()
        logger.info(f"🔧 Starting MCP tool: {tool_name}")
        
    def on_tool_end(self, output: str, **kwargs) -> None:
        logger.info(f"✅ MCP tool completed successfully")
        
    def on_tool_error(self, error: Union[Exception, KeyboardInterrupt], **kwargs) -> None:
        logger.error(f"❌ MCP tool error: {error}")
        
    def on_llm_start(self, serialized: Dict[str, Any], prompts: List[str], **kwargs) -> None:
        logger.info("🤖 LLM processing...")
        
    def on_llm_end(self, response: LLMResult, **kwargs) -> None:
        logger.info("✅ LLM response generated")


# Pydantic models for tool inputs
class NewsSearchInput(BaseModel):
    query: str = Field(description="Search query for news articles")
    category: str = Field(default="general", description="News category (technology, business, health, etc.)")
    limit: int = Field(default=5, description="Maximum number of articles to return")


class TemplateRenderInput(BaseModel):
    template_id: str = Field(description="Template identifier to use for rendering")
    parameters: Dict[str, Any] = Field(description="Parameters to pass to the template")


class DatabaseQueryInput(BaseModel):
    query: str = Field(description="SQL query to execute")
    parameters: List[Any] = Field(default=[], description="Query parameters")


class AnalyticsTrackInput(BaseModel):
    content_id: str = Field(description="Content identifier to track")
    metrics: Dict[str, Any] = Field(description="Metrics data to track")


class MCPNewsTool(BaseTool):
    """LangChain tool for news data server"""
    
    name = "search_news"
    description = "Search for news articles by query and category. Returns relevant news data."
    args_schema: Type[BaseModel] = NewsSearchInput
    
    def __init__(self, mcp_client: MCPClient):
        super().__init__()
        self.mcp_client = mcp_client
    
    async def _arun(self, query: str, category: str = "general", limit: int = 5) -> str:
        """Async implementation of the news search tool"""
        try:
            parameters = {
                "query": query,
                "category": category,
                "limit": limit
            }
            
            result = await self.mcp_client.call_tool("search_news", parameters)
            
            # Format the result for the agent
            news_items = result.get("news_items", [])
            if not news_items:
                return f"No news found for query: {query}"
            
            formatted_result = f"Found {len(news_items)} news articles for '{query}':\n\n"
            for i, item in enumerate(news_items, 1):
                formatted_result += f"{i}. {item.get('title', 'No title')}\n"
                formatted_result += f"   Source: {item.get('source', 'Unknown')}\n"
                formatted_result += f"   Summary: {item.get('description', 'No description')[:200]}...\n\n"
            
            return formatted_result
            
        except Exception as e:
            logger.error(f"News search failed: {e}")
            return f"Error searching news: {str(e)}"
    
    def _run(self, query: str, category: str = "general", limit: int = 5) -> str:
        """Sync wrapper for async implementation"""
        return asyncio.run(self._arun(query, category, limit))


class MCPTemplateTool(BaseTool):
    """LangChain tool for template server"""
    
    name = "render_template"
    description = "Render content using a template. Useful for generating formatted articles, reports, or documents."
    args_schema: Type[BaseModel] = TemplateRenderInput
    
    def __init__(self, mcp_client: MCPClient):
        super().__init__()
        self.mcp_client = mcp_client
    
    async def _arun(self, template_id: str, parameters: Dict[str, Any]) -> str:
        """Async implementation of the template rendering tool"""
        try:
            tool_parameters = {
                "template_id": template_id,
                "parameters": parameters
            }
            
            result = await self.mcp_client.call_tool("render_template", tool_parameters)
            
            rendered_content = result.get("rendered_content", "")
            if not rendered_content:
                return f"Template rendering failed for template: {template_id}"
            
            return f"Template '{template_id}' rendered successfully:\n\n{rendered_content}"
            
        except Exception as e:
            logger.error(f"Template rendering failed: {e}")
            return f"Error rendering template: {str(e)}"
    
    def _run(self, template_id: str, parameters: Dict[str, Any]) -> str:
        """Sync wrapper for async implementation"""
        return asyncio.run(self._arun(template_id, parameters))


class MCPDatabaseTool(BaseTool):
    """LangChain tool for database server"""
    
    name = "store_content"
    description = "Store content in the database. Use this to save articles, data, or any structured information."
    args_schema: Type[BaseModel] = DatabaseQueryInput
    
    def __init__(self, mcp_client: MCPClient):
        super().__init__()
        self.mcp_client = mcp_client
    
    async def _arun(self, query: str, parameters: List[Any] = None) -> str:
        """Async implementation of the database tool"""
        try:
            if parameters is None:
                parameters = []
                
            tool_parameters = {
                "query": query,
                "parameters": parameters
            }
            
            result = await self.mcp_client.call_tool("execute_query", tool_parameters)
            
            if result.get("success", False):
                rows_affected = result.get("rows_affected", 0)
                return f"Database operation successful. {rows_affected} rows affected."
            else:
                error_msg = result.get("error", "Unknown database error")
                return f"Database operation failed: {error_msg}"
                
        except Exception as e:
            logger.error(f"Database operation failed: {e}")
            return f"Error executing database operation: {str(e)}"
    
    def _run(self, query: str, parameters: List[Any] = None) -> str:
        """Sync wrapper for async implementation"""
        return asyncio.run(self._arun(query, parameters))


class MCPAnalyticsTool(BaseTool):
    """LangChain tool for analytics server"""
    
    name = "track_analytics"
    description = "Track analytics and metrics for content. Use this to record performance data and insights."
    args_schema: Type[BaseModel] = AnalyticsTrackInput
    
    def __init__(self, mcp_client: MCPClient):
        super().__init__()
        self.mcp_client = mcp_client
    
    async def _arun(self, content_id: str, metrics: Dict[str, Any]) -> str:
        """Async implementation of the analytics tool"""
        try:
            tool_parameters = {
                "content_id": content_id,
                "metrics": metrics
            }
            
            result = await self.mcp_client.call_tool("track_content_metrics", tool_parameters)
            
            if result.get("success", False):
                tracking_id = result.get("tracking_id", "unknown")
                return f"Analytics tracking successful. Tracking ID: {tracking_id}"
            else:
                error_msg = result.get("error", "Unknown analytics error")
                return f"Analytics tracking failed: {error_msg}"
                
        except Exception as e:
            logger.error(f"Analytics tracking failed: {e}")
            return f"Error tracking analytics: {str(e)}"
    
    def _run(self, content_id: str, metrics: Dict[str, Any]) -> str:
        """Sync wrapper for async implementation"""
        return asyncio.run(self._arun(content_id, metrics))


class NewsContentAgent:
    """LangChain agent integrated with MCP servers for news content generation"""
    
    def __init__(self, openai_api_key: str, model: str = "gpt-4-turbo-preview"):
        self.openai_api_key = openai_api_key
        self.model = model
        
        # MCP Server configurations
        self.server_configs = {
            "news": MCPServerConfig("news-data-server", "http://localhost:3001"),
            "template": MCPServerConfig("template-server", "http://localhost:3002"),
            "database": MCPServerConfig("database-server", "http://localhost:3003"),
            "analytics": MCPServerConfig("analytics-server", "http://localhost:3004"),
        }
        
        # Initialize components
        self.mcp_clients = {}
        self.tools = []
        self.llm = None
        self.agent_executor = None
        self.callback_handler = MCPCallbackHandler()
        
    async def initialize(self):
        """Initialize MCP clients and LangChain agent"""
        logger.info("🚀 Initializing News Content Agent...")
        
        # Initialize MCP clients
        for name, config in self.server_configs.items():
            client = MCPClient(config)
            self.mcp_clients[name] = client
            
            # Check server health
            if await client.health_check():
                logger.info(f"✅ {config.name} is healthy")
            else:
                logger.warning(f"⚠️ {config.name} is not responding")
        
        # Create MCP tools
        self.tools = [
            MCPNewsTool(self.mcp_clients["news"]),
            MCPTemplateTool(self.mcp_clients["template"]),
            MCPDatabaseTool(self.mcp_clients["database"]),
            MCPAnalyticsTool(self.mcp_clients["analytics"]),
        ]
        
        # Initialize LLM
        self.llm = ChatOpenAI(
            model=self.model,
            temperature=0.7,
            openai_api_key=self.openai_api_key
        )
        
        # Create agent prompt
        prompt = ChatPromptTemplate.from_messages([
            ("system", """You are a sophisticated news content agent with access to powerful MCP servers.

Your capabilities include:
1. **search_news**: Search for current news articles by topic and category
2. **render_template**: Generate formatted content using professional templates  
3. **store_content**: Save articles and data to the database
4. **track_analytics**: Record performance metrics and insights

You excel at:
- Finding relevant, current news on any topic
- Creating professional, well-formatted content
- Managing data storage and retrieval
- Tracking content performance and engagement

Always provide helpful, accurate, and well-structured responses. When using tools, explain what you're doing and why. If a tool fails, suggest alternatives or workarounds.

Current timestamp: {timestamp}"""),
            ("user", "{input}"),
            MessagesPlaceholder(variable_name="agent_scratchpad"),
            MessagesPlaceholder(variable_name="chat_history"),
        ])
        
        # Create memory for conversation context
        memory = ConversationBufferWindowMemory(
            memory_key="chat_history",
            return_messages=True,
            k=10  # Keep last 10 exchanges
        )
        
        # Create agent
        agent = create_openai_functions_agent(self.llm, self.tools, prompt)
        
        # Create agent executor
        self.agent_executor = AgentExecutor(
            agent=agent,
            tools=self.tools,
            memory=memory,
            verbose=True,
            callbacks=[self.callback_handler],
            max_iterations=10,
            early_stopping_method="generate"
        )
        
        logger.info("✅ News Content Agent initialized successfully")
    
    async def process_request(self, user_input: str) -> str:
        """Process a user request using the agent"""
        if not self.agent_executor:
            raise RuntimeError("Agent not initialized. Call initialize() first.")
        
        logger.info(f"🎯 Processing request: {user_input}")
        
        try:
            # Add current timestamp to the prompt
            current_time = datetime.now(timezone.utc).isoformat()
            
            # Execute the agent
            result = await self.agent_executor.ainvoke({
                "input": user_input,
                "timestamp": current_time
            })
            
            return result["output"]
            
        except Exception as e:
            logger.error(f"❌ Error processing request: {e}")
            return f"I encountered an error while processing your request: {str(e)}"
    
    async def create_news_article(self, topic: str, category: str = "general") -> str:
        """Specialized method for creating news articles"""
        request = f"""Create a comprehensive news article about '{topic}' in the {category} category. 

Please:
1. Search for the latest news on this topic
2. Use an appropriate template to create a professional article
3. Store the article in the database
4. Track analytics for the content

Provide me with a summary of what was accomplished."""
        
        return await self.process_request(request)
    
    async def get_news_summary(self, topic: str, count: int = 5) -> str:
        """Get a news summary for a specific topic"""
        request = f"Find and summarize the top {count} news articles about '{topic}'. Provide key insights and trends."
        return await self.process_request(request)
    
    async def cleanup(self):
        """Cleanup resources"""
        logger.info("🧹 Cleaning up News Content Agent...")
        for client in self.mcp_clients.values():
            await client.__aexit__(None, None, None)


async def main():
    """Example usage of the News Content Agent"""
    import os
    
    # Check for OpenAI API key
    openai_api_key = os.getenv("OPENAI_API_KEY")
    if not openai_api_key:
        print("❌ Please set OPENAI_API_KEY environment variable")
        return
    
    # Create and initialize agent
    agent = NewsContentAgent(openai_api_key)
    
    try:
        await agent.initialize()
        
        print("\n🤖 News Content Agent Ready!")
        print("=" * 50)
        
        # Example interactions
        examples = [
            "Find the latest news about artificial intelligence",
            "Create an article about renewable energy innovations",
            "Search for business news about electric vehicles and create a summary",
            "What are the current trends in healthcare technology?",
        ]
        
        for i, example in enumerate(examples, 1):
            print(f"\n📝 Example {i}: {example}")
            print("-" * 40)
            
            response = await agent.process_request(example)
            print(f"🤖 Agent Response:\n{response}")
            
            if i < len(examples):
                print("\n⏳ Waiting 3 seconds before next example...")
                await asyncio.sleep(3)
        
        print("\n✅ All examples completed successfully!")
        
    except KeyboardInterrupt:
        print("\n👋 Shutting down...")
    except Exception as e:
        print(f"\n❌ Error: {e}")
    finally:
        await agent.cleanup()


if __name__ == "__main__":
    asyncio.run(main())
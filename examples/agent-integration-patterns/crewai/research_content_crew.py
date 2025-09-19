#!/usr/bin/env python3
"""
CrewAI Integration Example with MCP Servers

This example demonstrates how to integrate CrewAI with our MCP server ecosystem
to create a collaborative research and content creation crew. The crew consists of
specialized agents that work together using different MCP servers:

1. Research Analyst - Uses news-data-server to gather information
2. Content Writer - Uses template-server to create professional content
3. Data Manager - Uses database-server to store and retrieve data
4. Performance Tracker - Uses analytics-server to monitor success metrics

Features:
- Multi-agent collaboration with specialized roles
- MCP server integration for each agent
- Task delegation and workflow coordination
- Error handling and fallback mechanisms
- Performance monitoring and reporting
"""

import asyncio
import json
import logging
import time
from typing import Any, Dict, List, Optional, Union
from dataclasses import dataclass
from datetime import datetime

import httpx
from pydantic import BaseModel, Field

# CrewAI imports
try:
    from crewai import Agent, Task, Crew, Process
    from crewai.tools import BaseTool
    from langchain.llms import OpenAI
    from langchain.chat_models import ChatOpenAI
except ImportError:
    print("❌ CrewAI not installed. Install with: pip install crewai")
    exit(1)

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


class MCPToolCallError(Exception):
    """Exception raised when MCP tool call fails"""
    pass


class MCPClient:
    """Async HTTP client for MCP server communication"""
    
    def __init__(self, config: MCPServerConfig):
        self.config = config
        self.client = httpx.AsyncClient(timeout=config.timeout)
        
    async def __aenter__(self):
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.client.aclose()
    
    async def call_tool(self, tool_name: str, parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Call an MCP tool with retry logic"""
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


class NewsResearchTool(BaseTool):
    """CrewAI tool for news research using MCP news-data-server"""
    
    name = "news_research"
    description = "Research current news articles on specific topics. Returns detailed news data and insights."
    
    def __init__(self, mcp_client: MCPClient):
        super().__init__()
        self.mcp_client = mcp_client
    
    def _run(self, query: str, category: str = "general", limit: int = 10) -> str:
        """Research news articles synchronously"""
        return asyncio.run(self._arun(query, category, limit))
    
    async def _arun(self, query: str, category: str = "general", limit: int = 10) -> str:
        """Research news articles asynchronously"""
        try:
            parameters = {
                "query": query,
                "category": category,
                "limit": limit
            }
            
            result = await self.mcp_client.call_tool("search_news", parameters)
            
            news_items = result.get("news_items", [])
            if not news_items:
                return f"No news found for query: {query}"
            
            # Format comprehensive research report
            report = f"🔍 RESEARCH REPORT: {query.upper()}\n"
            report += f"Category: {category.title()} | Articles Found: {len(news_items)}\n"
            report += "=" * 60 + "\n\n"
            
            for i, item in enumerate(news_items, 1):
                report += f"📰 Article {i}: {item.get('title', 'No title')}\n"
                report += f"   📅 Published: {item.get('published_at', 'Unknown date')}\n"
                report += f"   🏢 Source: {item.get('source', 'Unknown source')}\n"
                report += f"   📊 Engagement: {item.get('engagement_score', 0):.2f}\n"
                report += f"   📝 Summary: {item.get('description', 'No description')}\n"
                if item.get('url'):
                    report += f"   🔗 URL: {item['url']}\n"
                report += "\n" + "-" * 40 + "\n\n"
            
            # Add research insights
            report += "💡 KEY INSIGHTS:\n"
            sources = set(item.get('source', 'Unknown') for item in news_items)
            report += f"• Sources covered: {', '.join(sources)}\n"
            
            avg_engagement = sum(item.get('engagement_score', 0) for item in news_items) / len(news_items)
            report += f"• Average engagement score: {avg_engagement:.2f}\n"
            
            recent_articles = sum(1 for item in news_items if 'today' in item.get('published_at', '').lower())
            report += f"• Recent articles (today): {recent_articles}\n"
            
            return report
            
        except Exception as e:
            logger.error(f"News research failed: {e}")
            return f"❌ Research failed for '{query}': {str(e)}"


class ContentCreationTool(BaseTool):
    """CrewAI tool for content creation using MCP template-server"""
    
    name = "content_creation"
    description = "Create professional content using templates. Supports articles, reports, and various content formats."
    
    def __init__(self, mcp_client: MCPClient):
        super().__init__()
        self.mcp_client = mcp_client
    
    def _run(self, template_type: str, content_data: str, title: str = "", style: str = "professional") -> str:
        """Create content synchronously"""
        return asyncio.run(self._arun(template_type, content_data, title, style))
    
    async def _arun(self, template_type: str, content_data: str, title: str = "", style: str = "professional") -> str:
        """Create content asynchronously"""
        try:
            # Map template types to actual template IDs
            template_mapping = {
                "article": "blog_post",
                "report": "business_report", 
                "summary": "news_summary",
                "email": "email_template",
                "documentation": "api_documentation"
            }
            
            template_id = template_mapping.get(template_type.lower(), "blog_post")
            
            parameters = {
                "template_id": template_id,
                "parameters": {
                    "title": title or f"{template_type.title()} on Current Topics",
                    "content": content_data,
                    "style": style,
                    "created_at": datetime.now().isoformat(),
                    "author": "Research Content Crew",
                    "sections": self._extract_sections(content_data)
                }
            }
            
            result = await self.mcp_client.call_tool("render_template", parameters)
            
            rendered_content = result.get("rendered_content", "")
            if not rendered_content:
                return f"❌ Content creation failed for template: {template_id}"
            
            # Format the response
            response = f"✅ CONTENT CREATED SUCCESSFULLY\n"
            response += f"Template: {template_type.title()} ({template_id})\n"
            response += f"Style: {style.title()}\n"
            response += "=" * 50 + "\n\n"
            response += rendered_content
            
            return response
            
        except Exception as e:
            logger.error(f"Content creation failed: {e}")
            return f"❌ Content creation failed: {str(e)}"
    
    def _extract_sections(self, content: str) -> List[str]:
        """Extract sections from content for better template rendering"""
        # Simple section extraction based on paragraphs and bullet points
        lines = content.split('\n')
        sections = []
        current_section = []
        
        for line in lines:
            line = line.strip()
            if line.startswith('•') or line.startswith('-') or line.startswith('*'):
                if current_section:
                    sections.append('\n'.join(current_section))
                    current_section = []
                current_section.append(line)
            elif line and not line.startswith(' '):
                current_section.append(line)
                
        if current_section:
            sections.append('\n'.join(current_section))
            
        return sections[:5]  # Limit to 5 sections


class DataStorageTool(BaseTool):
    """CrewAI tool for data storage using MCP database-server"""
    
    name = "data_storage"
    description = "Store content, research data, and analytics in the database for future reference."
    
    def __init__(self, mcp_client: MCPClient):
        super().__init__()
        self.mcp_client = mcp_client
    
    def _run(self, content_type: str, title: str, content: str, metadata: str = "{}") -> str:
        """Store data synchronously"""
        return asyncio.run(self._arun(content_type, title, content, metadata))
    
    async def _arun(self, content_type: str, title: str, content: str, metadata: str = "{}") -> str:
        """Store data asynchronously"""
        try:
            # Parse metadata if it's a string
            if isinstance(metadata, str):
                try:
                    metadata_dict = json.loads(metadata)
                except json.JSONDecodeError:
                    metadata_dict = {"raw_metadata": metadata}
            else:
                metadata_dict = metadata
            
            # Create storage query
            query = """
            INSERT INTO crew_content (
                content_type, title, content, metadata, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            """
            
            current_time = datetime.now().isoformat()
            parameters = [
                content_type,
                title,
                content,
                json.dumps(metadata_dict),
                current_time,
                current_time
            ]
            
            result = await self.mcp_client.call_tool("execute_query", {
                "query": query,
                "parameters": parameters
            })
            
            if result.get("success", False):
                stored_id = result.get("inserted_id", "unknown")
                return f"✅ Content stored successfully!\nID: {stored_id}\nType: {content_type}\nTitle: {title}"
            else:
                error_msg = result.get("error", "Unknown database error")
                return f"❌ Storage failed: {error_msg}"
                
        except Exception as e:
            logger.error(f"Data storage failed: {e}")
            return f"❌ Data storage failed: {str(e)}"


class PerformanceTrackingTool(BaseTool):
    """CrewAI tool for performance tracking using MCP analytics-server"""
    
    name = "performance_tracking"
    description = "Track performance metrics and analytics for content and research activities."
    
    def __init__(self, mcp_client: MCPClient):
        super().__init__()
        self.mcp_client = mcp_client
    
    def _run(self, activity_type: str, content_id: str, metrics_data: str) -> str:
        """Track performance synchronously"""
        return asyncio.run(self._arun(activity_type, content_id, metrics_data))
    
    async def _arun(self, activity_type: str, content_id: str, metrics_data: str) -> str:
        """Track performance asynchronously"""
        try:
            # Parse metrics data
            if isinstance(metrics_data, str):
                try:
                    metrics = json.loads(metrics_data)
                except json.JSONDecodeError:
                    metrics = {"raw_data": metrics_data}
            else:
                metrics = metrics_data
            
            # Add default metrics
            metrics.update({
                "activity_type": activity_type,
                "timestamp": datetime.now().isoformat(),
                "crew_session_id": f"crew_{int(time.time())}",
                "content_length": len(str(metrics_data))
            })
            
            parameters = {
                "content_id": content_id,
                "metrics": metrics
            }
            
            result = await self.mcp_client.call_tool("track_content_metrics", parameters)
            
            if result.get("success", False):
                tracking_id = result.get("tracking_id", "unknown")
                performance_score = result.get("performance_score", 0.0)
                
                return f"📊 PERFORMANCE TRACKED\nTracking ID: {tracking_id}\nActivity: {activity_type}\nScore: {performance_score:.2f}"
            else:
                error_msg = result.get("error", "Unknown analytics error")
                return f"❌ Performance tracking failed: {error_msg}"
                
        except Exception as e:
            logger.error(f"Performance tracking failed: {e}")
            return f"❌ Performance tracking failed: {str(e)}"


class ResearchContentCrew:
    """CrewAI crew for research and content creation using MCP servers"""
    
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
        self.tools = {}
        self.agents = {}
        self.crew = None
        
    async def initialize(self):
        """Initialize MCP clients and CrewAI agents"""
        logger.info("🚀 Initializing Research Content Crew...")
        
        # Initialize MCP clients
        for name, config in self.server_configs.items():
            client = MCPClient(config)
            self.mcp_clients[name] = client
            logger.info(f"✅ {config.name} client initialized")
        
        # Initialize LLM for all agents
        llm = ChatOpenAI(
            model=self.model,
            temperature=0.7,
            openai_api_key=self.openai_api_key
        )
        
        # Create specialized tools
        self.tools = {
            "news_research": NewsResearchTool(self.mcp_clients["news"]),
            "content_creation": ContentCreationTool(self.mcp_clients["template"]),
            "data_storage": DataStorageTool(self.mcp_clients["database"]),
            "performance_tracking": PerformanceTrackingTool(self.mcp_clients["analytics"]),
        }
        
        # Create specialized agents
        self.agents = {
            "researcher": Agent(
                role='Research Analyst',
                goal='Conduct thorough research on assigned topics using current news and data sources',
                backstory="""You are an expert research analyst with access to real-time news data. 
                You excel at finding relevant information, analyzing trends, and providing comprehensive 
                research reports. You use the news_research tool to gather current information.""",
                tools=[self.tools["news_research"]],
                llm=llm,
                verbose=True,
                allow_delegation=False
            ),
            
            "writer": Agent(
                role='Content Writer',
                goal='Create engaging, professional content based on research findings',
                backstory="""You are a skilled content writer who transforms research data into 
                compelling articles, reports, and documentation. You use professional templates 
                and maintain high editorial standards. You use the content_creation tool to 
                generate polished content.""",
                tools=[self.tools["content_creation"]],
                llm=llm,
                verbose=True,
                allow_delegation=False
            ),
            
            "data_manager": Agent(
                role='Data Manager', 
                goal='Organize and store all research data and content for future reference',
                backstory="""You are a meticulous data manager responsible for organizing and 
                storing all crew outputs. You ensure data integrity, proper categorization, 
                and easy retrieval. You use the data_storage tool to maintain organized records.""",
                tools=[self.tools["data_storage"]],
                llm=llm,
                verbose=True,
                allow_delegation=False
            ),
            
            "analyst": Agent(
                role='Performance Analyst',
                goal='Monitor and analyze the performance of all crew activities',
                backstory="""You are a performance analyst who tracks metrics, analyzes outcomes, 
                and provides insights on crew effectiveness. You monitor content performance, 
                research quality, and operational efficiency. You use the performance_tracking 
                tool to record and analyze metrics.""",
                tools=[self.tools["performance_tracking"]],
                llm=llm,
                verbose=True,
                allow_delegation=False
            )
        }
        
        logger.info("✅ All agents initialized successfully")
    
    async def execute_research_project(self, topic: str, content_type: str = "article", category: str = "general") -> str:
        """Execute a complete research and content creation project"""
        logger.info(f"🎯 Starting research project: {topic}")
        
        # Define tasks for the crew
        research_task = Task(
            description=f"""Conduct comprehensive research on '{topic}' in the {category} category.
            
            Your research should include:
            1. Find at least 5-10 current news articles on this topic
            2. Analyze trends and key insights
            3. Identify main themes and important developments
            4. Provide a detailed research report with sources
            
            Use the news_research tool to gather current information.""",
            agent=self.agents["researcher"]
        )
        
        writing_task = Task(
            description=f"""Create a professional {content_type} based on the research findings.
            
            Your content should:
            1. Be based on the research data provided
            2. Use appropriate professional formatting
            3. Include key insights and analysis
            4. Be engaging and informative
            5. Maintain journalistic standards
            
            Use the content_creation tool with template type '{content_type}'.""",
            agent=self.agents["writer"],
            dependencies=[research_task]
        )
        
        storage_task = Task(
            description=f"""Store all research data and created content for future reference.
            
            You should store:
            1. The research report with proper categorization
            2. The final content piece with metadata
            3. Ensure proper indexing and searchability
            
            Use the data_storage tool to organize and save all materials.""",
            agent=self.agents["data_manager"],
            dependencies=[research_task, writing_task]
        )
        
        analytics_task = Task(
            description=f"""Track and analyze the performance of this research project.
            
            You should track:
            1. Research quality and comprehensiveness
            2. Content creation metrics
            3. Overall project effectiveness
            4. Time and resource utilization
            
            Use the performance_tracking tool to record metrics.""",
            agent=self.agents["analyst"],
            dependencies=[research_task, writing_task, storage_task]
        )
        
        # Create and execute crew
        crew = Crew(
            agents=list(self.agents.values()),
            tasks=[research_task, writing_task, storage_task, analytics_task],
            process=Process.sequential,
            verbose=2
        )
        
        try:
            result = crew.kickoff()
            logger.info("✅ Research project completed successfully")
            return f"🎉 RESEARCH PROJECT COMPLETED\nTopic: {topic}\nResult: {result}"
            
        except Exception as e:
            logger.error(f"❌ Research project failed: {e}")
            return f"❌ Research project failed: {str(e)}"
    
    async def quick_news_summary(self, topic: str, count: int = 5) -> str:
        """Quick news summary task using just the researcher"""
        task = Task(
            description=f"""Provide a quick news summary on '{topic}'.
            
            Find the top {count} news articles and provide:
            1. Brief overview of each article
            2. Key trends and insights
            3. Summary of current situation
            
            Focus on recent, relevant news.""",
            agent=self.agents["researcher"]
        )
        
        crew = Crew(
            agents=[self.agents["researcher"]],
            tasks=[task],
            process=Process.sequential,
            verbose=1
        )
        
        try:
            result = crew.kickoff()
            return f"📰 NEWS SUMMARY: {topic}\n{result}"
        except Exception as e:
            return f"❌ News summary failed: {str(e)}"
    
    async def cleanup(self):
        """Cleanup resources"""
        logger.info("🧹 Cleaning up Research Content Crew...")
        for client in self.mcp_clients.values():
            await client.__aexit__(None, None, None)


async def main():
    """Example usage of the Research Content Crew"""
    import os
    
    # Check for OpenAI API key
    openai_api_key = os.getenv("OPENAI_API_KEY")
    if not openai_api_key:
        print("❌ Please set OPENAI_API_KEY environment variable")
        return
    
    # Create and initialize crew
    crew = ResearchContentCrew(openai_api_key)
    
    try:
        await crew.initialize()
        
        print("\n🤖 Research Content Crew Ready!")
        print("=" * 60)
        
        # Example 1: Quick news summary
        print("\n📰 Example 1: Quick News Summary")
        print("-" * 40)
        
        summary = await crew.quick_news_summary("artificial intelligence", 3)
        print(summary)
        
        print("\n⏳ Waiting 5 seconds before next example...")
        await asyncio.sleep(5)
        
        # Example 2: Full research project
        print("\n📊 Example 2: Full Research Project")
        print("-" * 40)
        
        project_result = await crew.execute_research_project(
            topic="renewable energy innovations",
            content_type="report",
            category="technology"
        )
        print(project_result)
        
        print("\n✅ All examples completed successfully!")
        
    except KeyboardInterrupt:
        print("\n👋 Shutting down...")
    except Exception as e:
        print(f"\n❌ Error: {e}")
    finally:
        await crew.cleanup()


if __name__ == "__main__":
    print("🚀 CrewAI + MCP Integration Example")
    print("This example requires CrewAI and all MCP servers to be running.")
    print("Install CrewAI with: pip install crewai")
    print("Make sure all MCP servers are running on their default ports.")
    print()
    
    asyncio.run(main())
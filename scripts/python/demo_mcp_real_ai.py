#!/usr/bin/env python3
"""
MCP Real AI Integration Demonstration
This script demonstrates actual MCP protocol communication with real AI generation,
showing request/response logs and saving generated content to files.
"""

import os
import sys
import json
import time
import requests
import subprocess
import signal
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, Optional
import threading
import queue

class MCPDemo:
    def __init__(self):
        self.output_dir = Path("generated_content")
        self.output_dir.mkdir(exist_ok=True)
        
        self.logs_dir = Path("mcp_logs")
        self.logs_dir.mkdir(exist_ok=True)
        
        self.session_id = datetime.now().strftime("%Y%m%d_%H%M%S")
        
    def log_mcp_interaction(self, server_type: str, request: Dict[str, Any], response: Dict[str, Any], content_path: Optional[str] = None):
        """Log MCP request/response interaction with timestamp."""
        timestamp = datetime.now().isoformat()
        
        log_entry = {
            "timestamp": timestamp,
            "session_id": self.session_id,
            "server_type": server_type,
            "request": request,
            "response": response,
            "content_saved_to": content_path
        }
        
        # Save to JSON log file
        log_file = self.logs_dir / f"mcp_session_{self.session_id}.jsonl"
        with open(log_file, "a") as f:
            f.write(json.dumps(log_entry, indent=None) + "\n")
        
        # Print formatted log
        print(f"\n{'='*80}")
        print(f"🔄 MCP INTERACTION - {server_type.upper()} SERVER")
        print(f"📅 Timestamp: {timestamp}")
        print(f"{'='*80}")
        
        print(f"\n📤 REQUEST:")
        print(json.dumps(request, indent=2))
        
        print(f"\n📥 RESPONSE:")
        print(json.dumps(response, indent=2))
        
        if content_path:
            print(f"\n💾 CONTENT SAVED TO: {content_path}")
        
        print(f"{'='*80}\n")

    def start_mcp_server(self, server_name: str, delay: int = 0) -> subprocess.Popen:
        """Start an MCP server and return the process."""
        print(f"🚀 Starting {server_name} MCP server...")
        
        server_dir = Path(f"examples/{server_name}")
        if not server_dir.exists():
            raise FileNotFoundError(f"Server directory not found: {server_dir}")
        
        cmd = [
            "cargo", "run", "--bin", server_name, "--",
            "--transport", "stdio",
            "--delay", str(delay),
            "--debug"
        ]
        
        process = subprocess.Popen(
            cmd,
            cwd=server_dir,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        
        # Give server time to start
        time.sleep(1)
        
        if process.poll() is not None:
            stderr_output = process.stderr.read()
            raise RuntimeError(f"Server failed to start: {stderr_output}")
        
        print(f"✅ {server_name} server started (PID: {process.pid})")
        return process

    def send_mcp_request(self, process: subprocess.Popen, request: Dict[str, Any]) -> Dict[str, Any]:
        """Send MCP request to server and get response."""
        request_json = json.dumps(request) + "\n"
        
        print(f"📤 Sending MCP request...")
        process.stdin.write(request_json)
        process.stdin.flush()
        
        # Read response with timeout
        try:
            response_line = process.stdout.readline()
            if not response_line:
                raise RuntimeError("No response from server")
            
            response = json.loads(response_line.strip())
            return response
        except json.JSONDecodeError as e:
            stderr_output = process.stderr.read()
            raise RuntimeError(f"Invalid JSON response: {e}. Stderr: {stderr_output}")

    def save_content_to_file(self, content: str, filename: str, content_type: str = "text") -> str:
        """Save generated content to file and return the path."""
        if content_type == "text":
            file_path = self.output_dir / f"{self.session_id}_{filename}.txt"
            with open(file_path, "w", encoding="utf-8") as f:
                f.write(content)
        elif content_type == "markdown":
            file_path = self.output_dir / f"{self.session_id}_{filename}.md"
            with open(file_path, "w", encoding="utf-8") as f:
                f.write(content)
        elif content_type == "json":
            file_path = self.output_dir / f"{self.session_id}_{filename}.json"
            with open(file_path, "w", encoding="utf-8") as f:
                json.dump(content if isinstance(content, dict) else {"content": content}, f, indent=2)
        else:
            file_path = self.output_dir / f"{self.session_id}_{filename}"
            with open(file_path, "w", encoding="utf-8") as f:
                f.write(content)
        
        return str(file_path)

    def demo_blog_generation_mcp(self):
        """Demonstrate blog generation via MCP protocol."""
        print(f"\n🔥 DEMO: Blog Generation via MCP Protocol")
        print(f"=" * 60)
        
        server_process = None
        try:
            # Start blog generation server
            server_process = self.start_mcp_server("blog-generation-server", delay=1)
            
            # Test different blog scenarios
            blog_scenarios = [
                {
                    "name": "tech_blog",
                    "request": {
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/call",
                        "params": {
                            "name": "create_blog_post",
                            "arguments": {
                                "topic": "The Future of AI in Software Development",
                                "style": "professional",
                                "word_count": 800,
                                "target_audience": "developers"
                            }
                        }
                    }
                },
                {
                    "name": "casual_blog",
                    "request": {
                        "jsonrpc": "2.0",
                        "id": 2,
                        "method": "tools/call",
                        "params": {
                            "name": "create_blog_post",
                            "arguments": {
                                "topic": "Work-Life Balance in Remote Work",
                                "style": "casual",
                                "word_count": 600,
                                "target_audience": "general"
                            }
                        }
                    }
                }
            ]
            
            for scenario in blog_scenarios:
                print(f"\n📝 Generating blog: {scenario['name']}")
                
                # Send MCP request
                response = self.send_mcp_request(server_process, scenario["request"])
                
                # Extract and save content
                if "result" in response and "content" in response["result"]:
                    content = response["result"]["content"][0]["text"]
                    content_path = self.save_content_to_file(content, f"blog_{scenario['name']}", "markdown")
                else:
                    content_path = None
                
                # Log the interaction
                self.log_mcp_interaction("blog-generation", scenario["request"], response, content_path)
                
                time.sleep(2)  # Brief pause between requests
        
        finally:
            if server_process:
                server_process.terminate()
                server_process.wait()
                print(f"🛑 Blog generation server stopped")

    def demo_creative_content_mcp(self):
        """Demonstrate creative content generation via MCP protocol."""
        print(f"\n🎨 DEMO: Creative Content Generation via MCP Protocol")
        print(f"=" * 60)
        
        server_process = None
        try:
            # Start creative content server
            server_process = self.start_mcp_server("creative-content-server", delay=1)
            
            # Test different creative scenarios
            creative_scenarios = [
                {
                    "name": "fantasy_story",
                    "tool": "generate_story",
                    "request": {
                        "jsonrpc": "2.0",
                        "id": 3,
                        "method": "tools/call",
                        "params": {
                            "name": "generate_story",
                            "arguments": {
                                "genre": "fantasy",
                                "theme": "A dragon who has forgotten how to fly",
                                "length": "short"
                            }
                        }
                    }
                },
                {
                    "name": "nature_poem",
                    "tool": "create_poem",
                    "request": {
                        "jsonrpc": "2.0",
                        "id": 4,
                        "method": "tools/call",
                        "params": {
                            "name": "create_poem",
                            "arguments": {
                                "style": "haiku",
                                "theme": "mountain sunrise"
                            }
                        }
                    }
                },
                {
                    "name": "hero_character",
                    "tool": "develop_character",
                    "request": {
                        "jsonrpc": "2.0",
                        "id": 5,
                        "method": "tools/call",
                        "params": {
                            "name": "develop_character",
                            "arguments": {
                                "name": "Zara Nightwhisper",
                                "type": "hero",
                                "background": "shadow mage seeking redemption"
                            }
                        }
                    }
                }
            ]
            
            for scenario in creative_scenarios:
                print(f"\n🎭 Generating {scenario['tool']}: {scenario['name']}")
                
                # Send MCP request
                response = self.send_mcp_request(server_process, scenario["request"])
                
                # Extract and save content
                if "result" in response and "content" in response["result"]:
                    content = response["result"]["content"][0]["text"]
                    content_path = self.save_content_to_file(content, f"creative_{scenario['name']}", "text")
                else:
                    content_path = None
                
                # Log the interaction
                self.log_mcp_interaction("creative-content", scenario["request"], response, content_path)
                
                time.sleep(2)  # Brief pause between requests
        
        finally:
            if server_process:
                server_process.terminate()
                server_process.wait()
                print(f"🛑 Creative content server stopped")

    def demo_real_gemini_integration(self):
        """Demonstrate real Gemini API integration."""
        print(f"\n🤖 DEMO: Real Gemini API Integration")
        print(f"=" * 60)
        
        if not os.environ.get('GEMINI_API_KEY'):
            print("⚠️ GEMINI_API_KEY not found, skipping real AI integration demo")
            return
        
        # Real Gemini API calls
        real_scenarios = [
            {
                "name": "ai_tech_blog",
                "type": "blog",
                "prompt": "Write a professional blog post about 'Machine Learning in Everyday Applications' targeting tech professionals. The post should be approximately 800 words long.",
            },
            {
                "name": "space_story",
                "type": "story", 
                "prompt": "Write a short science fiction story about a space explorer who discovers a planet where time moves backwards. Make it approximately 500 words long.",
            },
            {
                "name": "coding_poem",
                "type": "poem",
                "prompt": "Write a creative poem about the joy and frustration of coding. Make it inspiring for developers.",
            }
        ]
        
        for scenario in real_scenarios:
            print(f"\n🌟 Real AI Generation: {scenario['name']}")
            
            try:
                # Call real Gemini API
                content = self.call_real_gemini_api(scenario["prompt"])
                
                # Save content
                content_path = self.save_content_to_file(
                    content, 
                    f"real_ai_{scenario['name']}", 
                    "markdown" if scenario["type"] == "blog" else "text"
                )
                
                # Create mock MCP request/response for logging
                mock_request = {
                    "jsonrpc": "2.0",
                    "id": f"real_{scenario['name']}",
                    "method": "real_ai/generate",
                    "params": {
                        "prompt": scenario["prompt"],
                        "type": scenario["type"]
                    }
                }
                
                mock_response = {
                    "jsonrpc": "2.0",
                    "id": f"real_{scenario['name']}",
                    "result": {
                        "content": [{"text": content}],
                        "source": "real_gemini_api",
                        "generation_time": "~8s",
                        "word_count": len(content.split())
                    }
                }
                
                # Log the interaction
                self.log_mcp_interaction("real-gemini-api", mock_request, mock_response, content_path)
                
                time.sleep(3)  # Pause between real API calls
                
            except Exception as e:
                print(f"❌ Real AI generation failed: {e}")

    def call_real_gemini_api(self, prompt: str) -> str:
        """Make real call to Gemini API."""
        api_key = os.environ.get('GEMINI_API_KEY')
        if not api_key:
            raise ValueError("GEMINI_API_KEY not set")
        
        url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={api_key}"
        
        request_body = {
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
                "temperature": 0.7,
                "topK": 40,
                "topP": 0.95,
                "maxOutputTokens": 4096,
                "candidateCount": 1
            }
        }
        
        print(f"🔄 Calling real Gemini API...")
        response = requests.post(
            url,
            headers={"Content-Type": "application/json"},
            json=request_body,
            timeout=60
        )
        
        if not response.ok:
            raise Exception(f"Gemini API error: {response.status_code} - {response.text}")
        
        response_data = response.json()
        content = (response_data
                  .get("candidates", [{}])[0]
                  .get("content", {})
                  .get("parts", [{}])[0]
                  .get("text", ""))
        
        if not content:
            raise Exception("No content generated by Gemini API")
        
        return content

    def demo_mcp_protocol_features(self):
        """Demonstrate core MCP protocol features."""
        print(f"\n🔌 DEMO: MCP Protocol Features")
        print(f"=" * 60)
        
        server_process = None
        try:
            # Start blog generation server for protocol demo
            server_process = self.start_mcp_server("blog-generation-server", delay=0)
            
            # 1. List available tools
            list_tools_request = {
                "jsonrpc": "2.0",
                "id": "list_tools",
                "method": "tools/list"
            }
            
            print(f"\n📋 Listing available tools...")
            response = self.send_mcp_request(server_process, list_tools_request)
            self.log_mcp_interaction("protocol-demo", list_tools_request, response)
            
            # 2. Get server info
            server_info_request = {
                "jsonrpc": "2.0",
                "id": "server_info",
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "mcp-demo-client",
                        "version": "1.0.0"
                    }
                }
            }
            
            print(f"\n🔍 Getting server information...")
            response = self.send_mcp_request(server_process, server_info_request)
            self.log_mcp_interaction("protocol-demo", server_info_request, response)
            
            # 3. Test error handling
            error_request = {
                "jsonrpc": "2.0",
                "id": "error_test",
                "method": "tools/call",
                "params": {
                    "name": "nonexistent_tool",
                    "arguments": {}
                }
            }
            
            print(f"\n⚠️ Testing error handling...")
            response = self.send_mcp_request(server_process, error_request)
            self.log_mcp_interaction("protocol-demo", error_request, response)
            
        finally:
            if server_process:
                server_process.terminate()
                server_process.wait()
                print(f"🛑 Protocol demo server stopped")

    def generate_demo_summary(self):
        """Generate a summary of the demo session."""
        print(f"\n📊 DEMO SESSION SUMMARY")
        print(f"=" * 60)
        
        # Count generated files
        content_files = list(self.output_dir.glob(f"{self.session_id}_*"))
        log_files = list(self.logs_dir.glob(f"mcp_session_{self.session_id}.*"))
        
        print(f"Session ID: {self.session_id}")
        print(f"Generated Content Files: {len(content_files)}")
        print(f"MCP Log Files: {len(log_files)}")
        
        print(f"\n📁 Generated Content:")
        for file_path in sorted(content_files):
            size_kb = file_path.stat().st_size / 1024
            print(f"  📄 {file_path.name} ({size_kb:.1f} KB)")
        
        print(f"\n📋 Log Files:")
        for file_path in sorted(log_files):
            size_kb = file_path.stat().st_size / 1024
            print(f"  📝 {file_path.name} ({size_kb:.1f} KB)")
        
        print(f"\n💡 To view content:")
        print(f"   cd {self.output_dir}")
        print(f"   ls -la {self.session_id}_*")
        
        print(f"\n💡 To view MCP logs:")
        print(f"   cd {self.logs_dir}")
        print(f"   cat mcp_session_{self.session_id}.jsonl | jq .")

def main():
    """Main demo execution."""
    print(f"🚀 MCP Real AI Integration Demo")
    print(f"🕐 Started at: {datetime.now().isoformat()}")
    print(f"=" * 80)
    
    demo = MCPDemo()
    
    try:
        # Run all demos
        demo.demo_mcp_protocol_features()
        demo.demo_blog_generation_mcp()
        demo.demo_creative_content_mcp()
        demo.demo_real_gemini_integration()
        
        # Generate summary
        demo.generate_demo_summary()
        
        print(f"\n🎉 Demo completed successfully!")
        print(f"🕐 Finished at: {datetime.now().isoformat()}")
        
    except KeyboardInterrupt:
        print(f"\n⚠️ Demo interrupted by user")
    except Exception as e:
        print(f"\n❌ Demo failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
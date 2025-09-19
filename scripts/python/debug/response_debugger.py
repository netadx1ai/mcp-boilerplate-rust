#!/usr/bin/env python3
"""
Debug script to examine the actual MCP image generation response structure
"""

import json
import subprocess
import sys
import os
import time
import requests
from typing import Dict, Any, Optional

class MCPImageDebugger:
    """Debug client to examine MCP image generation responses"""
    
    def __init__(self, server_host="127.0.0.1", server_port=3001):
        self.base_url = f"http://{server_host}:{server_port}"
        self.server_process = None
        
    def start_server(self, use_ai=True, provider="gemini", delay=0):
        """Start the MCP image generation server"""
        print("🚀 Starting MCP Image Generation Server...")
        
        # Check if API key is available for AI mode
        api_key = os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")
        if use_ai and not api_key:
            print("⚠️  No API key found, falling back to mock mode")
            use_ai = False
        
        cmd = [
            "./target/debug/image-generation-server",
            "--transport", "http",
            "--port", str(self.base_url.split(":")[-1]),
            "--delay", str(delay)
        ]
        
        if use_ai:
            cmd.extend(["--use-ai", "--provider", provider])
            print(f"✅ Using AI mode with {provider}")
        else:
            print("🎭 Using mock mode")
        
        try:
            self.server_process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                cwd="/Volumes/T72/Work2025AI/MCP-Genertic/mcp-boilerplate-rust"
            )
            
            # Wait for server to start
            print("⏳ Waiting for server startup...")
            time.sleep(3)
            
            # Check if server is running
            if self.server_process.poll() is not None:
                stdout, stderr = self.server_process.communicate()
                print("❌ Server failed to start:")
                print("STDERR:", stderr.decode())
                return False
            
            # Test server health
            try:
                response = requests.get(f"{self.base_url}/health", timeout=5)
                if response.status_code == 200:
                    print("✅ Server is healthy and ready!")
                    return True
            except:
                pass
            
            print("✅ Server started (health check skipped)")
            return True
            
        except Exception as e:
            print(f"❌ Failed to start server: {e}")
            return False
    
    def stop_server(self):
        """Stop the MCP server"""
        if self.server_process and self.server_process.poll() is None:
            print("🛑 Stopping server...")
            self.server_process.terminate()
            try:
                self.server_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.server_process.kill()
            print("✅ Server stopped")
    
    def send_mcp_request(self, method: str, params: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Send a request using MCP protocol via HTTP"""
        
        url = f"{self.base_url}/mcp/tools/call"
        request_data = params
            
        try:
            print(f"📡 Sending request to: {url}")
            print(f"📦 Request data: {json.dumps(request_data, indent=2)}")
            
            response = requests.post(
                url,
                json=request_data,
                headers={
                    "Content-Type": "application/json",
                    "Accept": "application/json"
                },
                timeout=30
            )
            
            print(f"📊 Response status: {response.status_code}")
            print(f"📋 Response headers: {dict(response.headers)}")
            
            if response.status_code == 200:
                print(f"✅ Success!")
                return response.json()
            else:
                print(f"❌ HTTP {response.status_code}: {response.text[:500]}")
                return None
                
        except requests.exceptions.RequestException as e:
            print(f"❌ Request failed: {e}")
            return None
    
    def debug_response_structure(self, response: Dict[str, Any], depth=0, max_depth=5):
        """Recursively debug the response structure"""
        indent = "  " * depth
        
        if depth > max_depth:
            print(f"{indent}... (max depth reached)")
            return
            
        if isinstance(response, dict):
            print(f"{indent}Dict with {len(response)} keys:")
            for key, value in response.items():
                print(f"{indent}  '{key}': {type(value).__name__}", end="")
                
                if isinstance(value, str):
                    preview = value[:50] + "..." if len(value) > 50 else value
                    print(f" = '{preview}'")
                    
                    # Check if it might be base64 data
                    if len(value) > 100 and all(c in 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=' for c in value):
                        print(f"{indent}    ^ Possible base64 data (length: {len(value)})")
                        
                elif isinstance(value, (int, float, bool)):
                    print(f" = {value}")
                elif isinstance(value, list):
                    print(f" = List with {len(value)} items")
                    if value and depth < max_depth:
                        print(f"{indent}    First item:")
                        self.debug_response_structure(value[0], depth + 2, max_depth)
                else:
                    print()
                    
                if isinstance(value, dict) and depth < max_depth:
                    self.debug_response_structure(value, depth + 1, max_depth)
                    
        elif isinstance(response, list):
            print(f"{indent}List with {len(response)} items")
            for i, item in enumerate(response[:3]):  # Show first 3 items
                print(f"{indent}  [{i}]: {type(item).__name__}")
                if isinstance(item, dict) and depth < max_depth:
                    self.debug_response_structure(item, depth + 1, max_depth)
        else:
            print(f"{indent}{type(response).__name__}: {str(response)[:100]}")
    
    def generate_and_debug(self, prompt: str, style: str = "photorealistic", size: str = "1024x1024"):
        """Generate an image and debug the response"""
        
        print(f"🎨 Generating image for debugging...")
        print(f"📝 Prompt: {prompt}")
        print(f"🎭 Style: {style}")
        print(f"📏 Size: {size}")
        print("-" * 60)
        
        # Create parameters for the tools/call method
        params = {
            "name": "generate_image",
            "arguments": {
                "prompt": prompt,
                "style": style,
                "size": size
            }
        }
        
        # Send the request
        response = self.send_mcp_request("tools/call", params)
        
        if response:
            print("\n🔍 RESPONSE STRUCTURE DEBUG")
            print("=" * 60)
            self.debug_response_structure(response)
            
            print("\n📋 RAW RESPONSE JSON")
            print("=" * 60)
            print(json.dumps(response, indent=2)[:2000] + "..." if len(json.dumps(response)) > 2000 else json.dumps(response, indent=2))
            
            return response
        else:
            print("❌ No response to debug")
            return None

def main():
    """Main function to debug image generation responses"""
    print("🔍 MCP Image Generation Response Debugger")
    print("=" * 60)
    
    # Get prompt from command line or use default
    if len(sys.argv) > 1:
        prompt = " ".join(sys.argv[1:])
    else:
        prompt = "A simple red circle on white background"
        print(f"Using simple test prompt: {prompt}")
    
    # Check build status
    print("\n🔍 Checking build status...")
    build_result = subprocess.run(
        ["cargo", "build", "--bin", "image-generation-server"],
        cwd="/Volumes/T72/Work2025AI/MCP-Genertic/mcp-boilerplate-rust",
        capture_output=True
    )
    
    if build_result.returncode != 0:
        print("❌ Build failed! Please run: cargo build --bin image-generation-server")
        sys.exit(1)
    
    print("✅ Build successful!")
    
    # Create debugger and start server
    debugger = MCPImageDebugger()
    
    try:
        # Start server with AI mode if API key available
        api_key_available = bool(os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY"))
        
        if not debugger.start_server(use_ai=api_key_available):
            print("❌ Failed to start server")
            sys.exit(1)
        
        # Generate image and debug response
        result = debugger.generate_and_debug(
            prompt=prompt,
            style="photorealistic", 
            size="1024x1024"
        )
        
        print(f"\n🎯 DEBUGGING SUMMARY")
        print("=" * 60)
        
        if result:
            # Look for image data patterns
            json_str = json.dumps(result)
            
            print(f"📊 Response contains {len(json_str)} characters")
            
            # Check for common image data indicators
            indicators = {
                "base64": "base64" in json_str.lower(),
                "image_data": "image_data" in json_str.lower(),
                "data:image": "data:image" in json_str.lower(),
                "png": "png" in json_str.lower(),
                "jpeg": "jpeg" in json_str.lower(),
                "url": "url" in json_str.lower(),
                "long_strings": any(len(str(v)) > 1000 for v in str(result).split())
            }
            
            print(f"🔍 Image data indicators found:")
            for indicator, found in indicators.items():
                status = "✅" if found else "❌"
                print(f"   {status} {indicator}")
            
        else:
            print("❌ No response received")
        
    except KeyboardInterrupt:
        print("\n⚠️  Interrupted by user")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # Always stop the server
        debugger.stop_server()

if __name__ == "__main__":
    main()
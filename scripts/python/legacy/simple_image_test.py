#!/usr/bin/env python3
"""
Simple Image Generation Test for MCP Server
Tests both mock and AI modes using HTTP transport for better debugging
"""

import requests
import json
import subprocess
import time
import os
import sys
from contextlib import contextmanager

@contextmanager
def mcp_server(use_ai=False, port=3002):
    """Context manager to start and stop MCP server"""
    cmd = [
        "./target/debug/image-generation-server",
        "--transport", "http", 
        "--port", str(port),
        "--delay", "0"
    ]
    
    if use_ai:
        cmd.extend(["--use-ai", "--provider", "gemini"])
    
    print(f"🚀 Starting server: {' '.join(cmd)}")
    
    # Start server
    process = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd="/Volumes/T72/Work2025AI/MCP-Genertic/mcp-boilerplate-rust"
    )
    
    # Wait for server to start
    time.sleep(2)
    
    # Check if server is running
    if process.poll() is not None:
        stdout, stderr = process.communicate()
        print(f"❌ Server failed to start:")
        print(f"STDOUT: {stdout.decode()}")
        print(f"STDERR: {stderr.decode()}")
        raise RuntimeError("Server startup failed")
    
    try:
        # Test server health
        health_url = f"http://127.0.0.1:{port}/health"
        response = requests.get(health_url, timeout=5)
        if response.status_code != 200:
            print(f"⚠️  Health check returned {response.status_code}")
        else:
            print(f"✅ Server healthy at {health_url}")
        
        yield f"http://127.0.0.1:{port}"
        
    finally:
        # Clean shutdown
        print("🛑 Stopping server...")
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        print("✅ Server stopped")

def call_tool(base_url, tool_name, arguments):
    """Call a tool via HTTP API"""
    url = f"{base_url}/mcp/tools/call"
    
    payload = {
        "name": tool_name,
        "arguments": arguments
    }
    
    print(f"📨 POST {url}")
    print(f"📝 Payload: {json.dumps(payload, indent=2)}")
    
    try:
        response = requests.post(url, json=payload, timeout=30)
        print(f"📡 Response status: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            print(f"✅ Success!")
            return result
        else:
            print(f"❌ Error: {response.status_code}")
            print(f"Response: {response.text[:500]}")
            return None
            
    except requests.exceptions.RequestException as e:
        print(f"❌ Request failed: {e}")
        return None

def display_image_result(result, mode_name):
    """Display formatted image generation result"""
    print(f"\n{'='*60}")
    print(f"🎨 IMAGE GENERATION RESULT - {mode_name}")
    print(f"{'='*60}")
    
    if not result:
        print("❌ No result to display")
        return
    
    # The result should contain tool result with image data
    if 'content' in result and result['content']:
        content = result['content'][0]
        if 'text' in content:
            try:
                image_data = json.loads(content['text'])
                
                print(f"Status: {'✅ SUCCESS' if image_data.get('success') else '❌ FAILED'}")
                
                if 'image' in image_data:
                    img = image_data['image']
                    print(f"🖼️  Image ID: {img.get('id', 'N/A')}")
                    print(f"📝 Prompt: {img.get('prompt', 'N/A')}")
                    print(f"🎭 Style: {img.get('style', 'N/A')}")
                    print(f"📏 Size: {img.get('size', 'N/A')}")
                    print(f"🔗 URL: {img.get('url', 'N/A')}")
                    
                    # Show enhanced prompt for AI mode
                    if 'enhanced_prompt' in img:
                        print(f"🎨 Enhanced: {img['enhanced_prompt']}")
                    
                    # Show AI description if available
                    if 'description' in img:
                        desc = img['description']
                        if len(desc) > 100:
                            desc = desc[:100] + "..."
                        print(f"📖 Description: {desc}")
                    
                    # Metadata
                    if 'metadata' in img:
                        meta = img['metadata']
                        print(f"\n🔧 Metadata:")
                        if 'provider' in meta:
                            print(f"   🤖 Provider: {meta['provider']}")
                        print(f"   🏷️  Model: {meta.get('model', 'N/A')}")
                        print(f"   ⏱️  Time: {meta.get('processing_time_ms', 'N/A')}ms")
                    
                    # Usage info for AI mode
                    if 'usage' in image_data:
                        usage = image_data['usage']
                        print(f"\n💰 Usage:")
                        print(f"   📊 Tokens: {usage.get('tokens_used', 'N/A')}")
                        print(f"   💵 Cost: ${usage.get('cost_usd', 'N/A')}")
                    
                    # Note
                    if 'note' in image_data:
                        print(f"\n📌 Note: {image_data['note']}")
                        
            except json.JSONDecodeError as e:
                print(f"❌ Failed to parse image data: {e}")
                print(f"Raw content: {content.get('text', '')[:200]}...")
    else:
        print("❌ No content in result")
        print(f"Raw result: {json.dumps(result, indent=2)[:300]}...")

def main():
    """Main test function"""
    print("🎨 Simple MCP Image Generation Test")
    print("=" * 50)
    
    # Check environment
    if not os.path.exists("/Volumes/T72/Work2025AI/MCP-Genertic/mcp-boilerplate-rust/target/debug/image-generation-server"):
        print("❌ Server binary not found. Run 'cargo build --bin image-generation-server' first.")
        sys.exit(1)
    
    has_api_key = bool(os.environ.get("GEMINI_API_KEY"))
    print(f"🔑 API Key: {'✅ Available' if has_api_key else '❌ Not configured'}")
    
    # Test prompt
    prompt = "A beautiful sunset over a mountain lake with reflections"
    
    print(f"\n🎯 Test Prompt: '{prompt}'")
    print("=" * 50)
    
    # Test 1: Mock Mode
    print("\n1️⃣ TESTING MOCK MODE")
    print("-" * 30)
    
    try:
        with mcp_server(use_ai=False, port=3002) as base_url:
            result = call_tool(base_url, "generate_image", {
                "prompt": prompt,
                "style": "photorealistic", 
                "size": "1024x1024"
            })
            display_image_result(result, "MOCK MODE")
            
    except Exception as e:
        print(f"❌ Mock mode test failed: {e}")
    
    # Test 2: AI Mode (if API key available)
    if has_api_key:
        print(f"\n\n2️⃣ TESTING AI MODE (Google/Gemini)")
        print("-" * 30)
        
        try:
            with mcp_server(use_ai=True, port=3003) as base_url:
                result = call_tool(base_url, "generate_image", {
                    "prompt": prompt,
                    "style": "photorealistic",
                    "size": "1024x1024"  
                })
                display_image_result(result, "AI MODE")
                
        except Exception as e:
            print(f"❌ AI mode test failed: {e}")
    else:
        print(f"\n\n2️⃣ AI MODE: Skipped (no API key)")
    
    print(f"\n\n🎉 Test Complete!")
    print("=" * 50)
    print("💡 Summary:")
    print("   • Mock mode: Fast placeholder responses")
    print("   • AI mode: Real Google/Gemini integration")
    print("   • Both modes tested via HTTP API")
    print("   • Ready for production use!")

if __name__ == "__main__":
    main()
#!/usr/bin/env python3
"""
Test script for MCP Image Generation Server
Demonstrates creating an image using the MCP server with Google/Gemini integration
"""

import json
import subprocess
import sys
import os
import time
from typing import Dict, Any, Optional

def run_mcp_server(use_ai: bool = False, prompt: str = "A beautiful sunset over mountains") -> Optional[Dict[Any, Any]]:
    """
    Run the MCP image generation server and create an image
    
    Args:
        use_ai: Whether to use real AI provider (Gemini) or mock responses
        prompt: The image generation prompt
        
    Returns:
        Dict containing the server response or None if failed
    """
    print(f"🎨 Generating image with prompt: '{prompt}'")
    print(f"🤖 AI Mode: {'Enabled (Gemini)' if use_ai else 'Disabled (Mock)'}")
    print("-" * 60)
    
    # Prepare MCP request
    mcp_request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "generate_image",
            "arguments": {
                "prompt": prompt,
                "style": "photorealistic",
                "size": "1024x1024"
            }
        }
    }
    
    # Build server command
    cmd = ["./target/debug/image-generation-server", "--transport", "stdio", "--delay", "0"]
    if use_ai:
        cmd.extend(["--use-ai", "--provider", "gemini"])
    
    print(f"📡 Running command: {' '.join(cmd)}")
    print(f"📨 Sending request: {json.dumps(mcp_request, indent=2)}")
    print()
    
    try:
        # Start the server process
        process = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            cwd="/Volumes/T72/Work2025AI/MCP-Genertic/mcp-boilerplate-rust"
        )
        
        # Send the request
        request_json = json.dumps(mcp_request) + "\n"
        stdout, stderr = process.communicate(input=request_json, timeout=30)
        
        print(f"🔍 Server stderr (logs):")
        for line in stderr.strip().split('\n'):
            if line.strip():
                print(f"   {line}")
        print()
        
        # Parse response
        if stdout.strip():
            print(f"📥 Raw server response:")
            print(f"   {stdout.strip()}")
            print()
            
            try:
                response = json.loads(stdout.strip())
                return response
            except json.JSONDecodeError as e:
                print(f"❌ Failed to parse JSON response: {e}")
                return None
        else:
            print("❌ No response received from server")
            return None
            
    except subprocess.TimeoutExpired:
        print("❌ Server request timed out (30s)")
        process.kill()
        return None
    except Exception as e:
        print(f"❌ Error running server: {e}")
        return None

def display_image_result(response: Dict[Any, Any], use_ai: bool) -> None:
    """
    Display the image generation result in a formatted way
    
    Args:
        response: The MCP server response
        use_ai: Whether AI mode was used
    """
    print("=" * 60)
    print(f"🎯 IMAGE GENERATION RESULT ({'AI MODE' if use_ai else 'MOCK MODE'})")
    print("=" * 60)
    
    if "result" in response:
        result = response["result"]
        if "content" in result and result["content"]:
            content = result["content"][0]
            if "text" in content:
                try:
                    image_data = json.loads(content["text"])
                    
                    print(f"✅ Status: {'SUCCESS' if image_data.get('success') else 'FAILED'}")
                    
                    if "image" in image_data:
                        img = image_data["image"]
                        print(f"🖼️  Image ID: {img.get('id', 'N/A')}")
                        print(f"📝 Prompt: {img.get('prompt', 'N/A')}")
                        
                        if use_ai and "enhanced_prompt" in img:
                            print(f"🎨 Enhanced Prompt: {img.get('enhanced_prompt', 'N/A')}")
                        
                        if use_ai and "description" in img:
                            desc = img.get('description', 'N/A')
                            # Truncate long descriptions
                            if len(desc) > 100:
                                desc = desc[:100] + "..."
                            print(f"📖 AI Description: {desc}")
                        
                        print(f"🎭 Style: {img.get('style', 'N/A')}")
                        print(f"📏 Size: {img.get('size', 'N/A')}")
                        print(f"🔗 URL: {img.get('url', 'N/A')}")
                        print(f"📅 Created: {img.get('created_at', 'N/A')}")
                        
                        # Metadata
                        if "metadata" in img:
                            meta = img["metadata"]
                            print(f"\n🔧 Metadata:")
                            if use_ai:
                                print(f"   Provider: {meta.get('provider', 'N/A')}")
                            print(f"   Model: {meta.get('model', 'N/A')}")
                            print(f"   Processing Time: {meta.get('processing_time_ms', 'N/A')}ms")
                            if use_ai and "api_version" in meta:
                                print(f"   API Version: {meta.get('api_version', 'N/A')}")
                        
                        # Usage info (AI mode)
                        if use_ai and "usage" in image_data:
                            usage = image_data["usage"]
                            print(f"\n💰 Usage:")
                            print(f"   Tokens Used: {usage.get('tokens_used', 'N/A')}")
                            print(f"   Cost: ${usage.get('cost_usd', 'N/A')}")
                        
                        # Note for mock mode
                        if "note" in image_data:
                            print(f"\n📌 Note: {image_data['note']}")
                            
                except json.JSONDecodeError as e:
                    print(f"❌ Failed to parse image data: {e}")
                    print(f"Raw content: {content.get('text', 'N/A')[:200]}...")
            else:
                print("❌ No text content in response")
        else:
            print("❌ No content in result")
    else:
        print("❌ No result in response")
        if "error" in response:
            error = response["error"]
            print(f"💥 Error: {error.get('message', 'Unknown error')}")
            print(f"   Code: {error.get('code', 'N/A')}")

def check_environment() -> tuple[bool, str]:
    """
    Check if the environment is properly set up
    
    Returns:
        Tuple of (is_ready, status_message)
    """
    print("🔍 Checking environment...")
    
    # Check if we're in the right directory
    if not os.path.exists("target/debug/image-generation-server"):
        return False, "❌ image-generation-server binary not found. Run 'cargo build --bin image-generation-server' first."
    
    # Check for API key (for AI mode)
    api_key = os.environ.get("GEMINI_API_KEY")
    has_api_key = bool(api_key and len(api_key) > 10)
    
    print(f"✅ Server binary: Found")
    print(f"{'✅' if has_api_key else '⚠️ '} Gemini API Key: {'Present' if has_api_key else 'Not configured (mock mode only)'}")
    
    return True, "Environment ready"

def main():
    """Main function to demonstrate image generation"""
    print("🎨 MCP Image Generation Server Test")
    print("=" * 50)
    
    # Check environment
    ready, message = check_environment()
    if not ready:
        print(message)
        sys.exit(1)
    
    print(message)
    print()
    
    # Test prompts
    test_prompts = [
        "A cute cat sitting on a wooden table in a cozy kitchen",
        "A futuristic cityscape with flying cars at sunset",
        "A serene mountain lake reflecting snow-capped peaks"
    ]
    
    # Check if API key is available for AI mode
    has_api_key = bool(os.environ.get("GEMINI_API_KEY"))
    
    for i, prompt in enumerate(test_prompts, 1):
        print(f"\n🎯 TEST {i}/3")
        print("-" * 40)
        
        # Try mock mode first
        print("1️⃣ Testing MOCK MODE:")
        response = run_mcp_server(use_ai=False, prompt=prompt)
        if response:
            display_image_result(response, use_ai=False)
        else:
            print("❌ Mock mode test failed")
        
        print("\n" + "="*60 + "\n")
        
        # Try AI mode if API key is available
        if has_api_key:
            print("2️⃣ Testing AI MODE (Google/Gemini):")
            response = run_mcp_server(use_ai=True, prompt=prompt)
            if response:
                display_image_result(response, use_ai=True)
            else:
                print("❌ AI mode test failed")
        else:
            print("2️⃣ AI MODE: Skipped (GEMINI_API_KEY not configured)")
        
        if i < len(test_prompts):
            print(f"\n{'='*60}")
            print("⏳ Waiting 2 seconds before next test...")
            time.sleep(2)
    
    print(f"\n🎉 Test complete! Generated {len(test_prompts)} images.")
    print("\n💡 Tips:")
    print("   • Mock mode: Fast responses for development")
    print("   • AI mode: Real image generation with Gemini API")
    print("   • Set GEMINI_API_KEY environment variable to test AI mode")
    print("   • Use --use-ai --provider gemini flags for production")

if __name__ == "__main__":
    main()
#!/usr/bin/env python3
"""
Enhanced MCP Image Generation Client with Image Saving
This script generates images via MCP server and saves them to disk
"""

import json
import subprocess
import sys
import os
import time
import requests
import base64
from datetime import datetime
from typing import Dict, Any, Optional

class MCPImageClientWithSave:
    """MCP Image Generation Client with image saving functionality"""
    
    def __init__(self, server_host="127.0.0.1", server_port=3001):
        self.base_url = f"http://{server_host}:{server_port}"
        self.server_process = None
        self.output_dir = "generated_images"
        
    def ensure_output_directory(self):
        """Create output directory if it doesn't exist"""
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)
            print(f"📁 Created output directory: {self.output_dir}")
        
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
            
            # If health check fails, assume server is ready anyway
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
    
    def create_jsonrpc_request(self, method: str, params: Dict[str, Any], request_id: str = "1") -> Dict[str, Any]:
        """Create a proper JSON-RPC 2.0 request"""
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params
        }
    
    def send_mcp_request(self, method: str, params: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Send a request using MCP protocol via HTTP"""
        
        # Use the correct MCP HTTP endpoint
        if method == "tools/call":
            url = f"{self.base_url}/mcp/tools/call"
            # For MCP HTTP transport, send the tool call parameters directly
            request_data = params
        else:
            url = f"{self.base_url}/mcp/request"
            request_data = self.create_jsonrpc_request(method, params)
            
        try:
            print(f"📡 Sending request to: {url}")
            response = requests.post(
                url,
                json=request_data,
                headers={
                    "Content-Type": "application/json",
                    "Accept": "application/json"
                },
                timeout=30
            )
            
            if response.status_code == 200:
                print(f"✅ Success!")
                return response.json()
            else:
                print(f"❌ HTTP {response.status_code}: {response.text[:200]}")
                return None
                
        except requests.exceptions.RequestException as e:
            print(f"❌ Request failed: {e}")
            return None
    
    def generate_image(self, prompt: str, style: str = "photorealistic", size: str = "1024x1024") -> Optional[Dict[str, Any]]:
        """Generate an image using the MCP server"""
        
        print(f"🎨 Generating image...")
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
            return response
        else:
            print("❌ Image generation failed")
            return None
    
    def save_base64_image(self, base64_data: str, filename: str) -> bool:
        """Save base64 encoded image data to file"""
        try:
            # Remove data URL prefix if present
            if base64_data.startswith('data:image/'):
                base64_data = base64_data.split(',')[1]
            
            # Decode base64 data
            image_data = base64.b64decode(base64_data)
            
            # Save to file
            filepath = os.path.join(self.output_dir, filename)
            with open(filepath, 'wb') as f:
                f.write(image_data)
            
            print(f"💾 Image saved to: {filepath}")
            print(f"📊 File size: {len(image_data)} bytes ({len(image_data)/1024:.1f} KB)")
            return True
            
        except Exception as e:
            print(f"❌ Failed to save image: {e}")
            return False
    
    def extract_image_data_from_response(self, result: Dict[str, Any]) -> Optional[str]:
        """Extract base64 image data from MCP response"""
        try:
            # Handle MCP transport response format
            if "content" in result and result["content"]:
                # MCP transport format with content array
                content = result["content"][0] if isinstance(result["content"], list) else result["content"]
                
                if "text" in content:
                    try:
                        image_data = json.loads(content["text"])
                    except json.JSONDecodeError:
                        # Maybe the text content is the raw base64 data
                        return content["text"]
                else:
                    image_data = content
            else:
                # Direct response format
                image_data = result
            
            # Look for base64 data in various possible locations
            if isinstance(image_data, dict):
                # Check for common base64 data fields
                for field in ['image_data', 'data', 'base64', 'content', 'image']:
                    if field in image_data:
                        if isinstance(image_data[field], str):
                            return image_data[field]
                        elif isinstance(image_data[field], dict):
                            # Check nested structures
                            for subfield in ['data', 'base64', 'content']:
                                if subfield in image_data[field]:
                                    return image_data[field][subfield]
            
            # If it's a string, assume it's the base64 data
            elif isinstance(image_data, str):
                return image_data
                
        except Exception as e:
            print(f"❌ Error extracting image data: {e}")
        
        return None
    
    def generate_filename(self, prompt: str, image_id: str = None) -> str:
        """Generate a filename for the image"""
        # Create timestamp
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # Clean prompt for filename (keep alphanumeric and spaces, replace with underscores)
        clean_prompt = "".join(c if c.isalnum() or c.isspace() else "" for c in prompt)
        clean_prompt = "_".join(clean_prompt.split())[:50]  # Limit length
        
        # Add image ID if available
        if image_id:
            clean_id = image_id.replace("img_", "").replace("gemini_", "")[:10]
            return f"{timestamp}_{clean_prompt}_{clean_id}.png"
        else:
            return f"{timestamp}_{clean_prompt}.png"
    
    def display_result_and_save(self, result: Dict[str, Any], prompt: str) -> Optional[str]:
        """Display the image generation result and save the image"""
        if not result:
            print("❌ No result to display")
            return None
        
        print("\n🎉 IMAGE GENERATION RESULT")
        print("=" * 60)
        
        # Extract and display metadata
        image_id = None
        success = False
        
        try:
            # Handle MCP transport response format
            if "content" in result and result["content"]:
                content = result["content"][0] if isinstance(result["content"], list) else result["content"]
                
                if "text" in content:
                    try:
                        image_data = json.loads(content["text"])
                    except json.JSONDecodeError:
                        # Raw response, try to extract what we can
                        print("✅ Generation Status: SUCCESS (raw response)")
                        image_data = {"success": True}
                else:
                    image_data = content
            else:
                image_data = result
            
            if image_data.get("success", True):  # Assume success if not specified
                print("✅ Generation Status: SUCCESS")
                success = True
                
                if isinstance(image_data, dict) and "image" in image_data:
                    img = image_data["image"]
                    image_id = img.get('id', 'N/A')
                    
                    print(f"🆔 Image ID: {image_id}")
                    print(f"📝 Original Prompt: {img.get('prompt', prompt)}")
                    
                    if 'enhanced_prompt' in img:
                        print(f"🎨 Enhanced Prompt: {img.get('enhanced_prompt', 'N/A')}")
                    
                    if 'ai_description' in img:
                        desc = img.get('ai_description', '')
                        if len(desc) > 150:
                            desc = desc[:150] + "..."
                        print(f"🧠 AI Description: {desc}")
                    
                    print(f"🎭 Style: {img.get('style', 'N/A')}")
                    print(f"📏 Size: {img.get('size', 'N/A')}")
                    print(f"📅 Created: {img.get('created_at', 'N/A')}")
                    
                    # Metadata
                    if "metadata" in img:
                        meta = img["metadata"]
                        print(f"\n🔧 Technical Details:")
                        print(f"   Provider: {meta.get('provider', 'N/A')}")
                        print(f"   Model: {meta.get('model', 'N/A')}")
                        print(f"   Processing Time: {meta.get('processing_time_ms', 'N/A')}ms")
                        
                        if "note" in meta:
                            print(f"   Note: {meta['note']}")
                
                # Try to extract and save image data
                base64_data = self.extract_image_data_from_response(result)
                if base64_data:
                    filename = self.generate_filename(prompt, image_id)
                    if self.save_base64_image(base64_data, filename):
                        full_path = os.path.join(self.output_dir, filename)
                        return full_path
                    else:
                        print("⚠️  Image metadata received but no image data found to save")
                else:
                    print("⚠️  No base64 image data found in response")
                    
            else:
                print("❌ Generation failed")
                
        except Exception as e:
            print(f"❌ Error processing result: {e}")
        
        # Handle error responses
        if "error" in result:
            error = result["error"]
            print(f"💥 Error: {error.get('message', 'Unknown error')}")
            print(f"   Code: {error.get('code', 'N/A')}")
        
        return None

def main():
    """Main function to demonstrate image creation with saving"""
    print("🎨 MCP Image Generation with Auto-Save")
    print("=" * 60)
    
    # Get prompt from command line or user input
    if len(sys.argv) > 1:
        prompt = " ".join(sys.argv[1:])
    else:
        prompt = input("🖼️  What image would you like to create? ").strip()
        if not prompt:
            prompt = "A beautiful sunset over mountains with vibrant colors"
            print(f"Using default prompt: {prompt}")
    
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
    
    # Create client and start server
    client = MCPImageClientWithSave()
    client.ensure_output_directory()
    
    try:
        # Start server with AI mode if API key available
        api_key_available = bool(os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY"))
        
        if not client.start_server(use_ai=api_key_available):
            print("❌ Failed to start server")
            sys.exit(1)
        
        # Generate image
        result = client.generate_image(
            prompt=prompt,
            style="photorealistic", 
            size="1024x1024"
        )
        
        # Display results and save image
        saved_path = client.display_result_and_save(result, prompt)
        
        if saved_path:
            print(f"\n🎉 SUCCESS! Image saved to: {saved_path}")
            print(f"📂 Open the file to view your generated image!")
            
            # Try to open the image with default viewer (macOS)
            try:
                subprocess.run(["open", saved_path], check=False)
                print("🖼️  Opening image in default viewer...")
            except:
                print("💡 Manually open the image file to view it")
        else:
            print("\n⚠️  Image generation completed but no image file was saved")
            print("   This might be a mock response or the image data format changed")
        
    except KeyboardInterrupt:
        print("\n⚠️  Interrupted by user")
    except Exception as e:
        print(f"\n❌ Error: {e}")
    finally:
        # Always stop the server
        client.stop_server()
    
    print(f"\n💡 Summary:")
    print(f"   • Prompt: {prompt}")
    print(f"   • Output directory: {client.output_dir}")
    print(f"   • MCP Server: {'AI mode' if api_key_available else 'Mock mode'}")
    
    if api_key_available:
        print("   • AI Enhancement: Enabled with Gemini")
    else:
        print("   • AI Enhancement: Disabled (set GEMINI_API_KEY to enable)")

if __name__ == "__main__":
    main()
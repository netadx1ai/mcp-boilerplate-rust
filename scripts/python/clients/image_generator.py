#!/usr/bin/env python3
"""
Working MCP Image Generation Client with Proper Image Saving
This script correctly extracts and saves base64 image data from MCP responses
"""

import json
import subprocess
import sys
import os
import time
import requests
import base64
import re
from datetime import datetime
from typing import Dict, Any, Optional
from pathlib import Path

class WorkingMCPImageClient:
    """MCP Image Generation Client that properly saves images"""
    
    def __init__(self, server_host="127.0.0.1", server_port=3001):
        self.base_url = f"http://{server_host}:{server_port}"
        self.server_process = None
        
        # Find project root (contains Cargo.toml)
        self.project_root = self._find_project_root()
        self.output_dir = os.path.join(self.project_root, "generated_images")
    
    def _find_project_root(self) -> str:
        """Find the project root directory (contains Cargo.toml)"""
        current = Path(__file__).resolve()
        for parent in [current] + list(current.parents):
            if (parent / "Cargo.toml").exists():
                return str(parent)
        # Fallback to current directory
        return os.getcwd()
        
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
                cwd=self.project_root
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
    
    def extract_image_urls(self, json_text: str) -> list:
        """Extract all data URLs (base64 images) from JSON text"""
        # Look for data:image URLs in the JSON
        data_url_pattern = r'data:image/[^;]+;base64,[A-Za-z0-9+/=]+'
        matches = re.findall(data_url_pattern, json_text)
        return matches
    
    def save_data_url_image(self, data_url: str, filename: str) -> bool:
        """Save a data URL image to file"""
        try:
            # Extract the base64 part after the comma
            if ',' in data_url:
                base64_data = data_url.split(',')[1]
            else:
                base64_data = data_url
            
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
    
    def generate_filename(self, prompt: str, image_id: str = None, suffix: str = "main") -> str:
        """Generate a filename for the image"""
        # Create timestamp
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # Clean prompt for filename (keep alphanumeric and spaces, replace with underscores)
        clean_prompt = "".join(c if c.isalnum() or c.isspace() else "" for c in prompt)
        clean_prompt = "_".join(clean_prompt.split())[:30]  # Limit length
        
        # Add image ID if available
        if image_id:
            clean_id = image_id.replace("img_", "").replace("gemini_", "")[:8]
            return f"{timestamp}_{clean_prompt}_{clean_id}_{suffix}.png"
        else:
            return f"{timestamp}_{clean_prompt}_{suffix}.png"
    
    def process_response_and_save(self, result: Dict[str, Any], prompt: str) -> list:
        """Process the MCP response and save all found images"""
        if not result:
            print("❌ No result to process")
            return []
        
        print("\n🎉 IMAGE GENERATION RESULT")
        print("=" * 60)
        
        saved_files = []
        image_id = None
        
        try:
            # Extract the JSON text from the MCP response
            if "content" in result and result["content"]:
                content = result["content"][0] if isinstance(result["content"], list) else result["content"]
                
                if "text" in content:
                    json_text = content["text"]
                    
                    # Parse the JSON to get metadata
                    try:
                        image_data = json.loads(json_text)
                        
                        if "image" in image_data:
                            img = image_data["image"]
                            image_id = img.get('id', 'unknown')
                            
                            print("✅ Generation Status: SUCCESS")
                            print(f"🆔 Image ID: {image_id}")
                            print(f"📝 Original Prompt: {img.get('prompt', prompt)}")
                            
                            if 'enhanced_prompt' in img:
                                enhanced = img.get('enhanced_prompt', '')
                                if len(enhanced) > 100:
                                    enhanced = enhanced[:100] + "..."
                                print(f"🎨 Enhanced Prompt: {enhanced}")
                            
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
                                print(f"   Real Image: {meta.get('real_image', 'N/A')}")
                        
                    except json.JSONDecodeError as e:
                        print(f"⚠️  Could not parse image metadata: {e}")
                    
                    # Extract all data URLs from the JSON text
                    print(f"\n🔍 Searching for image data...")
                    data_urls = self.extract_image_urls(json_text)
                    
                    if data_urls:
                        print(f"✅ Found {len(data_urls)} image(s) in response")
                        
                        for i, data_url in enumerate(data_urls):
                            # Determine image type and suffix
                            if 'thumbnail' in json_text.lower() and i == 0:
                                suffix = "thumbnail"
                            elif len(data_urls) > 1:
                                suffix = f"image_{i+1}"
                            else:
                                suffix = "main"
                            
                            filename = self.generate_filename(prompt, image_id, suffix)
                            
                            print(f"\n📷 Processing image {i+1}/{len(data_urls)}...")
                            print(f"   Type: {suffix}")
                            print(f"   Size: {len(data_url)} characters")
                            
                            if self.save_data_url_image(data_url, filename):
                                full_path = os.path.join(self.output_dir, filename)
                                saved_files.append(full_path)
                            else:
                                print(f"❌ Failed to save image {i+1}")
                    else:
                        print("❌ No image data URLs found in response")
                        
                        # Fallback: look for any base64-like strings
                        base64_pattern = r'[A-Za-z0-9+/]{100,}={0,2}'
                        potential_base64 = re.findall(base64_pattern, json_text)
                        
                        if potential_base64:
                            print(f"🔍 Found {len(potential_base64)} potential base64 strings")
                            for i, b64_data in enumerate(potential_base64[:3]):  # Max 3
                                try:
                                    # Try to decode as image
                                    image_data = base64.b64decode(b64_data)
                                    if len(image_data) > 1000:  # Reasonable image size
                                        filename = self.generate_filename(prompt, image_id, f"raw_{i+1}")
                                        filepath = os.path.join(self.output_dir, filename)
                                        with open(filepath, 'wb') as f:
                                            f.write(image_data)
                                        print(f"💾 Saved potential image: {filepath}")
                                        saved_files.append(filepath)
                                except:
                                    continue
                else:
                    print("❌ No text content found in response")
            else:
                print("❌ No content found in response")
                
        except Exception as e:
            print(f"❌ Error processing response: {e}")
            import traceback
            traceback.print_exc()
        
        return saved_files

def main():
    """Main function to generate and save images"""
    print("🎨 Working MCP Image Generation with Auto-Save")
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
    
    # Find project root
    project_root = Path(__file__).resolve()
    for parent in [project_root] + list(project_root.parents):
        if (parent / "Cargo.toml").exists():
            project_root = str(parent)
            break
    else:
        project_root = os.getcwd()
    
    build_result = subprocess.run(
        ["cargo", "build", "--bin", "image-generation-server"],
        cwd=project_root,
        capture_output=True
    )
    
    if build_result.returncode != 0:
        print("❌ Build failed! Please run: cargo build --bin image-generation-server")
        sys.exit(1)
    
    print("✅ Build successful!")
    
    # Create client and start server
    client = WorkingMCPImageClient()
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
        
        # Process response and save images
        saved_files = client.process_response_and_save(result, prompt)
        
        if saved_files:
            print(f"\n🎉 SUCCESS! {len(saved_files)} image(s) saved:")
            for filepath in saved_files:
                print(f"   📁 {filepath}")
            
            print(f"\n📂 All images saved in: {client.output_dir}/")
            
            # Try to open the first image with default viewer (macOS)
            if saved_files:
                try:
                    subprocess.run(["open", saved_files[0]], check=False)
                    print("🖼️  Opening first image in default viewer...")
                except:
                    print("💡 Manually open the image files to view them")
        else:
            print("\n⚠️  Image generation completed but no images were saved")
            print("   Check the server response format or API integration")
        
    except KeyboardInterrupt:
        print("\n⚠️  Interrupted by user")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # Always stop the server
        client.stop_server()
    
    print(f"\n💡 Summary:")
    print(f"   • Prompt: {prompt}")
    print(f"   • Output directory: {client.output_dir}")
    print(f"   • Images saved: {len(saved_files) if 'saved_files' in locals() else 0}")
    print(f"   • MCP Server: {'AI mode' if api_key_available else 'Mock mode'}")

if __name__ == "__main__":
    main()
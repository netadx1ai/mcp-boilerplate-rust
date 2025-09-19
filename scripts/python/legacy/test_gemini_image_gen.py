#!/usr/bin/env python3
"""
Comprehensive Test Script for MCP Image Generation Server with Google Gemini Integration
Tests both mock and real AI modes with proper error handling and environment validation
"""

import json
import subprocess
import sys
import os
import time
import platform
from typing import Dict, Any, Optional, List
from dataclasses import dataclass
from pathlib import Path

@dataclass
class TestConfig:
    """Configuration for test execution"""
    gemini_api_key: Optional[str]
    server_binary: str
    timeout_seconds: int = 30
    mock_delay: int = 0
    ai_delay: int = 2
    test_prompts: List[str] = None
    
    def __post_init__(self):
        if self.test_prompts is None:
            self.test_prompts = [
                "A cute cat sitting on a wooden table in a cozy kitchen",
                "A futuristic cityscape with flying cars at sunset",
                "A serene mountain lake reflecting snow-capped peaks",
                "A vibrant abstract painting with geometric shapes",
                "A steampunk robot in a Victorian library"
            ]

class GeminiImageTester:
    """Main test class for Gemini image generation"""
    
    def __init__(self, config: TestConfig):
        self.config = config
        self.test_results = []
        self.project_root = Path("/Volumes/T72/Work2025AI/MCP-Genertic/mcp-boilerplate-rust")
        
    def print_header(self, title: str, char: str = "=", width: int = 70):
        """Print a formatted header"""
        print(f"\n{char * width}")
        print(f"{title:^{width}}")
        print(f"{char * width}")
    
    def print_section(self, title: str, char: str = "-", width: int = 50):
        """Print a formatted section header"""
        print(f"\n{char * width}")
        print(f" {title}")
        print(f"{char * width}")
    
    def check_environment(self) -> tuple[bool, List[str]]:
        """
        Comprehensive environment check
        
        Returns:
            Tuple of (is_ready, issues_list)
        """
        issues = []
        
        # Check if we're on macOS (where the project is located)
        if platform.system() != "Darwin":
            issues.append(f"⚠️  Running on {platform.system()}, expected macOS")
        
        # Check project directory
        if not self.project_root.exists():
            issues.append(f"❌ Project directory not found: {self.project_root}")
            return False, issues
        
        # Check Cargo.toml
        cargo_toml = self.project_root / "Cargo.toml"
        if not cargo_toml.exists():
            issues.append(f"❌ Cargo.toml not found: {cargo_toml}")
        
        # Check if server binary exists or can be built
        binary_path = self.project_root / "target" / "debug" / "image-generation-server"
        if not binary_path.exists():
            issues.append(f"⚠️  Server binary not found at {binary_path}")
            issues.append("   Will attempt to build during test...")
        else:
            issues.append(f"✅ Server binary found: {binary_path}")
        
        # Check API key
        if self.config.gemini_api_key:
            if len(self.config.gemini_api_key) < 10:
                issues.append("⚠️  GEMINI_API_KEY seems too short")
            else:
                issues.append("✅ GEMINI_API_KEY configured (AI mode available)")
        else:
            issues.append("⚠️  GEMINI_API_KEY not set (AI mode disabled)")
        
        # Check network connectivity (basic)
        try:
            import urllib.request
            urllib.request.urlopen('https://google.com', timeout=5)
            issues.append("✅ Network connectivity: OK")
        except:
            issues.append("⚠️  Network connectivity: Issues detected")
        
        # Check Python version
        python_version = f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}"
        issues.append(f"ℹ️  Python version: {python_version}")
        
        return len([i for i in issues if i.startswith("❌")]) == 0, issues
    
    def build_server(self) -> bool:
        """
        Build the server binary if needed
        
        Returns:
            True if build successful or binary already exists
        """
        binary_path = self.project_root / "target" / "debug" / "image-generation-server"
        if binary_path.exists():
            return True
            
        print("🔨 Building image-generation-server...")
        try:
            result = subprocess.run(
                ["cargo", "build", "--bin", "image-generation-server"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=120  # 2 minute timeout for build
            )
            
            if result.returncode == 0:
                print("✅ Build successful!")
                if result.stderr:
                    print("Build warnings/info:")
                    for line in result.stderr.strip().split('\n'):
                        if line.strip():
                            print(f"   {line}")
                return True
            else:
                print("❌ Build failed!")
                print("STDOUT:", result.stdout)
                print("STDERR:", result.stderr)
                return False
                
        except subprocess.TimeoutExpired:
            print("❌ Build timed out after 2 minutes")
            return False
        except Exception as e:
            print(f"❌ Build error: {e}")
            return False
    
    def run_mcp_request(self, prompt: str, use_ai: bool = False, style: str = "photorealistic") -> Optional[Dict[Any, Any]]:
        """
        Execute a single MCP request to the image generation server
        
        Args:
            prompt: Image generation prompt
            use_ai: Whether to use real AI (Gemini) or mock mode
            style: Image style parameter
            
        Returns:
            Dict containing server response or None if failed
        """
        # Prepare MCP request
        mcp_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "generate_image",
                "arguments": {
                    "prompt": prompt,
                    "style": style,
                    "size": "1024x1024"
                }
            }
        }
        
        # Build command
        cmd = [
            str(self.project_root / "target" / "debug" / "image-generation-server"),
            "--transport", "stdio",
            "--delay", str(self.config.ai_delay if use_ai else self.config.mock_delay)
        ]
        
        if use_ai:
            cmd.extend(["--use-ai", "--provider", "gemini"])
        
        print(f"🚀 Command: {' '.join(cmd)}")
        print(f"📨 Request: {json.dumps(mcp_request, indent=2)}")
        
        try:
            # Set environment for subprocess
            env = os.environ.copy()
            if use_ai and self.config.gemini_api_key:
                env["GEMINI_API_KEY"] = self.config.gemini_api_key
            
            # Start server process
            process = subprocess.Popen(
                cmd,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                cwd=self.project_root,
                env=env
            )
            
            # Send request
            request_json = json.dumps(mcp_request) + "\n"
            stdout, stderr = process.communicate(
                input=request_json, 
                timeout=self.config.timeout_seconds
            )
            
            # Log server output
            if stderr.strip():
                print("📋 Server logs:")
                for line in stderr.strip().split('\n'):
                    if line.strip():
                        print(f"   {line}")
            
            # Parse response
            if stdout.strip():
                print(f"📥 Raw response: {stdout.strip()[:200]}{'...' if len(stdout.strip()) > 200 else ''}")
                try:
                    response = json.loads(stdout.strip())
                    return response
                except json.JSONDecodeError as e:
                    print(f"❌ JSON decode error: {e}")
                    print(f"Raw stdout: {stdout}")
                    return None
            else:
                print("❌ No response received")
                return None
                
        except subprocess.TimeoutExpired:
            print(f"❌ Request timed out after {self.config.timeout_seconds}s")
            process.kill()
            return None
        except Exception as e:
            print(f"❌ Execution error: {e}")
            return None
    
    def analyze_response(self, response: Dict[Any, Any], use_ai: bool) -> Dict[str, Any]:
        """
        Analyze and extract key information from MCP response
        
        Args:
            response: The MCP server response
            use_ai: Whether AI mode was used
            
        Returns:
            Dict with analyzed results
        """
        analysis = {
            "success": False,
            "error": None,
            "image_data": None,
            "ai_enhanced": False,
            "processing_time_ms": None,
            "provider": None,
            "model": None,
            "description_quality": None
        }
        
        if "error" in response:
            analysis["error"] = response["error"].get("message", "Unknown error")
            return analysis
        
        if "result" not in response:
            analysis["error"] = "No result in response"
            return analysis
        
        result = response["result"]
        if "content" not in result or not result["content"]:
            analysis["error"] = "No content in result"
            return analysis
        
        try:
            content = result["content"][0]
            if "text" not in content:
                analysis["error"] = "No text in content"
                return analysis
            
            image_data = json.loads(content["text"])
            analysis["success"] = image_data.get("success", False)
            analysis["image_data"] = image_data
            
            if "image" in image_data:
                img = image_data["image"]
                
                # Extract metadata
                metadata = img.get("metadata", {})
                analysis["provider"] = metadata.get("provider")
                analysis["model"] = metadata.get("model")
                analysis["processing_time_ms"] = metadata.get("processing_time_ms")
                
                # Check for AI enhancement
                if use_ai:
                    analysis["ai_enhanced"] = "ai_description" in img
                    if analysis["ai_enhanced"]:
                        desc = img.get("ai_description", "")
                        analysis["description_quality"] = len(desc.split())  # Word count as quality metric
            
            return analysis
            
        except json.JSONDecodeError as e:
            analysis["error"] = f"Failed to parse image data: {e}"
            return analysis
    
    def display_detailed_result(self, response: Dict[Any, Any], analysis: Dict[str, Any], use_ai: bool, prompt: str):
        """Display comprehensive test results"""
        
        mode = "🤖 AI MODE (Gemini)" if use_ai else "🎭 MOCK MODE"
        print(f"\n{mode}")
        print(f"📝 Prompt: {prompt}")
        
        if not analysis["success"]:
            print(f"❌ FAILED: {analysis.get('error', 'Unknown error')}")
            return
        
        print("✅ SUCCESS")
        
        image_data = analysis["image_data"]
        if "image" in image_data:
            img = image_data["image"]
            
            print(f"🆔 Image ID: {img.get('id', 'N/A')}")
            print(f"🎨 Style: {img.get('style', 'N/A')}")
            print(f"📏 Size: {img.get('size', 'N/A')}")
            print(f"🔗 URL: {img.get('url', 'N/A')}")
            
            if use_ai and "ai_description" in img:
                desc = img["ai_description"]
                print(f"🧠 AI Description ({len(desc)} chars): {desc[:100]}{'...' if len(desc) > 100 else ''}")
            
            # Metadata
            if "metadata" in img:
                meta = img["metadata"]
                print(f"⚙️  Provider: {meta.get('provider', 'N/A')}")
                print(f"🏷️  Model: {meta.get('model', 'N/A')}")
                print(f"⏱️  Processing: {meta.get('processing_time_ms', 'N/A')}ms")
                
                if "note" in meta:
                    print(f"📌 Note: {meta['note']}")
            
            # Usage info
            if "usage" in image_data:
                usage = image_data["usage"]
                print(f"💰 Estimated cost: ${usage.get('estimated_cost_usd', 'N/A')}")
                print(f"🎯 Tokens used: {usage.get('tokens_used', 'N/A')}")
    
    def run_comprehensive_test(self):
        """Run the complete test suite"""
        
        self.print_header("🎨 MCP GEMINI IMAGE GENERATION COMPREHENSIVE TEST", "=", 80)
        
        # Environment check
        self.print_section("🔍 Environment Check")
        ready, issues = self.check_environment()
        
        for issue in issues:
            print(issue)
        
        if not ready:
            print("\n❌ Environment check failed. Please fix the issues above.")
            return False
        
        print("\n✅ Environment check passed!")
        
        # Build check
        self.print_section("🔨 Build Check")
        if not self.build_server():
            print("❌ Failed to build server. Aborting tests.")
            return False
        
        # Test execution
        test_modes = [
            ("Mock Mode", False),
        ]
        
        if self.config.gemini_api_key:
            test_modes.append(("AI Mode (Gemini)", True))
        else:
            print("\n⚠️  Skipping AI mode tests (no API key)")
        
        all_passed = True
        
        for mode_name, use_ai in test_modes:
            self.print_section(f"🧪 Testing {mode_name}")
            
            mode_passed = True
            
            for i, prompt in enumerate(self.config.test_prompts[:3], 1):  # Test first 3 prompts
                print(f"\n🎯 Test {i}/3: {prompt[:50]}{'...' if len(prompt) > 50 else ''}")
                
                response = self.run_mcp_request(prompt, use_ai=use_ai)
                
                if response is None:
                    print("❌ No response received")
                    mode_passed = False
                    continue
                
                analysis = self.analyze_response(response, use_ai)
                self.display_detailed_result(response, analysis, use_ai, prompt)
                
                if not analysis["success"]:
                    mode_passed = False
                
                # Brief pause between tests
                if i < 3:
                    time.sleep(1)
            
            if mode_passed:
                print(f"\n✅ {mode_name} tests: PASSED")
            else:
                print(f"\n❌ {mode_name} tests: FAILED")
                all_passed = False
        
        # Summary
        self.print_section("📊 Test Summary")
        
        if all_passed:
            print("🎉 ALL TESTS PASSED!")
            print("\n💡 Next steps:")
            print("   • Mock mode is working for development")
            if self.config.gemini_api_key:
                print("   • Gemini AI integration is functional")
                print("   • Ready for production use with real image generation APIs")
            else:
                print("   • Set GEMINI_API_KEY to test AI mode")
            print("   • Consider integrating with actual image generation services")
        else:
            print("❌ SOME TESTS FAILED")
            print("\n🔧 Troubleshooting:")
            print("   • Check server logs above for error details")
            print("   • Verify API key is valid")
            print("   • Ensure network connectivity")
            print("   • Try running individual tests manually")
        
        return all_passed

def main():
    """Main function"""
    print("🎨 MCP Gemini Image Generation Tester")
    print("=" * 50)
    
    # Get API key from environment
    gemini_api_key = os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")
    
    # Create test configuration
    config = TestConfig(
        gemini_api_key=gemini_api_key,
        server_binary="image-generation-server",
        timeout_seconds=30,
        mock_delay=0,
        ai_delay=1  # Fast for testing
    )
    
    # Create and run tester
    tester = GeminiImageTester(config)
    success = tester.run_comprehensive_test()
    
    # Exit with appropriate code
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
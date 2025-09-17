#!/usr/bin/env python3
"""
Convenient Image Generation Wrapper Script
Run this from the project root to generate images using MCP
"""

import sys
import os
from pathlib import Path

# Add the clients directory to Python path
project_root = Path(__file__).parent
clients_dir = project_root / "scripts" / "python" / "clients"
sys.path.insert(0, str(clients_dir))

# Import and run the image generator
try:
    from image_generator import main
    
    if __name__ == "__main__":
        print("🎨 MCP Image Generator")
        print("=" * 50)
        print(f"📂 Project Root: {project_root}")
        print(f"📁 Output Directory: {project_root}/generated_images/")
        print()
        
        # Run the main image generator
        main()
        
except ImportError as e:
    print(f"❌ Error importing image generator: {e}")
    print("Make sure the scripts/python/clients/image_generator.py file exists")
    sys.exit(1)
except Exception as e:
    print(f"❌ Error running image generator: {e}")
    sys.exit(1)
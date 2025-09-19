#!/bin/bash

# MCP Image Generation Server - Google Gemini Environment Setup
# This script helps you configure the environment for Google Gemini AI integration

set -e

echo "🎨 MCP Image Generation Server - Gemini Setup"
echo "=============================================="
echo

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "examples/image-generation-server" ]]; then
    echo "❌ Error: This script must be run from the mcp-boilerplate-rust project root"
    echo "   Current directory: $(pwd)"
    echo "   Expected files: Cargo.toml, examples/image-generation-server/"
    exit 1
fi

echo "✅ Project directory verified"
echo

# Function to check if API key looks valid
validate_api_key() {
    local key="$1"
    if [[ -z "$key" ]]; then
        return 1
    fi
    if [[ ${#key} -lt 20 ]]; then
        echo "⚠️  Warning: API key seems short (${#key} characters). Google API keys are typically longer."
        return 1
    fi
    if [[ "$key" =~ ^[A-Za-z0-9_-]+$ ]]; then
        return 0
    else
        echo "⚠️  Warning: API key contains unexpected characters."
        return 1
    fi
}

# Check current environment
echo "🔍 Checking current environment..."

CURRENT_GEMINI_KEY="${GEMINI_API_KEY:-}"
CURRENT_GOOGLE_KEY="${GOOGLE_API_KEY:-}"

if [[ -n "$CURRENT_GEMINI_KEY" ]]; then
    echo "✅ GEMINI_API_KEY is currently set"
    if validate_api_key "$CURRENT_GEMINI_KEY"; then
        echo "   Key format looks valid (${#CURRENT_GEMINI_KEY} characters)"
    fi
elif [[ -n "$CURRENT_GOOGLE_KEY" ]]; then
    echo "✅ GOOGLE_API_KEY is currently set"
    if validate_api_key "$CURRENT_GOOGLE_KEY"; then
        echo "   Key format looks valid (${#CURRENT_GOOGLE_KEY} characters)"
    fi
else
    echo "⚠️  No API key found in environment"
fi

echo

# API Key setup
echo "🔑 Google Gemini API Key Setup"
echo "------------------------------"
echo
echo "To use the image generation server with Google Gemini AI, you need an API key."
echo
echo "📝 How to get a Gemini API key:"
echo "   1. Go to Google AI Studio: https://makersuite.google.com/app/apikey"
echo "   2. Sign in with your Google account"
echo "   3. Click 'Create API Key'"
echo "   4. Copy the generated key"
echo

# Prompt for API key
read -p "🔐 Enter your Gemini API key (or press Enter to skip): " NEW_API_KEY

if [[ -n "$NEW_API_KEY" ]]; then
    echo
    echo "🔍 Validating API key..."
    
    if validate_api_key "$NEW_API_KEY"; then
        echo "✅ API key format looks good"
        
        # Test the API key
        echo "🧪 Testing API key with Google Gemini..."
        
        # Create a simple test request
        TEST_RESPONSE=$(curl -s -w "%{http_code}" \
            -H "Content-Type: application/json" \
            -d '{
                "contents": [{
                    "parts": [{
                        "text": "Hello, respond with just the word SUCCESS if you can read this."
                    }]
                }],
                "generationConfig": {
                    "maxOutputTokens": 10
                }
            }' \
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=${NEW_API_KEY}" 2>/dev/null)
        
        HTTP_CODE="${TEST_RESPONSE: -3}"
        RESPONSE_BODY="${TEST_RESPONSE%???}"
        
        if [[ "$HTTP_CODE" == "200" ]]; then
            echo "✅ API key test successful!"
            
            # Set environment variables
            export GEMINI_API_KEY="$NEW_API_KEY"
            
            # Create/update .env file
            echo "💾 Saving to .env file..."
            
            ENV_FILE=".env"
            if [[ -f "$ENV_FILE" ]]; then
                # Remove existing GEMINI_API_KEY or GOOGLE_API_KEY lines
                sed -i.bak '/^GEMINI_API_KEY=/d; /^GOOGLE_API_KEY=/d' "$ENV_FILE"
            fi
            
            echo "GEMINI_API_KEY=$NEW_API_KEY" >> "$ENV_FILE"
            echo "✅ API key saved to $ENV_FILE"
            
            # Add to shell profile for persistence
            echo
            echo "🔧 Adding to shell profile for persistence..."
            
            SHELL_PROFILE=""
            if [[ -f "$HOME/.zshrc" ]]; then
                SHELL_PROFILE="$HOME/.zshrc"
            elif [[ -f "$HOME/.bashrc" ]]; then
                SHELL_PROFILE="$HOME/.bashrc"
            elif [[ -f "$HOME/.bash_profile" ]]; then
                SHELL_PROFILE="$HOME/.bash_profile"
            fi
            
            if [[ -n "$SHELL_PROFILE" ]]; then
                if ! grep -q "GEMINI_API_KEY" "$SHELL_PROFILE"; then
                    echo "export GEMINI_API_KEY=\"$NEW_API_KEY\"" >> "$SHELL_PROFILE"
                    echo "✅ Added to $SHELL_PROFILE"
                else
                    echo "ℹ️  API key already in $SHELL_PROFILE"
                fi
            fi
            
        else
            echo "❌ API key test failed (HTTP $HTTP_CODE)"
            echo "   Response: ${RESPONSE_BODY:0:100}..."
            echo "   Please check your API key and try again"
            exit 1
        fi
    else
        echo "⚠️  API key format validation failed, but continuing anyway..."
        export GEMINI_API_KEY="$NEW_API_KEY"
    fi
else
    echo "⏭️  Skipping API key setup"
fi

echo

# Build the server
echo "🔨 Building the image generation server..."
echo "----------------------------------------"

if cargo build --bin image-generation-server; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed"
    exit 1
fi

echo

# Test the setup
echo "🧪 Testing the setup..."
echo "----------------------"

echo "Testing mock mode (no API required)..."
MOCK_TEST_OUTPUT=$(echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"generate_image","arguments":{"prompt":"test"}}}' | \
    timeout 10s ./target/debug/image-generation-server --transport stdio --delay 0 2>/dev/null || echo "TIMEOUT")

if echo "$MOCK_TEST_OUTPUT" | grep -q '"success"'; then
    echo "✅ Mock mode test passed"
else
    echo "⚠️  Mock mode test had issues"
fi

if [[ -n "$GEMINI_API_KEY" ]]; then
    echo "Testing AI mode with Gemini..."
    AI_TEST_OUTPUT=$(echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"generate_image","arguments":{"prompt":"test cat"}}}' | \
        timeout 15s ./target/debug/image-generation-server --transport stdio --use-ai --provider gemini --delay 0 2>/dev/null || echo "TIMEOUT")
    
    if echo "$AI_TEST_OUTPUT" | grep -q '"success"'; then
        echo "✅ AI mode test passed"
    else
        echo "⚠️  AI mode test had issues"
    fi
else
    echo "⏭️  Skipping AI mode test (no API key)"
fi

echo

# Final instructions
echo "🎉 Setup Complete!"
echo "=================="
echo
echo "📋 Quick Start Commands:"
echo
echo "Mock mode (fast, no API key needed):"
echo "  cargo run --bin image-generation-server -- --transport stdio"
echo

if [[ -n "$GEMINI_API_KEY" ]]; then
    echo "AI mode with Gemini (real image descriptions):"
    echo "  cargo run --bin image-generation-server -- --transport stdio --use-ai --provider gemini"
    echo
fi

echo "HTTP server mode:"
echo "  cargo run --bin image-generation-server -- --transport http --port 3001"
echo

echo "Run comprehensive tests:"
echo "  python3 test_gemini_image_gen.py"
echo

echo "💡 Tips:"
echo "  • Mock mode: Great for development and testing"
echo "  • AI mode: Uses real Gemini API for enhanced image descriptions"
echo "  • The server generates descriptions and placeholder URLs"
echo "  • For actual image generation, integrate with Imagen or other services"
echo

if [[ -n "$NEW_API_KEY" ]]; then
    echo "🔐 Security reminder:"
    echo "  • Your API key is saved in .env and your shell profile"
    echo "  • Keep your API key secret and don't commit it to version control"
    echo "  • Add .env to your .gitignore file"
    echo
fi

echo "🚀 Ready to generate images with MCP!"
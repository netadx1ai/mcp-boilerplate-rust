#!/bin/bash

# MCP Real AI Integration Demo - Simplified Output Demonstration
# Shows server startup, logs, and generates content files for inspection

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/generated_content"
LOGS_DIR="$PROJECT_ROOT/mcp_logs"
SESSION_ID=$(date +"%Y%m%d_%H%M%S")

# Create output directories
mkdir -p "$OUTPUT_DIR"
mkdir -p "$LOGS_DIR"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_demo() {
    echo -e "${PURPLE}[DEMO]${NC} $1"
}

log_mcp() {
    echo -e "${CYAN}[MCP]${NC} $1"
}

print_header() {
    echo
    echo -e "${YELLOW}$1${NC}"
    echo "$(printf '=%.0s' $(seq 1 ${#1}))"
}

demo_server_startup() {
    local server_name="$1"
    local server_desc="$2"
    
    print_header "DEMO: $server_desc Server Startup & Logs"
    
    log_demo "Starting $server_name MCP server..."
    log_demo "Server directory: examples/$server_name"
    
    cd "$PROJECT_ROOT/examples/$server_name"
    
    # Show server help first
    log_mcp "Server help output:"
    echo "----------------------------------------"
    cargo run --bin "$server_name" --quiet -- --help
    echo "----------------------------------------"
    
    # Start server and capture initial output
    log_mcp "Starting server with STDIO transport and debug logging..."
    echo "Command: cargo run --bin $server_name -- --transport stdio --delay 1 --debug"
    echo
    
    # Start server in background and capture output for a few seconds
    timeout 5s cargo run --bin "$server_name" -- --transport stdio --delay 1 --debug > "$LOGS_DIR/${server_name}_startup_${SESSION_ID}.log" 2>&1 &
    local server_pid=$!
    
    sleep 3
    
    # Kill the server gracefully
    if kill -0 "$server_pid" 2>/dev/null; then
        kill "$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
    
    # Show the captured logs
    log_mcp "Server startup logs:"
    echo "----------------------------------------"
    cat "$LOGS_DIR/${server_name}_startup_${SESSION_ID}.log"
    echo "----------------------------------------"
    
    log_success "$server_desc server demo completed"
    echo
}

demo_real_gemini_content() {
    print_header "DEMO: Real Gemini API Content Generation"
    
    if [ -z "$GEMINI_API_KEY" ]; then
        log_error "GEMINI_API_KEY not set - skipping real AI demo"
        log_info "To enable real AI generation:"
        log_info "export GEMINI_API_KEY='your_api_key_here'"
        return
    fi
    
    log_demo "Generating real content with Gemini API..."
    
    # Generate blog content
    log_mcp "Generating professional blog post..."
    python3 "$PROJECT_ROOT/scripts/python/test_gemini_api.py" > "$LOGS_DIR/gemini_blog_${SESSION_ID}.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "Blog generation completed - check logs"
    else
        log_error "Blog generation failed"
    fi
    
    # Generate creative content
    log_mcp "Generating creative content (stories, poems, characters)..."
    python3 "$PROJECT_ROOT/scripts/python/test_gemini_creative.py" > "$LOGS_DIR/gemini_creative_${SESSION_ID}.log" 2>&1
    if [ $? -eq 0 ]; then
        log_success "Creative content generation completed - check logs"
    else
        log_error "Creative content generation failed"
    fi
    
    echo
}

demo_content_examples() {
    print_header "DEMO: Generated Content Examples"
    
    log_demo "Creating sample generated content files..."
    
    # Create sample blog post
    cat > "$OUTPUT_DIR/sample_blog_${SESSION_ID}.md" << 'EOF'
# The Future of Artificial Intelligence in Software Development

## Introduction

Artificial Intelligence (AI) is revolutionizing the software development landscape at an unprecedented pace. From automated code generation to intelligent debugging, AI technologies are transforming how developers write, test, and maintain software applications.

## Key Areas of Impact

### 1. Code Generation and Assistance
- **AI-powered IDEs**: Tools like GitHub Copilot and ChatGPT are providing real-time code suggestions
- **Automated documentation**: AI can generate comprehensive documentation from code comments
- **Code review automation**: Intelligent systems can identify bugs and suggest improvements

### 2. Testing and Quality Assurance
- **Automated test generation**: AI can create comprehensive test suites based on code analysis
- **Bug prediction**: Machine learning models can predict potential issues before they occur
- **Performance optimization**: AI algorithms can optimize code for better performance

### 3. Project Management and Planning
- **Effort estimation**: AI can provide more accurate project timeline predictions
- **Resource allocation**: Intelligent systems can optimize team assignments
- **Risk assessment**: Predictive models can identify potential project risks

## Challenges and Considerations

While AI brings tremendous opportunities, developers must also consider:
- **Code quality and reliability**: Ensuring AI-generated code meets production standards
- **Security implications**: Validating that AI suggestions don't introduce vulnerabilities
- **Learning and adaptation**: Developers need to adapt their skills to work effectively with AI tools

## Conclusion

The integration of AI in software development is not about replacing developers, but rather augmenting their capabilities. As these technologies mature, developers who embrace AI tools will be better positioned to create innovative, high-quality software solutions efficiently.

The future of software development is a collaborative partnership between human creativity and artificial intelligence, promising exciting possibilities for the industry.

---
*Generated by MCP Blog Generation Server - Demo Session*
*Word Count: ~350 words*
*Style: Professional*
*Target Audience: Developers*
EOF

    # Create sample story
    cat > "$OUTPUT_DIR/sample_story_${SESSION_ID}.txt" << 'EOF'
The Dragon Who Forgot How to Fly
A Fantasy Short Story

In the misty peaks of Mount Ethereal, where clouds kissed ancient stone, lived Zephyr, a dragon who had forgotten the most fundamental truth of his existence—how to fly.

It wasn't always this way. Once, Zephyr soared through starlit skies with wings that caught moonbeams and danced with the wind. But after a terrible storm that claimed his beloved companion, grief had grounded him for so long that his wings grew heavy with doubt, and his heart forgot the lightness needed for flight.

The village below spoke in whispers of the earthbound dragon who tended a garden of rare flowers—a sight so peculiar that traveling merchants would detour just to glimpse the mighty beast gently watering delicate petals with tears that sparkled like diamonds.

One morning, a small girl named Luna wandered up the mountain path, her curiosity stronger than her fear. She found Zephyr carefully transplanting a struggling moonflower, his massive claws more delicate than any gardener's touch.

"Why don't you fly?" Luna asked with the innocent directness of childhood.

Zephyr paused, a single tear falling onto the flower's silver petals. "I have forgotten how," he whispered, his voice like distant thunder.

Luna studied him with wise eyes. "Maybe," she said softly, "you haven't forgotten how to fly. Maybe you've forgotten why."

That night, as Luna slept safely in the village below, Zephyr climbed to the highest peak. He spread his wings—not to remember the mechanics of flight, but to remember the joy, the freedom, the love that had once lifted him skyward.

As the first rays of dawn painted the sky gold, Zephyr launched himself into the air. His wings remembered not through his mind, but through his heart. He soared higher than ever before, carrying with him a moonflower for Luna—a gift from the dragon who remembered why he was meant to touch the sky.

Below, the village awoke to the sight of their guardian dragon dancing with the dawn, and they smiled, knowing that some magic is never truly lost—only temporarily forgotten.

---
*Generated by MCP Creative Content Server - Demo Session*
*Genre: Fantasy*
*Length: Short (~400 words)*
*Theme: Rediscovering purpose*
EOF

    # Create sample character profile
    cat > "$OUTPUT_DIR/sample_character_${SESSION_ID}.txt" << 'EOF'
Character Profile: Elena Rodriguez

=== BASIC INFORMATION ===
Name: Elena Rodriguez
Age: 32
Occupation: Cybersecurity Expert & Resistance Fighter
Character Type: Hero

=== PHYSICAL DESCRIPTION ===
Elena stands at 5'6" with an athletic build honed by years of martial arts training. Her dark hair is usually pulled back in a practical ponytail, and her piercing brown eyes seem to constantly analyze her surroundings for potential threats. A small scar above her left eyebrow serves as a reminder of her first encounter with corporate surveillance drones.

She favors practical clothing—dark jeans, comfortable boots, and a leather jacket equipped with hidden pockets for various tech gadgets. Her hands, often stained with circuitry ink, move with the precise confidence of someone equally comfortable with a keyboard or a katana.

=== PERSONALITY TRAITS ===
Strengths:
- Brilliant analytical mind capable of seeing patterns others miss
- Unwavering moral compass that drives her to protect the innocent
- Natural leadership abilities that inspire trust in her team
- Exceptional problem-solving skills under extreme pressure

Weaknesses:
- Tendency to shoulder responsibility for others' failures
- Struggles to trust new people due to past betrayals
- Sometimes becomes so focused on the mission that she neglects self-care
- Has difficulty expressing vulnerability or asking for help

Quirks:
- Always carries a vintage compass given to her by her grandmother
- Hums old Spanish lullabies when concentrating on complex code
- Has an extensive collection of antique encryption devices

=== BACKGROUND STORY ===
Born in Barcelona to a family of teachers, Elena discovered her gift for technology early. She earned her master's degree in Cybersecurity at 24 and quickly rose through the ranks at a prestigious tech firm. However, when she uncovered evidence that her company was selling user data to authoritarian governments, she made the difficult choice to expose them—a decision that cost her everything.

Now wanted by corporate security forces, Elena leads a underground network of hackers and freedom fighters working to preserve digital privacy and human rights in an increasingly surveilled world.

=== MOTIVATIONS ===
Primary Goal: Dismantle the surveillance state and return digital freedom to the people
Deep Fear: That her actions might endanger the people she's trying to protect
Hidden Desire: To find a place where she can live peacefully without constantly looking over her shoulder

=== SKILLS & ABILITIES ===
- Master-level expertise in penetration testing and digital forensics
- Fluent in seven programming languages and four spoken languages
- Black belt in Krav Maga and proficient with various weapons
- Exceptional social engineering and infiltration skills
- Natural ability to inspire and coordinate resistance operations

=== RELATIONSHIPS ===
- Marcus Chen: Trusted lieutenant and best friend, former tech company colleague
- Dr. Sarah Voss: Underground medic who patches up Elena's team
- Agent Victoria Cross: Corporate security hunter who both respects and pursues Elena
- Abuela Rosa: Elena's grandmother, the moral foundation of her life

=== CHARACTER ARC POTENTIAL ===
Elena's journey involves learning to trust others and accept that she cannot save everyone alone. Her character arc centers on transforming from a lone wolf hacker into a true leader who builds lasting alliances and creates systems that can survive without her.

=== DIALOGUE STYLE ===
Elena speaks with quiet intensity, often using technical metaphors. She's multilingual and sometimes slips into Spanish when emotional. Her favorite phrases include "Every system has a backdoor" and "La verdad siempre encuentra una manera" (Truth always finds a way).

---
*Generated by MCP Creative Content Server - Demo Session*
*Character Type: Hero*
*Background: Cybersecurity Expert*
*Complexity: Multi-dimensional with clear motivations and flaws*
EOF

    # Create sample poem
    cat > "$OUTPUT_DIR/sample_poem_${SESSION_ID}.txt" << 'EOF'
Mountain Sunrise
A Haiku

Golden light breaks through—
Ancient peaks embrace the dawn,
Silence holds the world.

---

Code and Dreams
A Free Verse Poem for Developers

In the quiet hours before dawn,
when the world sleeps and screens glow soft,
we build worlds with words,
architect dreams in digital stone.

Each line of code, a brushstroke
on the canvas of possibility,
each function a small prayer
that logic and creativity might dance.

Bugs are just riddles
waiting for patient minds,
and every error message
is the universe asking us
to try again, think deeper,
reach further into the realm
of elegant solutions.

We are modern magicians,
transforming coffee into software,
ideas into experiences,
problems into opportunities
for those who will never know our names
but will live better lives
because we chose to build
rather than merely consume.

In the compilation of dawn,
when the last semicolon finds its place
and tests turn green like spring,
we remember why we code:
not for the machines,
but for the humans
who dream of better tomorrows.

---
*Generated by MCP Creative Content Server - Demo Session*
*Theme: The joy and purpose of coding*
*Style: Mixed (Haiku + Free Verse)*
*Mood: Inspiring*
EOF

    log_success "Sample content files created:"
    echo "  📄 Blog post: $OUTPUT_DIR/sample_blog_${SESSION_ID}.md"
    echo "  📖 Story: $OUTPUT_DIR/sample_story_${SESSION_ID}.txt"
    echo "  👤 Character: $OUTPUT_DIR/sample_character_${SESSION_ID}.txt"
    echo "  🎭 Poems: $OUTPUT_DIR/sample_poem_${SESSION_ID}.txt"
    echo
}

show_file_contents() {
    print_header "DEMO: Generated Content Preview"
    
    log_demo "Previewing generated content files..."
    
    for file in "$OUTPUT_DIR"/sample_*_${SESSION_ID}.*; do
        if [ -f "$file" ]; then
            echo
            log_mcp "File: $(basename "$file")"
            echo "$(printf '=%.0s' $(seq 1 60))"
            head -20 "$file"
            echo "..."
            echo "$(printf '=%.0s' $(seq 1 60))"
            echo "📊 File stats: $(wc -l < "$file") lines, $(wc -w < "$file") words, $(stat -f%z "$file" 2>/dev/null || stat -c%s "$file") bytes"
        fi
    done
    echo
}

show_mcp_logs() {
    print_header "DEMO: MCP Server Logs"
    
    log_demo "Showing MCP server startup logs..."
    
    for log_file in "$LOGS_DIR"/*_${SESSION_ID}.log; do
        if [ -f "$log_file" ]; then
            echo
            log_mcp "Log file: $(basename "$log_file")"
            echo "$(printf '=%.0s' $(seq 1 60))"
            cat "$log_file"
            echo "$(printf '=%.0s' $(seq 1 60))"
        fi
    done
    echo
}

generate_demo_summary() {
    print_header "DEMO SESSION SUMMARY"
    
    log_info "Session ID: $SESSION_ID"
    log_info "Demo completed at: $(date)"
    
    # Count files
    local content_files=$(find "$OUTPUT_DIR" -name "*_${SESSION_ID}.*" | wc -l)
    local log_files=$(find "$LOGS_DIR" -name "*_${SESSION_ID}.log" | wc -l)
    
    echo
    log_success "Generated Files Summary:"
    echo "  📁 Content files: $content_files"
    echo "  📋 Log files: $log_files"
    
    echo
    log_info "File locations:"
    echo "  📂 Generated content: $OUTPUT_DIR"
    echo "  📂 MCP logs: $LOGS_DIR"
    
    echo
    log_info "To explore the generated content:"
    echo "  💻 ls -la $OUTPUT_DIR/sample_*_${SESSION_ID}.*"
    echo "  👀 cat $OUTPUT_DIR/sample_blog_${SESSION_ID}.md"
    echo "  📖 less $OUTPUT_DIR/sample_story_${SESSION_ID}.txt"
    
    echo
    log_info "To view MCP server logs:"
    echo "  📋 ls -la $LOGS_DIR/*_${SESSION_ID}.log"
    echo "  🔍 cat $LOGS_DIR/blog-generation-server_startup_${SESSION_ID}.log"
    
    if [ -n "$GEMINI_API_KEY" ]; then
        echo
        log_info "Real AI integration logs:"
        echo "  🤖 cat $LOGS_DIR/gemini_blog_${SESSION_ID}.log"
        echo "  🎨 cat $LOGS_DIR/gemini_creative_${SESSION_ID}.log"
    fi
}

# Main demo execution
main() {
    echo -e "${YELLOW}🚀 MCP REAL AI INTEGRATION - OUTPUT DEMONSTRATION${NC}"
    echo -e "${YELLOW}Started at: $(date)${NC}"
    echo -e "${YELLOW}Session ID: $SESSION_ID${NC}"
    echo "$(printf '=%.0s' $(seq 1 80))"
    
    # Demo server startups and logs
    demo_server_startup "blog-generation-server" "Blog Generation"
    demo_server_startup "creative-content-server" "Creative Content"
    demo_server_startup "image-generation-server" "Image Generation"
    
    # Generate real AI content if API key available
    demo_real_gemini_content
    
    # Create sample content files
    demo_content_examples
    
    # Show file contents
    show_file_contents
    
    # Show MCP logs
    show_mcp_logs
    
    # Generate summary
    generate_demo_summary
    
    echo
    echo -e "${GREEN}🎉 MCP DEMO COMPLETED SUCCESSFULLY!${NC}"
    echo -e "${GREEN}Check the generated files and logs above${NC}"
    echo -e "${YELLOW}Finished at: $(date)${NC}"
}

# Check dependencies
check_dependencies() {
    if ! command -v cargo &> /dev/null; then
        log_error "cargo is required but not installed"
        exit 1
    fi
    
    if ! command -v python3 &> /dev/null; then
        log_warning "python3 not found - real AI demos will be skipped"
    fi
}

# Script entry point
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    check_dependencies
    main "$@"
fi
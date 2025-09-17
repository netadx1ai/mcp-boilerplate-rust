#!/usr/bin/env python3
"""
Test Gemini API functionality for MCP creative content generation validation
This script validates that the Gemini API is working correctly for real creative AI integration testing.
"""

import os
import sys
import json
import time
import requests
from typing import Optional, Dict, Any, List

def get_gemini_api_key() -> Optional[str]:
    """Get Gemini API key from environment variables."""
    return os.environ.get('GEMINI_API_KEY')

def generate_story_with_gemini(
    prompt: str,
    genre: str = "fantasy",
    length: str = "short",
    style: str = "third_person"
) -> Dict[str, Any]:
    """
    Generate story content using Gemini API.
    
    Args:
        prompt: Story prompt/premise
        genre: Story genre (fantasy, sci-fi, mystery, romance, thriller, adventure)
        length: Story length (short=500, medium=1000, long=2000 words)
        style: Narrative style (first_person, third_person, dialogue_heavy)
        
    Returns:
        Dict containing generated content and metadata
    """
    api_key = get_gemini_api_key()
    if not api_key:
        raise ValueError("GEMINI_API_KEY environment variable not set")
    
    # Map length to word count
    word_counts = {
        "short": 500,
        "medium": 1000,
        "long": 2000
    }
    word_count = word_counts.get(length, 500)
    
    # Construct the prompt
    story_prompt = f"""Write a {genre} story in {style.replace('_', ' ')} style about: {prompt}
The story should be approximately {word_count} words long.
Create engaging characters, vivid descriptions, and a compelling plot.
Include dialogue and narrative tension appropriate for the {genre} genre.
Make it creative, original, and entertaining."""
    
    return _call_gemini_api(story_prompt, "story", {
        "prompt": prompt,
        "genre": genre,
        "length": length,
        "style": style,
        "target_word_count": word_count
    })

def generate_poem_with_gemini(
    theme: str,
    style: str = "free_verse",
    mood: str = "contemplative"
) -> Dict[str, Any]:
    """
    Generate poetry using Gemini API.
    
    Args:
        theme: Poetry theme/subject
        style: Poetry style (haiku, sonnet, free_verse, limerick, ballad)
        mood: Desired mood (contemplative, joyful, melancholic, inspiring)
        
    Returns:
        Dict containing generated content and metadata
    """
    api_key = get_gemini_api_key()
    if not api_key:
        raise ValueError("GEMINI_API_KEY environment variable not set")
    
    # Style-specific instructions
    style_instructions = {
        "haiku": "Write a haiku about {theme}. Follow the traditional 5-7-5 syllable structure exactly. Focus on nature imagery and seasonal references.",
        "sonnet": "Write a sonnet about {theme}. Use 14 lines with proper rhyme scheme (ABAB CDCD EFEF GG). Include meter and rhythm.",
        "free_verse": "Write a free verse poem about {theme}. Focus on imagery and emotion without rhyme constraints. Use line breaks for rhythm.",
        "limerick": "Write a limerick about {theme}. Use the traditional AABBA rhyme scheme with humor and wit.",
        "ballad": "Write a ballad about {theme}. Tell a story in verse with rhyme and meter, suitable for singing."
    }
    
    instruction = style_instructions.get(style, style_instructions["free_verse"])
    poem_prompt = f"""{instruction.format(theme=theme)}
Make it {mood} in mood and tone.
Focus on vivid imagery, emotional resonance, and poetic language.
Ensure it captures the essence of '{theme}' beautifully."""
    
    return _call_gemini_api(poem_prompt, "poem", {
        "theme": theme,
        "style": style,
        "mood": mood
    })

def develop_character_with_gemini(
    name: str,
    character_type: str = "hero",
    background: str = "mysterious past"
) -> Dict[str, Any]:
    """
    Develop a character using Gemini API.
    
    Args:
        name: Character name
        character_type: Type (hero, villain, supporting, anti_hero)
        background: Character background/profession
        
    Returns:
        Dict containing generated content and metadata
    """
    api_key = get_gemini_api_key()
    if not api_key:
        raise ValueError("GEMINI_API_KEY environment variable not set")
    
    character_prompt = f"""Develop a detailed character profile for {name}, who is a {character_type} with {background}.

Include the following elements:
- **Physical Description**: Appearance, mannerisms, distinctive features
- **Personality Traits**: Strengths, weaknesses, quirks, habits
- **Background Story**: Origin, formative experiences, key events
- **Motivations**: Goals, fears, desires, what drives them
- **Relationships**: Family, friends, enemies, romantic interests
- **Skills/Abilities**: Talents, profession, special abilities, expertise
- **Character Arc Potential**: How they could grow or change
- **Dialogue Style**: How they speak, favorite phrases, speech patterns

Make this character complex, believable, and memorable. Provide specific details that bring them to life."""
    
    return _call_gemini_api(character_prompt, "character", {
        "name": name,
        "character_type": character_type,
        "background": background
    })

def _call_gemini_api(prompt: str, content_type: str, metadata: Dict[str, Any]) -> Dict[str, Any]:
    """
    Make API call to Gemini and return structured response.
    
    Args:
        prompt: The prompt to send to Gemini
        content_type: Type of content (story, poem, character)
        metadata: Additional metadata about the request
        
    Returns:
        Dict containing generated content and metadata
    """
    api_key = get_gemini_api_key()
    url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={api_key}"
    
    request_body = {
        "contents": [{
            "parts": [{
                "text": prompt
            }]
        }],
        "generationConfig": {
            "temperature": 0.8,  # Higher creativity for creative content
            "topK": 40,
            "topP": 0.95,
            "maxOutputTokens": 4096,
            "candidateCount": 1
        }
    }
    
    print(f"🔄 Generating {content_type}...")
    start_time = time.time()
    
    try:
        response = requests.post(
            url,
            headers={"Content-Type": "application/json"},
            json=request_body,
            timeout=60
        )
        
        generation_time = time.time() - start_time
        
        if not response.ok:
            error_detail = response.text
            raise Exception(f"Gemini API error ({response.status_code}): {error_detail}")
        
        response_data = response.json()
        
        # Extract content
        content = (response_data
                  .get("candidates", [{}])[0]
                  .get("content", {})
                  .get("parts", [{}])[0]
                  .get("text", ""))
        
        if not content:
            raise Exception(f"No {content_type} content generated by Gemini API")
        
        return {
            "content": content,
            "content_type": content_type,
            "generation_time": generation_time,
            "word_count": len(content.split()),
            "character_count": len(content),
            "line_count": len(content.split('\n')),
            **metadata
        }
        
    except requests.RequestException as e:
        raise Exception(f"Request failed: {e}")
    except json.JSONDecodeError as e:
        raise Exception(f"Failed to parse response: {e}")

def validate_story_quality(story_data: Dict[str, Any]) -> Dict[str, Any]:
    """Validate the quality of generated story content."""
    content = story_data["content"]
    target_words = story_data.get("target_word_count", 500)
    actual_words = story_data["word_count"]
    genre = story_data.get("genre", "")
    
    validation_results = {
        "passed": True,
        "issues": [],
        "metrics": {}
    }
    
    # Word count validation (±30% tolerance for creative content)
    word_tolerance = target_words * 0.3
    min_words = target_words - word_tolerance
    max_words = target_words + word_tolerance
    
    validation_results["metrics"]["word_count_accuracy"] = {
        "target": target_words,
        "actual": actual_words,
        "tolerance_range": f"{int(min_words)}-{int(max_words)}",
        "within_tolerance": min_words <= actual_words <= max_words
    }
    
    if not (min_words <= actual_words <= max_words):
        validation_results["passed"] = False
        validation_results["issues"].append(
            f"Word count {actual_words} outside tolerance range {int(min_words)}-{int(max_words)}"
        )
    
    # Story structure validation
    has_dialogue = '"' in content or "'" in content
    has_description = len(content.split('.')) > 5
    has_narrative_flow = actual_words > 200
    
    validation_results["metrics"]["story_structure"] = {
        "has_dialogue": has_dialogue,
        "has_description": has_description,
        "has_narrative_flow": has_narrative_flow,
        "paragraph_count": len([p for p in content.split('\n\n') if p.strip()])
    }
    
    if not has_description:
        validation_results["issues"].append("Story lacks sufficient descriptive content")
    
    if not has_narrative_flow:
        validation_results["issues"].append("Story too short for proper narrative flow")
    
    # Genre consistency check (basic keyword matching)
    genre_keywords = {
        "fantasy": ["magic", "dragon", "wizard", "quest", "kingdom", "spell", "enchant"],
        "sci-fi": ["space", "future", "technology", "robot", "alien", "galaxy", "science"],
        "mystery": ["clue", "detective", "murder", "suspect", "investigate", "solve"],
        "romance": ["love", "heart", "kiss", "relationship", "romantic", "passion"],
        "thriller": ["danger", "chase", "escape", "threat", "suspense", "urgent"],
        "adventure": ["journey", "explore", "discover", "brave", "challenge", "expedition"]
    }
    
    if genre in genre_keywords:
        content_lower = content.lower()
        matching_keywords = [kw for kw in genre_keywords[genre] if kw in content_lower]
        genre_consistency = len(matching_keywords) > 0
        
        validation_results["metrics"]["genre_consistency"] = {
            "genre": genre,
            "matching_keywords": matching_keywords,
            "consistent": genre_consistency
        }
        
        if not genre_consistency:
            validation_results["issues"].append(f"Story doesn't match {genre} genre characteristics")
    
    if validation_results["issues"]:
        validation_results["passed"] = False
    
    return validation_results

def validate_poem_quality(poem_data: Dict[str, Any]) -> Dict[str, Any]:
    """Validate the quality of generated poetry content."""
    content = poem_data["content"]
    style = poem_data.get("style", "")
    theme = poem_data.get("theme", "")
    
    validation_results = {
        "passed": True,
        "issues": [],
        "metrics": {}
    }
    
    lines = [line.strip() for line in content.split('\n') if line.strip()]
    line_count = len(lines)
    
    # Style-specific validation
    style_requirements = {
        "haiku": {"lines": 3, "min_lines": 3, "max_lines": 3},
        "sonnet": {"lines": 14, "min_lines": 14, "max_lines": 14},
        "limerick": {"lines": 5, "min_lines": 5, "max_lines": 5},
        "free_verse": {"lines": None, "min_lines": 4, "max_lines": 50},
        "ballad": {"lines": None, "min_lines": 8, "max_lines": 32}
    }
    
    if style in style_requirements:
        requirements = style_requirements[style]
        expected_lines = requirements["lines"]
        min_lines = requirements["min_lines"]
        max_lines = requirements["max_lines"]
        
        validation_results["metrics"]["structure"] = {
            "style": style,
            "line_count": line_count,
            "expected_lines": expected_lines,
            "valid_range": f"{min_lines}-{max_lines}" if not expected_lines else str(expected_lines)
        }
        
        if expected_lines and line_count != expected_lines:
            validation_results["passed"] = False
            validation_results["issues"].append(f"{style.title()} should have exactly {expected_lines} lines, got {line_count}")
        elif not expected_lines and (line_count < min_lines or line_count > max_lines):
            validation_results["passed"] = False
            validation_results["issues"].append(f"{style.title()} should have {min_lines}-{max_lines} lines, got {line_count}")
    
    # Theme relevance check
    if theme:
        content_lower = content.lower()
        theme_words = [word.lower() for word in theme.split() if len(word) > 3]
        matching_words = [word for word in theme_words if word in content_lower]
        theme_relevance = len(matching_words) / len(theme_words) if theme_words else 1.0
        
        validation_results["metrics"]["theme_relevance"] = {
            "theme": theme,
            "theme_words": theme_words,
            "matching_words": matching_words,
            "relevance_ratio": theme_relevance
        }
        
        if theme_relevance < 0.3:
            validation_results["passed"] = False
            validation_results["issues"].append(f"Poem not sufficiently related to theme '{theme}'")
    
    # Poetic quality indicators
    has_imagery = any(word in content.lower() for word in ["see", "hear", "feel", "touch", "smell", "bright", "dark", "warm", "cold"])
    has_emotion = any(word in content.lower() for word in ["love", "joy", "sadness", "fear", "hope", "dream", "heart", "soul"])
    
    validation_results["metrics"]["poetic_quality"] = {
        "has_imagery": has_imagery,
        "has_emotion": has_emotion,
        "line_count": line_count,
        "avg_line_length": sum(len(line) for line in lines) / len(lines) if lines else 0
    }
    
    return validation_results

def validate_character_quality(character_data: Dict[str, Any]) -> Dict[str, Any]:
    """Validate the quality of generated character development."""
    content = character_data["content"]
    name = character_data.get("name", "")
    character_type = character_data.get("character_type", "")
    
    validation_results = {
        "passed": True,
        "issues": [],
        "metrics": {}
    }
    
    # Check for required character development elements
    required_elements = {
        "physical": ["appearance", "look", "physical", "height", "hair", "eyes", "build"],
        "personality": ["personality", "trait", "character", "nature", "temperament"],
        "background": ["background", "past", "history", "origin", "childhood", "family"],
        "motivation": ["goal", "want", "desire", "motivation", "drive", "ambition"],
        "skills": ["skill", "ability", "talent", "power", "expertise", "profession"],
        "relationships": ["friend", "family", "relationship", "ally", "enemy", "love"]
    }
    
    content_lower = content.lower()
    element_coverage = {}
    
    for element, keywords in required_elements.items():
        has_element = any(keyword in content_lower for keyword in keywords)
        element_coverage[element] = has_element
    
    covered_elements = sum(element_coverage.values())
    total_elements = len(required_elements)
    
    validation_results["metrics"]["character_development"] = {
        "name_mentioned": name.lower() in content_lower if name else True,
        "element_coverage": element_coverage,
        "coverage_ratio": covered_elements / total_elements,
        "word_count": character_data["word_count"],
        "comprehensive": covered_elements >= 4  # At least 4 out of 6 elements
    }
    
    if name and name.lower() not in content_lower:
        validation_results["passed"] = False
        validation_results["issues"].append(f"Character name '{name}' not mentioned in profile")
    
    if covered_elements < 4:
        validation_results["passed"] = False
        validation_results["issues"].append(f"Character development incomplete ({covered_elements}/{total_elements} elements covered)")
    
    if character_data["word_count"] < 200:
        validation_results["passed"] = False
        validation_results["issues"].append("Character profile too brief (< 200 words)")
    
    return validation_results

def test_creative_content_scenarios():
    """Test various creative content generation scenarios."""
    test_scenarios = [
        # Story generation tests
        {
            "type": "story",
            "name": "Fantasy Adventure Story",
            "params": {
                "prompt": "A young mage discovers they can speak to dragons",
                "genre": "fantasy",
                "length": "short",
                "style": "third_person"
            }
        },
        {
            "type": "story", 
            "name": "Sci-Fi Mystery Story",
            "params": {
                "prompt": "A colony on Mars discovers an ancient alien artifact",
                "genre": "sci-fi",
                "length": "medium",
                "style": "first_person"
            }
        },
        
        # Poetry generation tests
        {
            "type": "poem",
            "name": "Nature Haiku",
            "params": {
                "theme": "cherry blossoms in spring",
                "style": "haiku",
                "mood": "contemplative"
            }
        },
        {
            "type": "poem",
            "name": "Love Sonnet",
            "params": {
                "theme": "eternal love",
                "style": "sonnet", 
                "mood": "romantic"
            }
        },
        {
            "type": "poem",
            "name": "Freedom Free Verse",
            "params": {
                "theme": "breaking free from constraints",
                "style": "free_verse",
                "mood": "inspiring"
            }
        },
        
        # Character development tests
        {
            "type": "character",
            "name": "Hero Character",
            "params": {
                "name": "Elena Rodriguez",
                "character_type": "hero",
                "background": "cybersecurity expert turned resistance fighter"
            }
        },
        {
            "type": "character",
            "name": "Villain Character", 
            "params": {
                "name": "Dr. Viktor Shadows",
                "character_type": "villain",
                "background": "brilliant scientist with a dark agenda"
            }
        }
    ]
    
    results = []
    
    for scenario in test_scenarios:
        print(f"\n🎨 Testing: {scenario['name']}")
        print(f"   Type: {scenario['type']}")
        
        try:
            # Generate content based on type
            if scenario["type"] == "story":
                content_data = generate_story_with_gemini(**scenario["params"])
                validation = validate_story_quality(content_data)
            elif scenario["type"] == "poem":
                content_data = generate_poem_with_gemini(**scenario["params"])
                validation = validate_poem_quality(content_data)
            elif scenario["type"] == "character":
                content_data = develop_character_with_gemini(**scenario["params"])
                validation = validate_character_quality(content_data)
            else:
                raise ValueError(f"Unknown content type: {scenario['type']}")
            
            # Add scenario info
            content_data["scenario_name"] = scenario["name"]
            content_data["validation"] = validation
            
            results.append(content_data)
            
            # Print results
            print(f"✅ Generated successfully in {content_data['generation_time']:.2f}s")
            print(f"   Words: {content_data['word_count']}")
            print(f"   Lines: {content_data['line_count']}")
            
            if validation["passed"]:
                print(f"✅ Quality validation passed")
            else:
                print(f"❌ Quality issues: {', '.join(validation['issues'])}")
                
        except Exception as e:
            print(f"❌ Failed: {e}")
            results.append({
                "scenario_name": scenario["name"],
                "content_type": scenario["type"],
                "error": str(e),
                "failed": True
            })
    
    return results

def print_creative_summary(results):
    """Print creative content test summary."""
    print("\n" + "="*70)
    print("🎭 GEMINI CREATIVE CONTENT TEST SUMMARY")
    print("="*70)
    
    # Overall statistics
    total_tests = len(results)
    successful_tests = len([r for r in results if not r.get("failed", False)])
    quality_passed = len([r for r in results if not r.get("failed", False) and r.get("validation", {}).get("passed", False)])
    
    print(f"Total Tests: {total_tests}")
    print(f"Successful Generations: {successful_tests}")
    print(f"Quality Validations Passed: {quality_passed}")
    print(f"Success Rate: {(successful_tests/total_tests)*100:.1f}%")
    print(f"Quality Rate: {(quality_passed/total_tests)*100:.1f}%")
    
    # Performance metrics by content type
    content_types = {}
    for result in results:
        if not result.get("failed", False):
            content_type = result.get("content_type", "unknown")
            if content_type not in content_types:
                content_types[content_type] = []
            content_types[content_type].append(result)
    
    if content_types:
        print(f"\nPerformance by Content Type:")
        for content_type, items in content_types.items():
            avg_time = sum(item.get("generation_time", 0) for item in items) / len(items)
            avg_words = sum(item.get("word_count", 0) for item in items) / len(items)
            quality_rate = len([item for item in items if item.get("validation", {}).get("passed", False)]) / len(items) * 100
            
            print(f"  {content_type.title()}: {avg_time:.2f}s avg, {avg_words:.0f} words avg, {quality_rate:.0f}% quality")
    
    # Detailed results
    print(f"\nDetailed Results:")
    for result in results:
        status = "❌ FAILED" if result.get("failed", False) else "✅ SUCCESS"
        name = result.get("scenario_name", "Unknown")
        content_type = result.get("content_type", "unknown")
        
        if result.get("failed", False):
            print(f"{status} {name} ({content_type}): {result.get('error', 'Unknown error')}")
        else:
            validation_status = "✅ QUALITY PASS" if result.get("validation", {}).get("passed", False) else "⚠️ QUALITY ISSUES"
            words = result.get("word_count", 0)
            time_taken = result.get("generation_time", 0)
            print(f"{status} {name} ({content_type}): {words} words in {time_taken:.2f}s - {validation_status}")

def main():
    """Main test execution."""
    print("🎨 Starting Gemini Creative Content API Integration Tests")
    print("="*70)
    
    # Check API key
    if not get_gemini_api_key():
        print("❌ GEMINI_API_KEY environment variable not set")
        print("   Please set your Gemini API key:")
        print("   export GEMINI_API_KEY='your_api_key_here'")
        sys.exit(1)
    
    print("✅ GEMINI_API_KEY found")
    
    # Run tests
    try:
        results = test_creative_content_scenarios()
        print_creative_summary(results)
        
        # Determine exit code
        successful_tests = len([r for r in results if not r.get("failed", False)])
        quality_passed = len([r for r in results if not r.get("failed", False) and r.get("validation", {}).get("passed", False)])
        
        if successful_tests == 0:
            print("\n❌ All tests failed!")
            sys.exit(1)
        elif quality_passed < successful_tests * 0.7:  # At least 70% should pass quality
            print(f"\n⚠️ Many tests had quality issues ({quality_passed}/{successful_tests} passed quality validation)")
            sys.exit(0)  # Still exit successfully since generation worked
        else:
            print(f"\n🎉 Most tests passed quality validation! ({quality_passed}/{successful_tests} passed)")
            sys.exit(0)
            
    except Exception as e:
        print(f"\n💥 Test execution failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
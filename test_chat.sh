#!/bin/bash

# AI Terminal Chat Testing Script
echo "============================================================"
echo "🚀 AI Terminal Chat Testing Suite"
echo "============================================================"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Ollama URL
OLLAMA_URL="http://localhost:11434"

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to log test results
log_test() {
    local test_name="$1"
    local success="$2"
    local details="$3"
    
    if [ "$success" = "true" ]; then
        echo -e "${GREEN}✅ PASS${NC}: $test_name"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ FAIL${NC}: $test_name"
        ((TESTS_FAILED++))
    fi
    
    if [ -n "$details" ]; then
        echo "  Details: $details"
    fi
}

# Test 1: Ollama Connection
echo -e "\n${YELLOW}Test 1: Ollama Connection${NC}"
echo "----------------------------------------"
RESPONSE=$(curl -s -w "\n%{http_code}" "$OLLAMA_URL/api/tags" 2>/dev/null | tail -1)

if [ "$RESPONSE" = "200" ]; then
    MODELS=$(curl -s "$OLLAMA_URL/api/tags" | jq -r '.models[].name' 2>/dev/null | tr '\n' ', ' | sed 's/,$//')
    log_test "Ollama Connection" "true" "Found models: $MODELS"
else
    log_test "Ollama Connection" "false" "HTTP status: $RESPONSE"
    echo -e "\n⚠️ Cannot proceed without Ollama connection"
    exit 1
fi

# Get first available model
MODEL=$(curl -s "$OLLAMA_URL/api/tags" | jq -r '.models[0].name' 2>/dev/null)
if [ -z "$MODEL" ]; then
    echo -e "\n⚠️ No models available in Ollama"
    exit 1
fi

echo -e "\n📦 Using model: $MODEL"

# Test 2: Simple Generation
echo -e "\n${YELLOW}Test 2: Simple Generation${NC}"
echo "----------------------------------------"
START_TIME=$(date +%s)
RESPONSE=$(curl -s -X POST "$OLLAMA_URL/api/generate" \
    -H "Content-Type: application/json" \
    -d "{\"model\": \"$MODEL\", \"prompt\": \"Say hello in exactly 3 words\", \"stream\": false, \"options\": {\"temperature\": 0.1, \"num_predict\": 10}}" \
    2>/dev/null)
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

if [ -n "$RESPONSE" ]; then
    GENERATED=$(echo "$RESPONSE" | jq -r '.response' 2>/dev/null | head -c 50)
    if [ -n "$GENERATED" ]; then
        log_test "Simple Generation" "true" "Generated in ${DURATION}s: '$GENERATED...'"
    else
        log_test "Simple Generation" "false" "Empty response"
    fi
else
    log_test "Simple Generation" "false" "No response from server"
fi

# Test 3: Streaming Generation
echo -e "\n${YELLOW}Test 3: Streaming Generation${NC}"
echo "----------------------------------------"
TOKEN_COUNT=0
START_TIME=$(date +%s)

# Use curl with line-by-line processing
curl -s -N -X POST "$OLLAMA_URL/api/generate" \
    -H "Content-Type: application/json" \
    -d "{\"model\": \"$MODEL\", \"prompt\": \"Count: 1 2 3\", \"stream\": true, \"options\": {\"temperature\": 0.1, \"num_predict\": 10}}" \
    2>/dev/null | while IFS= read -r line; do
    if [ -n "$line" ]; then
        TOKEN=$(echo "$line" | jq -r '.response' 2>/dev/null)
        if [ -n "$TOKEN" ] && [ "$TOKEN" != "null" ]; then
            ((TOKEN_COUNT++))
        fi
        # Check for done flag
        DONE=$(echo "$line" | jq -r '.done' 2>/dev/null)
        if [ "$DONE" = "true" ]; then
            break
        fi
    fi
done

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

if [ $TOKEN_COUNT -gt 0 ]; then
    log_test "Streaming Generation" "true" "Streamed response in ${DURATION}s"
else
    log_test "Streaming Generation" "false" "No tokens received"
fi

# Test 4: Model Info
echo -e "\n${YELLOW}Test 4: Model Information${NC}"
echo "----------------------------------------"
MODEL_INFO=$(curl -s "$OLLAMA_URL/api/show" \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"$MODEL\"}" 2>/dev/null)

if [ -n "$MODEL_INFO" ]; then
    MODEL_SIZE=$(echo "$MODEL_INFO" | jq -r '.size' 2>/dev/null)
    if [ -n "$MODEL_SIZE" ] && [ "$MODEL_SIZE" != "null" ]; then
        log_test "Model Information" "true" "Model loaded successfully"
    else
        log_test "Model Information" "false" "Could not get model info"
    fi
else
    log_test "Model Information" "false" "No response"
fi

# Test 5: Quick Response Time
echo -e "\n${YELLOW}Test 5: Response Time Check${NC}"
echo "----------------------------------------"
START_TIME=$(date +%s%N)
curl -s -X POST "$OLLAMA_URL/api/generate" \
    -H "Content-Type: application/json" \
    -d "{\"model\": \"$MODEL\", \"prompt\": \"Hi\", \"stream\": false, \"options\": {\"num_predict\": 1}}" \
    --max-time 5 > /dev/null 2>&1

if [ $? -eq 0 ]; then
    END_TIME=$(date +%s%N)
    DURATION=$(( (END_TIME - START_TIME) / 1000000 ))
    log_test "Quick Response" "true" "Response in ${DURATION}ms"
else
    log_test "Quick Response" "false" "Timeout or error"
fi

# Summary
echo ""
echo "============================================================"
echo "📊 Test Summary"
echo "============================================================"
TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))
echo -e "\nResults: ${GREEN}$TESTS_PASSED${NC}/${TOTAL_TESTS} tests passed"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed! Chat functionality is working correctly.${NC}"
else
    echo -e "${RED}❌ Some tests failed. Please check the details above.${NC}"
fi

echo ""
echo "============================================================"
echo "🎯 Next Steps:"
echo "============================================================"
echo "1. Run TUI: cargo run --bin ai-terminal"
echo "2. Run GUI: cargo run --bin ai-terminal-gui"
echo "3. Test Python bridge integration"
echo "4. Create spec-kit structure for ongoing development"

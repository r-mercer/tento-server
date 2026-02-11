#!/bin/bash

# Test script for token refresh endpoint
# Usage: ./scripts/test_refresh_token.sh [refresh_token]

set -e

BASE_URL="http://localhost:8080"
REFRESH_TOKEN="${1:-}"

echo "=================================="
echo "Token Refresh Endpoint Test Script"
echo "=================================="
echo ""

# Function to format JSON output
format_json() {
    if command -v jq &> /dev/null; then
        echo "$1" | jq '.'
    else
        echo "$1"
    fi
}

# Test 1: Missing refresh_token parameter
echo "Test 1: Missing refresh_token in request body"
echo "----------------------------------------------"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -d '{}' \
    "${BASE_URL}/auth/refresh")
HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo "HTTP Status: $HTTP_CODE"
echo "Response:"
format_json "$BODY"
echo ""

# Test 2: Invalid refresh_token format
echo "Test 2: Invalid refresh_token format"
echo "-------------------------------------"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -d '{"refresh_token": "invalid_token_format"}' \
    "${BASE_URL}/auth/refresh")
HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo "HTTP Status: $HTTP_CODE"
echo "Response:"
format_json "$BODY"
echo ""

# Test 3: Valid refresh_token (if provided)
if [ -n "$REFRESH_TOKEN" ]; then
    echo "Test 3: Valid refresh_token"
    echo "----------------------------"
    RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
        -H "Content-Type: application/json" \
        -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}" \
        "${BASE_URL}/auth/refresh")
    HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
    BODY=$(echo "$RESPONSE" | sed '$d')

    echo "HTTP Status: $HTTP_CODE"
    echo "Response:"
    format_json "$BODY"
    echo ""

    if [ "$HTTP_CODE" = "200" ]; then
        echo "✓ Token refresh successful!"
        
        # Extract new tokens
        NEW_TOKEN=$(echo "$BODY" | jq -r '.token // empty')
        NEW_REFRESH=$(echo "$BODY" | jq -r '.refresh_token // empty')
        
        if [ -n "$NEW_TOKEN" ] && [ -n "$NEW_REFRESH" ]; then
            echo ""
            echo "New Access Token (first 50 chars): ${NEW_TOKEN:0:50}..."
            echo "New Refresh Token (first 50 chars): ${NEW_REFRESH:0:50}..."
        fi
    else
        echo "✗ Token refresh failed"
    fi
else
    echo "Test 3: Skipped (no valid refresh_token provided)"
    echo "--------------------------------------------------"
    echo "To test with a valid token, run:"
    echo "  ./scripts/test_refresh_token.sh YOUR_REFRESH_TOKEN"
    echo ""
    echo "To get a refresh token:"
    echo "  1. Start the server: cargo run"
    echo "  2. Complete GitHub OAuth flow"
    echo "  3. Copy the refresh_token from the response"
fi

echo ""
echo "=================================="
echo "Tests Complete"
echo "=================================="

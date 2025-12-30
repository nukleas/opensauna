#!/bin/bash

# Test the Hotworx API
# Load credentials from .env
source /Users/naderheidari/Code/hotworx/.env

PASSWORD_HASH=$(echo -n "$HOTWORX_PASSWORD" | shasum -a 256 | cut -d' ' -f1)
DEVICE_ID="test-device-$(date +%s)"

echo "=== Step 1: Login with password ==="
echo "Device ID: $DEVICE_ID"
echo ""

# Step 1: Login with password
LOGIN_RESPONSE=$(curl -s -X POST "https://sailposapi.hotworx.net/api/v1/loginwithpassword" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -H "User-Agent: okhttp/4.9.3" \
  -H "sec-ch-ua-platform: Android" \
  -H "application-version: 5.0.0" \
  -H "device-id: $DEVICE_ID" \
  -d "email_address=$HOTWORX_USERNAME&password=$PASSWORD_HASH&device_id=$DEVICE_ID")

echo "Login response: $LOGIN_RESPONSE"
echo ""

# Extract token from response
TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
echo "Token: ${TOKEN:0:50}..."
echo ""

echo "=== Step 2: Verify OTP (123456) ==="
# Step 2: Verify OTP - type is "password" for password login flow
OTP_RESPONSE=$(curl -s -X POST "https://sailposapi.hotworx.net/api/v1/verifyOtp" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -H "User-Agent: okhttp/4.9.3" \
  -H "sec-ch-ua-platform: Android" \
  -H "application-version: 5.0.0" \
  -H "device-id: $DEVICE_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -d "email_address=$HOTWORX_USERNAME&password=$PASSWORD_HASH&phone_number=&device_id=$DEVICE_ID&otp=123456&type=password")

echo "OTP response: $OTP_RESPONSE"

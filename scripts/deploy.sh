#!/bin/bash

# Soroban Time-Locked Savings Contract Deployment Script
# This script automates the deployment and initialization process

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
    echo -e "${GREEN}✓ Loaded .env file${NC}"
else
    echo -e "${RED}✗ .env file not found. Please copy .env.example to .env and configure it.${NC}"
    exit 1
fi

# Check required variables
if [ -z "$SOROBAN_NETWORK" ] || [ -z "$STELLAR_SECRET_KEY" ]; then
    echo -e "${RED}✗ Missing required environment variables${NC}"
    echo "Please ensure SOROBAN_NETWORK and STELLAR_SECRET_KEY are set in .env"
    exit 1
fi

echo -e "${YELLOW}Starting deployment process...${NC}\n"

# Step 1: Build the contract
echo -e "${YELLOW}Step 1: Building contract...${NC}"
cd contracts/hello-world
cargo build --release --target wasm32-unknown-unknown
echo -e "${GREEN}✓ Contract built successfully${NC}\n"

# Step 2: Deploy the contract
echo -e "${YELLOW}Step 2: Deploying contract to ${SOROBAN_NETWORK}...${NC}"
CONTRACT_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/hello_world.wasm \
    --source-account $STELLAR_SECRET_KEY \
    --network $SOROBAN_NETWORK)

echo -e "${GREEN}✓ Contract deployed!${NC}"
echo -e "Contract ID: ${GREEN}$CONTRACT_ID${NC}\n"

# Update .env file with contract ID
cd ../..
if grep -q "^CONTRACT_ID=" .env; then
    sed -i "s|^CONTRACT_ID=.*|CONTRACT_ID=$CONTRACT_ID|" .env
else
    echo "CONTRACT_ID=$CONTRACT_ID" >> .env
fi
echo -e "${GREEN}✓ Updated .env with CONTRACT_ID${NC}\n"

# Step 3: Set up token (if not already set)
if [ -z "$TOKEN_ADDRESS" ]; then
    echo -e "${YELLOW}Step 3: Setting up native XLM token...${NC}"
    TOKEN_ADDRESS=$(soroban lab token wrap \
        --asset native \
        --source-account $STELLAR_SECRET_KEY \
        --network $SOROBAN_NETWORK)
    
    echo -e "${GREEN}✓ Token wrapped!${NC}"
    echo -e "Token Address: ${GREEN}$TOKEN_ADDRESS${NC}\n"
    
    # Update .env
    if grep -q "^TOKEN_ADDRESS=" .env; then
        sed -i "s|^TOKEN_ADDRESS=.*|TOKEN_ADDRESS=$TOKEN_ADDRESS|" .env
    else
        echo "TOKEN_ADDRESS=$TOKEN_ADDRESS" >> .env
    fi
    echo -e "${GREEN}✓ Updated .env with TOKEN_ADDRESS${NC}\n"
else
    echo -e "${GREEN}✓ Using existing TOKEN_ADDRESS from .env${NC}\n"
fi

# Step 4: Get admin address
ADMIN_ADDRESS=$(soroban keys address admin-account 2>/dev/null || echo "")
if [ -z "$ADMIN_ADDRESS" ]; then
    echo -e "${YELLOW}Admin account not found. Using deployer account as admin.${NC}"
    ADMIN_ADDRESS=$(soroban keys address deployer-account)
fi

# Step 5: Initialize the contract
echo -e "${YELLOW}Step 4: Initializing contract...${NC}"
PENALTY=${EMERGENCY_PENALTY:-1000}

soroban contract invoke \
    --id $CONTRACT_ID \
    --source-account $STELLAR_SECRET_KEY \
    --network $SOROBAN_NETWORK \
    -- \
    initialize \
    --token $TOKEN_ADDRESS \
    --admin $ADMIN_ADDRESS \
    --emergency_penalty $PENALTY

echo -e "${GREEN}✓ Contract initialized successfully!${NC}\n"

# Summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}   Deployment Complete! 🎉${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "Contract ID:    ${GREEN}$CONTRACT_ID${NC}"
echo -e "Token Address:  ${GREEN}$TOKEN_ADDRESS${NC}"
echo -e "Admin Address:  ${GREEN}$ADMIN_ADDRESS${NC}"
echo -e "Network:        ${GREEN}$SOROBAN_NETWORK${NC}"
echo -e "Emergency Penalty: ${GREEN}$PENALTY basis points${NC}"
echo -e "${GREEN}========================================${NC}\n"

echo -e "${YELLOW}Next steps:${NC}"
echo -e "1. Test the contract with: ${GREEN}./scripts/test-contract.sh${NC}"
echo -e "2. Create a savings goal: ${GREEN}./scripts/create-goal.sh${NC}"
echo -e "3. View your goals: ${GREEN}./scripts/view-goals.sh${NC}"

#!/bin/bash

# Script to set up Stellar accounts for Soroban development

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

NETWORK=${1:-testnet}

echo -e "${YELLOW}Setting up Stellar accounts for $NETWORK...${NC}\n"

# Create deployer account
echo -e "${YELLOW}Creating deployer account...${NC}"
soroban keys generate deployer-account --network $NETWORK 2>/dev/null || echo "Deployer account already exists"
DEPLOYER_ADDRESS=$(soroban keys address deployer-account)
echo -e "Deployer Address: ${GREEN}$DEPLOYER_ADDRESS${NC}"

# Fund deployer account
echo -e "${YELLOW}Funding deployer account...${NC}"
soroban keys fund deployer-account --network $NETWORK
echo -e "${GREEN}✓ Deployer account funded${NC}\n"

# Create admin account
echo -e "${YELLOW}Creating admin account...${NC}"
soroban keys generate admin-account --network $NETWORK 2>/dev/null || echo "Admin account already exists"
ADMIN_ADDRESS=$(soroban keys address admin-account)
echo -e "Admin Address: ${GREEN}$ADMIN_ADDRESS${NC}"

# Fund admin account
echo -e "${YELLOW}Funding admin account...${NC}"
soroban keys fund admin-account --network $NETWORK
echo -e "${GREEN}✓ Admin account funded${NC}\n"

# Create test user accounts
for i in 1 2; do
    echo -e "${YELLOW}Creating test-user-$i account...${NC}"
    soroban keys generate test-user-$i --network $NETWORK 2>/dev/null || echo "test-user-$i already exists"
    USER_ADDRESS=$(soroban keys address test-user-$i)
    echo -e "Test User $i Address: ${GREEN}$USER_ADDRESS${NC}"
    
    echo -e "${YELLOW}Funding test-user-$i account...${NC}"
    soroban keys fund test-user-$i --network $NETWORK
    echo -e "${GREEN}✓ test-user-$i account funded${NC}\n"
done

# Get secret keys
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}   Accounts Created Successfully! 🎉${NC}"
echo -e "${GREEN}========================================${NC}\n"

echo -e "${YELLOW}Add these to your .env file:${NC}\n"

DEPLOYER_SECRET=$(soroban keys show deployer-account)
ADMIN_SECRET=$(soroban keys show admin-account)
USER1_SECRET=$(soroban keys show test-user-1)
USER2_SECRET=$(soroban keys show test-user-2)

echo "STELLAR_SECRET_KEY=$DEPLOYER_SECRET"
echo "ADMIN_SECRET_KEY=$ADMIN_SECRET"
echo "TEST_USER_1_SECRET=$USER1_SECRET"
echo "TEST_USER_2_SECRET=$USER2_SECRET"

echo -e "\n${GREEN}========================================${NC}"
echo -e "${YELLOW}Account Addresses:${NC}"
echo -e "Deployer: ${GREEN}$DEPLOYER_ADDRESS${NC}"
echo -e "Admin:    ${GREEN}$ADMIN_ADDRESS${NC}"
echo -e "User 1:   ${GREEN}$(soroban keys address test-user-1)${NC}"
echo -e "User 2:   ${GREEN}$(soroban keys address test-user-2)${NC}"
echo -e "${GREEN}========================================${NC}\n"

echo -e "${YELLOW}Next step: Run ${GREEN}./scripts/deploy.sh${YELLOW} to deploy your contract${NC}"

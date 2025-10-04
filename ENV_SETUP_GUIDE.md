# Environment Setup Guide

This guide will help you set up your `.env` file for the Soroban Time-Locked Savings contract.

## Quick Start

1. **Copy the example file:**
   ```bash
   cp .env.example .env
   ```

2. **Follow the steps below to populate your `.env` file**

---

## Step-by-Step Configuration

### 1. Get a Testnet Account

You need a Stellar account with testnet funds to deploy and test your contract.

**Option A: Using Stellar Laboratory (Recommended)**
1. Go to [Stellar Laboratory](https://laboratory.stellar.org/#account-creator?network=test)
2. Click "Generate keypair"
3. Copy the **Secret Key** (starts with `S`)
4. Click "Get test network lumens" to fund your account
5. Paste the secret key in your `.env` as `STELLAR_SECRET_KEY`

**Option B: Using Soroban CLI**
```bash
# Generate a new identity
soroban keys generate my-account --network testnet

# Fund the account
soroban keys fund my-account --network testnet

# Get the secret key
soroban keys show my-account
```

### 2. Configure Network Settings

For **Testnet** (recommended for development):
```env
SOROBAN_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
```

For **Futurenet** (bleeding edge features):
```env
SOROBAN_NETWORK=futurenet
SOROBAN_RPC_URL=https://rpc-futurenet.stellar.org
SOROBAN_NETWORK_PASSPHRASE="Test SDF Future Network ; October 2022"
```

### 3. Set Up Admin Account

The admin account has special privileges (update penalty rates, etc.)

**Option 1: Use the same account**
```env
ADMIN_SECRET_KEY=<same as STELLAR_SECRET_KEY>
```

**Option 2: Create a separate admin account**
```bash
soroban keys generate admin-account --network testnet
soroban keys fund admin-account --network testnet
soroban keys show admin-account
```

### 4. Deploy Your Contract

```bash
# Build the contract
cd contracts/hello-world
cargo build --release --target wasm32-unknown-unknown

# Deploy to testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hello_world.wasm \
  --source my-account \
  --network testnet

# This will output a CONTRACT_ID - copy it to your .env
```

### 5. Set Up Token Address

**Option A: Use Native XLM (Stellar Lumens)**
```bash
# Wrap native XLM as a Soroban token
soroban lab token wrap \
  --asset native \
  --source my-account \
  --network testnet

# Copy the output address to TOKEN_ADDRESS in .env
```

**Option B: Deploy a Custom Test Token**
```bash
# Deploy a test token
soroban contract deploy \
  --wasm <path-to-token-wasm> \
  --source my-account \
  --network testnet
```

**Option C: Use an Existing Token**
- Find testnet token addresses on [Stellar Expert](https://stellar.expert/explorer/testnet)

### 6. Initialize Your Contract

Once deployed, initialize it:

```bash
soroban contract invoke \
  --id <YOUR_CONTRACT_ID> \
  --source my-account \
  --network testnet \
  -- \
  initialize \
  --token <TOKEN_ADDRESS> \
  --admin <ADMIN_PUBLIC_KEY> \
  --emergency_penalty 1000
```

---

## Example Complete .env File

```env
# Network
SOROBAN_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Accounts
STELLAR_SECRET_KEY=SCZM7ZYXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVX
ADMIN_SECRET_KEY=SCZM7ZYXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVXVX

# Contract
CONTRACT_ID=CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM
TOKEN_ADDRESS=CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABBBBB
EMERGENCY_PENALTY=1000

# Testing
TEST_AMOUNT=10000000
TEST_LOCK_DURATION=86400
TEST_INTEREST_RATE=500
```

---

## Useful Commands

### Check Account Balance
```bash
soroban keys address my-account | xargs -I {} \
  curl "https://horizon-testnet.stellar.org/accounts/{}"
```

### Invoke Contract Functions

**Create a savings goal:**
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source my-account \
  --network testnet \
  -- \
  create_goal \
  --owner $(soroban keys address my-account) \
  --amount 10000000 \
  --lock_duration 86400 \
  --interest_rate 500
```

**Check goal details:**
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source my-account \
  --network testnet \
  -- \
  get_goal \
  --owner $(soroban keys address my-account) \
  --goal_id 0
```

**Withdraw (after lock period):**
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source my-account \
  --network testnet \
  -- \
  withdraw \
  --owner $(soroban keys address my-account) \
  --goal_id 0
```

---

## Security Best Practices

1. ✅ **NEVER commit `.env` to git** - it's already in `.gitignore`
2. ✅ **Use different accounts for testnet and mainnet**
3. ✅ **Keep your secret keys secure** - treat them like passwords
4. ✅ **Use environment-specific .env files** (`.env.testnet`, `.env.mainnet`)
5. ✅ **Rotate keys regularly** in production
6. ✅ **Use hardware wallets** for mainnet admin accounts

---

## Troubleshooting

### "Account not found"
- Make sure you funded your testnet account
- Check the account address: `soroban keys address my-account`

### "Contract not initialized"
- Run the `initialize` function first before using other functions

### "Insufficient balance"
- Fund your account: `soroban keys fund my-account --network testnet`
- Check balance on [Stellar Expert](https://stellar.expert/explorer/testnet)

### "Invalid token address"
- Verify the token contract is deployed
- Ensure you're using the correct network

---

## Next Steps

1. ✅ Set up your `.env` file
2. ✅ Deploy your contract
3. ✅ Initialize the contract
4. ✅ Test with small amounts first
5. ✅ Build a frontend (optional)
6. ✅ Deploy to mainnet (when ready)

For more help, see:
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Laboratory](https://laboratory.stellar.org/)
- [Soroban Examples](https://github.com/stellar/soroban-examples)

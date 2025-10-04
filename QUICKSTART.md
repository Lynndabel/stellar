# Quick Start Guide - Time-Locked Savings Contract

Get your Soroban savings contract up and running in 5 minutes!

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) installed
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup) installed
- Basic command line knowledge

## 5-Minute Setup

### 1. Set Up Environment Variables

```bash
# Copy the example env file
cp .env.example .env
```

### 2. Create and Fund Accounts (Automated)

```bash
# Make scripts executable (Linux/Mac)
chmod +x scripts/*.sh

# Create accounts and get testnet funds
./scripts/setup-accounts.sh testnet
```

This will:
- ✅ Create deployer, admin, and test user accounts
- ✅ Fund them with testnet XLM
- ✅ Display secret keys to add to your `.env`

**Copy the output and paste into your `.env` file!**

### 3. Deploy the Contract (Automated)

```bash
# Deploy and initialize the contract
./scripts/deploy.sh
```

This will:
- ✅ Build the contract
- ✅ Deploy to testnet
- ✅ Wrap native XLM as a token
- ✅ Initialize the contract
- ✅ Update your `.env` with contract details

### 4. Test the Contract

```bash
# Create a savings goal
soroban contract invoke \
  --id $(grep CONTRACT_ID .env | cut -d '=' -f2) \
  --source deployer-account \
  --network testnet \
  -- \
  create_goal \
  --owner $(soroban keys address deployer-account) \
  --amount 10000000 \
  --lock_duration 86400 \
  --interest_rate 500

# Check your goal
soroban contract invoke \
  --id $(grep CONTRACT_ID .env | cut -d '=' -f2) \
  --source deployer-account \
  --network testnet \
  -- \
  get_goal \
  --owner $(soroban keys address deployer-account) \
  --goal_id 0
```

## Manual Setup (Windows or Alternative)

If the scripts don't work on your system, follow these manual steps:

### 1. Create Accounts

```bash
# Create deployer account
soroban keys generate deployer-account --network testnet
soroban keys fund deployer-account --network testnet

# Create admin account
soroban keys generate admin-account --network testnet
soroban keys fund admin-account --network testnet

# Get secret keys
soroban keys show deployer-account
soroban keys show admin-account
```

Add to `.env`:
```env
STELLAR_SECRET_KEY=<deployer-secret-key>
ADMIN_SECRET_KEY=<admin-secret-key>
```

### 2. Build and Deploy

```bash
# Build
cd contracts/hello-world
cargo build --release --target wasm32-unknown-unknown

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hello_world.wasm \
  --source deployer-account \
  --network testnet
```

Copy the CONTRACT_ID to your `.env`

### 3. Set Up Token

```bash
# Wrap native XLM
soroban lab token wrap \
  --asset native \
  --source deployer-account \
  --network testnet
```

Copy the TOKEN_ADDRESS to your `.env`

### 4. Initialize Contract

```bash
soroban contract invoke \
  --id <YOUR_CONTRACT_ID> \
  --source deployer-account \
  --network testnet \
  -- \
  initialize \
  --token <TOKEN_ADDRESS> \
  --admin $(soroban keys address admin-account) \
  --emergency_penalty 1000
```

## What's Next?

### Test Different Features

**Create a savings goal:**
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer-account \
  --network testnet \
  -- \
  create_goal \
  --owner $(soroban keys address deployer-account) \
  --amount 10000000 \
  --lock_duration 86400 \
  --interest_rate 500
```

**Check current balance (with pending interest):**
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer-account \
  --network testnet \
  -- \
  get_current_balance \
  --owner $(soroban keys address deployer-account) \
  --goal_id 0
```

**Compound interest manually:**
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer-account \
  --network testnet \
  -- \
  compound_interest \
  --owner $(soroban keys address deployer-account) \
  --goal_id 0
```

**Emergency withdrawal (with penalty):**
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer-account \
  --network testnet \
  -- \
  emergency_withdraw \
  --owner $(soroban keys address deployer-account) \
  --goal_id 0
```

**Normal withdrawal (after lock period):**
```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer-account \
  --network testnet \
  -- \
  withdraw \
  --owner $(soroban keys address deployer-account) \
  --goal_id 0
```

### Build a Frontend

Consider building a web interface using:
- **Next.js** + **@stellar/stellar-sdk**
- **React** + **@soroban-react**
- **Vue** + **stellar-sdk**

### Deploy to Mainnet

When ready for production:
1. Change network to `mainnet` in `.env`
2. Use real accounts with proper security
3. Audit your contract code
4. Test thoroughly on testnet first
5. Deploy with `./scripts/deploy.sh`

## Troubleshooting

**"command not found: soroban"**
- Install Soroban CLI: `cargo install --locked soroban-cli`

**"Account not found"**
- Make sure you funded the account: `soroban keys fund <account> --network testnet`

**"Contract not initialized"**
- Run the initialize command before using other functions

**Scripts won't run (Windows)**
- Use Git Bash or WSL
- Or follow the manual setup steps

## Resources

- 📚 [Full Setup Guide](./ENV_SETUP_GUIDE.md)
- 📖 [Soroban Docs](https://soroban.stellar.org/docs)
- 🔬 [Stellar Laboratory](https://laboratory.stellar.org/)
- 💬 [Stellar Discord](https://discord.gg/stellar)

---

**Happy Building! 🚀**

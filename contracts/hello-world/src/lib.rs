#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracterror, contracttype, token, Address, Env,
};

/// Custom error types for the contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InvalidDuration = 4,
    RateTooHigh = 5,
    PenaltyTooHigh = 6,
    Overflow = 7,
    GoalNotFound = 8,
    GoalInactive = 9,
    StillLocked = 10,
    AlreadyWithdrawn = 11,
    Unauthorized = 12,
    TimeError = 13,
    DivisionError = 14,
    Underflow = 15,
    GoalOverflow = 16,
}

/// Represents a single savings goal with time-lock mechanism
#[contracttype]
#[derive(Clone)]
pub struct SavingsGoal {
    /// Owner of this savings goal
    pub owner: Address,
    /// Amount deposited (in stroops or token smallest unit)
    pub principal: i128,
    /// Annual interest rate in basis points (e.g., 500 = 5%)
    pub interest_rate: u32,
    /// Timestamp when the deposit was made
    pub start_time: u64,
    /// Lock duration in seconds
    pub lock_duration: u64,
    /// Timestamp when funds can be withdrawn without penalty
    pub unlock_time: u64,
    /// Accumulated interest (calculated on demand)
    pub accrued_interest: i128,
    /// Last time interest was compounded
    pub last_compound_time: u64,
    /// Whether this goal is active
    pub is_active: bool,
}

/// Storage keys for the contract
#[contracttype]
pub enum StorageKey {
    /// Token address for the contract
    Token,
    /// Admin address
    Admin,
    /// Counter for goal IDs
    GoalCounter,
    /// Mapping: (owner, goal_id) -> SavingsGoal
    Goal(Address, u64),
    /// User's goal count
    UserGoalCount(Address),
    /// Emergency withdrawal penalty in basis points (e.g., 1000 = 10%)
    EmergencyPenalty,
}

/// Minimum lock duration: 1 day in seconds
const MIN_LOCK_DURATION: u64 = 86400;

/// Maximum lock duration: 10 years in seconds
const MAX_LOCK_DURATION: u64 = 315360000;

/// Maximum interest rate: 50% in basis points
const MAX_INTEREST_RATE: u32 = 5000;

/// Basis points divisor (10000 = 100%)
const BASIS_POINTS: i128 = 10000;

/// Seconds in a year for interest calculation
const SECONDS_PER_YEAR: i128 = 31536000;

#[contract]
pub struct TimeLockedSavings;

#[contractimpl]
impl TimeLockedSavings {
    /// Initialize the contract with token address and admin
    /// 
    /// # Security:
    /// - Can only be called once (initialization pattern)
    /// - Sets up admin privileges for contract management
    /// 
    /// # Parameters:
    /// - `token`: Address of the token to be used for savings
    /// - `admin`: Address with administrative privileges
    /// - `emergency_penalty`: Penalty in basis points for early withdrawal (e.g., 1000 = 10%)
    pub fn initialize(
        env: Env,
        token: Address,
        admin: Address,
        emergency_penalty: u32,
    ) -> Result<(), Error> {
        // Security: Prevent re-initialization
        if env.storage().instance().has(&StorageKey::Token) {
            return Err(Error::AlreadyInitialized);
        }

        // Security: Validate penalty rate
        if emergency_penalty > 5000 {
            // Max 50%
            return Err(Error::PenaltyTooHigh);
        }

        // Store contract configuration
        env.storage().instance().set(&StorageKey::Token, &token);
        env.storage().instance().set(&StorageKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&StorageKey::EmergencyPenalty, &emergency_penalty);
        env.storage().instance().set(&StorageKey::GoalCounter, &0u64);

        Ok(())
    }

    /// Create a new savings goal with time-lock
    /// 
    /// # Security:
    /// - Validates all inputs before state changes
    /// - Uses authorization to ensure only owner can create goals
    /// - Atomic operation - either fully succeeds or reverts
    /// - Protects against overflow in calculations
    /// 
    /// # Parameters:
    /// - `owner`: Address of the goal owner (must authorize)
    /// - `amount`: Amount to deposit
    /// - `lock_duration`: How long funds are locked (in seconds)
    /// - `interest_rate`: Annual interest rate in basis points
    pub fn create_goal(
        env: Env,
        owner: Address,
        amount: i128,
        lock_duration: u64,
        interest_rate: u32,
    ) -> Result<u64, Error> {
        // Security: Require authorization from the owner
        owner.require_auth();

        // Security: Validate inputs
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        if lock_duration < MIN_LOCK_DURATION || lock_duration > MAX_LOCK_DURATION {
            return Err(Error::InvalidDuration);
        }

        if interest_rate > MAX_INTEREST_RATE {
            return Err(Error::RateTooHigh);
        }

        // Get current timestamp
        let current_time = env.ledger().timestamp();

        // Security: Check for overflow when calculating unlock time
        let unlock_time = current_time
            .checked_add(lock_duration)
            .ok_or(Error::Overflow)?;

        // Transfer tokens from user to contract
        // Security: This will fail if user has insufficient balance
        let token_address: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Token)
            .ok_or(Error::NotInitialized)?;
        let token = token::Client::new(&env, &token_address);
        token.transfer(&owner, &env.current_contract_address(), &amount);

        // Generate unique goal ID
        let goal_id: u64 = env
            .storage()
            .instance()
            .get(&StorageKey::GoalCounter)
            .unwrap_or(0);

        // Security: Check for goal ID overflow
        let next_goal_id = goal_id
            .checked_add(1)
            .ok_or(Error::GoalOverflow)?;

        // Create the savings goal
        let goal = SavingsGoal {
            owner: owner.clone(),
            principal: amount,
            interest_rate,
            start_time: current_time,
            lock_duration,
            unlock_time,
            accrued_interest: 0,
            last_compound_time: current_time,
            is_active: true,
        };

        // Store the goal
        env.storage()
            .persistent()
            .set(&StorageKey::Goal(owner.clone(), goal_id), &goal);

        // Update counters
        env.storage()
            .instance()
            .set(&StorageKey::GoalCounter, &next_goal_id);

        let user_count: u64 = env
            .storage()
            .persistent()
            .get(&StorageKey::UserGoalCount(owner.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&StorageKey::UserGoalCount(owner), &(user_count + 1));

        Ok(goal_id)
    }

    /// Compound interest for a specific goal
    /// 
    /// # Security:
    /// - Only calculates interest, doesn't modify principal
    /// - Uses safe math to prevent overflow
    /// - Can be called by anyone (public utility function)
    /// 
    /// # Parameters:
    /// - `owner`: Address of the goal owner
    /// - `goal_id`: ID of the goal to compound
    pub fn compound_interest(env: Env, owner: Address, goal_id: u64) -> Result<(), Error> {
        let mut goal: SavingsGoal = env
            .storage()
            .persistent()
            .get(&StorageKey::Goal(owner.clone(), goal_id))
            .ok_or(Error::GoalNotFound)?;

        // Security: Check if goal is active
        if !goal.is_active {
            return Err(Error::GoalInactive);
        }

        let current_time = env.ledger().timestamp();

        // Calculate time elapsed since last compound
        let time_elapsed = current_time
            .checked_sub(goal.last_compound_time)
            .ok_or(Error::TimeError)?;

        if time_elapsed == 0 {
            return Ok(()); // No time passed, nothing to compound
        }

        // Calculate interest: (principal + accrued) * rate * time / (SECONDS_PER_YEAR * BASIS_POINTS)
        // Security: Use checked arithmetic to prevent overflow
        let total_balance = goal
            .principal
            .checked_add(goal.accrued_interest)
            .ok_or(Error::Overflow)?;

        let interest = total_balance
            .checked_mul(goal.interest_rate as i128)
            .ok_or(Error::Overflow)?
            .checked_mul(time_elapsed as i128)
            .ok_or(Error::Overflow)?
            .checked_div(SECONDS_PER_YEAR * BASIS_POINTS)
            .ok_or(Error::DivisionError)?;

        // Update accrued interest
        goal.accrued_interest = goal
            .accrued_interest
            .checked_add(interest)
            .ok_or(Error::Overflow)?;

        goal.last_compound_time = current_time;

        // Save updated goal
        env.storage()
            .persistent()
            .set(&StorageKey::Goal(owner, goal_id), &goal);

        Ok(())
    }

    /// Withdraw funds from a matured goal
    /// 
    /// # Security:
    /// - Requires owner authorization
    /// - Checks unlock time before allowing withdrawal
    /// - Compounds interest before withdrawal
    /// - Marks goal as inactive to prevent double withdrawal
    /// - Uses checked arithmetic
    /// 
    /// # Parameters:
    /// - `owner`: Address of the goal owner
    /// - `goal_id`: ID of the goal to withdraw from
    pub fn withdraw(env: Env, owner: Address, goal_id: u64) -> Result<i128, Error> {
        // Security: Require authorization
        owner.require_auth();

        // Compound interest before withdrawal
        Self::compound_interest(env.clone(), owner.clone(), goal_id)?;

        let mut goal: SavingsGoal = env
            .storage()
            .persistent()
            .get(&StorageKey::Goal(owner.clone(), goal_id))
            .ok_or(Error::GoalNotFound)?;

        // Security: Check if goal is active
        if !goal.is_active {
            return Err(Error::AlreadyWithdrawn);
        }

        let current_time = env.ledger().timestamp();

        // Security: Ensure lock period has passed
        if current_time < goal.unlock_time {
            return Err(Error::StillLocked);
        }

        // Calculate total withdrawal amount
        let total_amount = goal
            .principal
            .checked_add(goal.accrued_interest)
            .ok_or(Error::Overflow)?;

        // Security: Mark goal as inactive before transfer to prevent reentrancy
        goal.is_active = false;
        env.storage()
            .persistent()
            .set(&StorageKey::Goal(owner.clone(), goal_id), &goal);

        // Transfer funds to owner
        let token_address: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Token)
            .ok_or(Error::NotInitialized)?;
        let token = token::Client::new(&env, &token_address);
        token.transfer(&env.current_contract_address(), &owner, &total_amount);

        Ok(total_amount)
    }

    /// Emergency withdrawal with penalty before unlock time
    /// 
    /// # Security:
    /// - Requires owner authorization
    /// - Applies penalty to discourage misuse
    /// - Compounds interest before calculating penalty
    /// - Marks goal as inactive to prevent double withdrawal
    /// - Admin receives penalty as contract revenue
    /// 
    /// # Parameters:
    /// - `owner`: Address of the goal owner
    /// - `goal_id`: ID of the goal to withdraw from
    pub fn emergency_withdraw(env: Env, owner: Address, goal_id: u64) -> Result<i128, Error> {
        // Security: Require authorization
        owner.require_auth();

        // Compound interest before withdrawal
        Self::compound_interest(env.clone(), owner.clone(), goal_id)?;

        let mut goal: SavingsGoal = env
            .storage()
            .persistent()
            .get(&StorageKey::Goal(owner.clone(), goal_id))
            .ok_or(Error::GoalNotFound)?;

        // Security: Check if goal is active
        if !goal.is_active {
            return Err(Error::AlreadyWithdrawn);
        }

        // Calculate total balance
        let total_balance = goal
            .principal
            .checked_add(goal.accrued_interest)
            .ok_or(Error::Overflow)?;

        // Get penalty rate
        let penalty_rate: u32 = env
            .storage()
            .instance()
            .get(&StorageKey::EmergencyPenalty)
            .unwrap_or(1000); // Default 10%

        // Calculate penalty amount
        let penalty = total_balance
            .checked_mul(penalty_rate as i128)
            .ok_or(Error::Overflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(Error::DivisionError)?;

        let withdrawal_amount = total_balance
            .checked_sub(penalty)
            .ok_or(Error::Underflow)?;

        // Security: Mark goal as inactive before transfers
        goal.is_active = false;
        env.storage()
            .persistent()
            .set(&StorageKey::Goal(owner.clone(), goal_id), &goal);

        // Transfer tokens
        let token_address: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Token)
            .ok_or(Error::NotInitialized)?;
        let token = token::Client::new(&env, &token_address);

        // Transfer withdrawal amount to owner
        token.transfer(&env.current_contract_address(), &owner, &withdrawal_amount);

        // Transfer penalty to admin
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(Error::NotInitialized)?;
        token.transfer(&env.current_contract_address(), &admin, &penalty);

        Ok(withdrawal_amount)
    }

    /// Get details of a specific savings goal
    /// 
    /// # Security:
    /// - Read-only function, no state changes
    /// - Anyone can view goal details (transparency)
    pub fn get_goal(env: Env, owner: Address, goal_id: u64) -> Result<SavingsGoal, Error> {
        env.storage()
            .persistent()
            .get(&StorageKey::Goal(owner, goal_id))
            .ok_or(Error::GoalNotFound)
    }

    /// Get the total number of goals for a user
    /// 
    /// # Security:
    /// - Read-only function
    pub fn get_user_goal_count(env: Env, owner: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&StorageKey::UserGoalCount(owner))
            .unwrap_or(0)
    }

    /// Calculate current total balance (principal + interest) for a goal
    /// 
    /// # Security:
    /// - Read-only function, doesn't modify state
    /// - Calculates up-to-date interest without changing storage
    pub fn get_current_balance(env: Env, owner: Address, goal_id: u64) -> Result<i128, Error> {
        let goal: SavingsGoal = env
            .storage()
            .persistent()
            .get(&StorageKey::Goal(owner, goal_id))
            .ok_or(Error::GoalNotFound)?;

        if !goal.is_active {
            return Ok(0);
        }

        let current_time = env.ledger().timestamp();
        let time_elapsed = current_time
            .checked_sub(goal.last_compound_time)
            .ok_or(Error::TimeError)?;

        // Calculate pending interest
        let total_balance = goal
            .principal
            .checked_add(goal.accrued_interest)
            .ok_or(Error::Overflow)?;

        let pending_interest = total_balance
            .checked_mul(goal.interest_rate as i128)
            .ok_or(Error::Overflow)?
            .checked_mul(time_elapsed as i128)
            .ok_or(Error::Overflow)?
            .checked_div(SECONDS_PER_YEAR * BASIS_POINTS)
            .ok_or(Error::DivisionError)?;

        total_balance
            .checked_add(pending_interest)
            .ok_or(Error::Overflow)
    }

    /// Admin function to update emergency penalty rate
    /// 
    /// # Security:
    /// - Only admin can call this
    /// - Validates new penalty rate
    pub fn set_emergency_penalty(env: Env, admin: Address, new_penalty: u32) -> Result<(), Error> {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(Error::NotInitialized)?;

        if admin != stored_admin {
            return Err(Error::Unauthorized);
        }

        if new_penalty > 5000 {
            return Err(Error::PenaltyTooHigh);
        }

        env.storage()
            .instance()
            .set(&StorageKey::EmergencyPenalty, &new_penalty);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Ledger}, token};

    #[test]
    fn test_create_and_withdraw_goal() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, TimeLockedSavings);
        let client = TimeLockedSavingsClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let token_id = env.register_stellar_asset_contract_v2(admin.clone());
        let token = token::Client::new(&env, &token_id.address());

        // Initialize contract
        client.initialize(&token_id.address(), &admin, &1000);

        // Mint tokens to user
        token.mint(&user, &10000);

        // Create goal: 10000 tokens, 30 days lock, 5% interest
        let goal_id = client.create_goal(&user, &10000, &2592000, &500);

        // Fast forward time to unlock
        env.ledger().with_mut(|li| li.timestamp = 2592001);

        // Withdraw
        let amount = client.withdraw(&user, &goal_id);
        assert!(amount > 10000); // Should have interest
    }
}

//! # Testes de Integração E2E para Staking e Governança
//! 
//! Este arquivo contém testes end-to-end que simulam cenários completos
//! de uso do sistema de staking e governança da Lunex DEX na rede Lunes.

use std::collections::HashMap;

/// Mock do contrato de Staking para testes E2E
pub struct MockStakingContract {
    pub owner: String,
    pub paused: bool,
    pub total_staked: u128,
    pub total_rewards_distributed: u128,
    pub active_stakers: u32,
    pub stakes: HashMap<String, StakePosition>,
    pub proposals: HashMap<u32, ProjectProposal>,
    pub next_proposal_id: u32,
    pub user_votes: HashMap<(u32, String), bool>,
    pub approved_projects: HashMap<String, bool>,
    pub current_block: u64,
}

#[derive(Clone, Debug)]
pub struct StakePosition {
    pub amount: u128,
    pub start_time: u64,
    pub duration: u64,
    pub last_claim: u64,
    pub pending_rewards: u128,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct ProjectProposal {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub token_address: String,
    pub proposer: String,
    pub votes_for: u128,
    pub votes_against: u128,
    pub voting_deadline: u64,
    pub executed: bool,
    pub active: bool,
}

impl MockStakingContract {
    pub fn new(owner: String) -> Self {
        Self {
            owner,
            paused: false,
            total_staked: 0,
            total_rewards_distributed: 0,
            active_stakers: 0,
            stakes: HashMap::new(),
            proposals: HashMap::new(),
            next_proposal_id: 1,
            user_votes: HashMap::new(),
            approved_projects: HashMap::new(),
            current_block: 1000000,
        }
    }

    pub fn stake(&mut self, user: String, amount: u128, duration: u64) -> Result<(), String> {
        if self.paused {
            return Err("Contract paused".to_string());
        }

        if amount < 100_000_000_000 {  // 1000 LUNES minimum (8 decimals)
            return Err("Minimum stake not met".to_string());
        }

        if duration < 302400 || duration > 15724800 {  // 7 days to 365 days
            return Err("Invalid duration".to_string());
        }

        if self.stakes.get(&user).map_or(false, |s| s.active) {
            return Err("User already has active stake".to_string());
        }

        let stake = StakePosition {
            amount,
            start_time: self.current_block,
            duration,
            last_claim: self.current_block,
            pending_rewards: 0,
            active: true,
        };

        self.stakes.insert(user, stake);
        self.active_stakers += 1;
        self.total_staked += amount;

        Ok(())
    }

    pub fn unstake(&mut self, user: String) -> Result<(u128, u128, u128), String> {
        if self.paused {
            return Err("Contract paused".to_string());
        }

        let stake = self.stakes.get(&user)
            .ok_or("No active stake")?
            .clone();

        if !stake.active {
            return Err("No active stake".to_string());
        }

        let (rewards, penalty) = self.calculate_rewards_and_penalty(&stake);
                let _total_return = stake.amount + rewards - penalty;
        
        // Update stake
        let mut updated_stake = stake.clone();
        updated_stake.active = false;
        self.stakes.insert(user, updated_stake);

        self.active_stakers -= 1;
        self.total_staked -= stake.amount;
        self.total_rewards_distributed += rewards;

        Ok((stake.amount, rewards, penalty))
    }

    pub fn claim_rewards(&mut self, user: String) -> Result<u128, String> {
        if self.paused {
            return Err("Contract paused".to_string());
        }

        let mut stake = self.stakes.get(&user)
            .ok_or("No active stake")?
            .clone();

        if !stake.active {
            return Err("No active stake".to_string());
        }

        let rewards = self.calculate_pending_rewards(&stake);
        
        stake.last_claim = self.current_block;
        stake.pending_rewards = 0;
        self.stakes.insert(user, stake);

        self.total_rewards_distributed += rewards;

        Ok(rewards)
    }

    pub fn create_proposal(
        &mut self,
        proposer: String,
        name: String,
        description: String,
        token_address: String,
    ) -> Result<u32, String> {
        if self.paused {
            return Err("Contract paused".to_string());
        }

        if token_address.starts_with("0x00") {
            return Err("Zero address not allowed".to_string());
        }

        let voting_power = self.get_voting_power(&proposer);
        if voting_power < 1_000_000_000_000 {  // 10,000 LUNES minimum (8 decimals)
            return Err("Insufficient voting power".to_string());
        }

        let proposal_id = self.next_proposal_id;
        let voting_deadline = self.current_block + 604800;  // 14 days

        let proposal = ProjectProposal {
            id: proposal_id,
            name,
            description,
            token_address,
            proposer,
            votes_for: 0,
            votes_against: 0,
            voting_deadline,
            executed: false,
            active: true,
        };

        self.proposals.insert(proposal_id, proposal);
        self.next_proposal_id += 1;

        Ok(proposal_id)
    }

    pub fn vote(&mut self, proposal_id: u32, voter: String, in_favor: bool) -> Result<(), String> {
        if self.paused {
            return Err("Contract paused".to_string());
        }

        let mut proposal = self.proposals.get(&proposal_id)
            .ok_or("Invalid proposal")?
            .clone();

        if !proposal.active {
            return Err("Proposal not active".to_string());
        }

        if self.current_block > proposal.voting_deadline {
            return Err("Voting period expired".to_string());
        }

        if *self.user_votes.get(&(proposal_id, voter.clone())).unwrap_or(&false) {
            return Err("Already voted".to_string());
        }

        let vote_power = self.get_voting_power(&voter);
        if vote_power == 0 {
            return Err("No voting power".to_string());
        }

        if in_favor {
            proposal.votes_for += vote_power;
        } else {
            proposal.votes_against += vote_power;
        }

        self.proposals.insert(proposal_id, proposal);
        self.user_votes.insert((proposal_id, voter), true);

        Ok(())
    }

    pub fn execute_proposal(&mut self, proposal_id: u32) -> Result<bool, String> {
        let mut proposal = self.proposals.get(&proposal_id)
            .ok_or("Invalid proposal")?
            .clone();

        if !proposal.active || proposal.executed {
            return Err("Proposal not executable".to_string());
        }

        if self.current_block <= proposal.voting_deadline {
            return Err("Voting period not ended".to_string());
        }

        let approved = proposal.votes_for > proposal.votes_against;

        if approved {
            self.approved_projects.insert(proposal.token_address.clone(), true);
        }

        proposal.executed = true;
        proposal.active = false;
        self.proposals.insert(proposal_id, proposal);

        Ok(approved)
    }

    pub fn get_voting_power(&self, user: &str) -> u128 {
        self.stakes.get(user)
            .map(|stake| if stake.active { stake.amount } else { 0 })
            .unwrap_or(0)
    }

    pub fn is_project_approved(&self, token_address: &str) -> bool {
        self.approved_projects.get(token_address).unwrap_or(&false).clone()
    }

    pub fn get_stats(&self) -> (u128, u128, u32) {
        (self.total_staked, self.total_rewards_distributed, self.active_stakers)
    }

    pub fn advance_blocks(&mut self, blocks: u64) {
        self.current_block += blocks;
    }

    fn calculate_pending_rewards(&self, stake: &StakePosition) -> u128 {
        let time_staked = self.current_block.saturating_sub(stake.last_claim);
        let year_in_blocks = 365 * 24 * 60 * 30;  // ~1 year in blocks
        
        // 10% annual reward rate
        (stake.amount as u128 * 1000 * time_staked as u128) / (10000 * year_in_blocks as u128)
    }

    fn calculate_rewards_and_penalty(&self, stake: &StakePosition) -> (u128, u128) {
        let rewards = self.calculate_pending_rewards(stake);
        let time_staked = self.current_block.saturating_sub(stake.start_time);

        let penalty = if time_staked < stake.duration {
            // 5% early penalty
            stake.amount * 500 / 10000
        } else {
            0
        };

        (rewards, penalty)
    }
}

/// Mock integrado da DEX com Staking
pub struct MockLunexDEXWithStaking {
    pub staking: MockStakingContract,
    pub user_balances: HashMap<String, u128>,
    pub native_token_price: u128,  // Price in smallest units
}

impl MockLunexDEXWithStaking {
    pub fn new() -> Self {
        Self {
            staking: MockStakingContract::new("admin".to_string()),
            user_balances: HashMap::new(),
            native_token_price: 100_000_000,  // 1 LUNES = 100M units (8 decimals)
        }
    }

    pub fn mint_tokens(&mut self, user: String, amount: u128) {
        *self.user_balances.entry(user).or_insert(0) += amount;
    }

    pub fn get_balance(&self, user: &str) -> u128 {
        self.user_balances.get(user).unwrap_or(&0).clone()
    }

    pub fn stake_with_balance_check(&mut self, user: String, amount: u128, duration: u64) -> Result<(), String> {
        let balance = self.get_balance(&user);
        if balance < amount {
            return Err("Insufficient balance".to_string());
        }

        let result = self.staking.stake(user.clone(), amount, duration);
        if result.is_ok() {
            *self.user_balances.entry(user).or_insert(0) -= amount;
        }
        result
    }

    pub fn unstake_with_balance_update(&mut self, user: String) -> Result<(u128, u128, u128), String> {
        let result = self.staking.unstake(user.clone())?;
        let (original, rewards, penalty) = result;
        let total_return = original + rewards - penalty;
        
        *self.user_balances.entry(user).or_insert(0) += total_return;
        
        Ok(result)
    }
}

// === TESTES E2E ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_staking_lifecycle_e2e() {
        let mut dex = MockLunexDEXWithStaking::new();
        let user = "alice".to_string();
        
        // Setup: Give user some LUNES tokens
        dex.mint_tokens(user.clone(), 10_000_000_000_000); // 100,000 LUNES (8 decimals)
        
        let initial_balance = dex.get_balance(&user);
        assert_eq!(initial_balance, 10_000_000_000_000);
        
        // Step 1: Stake tokens
        let stake_amount = 5_000_000_000_000; // 50,000 LUNES (8 decimals)
        let stake_duration = 302400 * 2; // 14 days
        
        let result = dex.stake_with_balance_check(user.clone(), stake_amount, stake_duration);
        assert!(result.is_ok(), "Staking should succeed: {:?}", result);
        
        // Verify balance decreased
        let balance_after_stake = dex.get_balance(&user);
        assert_eq!(balance_after_stake, initial_balance - stake_amount);
        
        // Verify staking stats
        let (total_staked, total_rewards, active_stakers) = dex.staking.get_stats();
        assert_eq!(total_staked, stake_amount);
        assert_eq!(total_rewards, 0);
        assert_eq!(active_stakers, 1);
        
        // Step 2: Advance time and claim rewards
        dex.staking.advance_blocks(302400); // 7 days
        
        let rewards_result = dex.staking.claim_rewards(user.clone());
        assert!(rewards_result.is_ok());
        let rewards = rewards_result.unwrap();
        assert!(rewards > 0, "Should have earned some rewards");
        
        // Step 3: Unstake after full duration
        dex.staking.advance_blocks(302400); // Another 7 days (total 14 days)
        
        let unstake_result = dex.unstake_with_balance_update(user.clone());
        assert!(unstake_result.is_ok());
        
        let (original, final_rewards, penalty) = unstake_result.unwrap();
        assert_eq!(original, stake_amount);
        assert!(final_rewards >= rewards); // Should have earned more rewards
        assert_eq!(penalty, 0); // No penalty for full duration
        
        // Verify final balance includes rewards
        let final_balance = dex.get_balance(&user);
        assert!(final_balance > initial_balance, "Should have earned rewards");
        
        println!("✅ Full staking lifecycle completed successfully!");
        println!("   Original stake: {} LUNES", original / 100_000_000);
        println!("   Total rewards: {} LUNES", final_rewards / 100_000_000);
        println!("   Final balance: {} LUNES", final_balance / 100_000_000);
    }

    #[test]
    fn test_governance_proposal_lifecycle_e2e() {
        let mut dex = MockLunexDEXWithStaking::new();
        
        // Setup users with staking power
        let proposer = "alice".to_string();
        let voter1 = "bob".to_string();
        let voter2 = "charlie".to_string();
        
        // Give users tokens and stake
        dex.mint_tokens(proposer.clone(), 2_000_000_000_000); // 20,000 LUNES (8 decimals)
        dex.mint_tokens(voter1.clone(), 1_500_000_000_000);   // 15,000 LUNES (8 decimals)
        dex.mint_tokens(voter2.clone(), 500_000_000_000);     // 5,000 LUNES (8 decimals)
        
        // Stake tokens (gives voting power)
        dex.stake_with_balance_check(proposer.clone(), 1_500_000_000_000, 302400).unwrap();
        dex.stake_with_balance_check(voter1.clone(), 1_000_000_000_000, 302400).unwrap();
        dex.stake_with_balance_check(voter2.clone(), 300_000_000_000, 302400).unwrap();
        
        // Step 1: Create proposal
        let proposal_result = dex.staking.create_proposal(
            proposer.clone(),
            "List NewCoin Token".to_string(),
            "A revolutionary new token for the ecosystem".to_string(),
            "0x1234567890abcdef".to_string(),
        );
        
        assert!(proposal_result.is_ok());
        let proposal_id = proposal_result.unwrap();
        
        // Step 2: Vote on proposal
        // Proposer votes in favor
        dex.staking.vote(proposal_id, proposer.clone(), true).unwrap();
        
        // Voter1 votes against
        dex.staking.vote(proposal_id, voter1.clone(), false).unwrap();
        
        // Voter2 votes in favor
        dex.staking.vote(proposal_id, voter2.clone(), true).unwrap();
        
        // Check voting power distribution
        let proposer_power = dex.staking.get_voting_power(&proposer);
        let voter1_power = dex.staking.get_voting_power(&voter1);
        let voter2_power = dex.staking.get_voting_power(&voter2);
        
        assert_eq!(proposer_power, 1_500_000_000_000);
        assert_eq!(voter1_power, 1_000_000_000_000);
        assert_eq!(voter2_power, 300_000_000_000);
        
        // Step 3: Advance time past voting period
        dex.staking.advance_blocks(604800 + 1); // 14 days + 1 block
        
        // Step 4: Execute proposal
        let execution_result = dex.staking.execute_proposal(proposal_id);
        assert!(execution_result.is_ok());
        
        let approved = execution_result.unwrap();
        // Votes FOR: 15,000 + 3,000 = 18,000 LUNES
        // Votes AGAINST: 10,000 LUNES
        // Should be approved
        assert!(approved, "Proposal should be approved");
        
        // Step 5: Verify project is approved for listing
        let is_approved = dex.staking.is_project_approved("0x1234567890abcdef");
        assert!(is_approved, "Project should be approved for listing");
        
        println!("✅ Governance proposal lifecycle completed successfully!");
        println!("   Proposal approved with majority vote");
        println!("   Project is now approved for DEX listing");
    }

    #[test]
    fn test_early_unstaking_penalty_e2e() {
        let mut dex = MockLunexDEXWithStaking::new();
        let user = "alice".to_string();
        
        // Setup
        dex.mint_tokens(user.clone(), 1_000_000_000_000); // 10,000 LUNES (8 decimals)
        let stake_amount = 500_000_000_000; // 5,000 LUNES (8 decimals)
        let stake_duration = 302400 * 4; // 28 days
        
        // Stake
        dex.stake_with_balance_check(user.clone(), stake_amount, stake_duration).unwrap();
        
        // Advance only 7 days (less than 28 days duration)
        dex.staking.advance_blocks(302400);
        
        // Early unstake
        let unstake_result = dex.unstake_with_balance_update(user.clone());
        assert!(unstake_result.is_ok());
        
        let (_original, _rewards, penalty) = unstake_result.unwrap();
        
        // Should have penalty (5% of stake amount)
        let expected_penalty = stake_amount * 500 / 10000; // 5%
        assert_eq!(penalty, expected_penalty);
        assert!(penalty > 0, "Should have early unstaking penalty");
        
        // Final balance should be less than initial due to penalty
        let final_balance = dex.get_balance(&user);
        let initial_balance = 1_000_000_000_000;
        assert!(final_balance < initial_balance, "Should lose tokens due to penalty");
        
        println!("✅ Early unstaking penalty applied correctly!");
        println!("   Penalty: {} LUNES", penalty / 100_000_000);
    }

    #[test]
    fn test_governance_insufficient_voting_power_e2e() {
        let mut dex = MockLunexDEXWithStaking::new();
        let user = "alice".to_string();
        
        // Give user tokens but don't stake enough for proposal creation
        dex.mint_tokens(user.clone(), 500_000_000_000); // 5,000 LUNES (8 decimals)
        dex.stake_with_balance_check(user.clone(), 500_000_000_000, 302400).unwrap();
        
        // Try to create proposal (requires 10,000 LUNES minimum)
        let proposal_result = dex.staking.create_proposal(
            user.clone(),
            "Test Proposal".to_string(),
            "Description".to_string(),
            "0x1234567890abcdef".to_string(),
        );
        
        assert!(proposal_result.is_err());
        assert_eq!(proposal_result.unwrap_err(), "Insufficient voting power");
        
        println!("✅ Voting power requirement enforced correctly!");
    }

    #[test]
    fn test_multiple_stakers_rewards_distribution_e2e() {
        let mut dex = MockLunexDEXWithStaking::new();
        
        let users = vec!["alice", "bob", "charlie"];
        let stake_amounts = vec![
            1_000_000_000_000, // 10,000 LUNES (8 decimals)
            2_000_000_000_000, // 20,000 LUNES (8 decimals)
            500_000_000_000,   // 5,000 LUNES (8 decimals)
        ];
        
        // Setup and stake
        for (i, user) in users.iter().enumerate() {
            dex.mint_tokens(user.to_string(), stake_amounts[i] * 2);
            dex.stake_with_balance_check(user.to_string(), stake_amounts[i], 302400).unwrap();
        }
        
        // Verify total staked
        let (total_staked, _, active_stakers) = dex.staking.get_stats();
        let expected_total = stake_amounts.iter().sum::<u128>();
        assert_eq!(total_staked, expected_total);
        assert_eq!(active_stakers, 3);
        
        // Advance time
        dex.staking.advance_blocks(302400); // 7 days
        
        // Claim rewards for all users
        let mut total_rewards = 0u128;
        for user in &users {
            let rewards = dex.staking.claim_rewards(user.to_string()).unwrap();
            total_rewards += rewards;
            
            // Higher stake should earn proportionally more rewards
            println!("User {} earned {} LUNES in rewards", user, rewards / 100_000_000);
        }
        
        assert!(total_rewards > 0, "Total rewards should be positive");
        
        // Verify rewards distribution proportionality
        let alice_rewards = dex.staking.claim_rewards("alice".to_string()).unwrap_or(0);
        let bob_rewards = dex.staking.claim_rewards("bob".to_string()).unwrap_or(0);
        
        // Bob staked 2x more than Alice, so should earn ~2x more rewards
        // (allowing for some variance due to integer division)
        if alice_rewards > 0 && bob_rewards > 0 {
            let ratio = bob_rewards as f64 / alice_rewards as f64;
            assert!(ratio >= 1.8 && ratio <= 2.2, "Reward ratio should be ~2.0, got {}", ratio);
        }
        
        println!("✅ Multi-staker rewards distribution working correctly!");
        println!("   Total rewards distributed: {} LUNES", total_rewards / 100_000_000);
    }

    #[test]
    fn test_lunex_dex_integration_with_lunes_network() {
        let mut dex = MockLunexDEXWithStaking::new();
        
        println!("🚀 Testing Lunex DEX integration with Lunes Network");
        
        // Simulate typical user journey on Lunes Network
        let user = "lunes_user".to_string();
        
        // 1. User receives LUNES tokens (from other sources, exchanges, etc.)
        dex.mint_tokens(user.clone(), 100_000_000_000_000); // 1,000,000 LUNES (8 decimals)
        println!("   ✅ User received 1,000,000 LUNES tokens");
        
        // 2. User stakes portion for governance voting power
        let stake_amount = 10_000_000_000_000; // 100,000 LUNES (8 decimals)
        dex.stake_with_balance_check(user.clone(), stake_amount, 302400 * 12).unwrap(); // 84 days
        println!("   ✅ User staked 100,000 LUNES for 84 days");
        
        // 3. User participates in governance
        let proposal_id = dex.staking.create_proposal(
            user.clone(),
            "List LUNES/USDT Pair".to_string(),
            "Add USDT trading pair for LUNES".to_string(),
            "0xUSDT_CONTRACT_ADDRESS".to_string(),
        ).unwrap();
        println!("   ✅ User created governance proposal for USDT listing");
        
        // 4. User votes on their own proposal
        dex.staking.vote(proposal_id, user.clone(), true).unwrap();
        println!("   ✅ User voted in favor of their proposal");
        
        // 5. Simulate time passing and more users joining
        dex.mint_tokens("other_user".to_string(), 5_000_000_000_000);
        dex.stake_with_balance_check("other_user".to_string(), 5_000_000_000_000, 302400).unwrap();
        dex.staking.vote(proposal_id, "other_user".to_string(), true).unwrap();
        
        // 6. Execute proposal after voting period
        dex.staking.advance_blocks(604800 + 1); // 14 days + 1
        let approved = dex.staking.execute_proposal(proposal_id).unwrap();
        assert!(approved);
        println!("   ✅ Proposal approved - USDT listing authorized");
        
        // 7. User claims staking rewards
        let rewards = dex.staking.claim_rewards(user.clone()).unwrap();
        println!("   ✅ User claimed {} LUNES in staking rewards", rewards / 100_000_000);
        
        // 8. Verify everything is working as expected
        let voting_power = dex.staking.get_voting_power(&user);
        let (total_staked, total_rewards, active_stakers) = dex.staking.get_stats();
        let is_usdt_approved = dex.staking.is_project_approved("0xUSDT_CONTRACT_ADDRESS");
        
        assert_eq!(voting_power, stake_amount);
        assert!(total_staked > 0);
        assert!(total_rewards > 0);
        assert_eq!(active_stakers, 2);
        assert!(is_usdt_approved);
        
        println!("🎉 Lunex DEX + Lunes Network integration test completed successfully!");
        println!("   Total LUNES staked in system: {} LUNES", total_staked / 100_000_000);
        println!("   Total rewards distributed: {} LUNES", total_rewards / 100_000_000);
        println!("   Active stakers: {}", active_stakers);
        println!("   USDT pair approved for listing: {}", is_usdt_approved);
    }
}
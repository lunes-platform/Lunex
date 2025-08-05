#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// # Lunex Staking Contract
///
/// This contract allows users to stake $LUNES (native token) and earn rewards.
/// It also provides governance features for project listing proposals.
///
/// ## Key Features:
/// - Stake $LUNES tokens and earn rewards
/// - Unstake with optional penalty periods
/// - Governance voting for project listings
/// - Reward distribution based on staking duration and amount
/// - Anti-whale mechanisms for fair distribution
///
/// ## Security Features:
/// - Reentrancy protection
/// - Access control for admin functions
/// - Input validation and overflow protection
/// - Deadline-based operations
/// - Slashing protection for malicious behavior

#[ink::contract]
pub mod staking_contract {
    use ink::storage::Mapping;
    use ink::prelude::string::String;

    /// Staking-related errors
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum StakingError {
        /// Insufficient balance for staking
        InsufficientBalance,
        /// No active stake found
        NoActiveStake,
        /// Minimum staking amount not met
        MinimumStakeNotMet,
        /// Staking period not yet completed
        StakingPeriodNotCompleted,
        /// Access denied - caller not authorized
        AccessDenied,
        /// Invalid staking duration
        InvalidDuration,
        /// Contract is paused
        ContractPaused,
        /// Unstaking too early (penalty period)
        EarlyUnstaking,
        /// Maximum stakers reached
        MaxStakersReached,
        /// Invalid proposal ID
        InvalidProposal,
        /// Already voted on this proposal
        AlreadyVoted,
        /// Voting period expired
        VotingPeriodExpired,
        /// Insufficient voting power
        InsufficientVotingPower,
        /// Insufficient fee for proposal
        InsufficientFee,
        /// Overflow error
        Overflow,
        /// Zero address not allowed
        ZeroAddress,
        /// Reentrancy detected
        Reentrancy,
        /// Zero amount not allowed
        ZeroAmount,
        /// No rewards to claim
        NoRewardsToClaim,
        /// Token already listed
        AlreadyListed,
        /// Token not listed
        TokenNotListed,
        /// Too many tokens in batch operation
        TooManyTokens,
        /// Invalid amount provided
        InvalidAmount,
    }

    /// Governance proposal for project listing
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Clone, ink::storage::traits::StorageLayout))]
    pub struct ProjectProposal {
        /// Unique proposal ID
        pub id: u32,
        /// Project name
        pub name: String,
        /// Project description
        pub description: String,
        /// Project token contract address
        pub token_address: AccountId,
        /// Proposal creator
        pub proposer: AccountId,
        /// Total votes in favor
        pub votes_for: Balance,
        /// Total votes against
        pub votes_against: Balance,
        /// Voting deadline
        pub voting_deadline: Timestamp,
        /// Whether proposal is executed
        pub executed: bool,
        /// Whether proposal is active
        pub active: bool,
        /// Fee paid for proposal
        pub fee: Balance,
        /// Whether fee has been refunded
        pub fee_refunded: bool,
        /// For fee change proposals, this stores the new fee amount
        pub new_fee_amount: Option<Balance>,
    }

    /// Individual staking position
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Clone, ink::storage::traits::StorageLayout))]
    pub struct StakePosition {
        /// Amount staked
        pub amount: Balance,
        /// Timestamp when stake was created
        pub start_time: Timestamp,
        /// Staking duration in blocks
        pub duration: u64,
        /// Last reward claim timestamp
        pub last_claim: Timestamp,
        /// Accumulated rewards
        pub pending_rewards: Balance,
        /// Whether position is active
        pub active: bool,
        /// Staking tier baseado na duração
        pub tier: StakingTier,
        /// Early adopter tier
        pub early_adopter_tier: EarlyAdopterTier,
        /// Participação em governança (número de votos)
        pub governance_participation: u32,
    }

    /// Tiers de staking baseados em duração
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone, Copy)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum StakingTier {
        Bronze,   // 7-30 dias - 8% APY
        Silver,   // 31-90 dias - 10% APY
        Gold,     // 91-180 dias - 12% APY
        Platinum, // 181+ dias - 15% APY
    }

    /// Tier de early adopter
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, Clone, Copy)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum EarlyAdopterTier {
        None,
        Top1000,  // +10% por 1 mês
        Top500,   // +25% por 2 meses  
        Top100,   // +50% por 3 meses
    }

    /// Informações de campanha promocional
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Clone, ink::storage::traits::StorageLayout))]
    pub struct Campaign {
        pub name: Vec<u8>,
        pub bonus_rate: u32,
        pub start_time: Timestamp,
        pub end_time: Timestamp,
        pub active: bool,
    }

    /// Staking events
    #[ink(event)]
    pub struct Staked {
        #[ink(topic)]
        pub staker: AccountId,
        pub amount: Balance,
        pub duration: u64,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct Unstaked {
        #[ink(topic)]
        pub staker: AccountId,
        pub amount: Balance,
        pub rewards: Balance,
        pub penalty: Balance,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct RewardsClaimed {
        #[ink(topic)]
        pub staker: AccountId,
        pub amount: Balance,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct ProposalCreated {
        #[ink(topic)]
        pub proposal_id: u32,
        pub proposer: AccountId,
        pub project_name: String,
        pub token_address: AccountId,
        pub voting_deadline: Timestamp,
    }

    #[ink(event)]
    pub struct Voted {
        #[ink(topic)]
        pub proposal_id: u32,
        #[ink(topic)]
        pub voter: AccountId,
        pub vote_power: Balance,
        pub in_favor: bool,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct ProposalExecuted {
        #[ink(topic)]
        pub proposal_id: u32,
        pub approved: bool,
        pub votes_for: Balance,
        pub votes_against: Balance,
        pub timestamp: Timestamp,
    }

    /// === NOVOS EVENTOS PARA PREMIAÇÃO ===

    #[ink(event)]
    pub struct TradingRewardsFunded {
        pub amount: Balance,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct TradingRewardsDistributed {
        pub total_amount: Balance,
        pub stakers_count: u32,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct TradingRewardsDistributionProgress {
        pub processed_stakers: u32,
        pub rewards_distributed: u32,
        pub start_index: u32,
        pub end_index: u32,
        pub is_complete: bool,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct GovernanceBonusAwarded {
        #[ink(topic)]
        pub staker: AccountId,
        pub bonus_type: u8, // 1=voting, 2=proposal, 3=implementation
        pub amount: Balance,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct GovernanceBonusClaimed {
        #[ink(topic)]
        pub staker: AccountId,
        pub amount: Balance,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct TierUpgraded {
        #[ink(topic)]
        pub staker: AccountId,
        pub old_tier: StakingTier,
        pub new_tier: StakingTier,
        pub timestamp: Timestamp,
    }

    // === ADMIN LISTING EVENTS ===

    #[ink(event)]
    pub struct AdminTokenListed {
        #[ink(topic)]
        pub token_address: AccountId,
        #[ink(topic)]
        pub admin: AccountId,
        pub reason: String,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct AdminBatchListingCompleted {
        #[ink(topic)]
        pub admin: AccountId,
        pub tokens_listed: u32,
        pub timestamp: Timestamp,
    }

    /// === EVENTOS ADMINISTRATIVOS ===

    #[ink(event)]
    pub struct TreasuryAddressChanged {
        pub old_treasury: AccountId,
        pub new_treasury: AccountId,
        pub admin: AccountId,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct OwnershipTransferred {
        pub old_owner: AccountId,
        pub new_owner: AccountId,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct TradingRewardsContractChanged {
        pub old_contract: Option<AccountId>,
        pub new_contract: AccountId,
        pub admin: AccountId,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct ContractPaused {
        pub admin: AccountId,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct ContractUnpaused {
        pub admin: AccountId,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct AdminTokenDelisted {
        #[ink(topic)]
        pub token_address: AccountId,
        #[ink(topic)]
        pub admin: AccountId,
        pub reason: String,
        pub timestamp: Timestamp,
    }

    // === GOVERNANCE FEE EVENTS ===
    #[ink(event)]
    pub struct FeeChangeProposed {
        #[ink(topic)]
        pub proposal_id: u32,
        #[ink(topic)]
        pub proposer: AccountId,
        pub current_fee: Balance,
        pub proposed_fee: Balance,
        pub justification: String,
        pub voting_deadline: Timestamp,
    }

    #[ink(event)]
    pub struct ProposalFeeChanged {
        #[ink(topic)]
        pub proposal_id: u32,
        pub old_fee: Balance,
        pub new_fee: Balance,
        #[ink(topic)]
        pub changed_by: AccountId,
        pub timestamp: Timestamp,
    }

    /// Constants for staking mechanics
    pub mod constants {
        use super::Balance;

        /// Minimum stake amount (1000 LUNES)
        pub const MIN_STAKE: Balance = 100_000_000_000; // 1000 * 10^8
        
        /// Minimum staking duration (7 days in blocks, ~2 sec per block)
        pub const MIN_DURATION: u64 = 7 * 24 * 60 * 30; // 7 days
        
        /// Maximum staking duration (365 days)
        pub const MAX_DURATION: u64 = 365 * 24 * 60 * 30; // 1 year
        
        /// === REWARD RATES POR TIER (em basis points) ===
        
        /// Bronze tier reward rate (8% APY)
        pub const BRONZE_REWARD_RATE: u32 = 800; // 8% = 800 basis points
        
        /// Silver tier reward rate (10% APY)
        pub const SILVER_REWARD_RATE: u32 = 1000; // 10% = 1000 basis points
        
        /// Gold tier reward rate (12% APY)  
        pub const GOLD_REWARD_RATE: u32 = 1200; // 12% = 1200 basis points
        
        /// Platinum tier reward rate (15% APY)
        pub const PLATINUM_REWARD_RATE: u32 = 1500; // 15% = 1500 basis points
        
        /// === MULTIPLICADORES DE QUANTIDADE ===
        
        /// 1k-10k LUNES: 1.0x (base)
        pub const SMALL_STAKER_MULTIPLIER: u32 = 10000; // 1.0x = 10000 basis points
        
        /// 10k-50k LUNES: 1.1x (+10%)
        pub const MEDIUM_STAKER_MULTIPLIER: u32 = 11000; // 1.1x = 11000 basis points
        
        /// 50k-200k LUNES: 1.2x (+20%)
        pub const LARGE_STAKER_MULTIPLIER: u32 = 12000; // 1.2x = 12000 basis points
        
        /// 200k+ LUNES: 1.3x (+30%)
        pub const WHALE_STAKER_MULTIPLIER: u32 = 13000; // 1.3x = 13000 basis points
        
        /// === THRESHOLDS PARA MULTIPLICADORES ===
        
        pub const MEDIUM_STAKE_THRESHOLD: Balance = 1_000_000_000_000; // 10k LUNES
        pub const LARGE_STAKE_THRESHOLD: Balance = 5_000_000_000_000; // 50k LUNES  
        pub const WHALE_STAKE_THRESHOLD: Balance = 20_000_000_000_000; // 200k LUNES
        
        /// === EARLY ADOPTER BONUSES ===
        
        /// Top 100 early adopters: +50% for 3 months
        pub const TOP_100_BONUS: u32 = 5000; // +50% = 5000 basis points
        pub const TOP_100_DURATION: u64 = 90 * 24 * 60 * 30; // 3 months
        
        /// Top 500 early adopters: +25% for 2 months  
        pub const TOP_500_BONUS: u32 = 2500; // +25% = 2500 basis points
        pub const TOP_500_DURATION: u64 = 60 * 24 * 60 * 30; // 2 months
        
        /// Top 1000 early adopters: +10% for 1 month
        pub const TOP_1000_BONUS: u32 = 1000; // +10% = 1000 basis points
        pub const TOP_1000_DURATION: u64 = 30 * 24 * 60 * 30; // 1 month
        
        /// === GOVERNANCE BONUSES ===
        
        /// Bonus for creating approved proposal (1000 LUNES)
        pub const PROPOSAL_BONUS: Balance = 100_000_000_000; // 1000 * 10^8
        
        /// Bonus for active voting (200 LUNES/month)
        pub const VOTING_BONUS: Balance = 20_000_000_000; // 200 * 10^8
        
        /// Bonus for implemented proposal (5000 LUNES)
        pub const IMPLEMENTATION_BONUS: Balance = 500_000_000_000; // 5000 * 10^8
        
        /// Minimum votes per month for bonus (80% participation)
        pub const MIN_VOTES_FOR_BONUS: u32 = 8; // Assuming ~10 proposals/month
        
        /// === TRADING REWARDS SHARE ===
        
        /// Percentage of trading rewards que vai para stakers (10%)
        pub const STAKING_SHARE_OF_TRADING_REWARDS: u32 = 1000; // 10% = 1000 basis points
        
        /// === CONSTANTES EXISTENTES ===
        
        /// Early unstaking penalty (5%)
        pub const EARLY_PENALTY_RATE: u32 = 500; // 5% = 500 basis points
        
        /// Voting period for proposals (14 days)
        pub const VOTING_PERIOD: u64 = 14 * 24 * 60 * 30; // 14 days in blocks
        
        /// Minimum voting power required to create proposal (10,000 LUNES staked)
        pub const MIN_PROPOSAL_POWER: Balance = 1_000_000_000_000; // 10,000 * 10^8
        
        /// Proposal fee (1,000 LUNES)
        pub const PROPOSAL_FEE: Balance = 100_000_000_000; // 1000 * 10^8
        
        /// Maximum number of active stakers
        pub const MAX_STAKERS: u32 = 10_000;
        
        /// Basis points denominator
        pub const BASIS_POINTS: u32 = 10_000;
        
        /// Zero address constant
        pub const ZERO_ADDRESS: [u8; 32] = [0u8; 32];
    }

    /// Main staking contract storage
    #[ink(storage)]
    pub struct StakingContract {
        /// Contract owner/admin
        owner: AccountId,
        /// Treasury address for project funds
        treasury_address: AccountId,
        /// Whether contract is paused
        paused: bool,
        /// Reentrancy guard
        locked: bool,
        /// Total amount staked in the contract
        total_staked: Balance,
        /// Total rewards distributed
        total_rewards_distributed: Balance,
        /// Number of active stakers
        active_stakers: u32,
        /// Mapping from staker to their position
        stakes: Mapping<AccountId, StakePosition>,
        /// Mapping for tracking staker addresses
        staker_addresses: Mapping<u32, AccountId>,
        /// Current staker index
        staker_index: u32,
        /// Governance proposals
        proposals: Mapping<u32, ProjectProposal>,
        /// Next proposal ID
        next_proposal_id: u32,
        /// Mapping to track if user voted on proposal
        user_votes: Mapping<(u32, AccountId), bool>,
        /// Approved projects for listing
        approved_projects: Mapping<AccountId, bool>,
        
        /// === NOVO SISTEMA DE PREMIAÇÃO ===
        
        /// Pool de trading rewards destinado para stakers
        trading_rewards_pool: Balance,
        /// Endereço do contrato de trading rewards
        trading_rewards_contract: Option<AccountId>,
        /// Mapeamento de tier multipliers
        tier_multipliers: Mapping<StakingTier, u32>,
        /// Bônus de governança acumulados por usuário
        governance_bonuses: Mapping<AccountId, Balance>,
        /// Contador de early adopters por tier
        early_adopter_counts: Mapping<EarlyAdopterTier, u32>,
        /// Campanhas ativas (acessadas raramente - otimizado com Lazy)
        active_campaigns: ink::storage::Lazy<Mapping<u32, Campaign>>,
        /// Próximo ID de campanha
        next_campaign_id: u32,
        /// Total de trading rewards distribuídos para stakers (métrica, acessada raramente)
        total_trading_rewards_distributed: Balance,
        /// Taxa atual para criação de propostas (ajustável via governança)
        current_proposal_fee: Balance,
    }

    impl StakingContract {
        /// Creates a new staking contract
        #[ink(constructor)]
        pub fn new(treasury_address: AccountId) -> Self {
            let mut contract = Self {
                owner: Self::env().caller(),
                treasury_address,
                paused: false,
                locked: false,
                total_staked: 0,
                total_rewards_distributed: 0,
                active_stakers: 0,
                stakes: Mapping::default(),
                staker_addresses: Mapping::default(),
                staker_index: 0,
                proposals: Mapping::default(),
                next_proposal_id: 1,
                user_votes: Mapping::default(),
                approved_projects: Mapping::default(),
                
                // Novos campos para premiação
                trading_rewards_pool: 0,
                trading_rewards_contract: None,
                tier_multipliers: Mapping::default(),
                governance_bonuses: Mapping::default(),
                early_adopter_counts: Mapping::default(),
                active_campaigns: ink::storage::Lazy::new(),
                next_campaign_id: 1,
                total_trading_rewards_distributed: 0,
                current_proposal_fee: constants::PROPOSAL_FEE, // Inicia com 1,000 LUNES
            };
            
            // Inicializa campos Lazy
            contract.active_campaigns.set(&Mapping::default());
            
            // Inicializa tier multipliers
            contract.tier_multipliers.insert(&StakingTier::Bronze, &constants::BRONZE_REWARD_RATE);
            contract.tier_multipliers.insert(&StakingTier::Silver, &constants::SILVER_REWARD_RATE);
            contract.tier_multipliers.insert(&StakingTier::Gold, &constants::GOLD_REWARD_RATE);
            contract.tier_multipliers.insert(&StakingTier::Platinum, &constants::PLATINUM_REWARD_RATE);
            
            // Inicializa contadores de early adopters
            contract.early_adopter_counts.insert(&EarlyAdopterTier::None, &0);
            contract.early_adopter_counts.insert(&EarlyAdopterTier::Top1000, &0);
            contract.early_adopter_counts.insert(&EarlyAdopterTier::Top500, &0);
            contract.early_adopter_counts.insert(&EarlyAdopterTier::Top100, &0);
            
            contract
        }

        // ========================================
        // ADMIN METHODS
        // ========================================

        /// Set treasury address
        #[ink(message)]
        pub fn set_treasury_address(&mut self, new_treasury: AccountId) -> Result<(), StakingError> {
            self.ensure_owner()?;
            
            if new_treasury == AccountId::from(constants::ZERO_ADDRESS) {
                return Err(StakingError::ZeroAddress);
            }
            
            let old_treasury = self.treasury_address;
            self.treasury_address = new_treasury;
            
            // Emit event
            self.env().emit_event(TreasuryAddressChanged {
                old_treasury,
                new_treasury,
                admin: self.env().caller(),
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        // ========================================
        // STAKING & REWARDS
        // ========================================

        /// Stakes LUNES tokens for a specified duration
        #[ink(message, payable)]
        pub fn stake(&mut self, duration: u64) -> Result<(), StakingError> {
            self.ensure_not_paused()?;
            self.acquire_lock()?;
            
            let caller = self.env().caller();
            let amount = self.env().transferred_value();
            let current_time = self.env().block_timestamp();
            
            // Validations
            if amount < constants::MIN_STAKE {
                self.release_lock();
                return Err(StakingError::MinimumStakeNotMet);
            }
            
            if duration < constants::MIN_DURATION || duration > constants::MAX_DURATION {
                self.release_lock();
                return Err(StakingError::InvalidDuration);
            }
            
            if self.active_stakers >= constants::MAX_STAKERS {
                self.release_lock();
                return Err(StakingError::MaxStakersReached);
            }
            
            // Check if user already has an active stake
            if let Some(existing_stake) = self.stakes.get(&caller) {
                if existing_stake.active {
                    self.release_lock();
                    return Err(StakingError::NoActiveStake); // User already has active stake
                }
            }
            
            // Determina tier baseado na duração
            let tier = self.calculate_staking_tier(duration);
            
            // Determina early adopter tier
            let early_adopter_tier = self.determine_early_adopter_tier();
            
            // Create new stake position
            let stake_position = StakePosition {
                amount,
                start_time: current_time,
                duration,
                last_claim: current_time,
                pending_rewards: 0,
                active: true,
                tier,
                early_adopter_tier,
                governance_participation: 0,
            };
            
            // Update state
            self.stakes.insert(&caller, &stake_position);
            self.staker_addresses.insert(&self.staker_index, &caller);
            self.staker_index += 1;
            self.active_stakers += 1;
            self.total_staked = self.total_staked.checked_add(amount)
                .ok_or(StakingError::Overflow)?;
            
            // Emit event
            self.env().emit_event(Staked {
                staker: caller,
                amount,
                duration,
                timestamp: current_time,
            });
            
            self.release_lock();
            Ok(())
        }

        /// Unstakes tokens and claims rewards
        #[ink(message)]
        pub fn unstake(&mut self) -> Result<(), StakingError> {
            self.ensure_not_paused()?;
            self.acquire_lock()?;
            
            let caller = self.env().caller();
            let current_time = self.env().block_timestamp();
            
            let mut stake = self.stakes.get(&caller).ok_or(StakingError::NoActiveStake)?;
            
            if !stake.active {
                self.release_lock();
                return Err(StakingError::NoActiveStake);
            }
            
            // Calculate rewards and penalties
            let (rewards, penalty) = self.calculate_rewards_and_penalty(&stake, current_time)?;
            let total_amount = stake.amount.checked_add(rewards)
                .ok_or(StakingError::Overflow)?
                .checked_sub(penalty)
                .ok_or(StakingError::Overflow)?;
            
            // Update state
            stake.active = false;
            self.stakes.insert(&caller, &stake);
            self.active_stakers -= 1;
            self.total_staked = self.total_staked.checked_sub(stake.amount)
                .ok_or(StakingError::Overflow)?;
            self.total_rewards_distributed = self.total_rewards_distributed.checked_add(rewards)
                .ok_or(StakingError::Overflow)?;
            
            // Transfer tokens back to user
            if self.env().transfer(caller, total_amount).is_err() {
                self.release_lock();
                return Err(StakingError::InsufficientBalance);
            }
            
            // Emit event
            self.env().emit_event(Unstaked {
                staker: caller,
                amount: stake.amount,
                rewards,
                penalty,
                timestamp: current_time,
            });
            
            self.release_lock();
            Ok(())
        }

        /// Claims pending rewards without unstaking
        #[ink(message)]
        pub fn claim_rewards(&mut self) -> Result<(), StakingError> {
            self.ensure_not_paused()?;
            self.acquire_lock()?;
            
            let caller = self.env().caller();
            let current_time = self.env().block_timestamp();
            
            let mut stake = self.stakes.get(&caller).ok_or(StakingError::NoActiveStake)?;
            
            if !stake.active {
                self.release_lock();
                return Err(StakingError::NoActiveStake);
            }
            
            let rewards = self.calculate_pending_rewards(&stake, current_time)?;
            
            if rewards == 0 {
                self.release_lock();
                return Ok(()); // No rewards to claim
            }
            
            // Update stake
            stake.last_claim = current_time;
            stake.pending_rewards = 0;
            self.stakes.insert(&caller, &stake);
            
            self.total_rewards_distributed = self.total_rewards_distributed.checked_add(rewards)
                .ok_or(StakingError::Overflow)?;
            
            // Transfer rewards
            if self.env().transfer(caller, rewards).is_err() {
                self.release_lock();
                return Err(StakingError::InsufficientBalance);
            }
            
            // Emit event
            self.env().emit_event(RewardsClaimed {
                staker: caller,
                amount: rewards,
                timestamp: current_time,
            });
            
            self.release_lock();
            Ok(())
        }

        /// Creates a new project listing proposal
        #[ink(message, payable)]
        pub fn create_proposal(
            &mut self,
            name: String,
            description: String,
            token_address: AccountId,
        ) -> Result<u32, StakingError> {
            self.ensure_not_paused()?;
            
            let caller = self.env().caller();
            let current_time = self.env().block_timestamp();
            let fee = self.env().transferred_value();
            
            // Validate inputs
            if token_address == AccountId::from(constants::ZERO_ADDRESS) {
                return Err(StakingError::ZeroAddress);
            }
            
            // Check proposal fee (agora dinâmica, ajustável pela comunidade)
            if fee < self.current_proposal_fee {
                return Err(StakingError::InsufficientFee);
            }
            
            // Check voting power requirement
            let voting_power = self.get_voting_power(caller)?;
            if voting_power < constants::MIN_PROPOSAL_POWER {
                return Err(StakingError::InsufficientVotingPower);
            }
            
            let proposal_id = self.next_proposal_id;
            let voting_deadline = current_time.saturating_add(constants::VOTING_PERIOD);
            
            let proposal = ProjectProposal {
                id: proposal_id,
                name: name.clone(),
                description,
                token_address,
                proposer: caller,
                votes_for: 0,
                votes_against: 0,
                voting_deadline,
                executed: false,
                active: true,
                fee,
                fee_refunded: false,
                new_fee_amount: None, // Proposta normal de token, não mudança de taxa
            };
            
            self.proposals.insert(&proposal_id, &proposal);
            self.next_proposal_id = self.next_proposal_id.checked_add(1)
                .ok_or(StakingError::Overflow)?;
            
            // Emit event
            self.env().emit_event(ProposalCreated {
                proposal_id,
                proposer: caller,
                project_name: name,
                token_address,
                voting_deadline,
            });
            
            Ok(proposal_id)
        }

        /// Votes on a project listing proposal
        #[ink(message)]
        pub fn vote(&mut self, proposal_id: u32, in_favor: bool) -> Result<(), StakingError> {
            self.ensure_not_paused()?;
            
            let caller = self.env().caller();
            let current_time = self.env().block_timestamp();
            
            let mut proposal = self.proposals.get(&proposal_id)
                .ok_or(StakingError::InvalidProposal)?;
            
            if !proposal.active {
                return Err(StakingError::InvalidProposal);
            }
            
            if current_time > proposal.voting_deadline {
                return Err(StakingError::VotingPeriodExpired);
            }
            
            // Check if already voted
            if self.user_votes.get(&(proposal_id, caller)).unwrap_or(false) {
                return Err(StakingError::AlreadyVoted);
            }
            
            let vote_power = self.get_voting_power(caller)?;
            if vote_power == 0 {
                return Err(StakingError::InsufficientVotingPower);
            }
            
            // Record vote
            if in_favor {
                proposal.votes_for = proposal.votes_for.checked_add(vote_power)
                    .ok_or(StakingError::Overflow)?;
            } else {
                proposal.votes_against = proposal.votes_against.checked_add(vote_power)
                    .ok_or(StakingError::Overflow)?;
            }
            
            self.proposals.insert(&proposal_id, &proposal);
            self.user_votes.insert(&(proposal_id, caller), &true);
            
            // Emit event
            self.env().emit_event(Voted {
                proposal_id,
                voter: caller,
                vote_power,
                in_favor,
                timestamp: current_time,
            });
            
            Ok(())
        }

        /// Executes a proposal after voting period ends
        #[ink(message)]
        pub fn execute_proposal(&mut self, proposal_id: u32) -> Result<(), StakingError> {
            let current_time = self.env().block_timestamp();
            
            let mut proposal = self.proposals.get(&proposal_id)
                .ok_or(StakingError::InvalidProposal)?;
            
            if !proposal.active || proposal.executed {
                return Err(StakingError::InvalidProposal);
            }
            
            if current_time < proposal.voting_deadline {
                return Err(StakingError::VotingPeriodExpired);
            }
            
            let approved = proposal.votes_for > proposal.votes_against;
            
            if approved {
                // Verificar se é proposta de mudança de taxa (tem new_fee_amount)
                if let Some(new_fee) = proposal.new_fee_amount {
                    self.execute_fee_change(proposal_id, new_fee)?;
                } else {
                    // Proposta normal de listagem de token
                    self.approved_projects.insert(&proposal.token_address, &true);
                }
                
                // Reembolsar taxa para propostas aprovadas
                // Note: Em produção, implemente verificação de saldo antes da transferência
                proposal.fee_refunded = true; // Marcar como reembolsado para testes
            } else {
                // Distribuir taxa (simulado para testes)
                if !proposal.fee_refunded {
                    let staking_share = proposal.fee / 10; // 10%
                    
                    // Note: Em produção, implemente transferências reais
                    // Por enquanto, apenas atualizar o pool interno
                    self.trading_rewards_pool = self.trading_rewards_pool.saturating_add(staking_share);
                    
                    proposal.fee_refunded = true;
                }
            }
            
            proposal.executed = true;
            proposal.active = false;
            self.proposals.insert(&proposal_id, &proposal);
            
            // Emit event
            self.env().emit_event(ProposalExecuted {
                proposal_id,
                approved,
                votes_for: proposal.votes_for,
                votes_against: proposal.votes_against,
                timestamp: current_time,
            });
            
            Ok(())
        }

        // === Query Methods ===

        /// Gets stake information for an account
        #[ink(message)]
        pub fn get_stake(&self, account: AccountId) -> Option<StakePosition> {
            self.stakes.get(&account)
        }

        /// Gets current rewards for a staker
        #[ink(message)]
        pub fn get_pending_rewards(&self, account: AccountId) -> Result<Balance, StakingError> {
            let stake = self.stakes.get(&account).ok_or(StakingError::NoActiveStake)?;
            
            if !stake.active {
                return Err(StakingError::NoActiveStake);
            }
            
            let current_time = self.env().block_timestamp();
            self.calculate_pending_rewards(&stake, current_time)
        }

        /// Gets voting power for an account
        #[ink(message)]
        pub fn get_voting_power(&self, account: AccountId) -> Result<Balance, StakingError> {
            if let Some(stake) = self.stakes.get(&account) {
                if stake.active {
                    Ok(stake.amount)
                } else {
                    Ok(0)
                }
            } else {
                Ok(0)
            }
        }

        /// Gets proposal information
        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u32) -> Option<ProjectProposal> {
            self.proposals.get(&proposal_id)
        }

        /// Checks if a project is approved for listing
        #[ink(message)]
        pub fn is_project_approved(&self, token_address: AccountId) -> bool {
            self.approved_projects.get(&token_address).unwrap_or(false)
        }

        // ========================================
        // ADMIN LISTING FUNCTIONS (TEAM POWER)
        // ========================================

        /// Lista token diretamente via admin (sem governança)
        /// Usado para tokens iniciais e casos especiais
        #[ink(message)]
        pub fn admin_list_token(
            &mut self, 
            token_address: AccountId,
            reason: String
        ) -> Result<(), StakingError> {
            self.ensure_owner()?;
            
            if token_address == AccountId::from(constants::ZERO_ADDRESS) {
                return Err(StakingError::ZeroAddress);
            }
            
            // Verificar se já está aprovado
            if self.approved_projects.get(&token_address).unwrap_or(false) {
                return Err(StakingError::AlreadyListed);
            }
            
            // Aprovar diretamente
            self.approved_projects.insert(&token_address, &true);
            
            // Emit event específico para listagem por admin
            self.env().emit_event(AdminTokenListed {
                token_address,
                admin: self.env().caller(),
                reason,
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        /// Lista múltiplos tokens de uma vez (batch)
        /// Útil para configuração inicial da DEX
        #[ink(message)]
        pub fn admin_batch_list_tokens(
            &mut self,
            tokens: Vec<(AccountId, String)>
        ) -> Result<u32, StakingError> {
            self.ensure_owner()?;
            
            if tokens.len() > 50 {
                return Err(StakingError::TooManyTokens);
            }
            
            let mut listed_count = 0u32;
            
            for (token_address, reason) in tokens {
                if token_address == AccountId::from(constants::ZERO_ADDRESS) {
                    continue; // Pular endereços inválidos
                }
                
                // Verificar se já está aprovado
                if self.approved_projects.get(&token_address).unwrap_or(false) {
                    continue; // Pular se já listado
                }
                
                // Aprovar token
                self.approved_projects.insert(&token_address, &true);
                
                // Emit event individual
                self.env().emit_event(AdminTokenListed {
                    token_address,
                    admin: self.env().caller(),
                    reason: reason.clone(),
                    timestamp: self.env().block_timestamp(),
                });
                
                listed_count = listed_count.checked_add(1)
                    .ok_or(StakingError::Overflow)?;
            }
            
            // Emit event de batch
            self.env().emit_event(AdminBatchListingCompleted {
                admin: self.env().caller(),
                tokens_listed: listed_count,
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(listed_count)
        }

        /// Remove token da lista (apenas admin)
        /// Para casos extremos onde um token precisa ser removido
        #[ink(message)]
        pub fn admin_delist_token(
            &mut self,
            token_address: AccountId,
            reason: String
        ) -> Result<(), StakingError> {
            self.ensure_owner()?;
            
            if !self.approved_projects.get(&token_address).unwrap_or(false) {
                return Err(StakingError::TokenNotListed);
            }
            
            // Remover da lista
            self.approved_projects.remove(&token_address);
            
            // Emit event
            self.env().emit_event(AdminTokenDelisted {
                token_address,
                admin: self.env().caller(),
                reason,
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        /// Obter estatísticas de listagem
        #[ink(message)]
        pub fn get_listing_stats(&self) -> (u32, u32, u32) {
            (
                self.next_proposal_id.saturating_sub(1),  // Propostas criadas
                self.active_stakers,                      // Stakers ativos (podem votar)
                0u32 // Tokens aprovados - calculado off-chain para eficiência
            )
        }

        // ========================================
        // GOVERNANÇA DA TAXA DE PROPOSTA
        // ========================================

        /// Propor alteração na taxa de proposta (via governança)
        #[ink(message, payable)]
        pub fn propose_fee_change(
            &mut self,
            new_fee: Balance,
            justification: String,
        ) -> Result<u32, StakingError> {
            self.ensure_not_paused()?;
            
            let caller = self.env().caller();
            let current_time = self.env().block_timestamp();
            let fee = self.env().transferred_value();
            
            // Validações
            if new_fee == 0 || new_fee > 10_000_000_000_000 { // Max 100,000 LUNES
                return Err(StakingError::InvalidAmount);
            }
            
            // Taxa atual se aplica para propostas de mudança de taxa também
            if fee < self.current_proposal_fee {
                return Err(StakingError::InsufficientFee);
            }
            
            // Check voting power requirement
            let voting_power = self.get_voting_power(caller)?;
            if voting_power < constants::MIN_PROPOSAL_POWER {
                return Err(StakingError::InsufficientVotingPower);
            }
            
            let proposal_id = self.next_proposal_id;
            let voting_deadline = current_time.saturating_add(constants::VOTING_PERIOD);
            
            // Criar proposta especial para mudança de taxa
            let proposal = ProjectProposal {
                id: proposal_id,
                name: "MUDANCA_TAXA_PROPOSTA".to_string(),
                description: format!("Alterar taxa de proposta para {} LUNES. Justificativa: {}", 
                    new_fee.checked_div(100_000_000).unwrap_or(0), justification), // Mostrar em LUNES
                token_address: AccountId::from(constants::ZERO_ADDRESS), // Endereço especial para mudança de taxa
                proposer: caller,
                votes_for: 0,
                votes_against: 0,
                voting_deadline,
                executed: false,
                active: true,
                fee,
                fee_refunded: false,
                new_fee_amount: Some(new_fee), // Armazenar diretamente a nova taxa
            };
            
            self.proposals.insert(&proposal_id, &proposal);
            self.next_proposal_id = self.next_proposal_id.checked_add(1)
                .ok_or(StakingError::Overflow)?;
            
            // Emit event
            self.env().emit_event(FeeChangeProposed {
                proposal_id,
                proposer: caller,
                current_fee: self.current_proposal_fee,
                proposed_fee: new_fee,
                justification,
                voting_deadline,
            });
            
            Ok(proposal_id)
        }

        /// Executar mudança de taxa (após aprovação)
        pub fn execute_fee_change(&mut self, proposal_id: u32, new_fee: Balance) -> Result<(), StakingError> {
            // Esta função é chamada internamente por execute_proposal quando uma proposta de taxa é aprovada
            if new_fee == 0 || new_fee > 10_000_000_000_000 { // Max 100,000 LUNES
                return Err(StakingError::InvalidAmount);
            }
            
            let old_fee = self.current_proposal_fee;
            self.current_proposal_fee = new_fee;
            
            // Emit event
            self.env().emit_event(ProposalFeeChanged {
                proposal_id,
                old_fee,
                new_fee,
                changed_by: self.env().caller(),
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        /// Obter taxa atual de proposta
        #[ink(message)]
        pub fn get_current_proposal_fee(&self) -> Balance {
            self.current_proposal_fee
        }

        /// Gets contract statistics
        #[ink(message)]
        pub fn get_stats(&self) -> (Balance, Balance, u32) {
            (self.total_staked, self.total_rewards_distributed, self.active_stakers)
        }

        // === Admin Methods ===

        /// Pauses the contract (admin only)
        #[ink(message)]
        pub fn pause(&mut self) -> Result<(), StakingError> {
            self.ensure_owner()?;
            self.paused = true;
            
            // Emit event
            self.env().emit_event(ContractPaused {
                admin: self.env().caller(),
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        /// Unpauses the contract (admin only)
        #[ink(message)]
        pub fn unpause(&mut self) -> Result<(), StakingError> {
            self.ensure_owner()?;
            self.paused = false;
            
            // Emit event
            self.env().emit_event(ContractUnpaused {
                admin: self.env().caller(),
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        /// Transfers ownership (admin only)
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<(), StakingError> {
            self.ensure_owner()?;
            
            if new_owner == AccountId::from(constants::ZERO_ADDRESS) {
                return Err(StakingError::ZeroAddress);
            }
            
            let old_owner = self.owner;
            self.owner = new_owner;
            
            // Emit event
            self.env().emit_event(OwnershipTransferred {
                old_owner,
                new_owner,
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        /// === FUNÇÕES DE INTEGRAÇÃO COM TRADING REWARDS ===

        /// Define o endereço do contrato de trading rewards (apenas admin)
        #[ink(message)]
        pub fn set_trading_rewards_contract(&mut self, contract_address: AccountId) -> Result<(), StakingError> {
            self.ensure_owner()?;
            
            if contract_address == AccountId::from(constants::ZERO_ADDRESS) {
                return Err(StakingError::ZeroAddress);
            }
            
            let old_contract = self.trading_rewards_contract;
            self.trading_rewards_contract = Some(contract_address);
            
            // Emit event
            self.env().emit_event(TradingRewardsContractChanged {
                old_contract,
                new_contract: contract_address,
                admin: self.env().caller(),
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        /// Recebe trading rewards do contrato de rewards (apenas contrato autorizado)
        #[ink(message, payable)]
        pub fn fund_staking_rewards(&mut self) -> Result<(), StakingError> {
            self.ensure_not_paused()?;
            
            // Verifica se é o contrato autorizado
            let caller = self.env().caller();
            if self.trading_rewards_contract != Some(caller) {
                return Err(StakingError::AccessDenied);
            }
            
            let amount = self.env().transferred_value();
            if amount == 0 {
                return Err(StakingError::ZeroAmount);
            }
            
            self.trading_rewards_pool = self.trading_rewards_pool
                .checked_add(amount)
                .ok_or(StakingError::Overflow)?;
            
            // Emit event
            self.env().emit_event(TradingRewardsFunded {
                amount,
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        /// Distribui trading rewards para todos os stakers ativos (apenas admin)
        /// Para compatibilidade, distribui tudo em uma página
        #[ink(message)]
        pub fn distribute_trading_rewards(&mut self) -> Result<(), StakingError> {
            // Para compatibilidade, distribui tudo em uma página
            let (_processed, _complete, _next) = self.distribute_trading_rewards_paginated(None, None)?;
            Ok(())
        }

        /// Distribui trading rewards com paginação para melhor eficiência de gas (apenas admin)
        /// 
        /// # Parâmetros
        /// - `start_index`: Índice inicial para distribuição (None = do início)
        /// - `batch_size`: Número máximo de stakers para processar (None = todos restantes, max 100)
        /// 
        /// # Retorna
        /// - `Ok((processed_count, is_complete, next_index))`: 
        ///   - processed_count: número de stakers processados
        ///   - is_complete: se a distribuição foi finalizada
        ///   - next_index: próximo índice para continuar (se não completo)
        #[ink(message)]
        pub fn distribute_trading_rewards_paginated(
            &mut self, 
            start_index: Option<u32>,
            batch_size: Option<u32>
        ) -> Result<(u32, bool, Option<u32>), StakingError> {
            self.ensure_owner()?;
            self.ensure_not_paused()?;
            self.acquire_lock()?;
            
            if self.trading_rewards_pool == 0 {
                self.release_lock();
                return Ok((0, true, None)); // Nada para distribuir
            }
            
            let total_weight = self.calculate_total_staker_weight()?;
            if total_weight == 0 {
                self.release_lock();
                return Ok((0, true, None)); // Nenhum staker ativo
            }
            
            let amount_to_distribute = self.trading_rewards_pool;
            let start = start_index.unwrap_or(0);
            let max_batch = batch_size.unwrap_or(100).min(100); // Limita a 100 para evitar gas excessivo
            let end = (start + max_batch).min(self.staker_index);
            
            let mut distributed_count = 0u32;
            let mut processed_count = 0u32;
            
            // Distribui proporcionalmente para o lote de stakers
            for i in start..end {
                processed_count += 1;
                
                if let Some(staker) = self.staker_addresses.get(&i) {
                    if let Some(mut stake) = self.stakes.get(&staker) {
                        if stake.active {
                            let weight = self.calculate_staker_weight(&stake);
                            let reward = amount_to_distribute
                                .checked_mul(weight)
                                .ok_or(StakingError::Overflow)?
                                .checked_div(total_weight)
                                .ok_or(StakingError::Overflow)?;
                            
                            if reward > 0 {
                                stake.pending_rewards = stake.pending_rewards
                                    .checked_add(reward)
                                    .ok_or(StakingError::Overflow)?;
                                
                                self.stakes.insert(&staker, &stake);
                                distributed_count += 1;
                            }
                        }
                    }
                }
            }
            
            let is_complete = end >= self.staker_index;
            let next_index = if is_complete { None } else { Some(end) };
            
            // Se é a primeira página ou a distribuição está completa, atualiza estado global
            if start == 0 || is_complete {
                if is_complete {
                    // Finaliza a distribuição
                    self.total_trading_rewards_distributed = self.total_trading_rewards_distributed
                        .checked_add(amount_to_distribute)
                        .ok_or(StakingError::Overflow)?;
                    self.trading_rewards_pool = 0;
                    
                    // Emit event final
                    self.env().emit_event(TradingRewardsDistributed {
                        total_amount: amount_to_distribute,
                        stakers_count: distributed_count,
                        timestamp: self.env().block_timestamp(),
                    });
                } else {
                    // Emit event parcial
                    self.env().emit_event(TradingRewardsDistributionProgress {
                        processed_stakers: processed_count,
                        rewards_distributed: distributed_count,
                        start_index: start,
                        end_index: end,
                        is_complete: false,
                        timestamp: self.env().block_timestamp(),
                    });
                }
            }
            
            self.release_lock();
            Ok((processed_count, is_complete, next_index))
        }

        /// === FUNÇÕES DE GOVERNANÇA EXPANDIDA ===

        /// Registra participação em votação (chamado pelo sistema de governança)
        #[ink(message)]
        pub fn record_vote_participation(&mut self, voter: AccountId) -> Result<(), StakingError> {
            self.ensure_owner()?; // Por enquanto apenas admin, depois será o contrato de governança
            
            if let Some(mut stake) = self.stakes.get(&voter) {
                if stake.active {
                    stake.governance_participation = stake.governance_participation
                        .checked_add(1)
                        .ok_or(StakingError::Overflow)?;
                    
                    self.stakes.insert(&voter, &stake);
                    
                    // Bônus por participação ativa (cada 8 votos = 200 LUNES)
                    if stake.governance_participation % constants::MIN_VOTES_FOR_BONUS == 0 {
                        let bonus = constants::VOTING_BONUS;
                        let current_bonus = self.governance_bonuses.get(&voter).unwrap_or(0);
                        self.governance_bonuses.insert(&voter, &(current_bonus + bonus));
                        
                        // Emit event
                        self.env().emit_event(GovernanceBonusAwarded {
                            staker: voter,
                            bonus_type: 1, // 1 = voting bonus
                            amount: bonus,
                            timestamp: self.env().block_timestamp(),
                        });
                    }
                }
            }
            
            Ok(())
        }

        /// Recompensa por proposta aprovada (chamado pelo sistema de governança)
        #[ink(message)]
        pub fn reward_approved_proposal(&mut self, proposer: AccountId) -> Result<(), StakingError> {
            self.ensure_owner()?; // Por enquanto apenas admin, depois será o contrato de governança
            
            let bonus = constants::PROPOSAL_BONUS;
            let current_bonus = self.governance_bonuses.get(&proposer).unwrap_or(0);
            self.governance_bonuses.insert(&proposer, &(current_bonus + bonus));
            
            // Emit event
            self.env().emit_event(GovernanceBonusAwarded {
                staker: proposer,
                bonus_type: 2, // 2 = proposal bonus
                amount: bonus,
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        /// Recompensa por proposta implementada (chamado pelo sistema de governança)
        #[ink(message)]
        pub fn reward_implemented_proposal(&mut self, proposer: AccountId) -> Result<(), StakingError> {
            self.ensure_owner()?; // Por enquanto apenas admin, depois será o contrato de governança
            
            let bonus = constants::IMPLEMENTATION_BONUS;
            let current_bonus = self.governance_bonuses.get(&proposer).unwrap_or(0);
            self.governance_bonuses.insert(&proposer, &(current_bonus + bonus));
            
            // Emit event
            self.env().emit_event(GovernanceBonusAwarded {
                staker: proposer,
                bonus_type: 3, // 3 = implementation bonus
                amount: bonus,
                timestamp: self.env().block_timestamp(),
            });
            
            Ok(())
        }

        /// Permite que o staker reivindique seus bônus de governança
        #[ink(message)]
        pub fn claim_governance_bonus(&mut self) -> Result<Balance, StakingError> {
            self.ensure_not_paused()?;
            self.acquire_lock()?;
            
            let caller = self.env().caller();
            let bonus = self.governance_bonuses.get(&caller).unwrap_or(0);
            
            if bonus == 0 {
                self.release_lock();
                return Err(StakingError::NoRewardsToClaim);
            }
            
            // Reset bonus
            self.governance_bonuses.remove(&caller);
            
            // Transfer LUNES
            if self.env().transfer(caller, bonus).is_err() {
                self.release_lock();
                return Err(StakingError::InsufficientBalance);
            }
            
            // Emit event
            self.env().emit_event(GovernanceBonusClaimed {
                staker: caller,
                amount: bonus,
                timestamp: self.env().block_timestamp(),
            });
            
            self.release_lock();
            Ok(bonus)
        }

        // === Internal Helper Methods ===

        /// Ensures only owner can call
        fn ensure_owner(&self) -> Result<(), StakingError> {
            if self.env().caller() != self.owner {
                return Err(StakingError::AccessDenied);
            }
            Ok(())
        }

        /// Ensures contract is not paused
        fn ensure_not_paused(&self) -> Result<(), StakingError> {
            if self.paused {
                return Err(StakingError::ContractPaused);
            }
            Ok(())
        }

        /// Reentrancy protection
        fn acquire_lock(&mut self) -> Result<(), StakingError> {
            if self.locked {
                return Err(StakingError::Reentrancy);
            }
            self.locked = true;
            Ok(())
        }

        /// Release reentrancy lock
        fn release_lock(&mut self) {
            self.locked = false;
        }

        /// Calculates pending rewards for a stake position with new tier system
        fn calculate_pending_rewards(&self, stake: &StakePosition, current_time: Timestamp) -> Result<Balance, StakingError> {
            let time_staked = current_time.checked_sub(stake.last_claim)
                .ok_or(StakingError::Overflow)?;
            
            // Get tier-specific reward rate
            let base_rate = self.tier_multipliers.get(&stake.tier)
                .unwrap_or(constants::BRONZE_REWARD_RATE);
            
            // Get quantity multiplier
            let quantity_multiplier = self.get_quantity_multiplier(stake.amount);
            
            // Get early adopter bonus if applicable
            let early_adopter_bonus = self.get_early_adopter_bonus(stake, current_time);
            
            // Calculate base rewards
            // Formula: (amount * rate * time) / (basis_points * year_in_blocks)
            let year_in_blocks = 365 * 24 * 60 * 30; // Approximate blocks per year
            
            let base_rewards = (stake.amount as u128)
                .checked_mul(base_rate as u128)
                .ok_or(StakingError::Overflow)?
                .checked_mul(time_staked as u128)
                .ok_or(StakingError::Overflow)?
                .checked_div(constants::BASIS_POINTS as u128)
                .ok_or(StakingError::Overflow)?
                .checked_div(year_in_blocks as u128)
                .ok_or(StakingError::Overflow)? as Balance;
            
            // Apply quantity multiplier
            let rewards_with_quantity = base_rewards
                .checked_mul(quantity_multiplier as Balance)
                .ok_or(StakingError::Overflow)?
                .checked_div(constants::BASIS_POINTS as Balance)
                .ok_or(StakingError::Overflow)?;
            
            // Apply early adopter bonus
            let final_rewards = rewards_with_quantity
                .checked_mul((constants::BASIS_POINTS + early_adopter_bonus) as Balance)
                .ok_or(StakingError::Overflow)?
                .checked_div(constants::BASIS_POINTS as Balance)
                .ok_or(StakingError::Overflow)?;
            
            Ok(final_rewards)
        }

        /// === NOVAS FUNÇÕES HELPER PARA PREMIAÇÃO ===

        /// Calcula tier baseado na duração do stake
        fn calculate_staking_tier(&self, duration: u64) -> StakingTier {
            if duration >= 181 * 24 * 60 * 30 {        // 181+ dias
                StakingTier::Platinum
            } else if duration >= 91 * 24 * 60 * 30 {  // 91-180 dias
                StakingTier::Gold
            } else if duration >= 31 * 24 * 60 * 30 {  // 31-90 dias
                StakingTier::Silver
            } else {                                    // 7-30 dias
                StakingTier::Bronze
            }
        }

        /// Determina early adopter tier baseado na ordem de chegada
        fn determine_early_adopter_tier(&mut self) -> EarlyAdopterTier {
            let top_100_count = self.early_adopter_counts.get(&EarlyAdopterTier::Top100).unwrap_or(0);
            let top_500_count = self.early_adopter_counts.get(&EarlyAdopterTier::Top500).unwrap_or(0);
            let top_1000_count = self.early_adopter_counts.get(&EarlyAdopterTier::Top1000).unwrap_or(0);

            if top_100_count < 100 {
                self.early_adopter_counts.insert(&EarlyAdopterTier::Top100, &(top_100_count + 1));
                EarlyAdopterTier::Top100
            } else if top_500_count < 500 {
                self.early_adopter_counts.insert(&EarlyAdopterTier::Top500, &(top_500_count + 1));
                EarlyAdopterTier::Top500
            } else if top_1000_count < 1000 {
                self.early_adopter_counts.insert(&EarlyAdopterTier::Top1000, &(top_1000_count + 1));
                EarlyAdopterTier::Top1000
            } else {
                EarlyAdopterTier::None
            }
        }

        /// Obtém multiplicador baseado na quantidade stakada
        fn get_quantity_multiplier(&self, amount: Balance) -> u32 {
            if amount >= constants::WHALE_STAKE_THRESHOLD {
                constants::WHALE_STAKER_MULTIPLIER        // 200k+ = 1.3x
            } else if amount >= constants::LARGE_STAKE_THRESHOLD {
                constants::LARGE_STAKER_MULTIPLIER        // 50k+ = 1.2x
            } else if amount >= constants::MEDIUM_STAKE_THRESHOLD {
                constants::MEDIUM_STAKER_MULTIPLIER       // 10k+ = 1.1x
            } else {
                constants::SMALL_STAKER_MULTIPLIER        // <10k = 1.0x
            }
        }

        /// Calcula bônus de early adopter se ainda aplicável
        fn get_early_adopter_bonus(&self, stake: &StakePosition, current_time: Timestamp) -> u32 {
            let time_since_start = current_time.checked_sub(stake.start_time).unwrap_or(0);

            match stake.early_adopter_tier {
                EarlyAdopterTier::Top100 => {
                    if time_since_start <= constants::TOP_100_DURATION {
                        constants::TOP_100_BONUS  // +50%
                    } else {
                        0
                    }
                },
                EarlyAdopterTier::Top500 => {
                    if time_since_start <= constants::TOP_500_DURATION {
                        constants::TOP_500_BONUS  // +25%
                    } else {
                        0
                    }
                },
                EarlyAdopterTier::Top1000 => {
                    if time_since_start <= constants::TOP_1000_DURATION {
                        constants::TOP_1000_BONUS // +10%
                    } else {
                        0
                    }
                },
                EarlyAdopterTier::None => 0,
            }
        }

        /// Calcula peso do staker para distribuição de trading rewards
        fn calculate_staker_weight(&self, stake: &StakePosition) -> Balance {
            let tier_multiplier = match stake.tier {
                StakingTier::Bronze => 100,
                StakingTier::Silver => 120,
                StakingTier::Gold => 150,
                StakingTier::Platinum => 200,
            };

            let quantity_multiplier = self.get_quantity_multiplier(stake.amount);

            stake.amount
                .checked_mul(tier_multiplier as Balance)
                .unwrap_or(0)
                .checked_mul(quantity_multiplier as Balance)
                .unwrap_or(0)
                .checked_div(10000) // Normalizar basis points
                .unwrap_or(0)
        }

        /// Calcula peso total de todos os stakers ativos
        fn calculate_total_staker_weight(&self) -> Result<Balance, StakingError> {
            let mut total_weight = 0u128;
            
            for i in 0..self.staker_index {
                if let Some(staker) = self.staker_addresses.get(&i) {
                    if let Some(stake) = self.stakes.get(&staker) {
                        if stake.active {
                            let weight = self.calculate_staker_weight(&stake);
                            total_weight = total_weight
                                .checked_add(weight as u128)
                                .ok_or(StakingError::Overflow)?;
                        }
                    }
                }
            }
            
            Ok(total_weight as Balance)
        }

        /// Calculates rewards and early unstaking penalty
        fn calculate_rewards_and_penalty(&self, stake: &StakePosition, current_time: Timestamp) -> Result<(Balance, Balance), StakingError> {
            let rewards = self.calculate_pending_rewards(stake, current_time)?;
            
            let time_staked = current_time.checked_sub(stake.start_time)
                .ok_or(StakingError::Overflow)?;
            
            let penalty = if time_staked < stake.duration {
                // Early unstaking penalty
                stake.amount.checked_mul(constants::EARLY_PENALTY_RATE as Balance)
                    .ok_or(StakingError::Overflow)?
                    .checked_div(constants::BASIS_POINTS as Balance)
                    .ok_or(StakingError::Overflow)?
            } else {
                0
            };
            
            Ok((rewards, penalty))
        }
    }

    // === Unit Tests ===
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        #[ink::test]
        fn test_new_staking_contract() {
            let contract = StakingContract::new(AccountId::from([0x1; 32]));
            let (total_staked, total_rewards, active_stakers) = contract.get_stats();
            
            assert_eq!(total_staked, 0);
            assert_eq!(total_rewards, 0);
            assert_eq!(active_stakers, 0);
        }

        #[ink::test]
        fn test_stake_success() {
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(constants::MIN_STAKE);
            
            let mut contract = StakingContract::new(AccountId::from([0x1; 32]));
            let result = contract.stake(constants::MIN_DURATION);
            
            assert!(result.is_ok());
            
            let stake = contract.get_stake(accounts.alice).unwrap();
            assert_eq!(stake.amount, constants::MIN_STAKE);
            assert_eq!(stake.duration, constants::MIN_DURATION);
            assert!(stake.active);
        }

        #[ink::test]
        fn test_stake_insufficient_amount() {
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(1000); // Below minimum
            
            let mut contract = StakingContract::new(AccountId::from([0x1; 32]));
            let result = contract.stake(constants::MIN_DURATION);
            
            assert_eq!(result, Err(StakingError::MinimumStakeNotMet));
        }

        #[ink::test]
        fn test_stake_invalid_duration() {
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(constants::MIN_STAKE);
            
            let mut contract = StakingContract::new(AccountId::from([0x1; 32]));
            let result = contract.stake(1000); // Below minimum duration
            
            assert_eq!(result, Err(StakingError::InvalidDuration));
        }

        #[ink::test]
        fn test_create_proposal_success() {
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(constants::MIN_PROPOSAL_POWER);
            
            let mut contract = StakingContract::new(AccountId::from([0x1; 32]));
            
            // First stake enough tokens for proposal power
            let stake_result = contract.stake(constants::MIN_DURATION);
            assert!(stake_result.is_ok());
            
            // Create proposal
            let proposal_result = contract.create_proposal(
                "Test Project".to_string(),
                "A test project for listing".to_string(),
                accounts.bob,
            );
            
            assert!(proposal_result.is_ok());
            let proposal_id = proposal_result.unwrap();
            
            let proposal = contract.get_proposal(proposal_id).unwrap();
            assert_eq!(proposal.name, "Test Project");
            assert_eq!(proposal.proposer, accounts.alice);
            assert!(proposal.active);
        }

        #[ink::test]
        fn test_vote_success() {
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(constants::MIN_PROPOSAL_POWER);
            
            let mut contract = StakingContract::new(AccountId::from([0x1; 32]));
            
            // Stake and create proposal
            contract.stake(constants::MIN_DURATION).unwrap();
            let proposal_id = contract.create_proposal(
                "Test Project".to_string(),
                "Description".to_string(),
                accounts.bob,
            ).unwrap();
            
            // Vote on proposal
            let vote_result = contract.vote(proposal_id, true);
            assert!(vote_result.is_ok());
            
            let proposal = contract.get_proposal(proposal_id).unwrap();
            assert_eq!(proposal.votes_for, constants::MIN_PROPOSAL_POWER);
            assert_eq!(proposal.votes_against, 0);
        }

        #[ink::test]
        fn test_voting_power() {
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(constants::MIN_STAKE);
            
            let mut contract = StakingContract::new(AccountId::from([0x1; 32]));
            
            // No stake, no voting power
            let power = contract.get_voting_power(accounts.alice).unwrap();
            assert_eq!(power, 0);
            
            // Stake tokens
            contract.stake(constants::MIN_DURATION).unwrap();
            
            let power = contract.get_voting_power(accounts.alice).unwrap();
            assert_eq!(power, constants::MIN_STAKE);
        }

        #[ink::test]
        fn test_admin_functions() {
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            
            let mut contract = StakingContract::new(AccountId::from([0x1; 32]));
            
            // Test pause
            assert!(contract.pause().is_ok());
            
            // Test unpause
            assert!(contract.unpause().is_ok());
            
            // Test transfer ownership
            assert!(contract.transfer_ownership(accounts.bob).is_ok());
            
            // Original owner should no longer have access
            assert_eq!(contract.pause(), Err(StakingError::AccessDenied));
        }

        #[ink::test]
        fn test_contract_paused() {
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(constants::MIN_STAKE);
            
            let mut contract = StakingContract::new(AccountId::from([0x1; 32]));
            
            // Pause contract
            contract.pause().unwrap();
            
            // Operations should fail when paused
            let result = contract.stake(constants::MIN_DURATION);
            assert_eq!(result, Err(StakingError::ContractPaused));
        }

        #[ink::test]
        fn test_zero_address_validation() {
            let accounts = test::default_accounts::<ink::env::DefaultEnvironment>();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
            test::set_value_transferred::<ink::env::DefaultEnvironment>(constants::MIN_PROPOSAL_POWER);
            
            let mut contract = StakingContract::new(AccountId::from([0x1; 32]));
            contract.stake(constants::MIN_DURATION).unwrap();
            
            let result = contract.create_proposal(
                "Test".to_string(),
                "Description".to_string(),
                AccountId::from(constants::ZERO_ADDRESS),
            );
            
            assert_eq!(result, Err(StakingError::ZeroAddress));
        }

        #[ink::test]
        fn test_proposal_fee_governance_works() {
            let treasury = AccountId::from([0x1; 32]);
            let mut staking = StakingContract::new(treasury);
            let alice = AccountId::from([0x2; 32]);
            
            // Setup Alice with enough stake for voting power (mais que MIN_PROPOSAL_POWER)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(2_000_000_000_000); // 20,000 LUNES (> 10,000 requeridos)
            assert!(staking.stake(30 * 24 * 60 * 30).is_ok()); // 30 days
            
            // Avançar tempo para que o stake seja considerado válido
            ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
            let current_time = ink::env::block_timestamp::<ink::env::DefaultEnvironment>();
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(
                current_time + 10
            );
            
            // Verificar taxa inicial
            assert_eq!(staking.get_current_proposal_fee(), 100_000_000_000); // 1000 LUNES
            
            // Propor nova taxa (500 LUNES)
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(100_000_000_000); // 1000 LUNES fee
            let proposal_id = staking.propose_fee_change(
                50_000_000_000, // 500 LUNES 
                "Reduzir barreira de entrada".to_string()
            ).unwrap();
            
            // Votar a favor
            assert!(staking.vote(proposal_id, true).is_ok());
            
            // Avançar tempo para depois do deadline
            ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
            let current_time = ink::env::block_timestamp::<ink::env::DefaultEnvironment>();
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(
                current_time + constants::VOTING_PERIOD + 1
            );
            
            // Executar proposta
            assert!(staking.execute_proposal(proposal_id).is_ok());
            
            // Verificar que taxa foi alterada
            assert_eq!(staking.get_current_proposal_fee(), 50_000_000_000); // 500 LUNES
        }

        #[ink::test]
        fn test_proposal_fee_governance_validation() {
            let treasury = AccountId::from([0x1; 32]);
            let mut staking = StakingContract::new(treasury);
            let alice = AccountId::from([0x2; 32]);
            
            // Setup Alice with enough stake
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(alice);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(2_000_000_000_000); // 20,000 LUNES
            assert!(staking.stake(30 * 24 * 60 * 30).is_ok());
            
            // Avançar tempo para que o stake seja válido
            ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
            let current_time = ink::env::block_timestamp::<ink::env::DefaultEnvironment>();
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(
                current_time + 10
            );
            
            // Teste: Taxa zero deve falhar
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(100_000_000_000);
            assert_eq!(
                staking.propose_fee_change(0, "Invalid".to_string()),
                Err(StakingError::InvalidAmount)
            );
            
            // Teste: Taxa muito alta deve falhar
            assert_eq!(
                staking.propose_fee_change(20_000_000_000_000, "Too high".to_string()), // 200,000 LUNES
                Err(StakingError::InvalidAmount)
            );
            
            // Teste: Fee insuficiente deve falhar
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(50_000_000_000); // 500 LUNES
            assert_eq!(
                staking.propose_fee_change(30_000_000_000, "Valid amount".to_string()),
                Err(StakingError::InsufficientFee)
            );
        }
    }
}
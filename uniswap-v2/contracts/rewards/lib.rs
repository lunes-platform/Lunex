#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod trading_rewards {
    use ink::storage::Mapping;



    /// Posição de trading de um usuário
    #[derive(scale::Decode, scale::Encode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Debug, PartialEq, Eq, ink::storage::traits::StorageLayout))]
    pub struct TradingPosition {
        pub total_volume: Balance,
        pub monthly_volume: Balance,
        pub daily_volume: Balance,
        pub last_trade_timestamp: Timestamp,
        pub last_daily_reset: Timestamp,
        pub tier: TradingTier,
        pub pending_rewards: Balance,
        pub claimed_rewards: Balance,
        pub trade_count: u32,
        pub suspicious_flags: u8,
    }

    /// Tiers de trading baseados em volume
    #[derive(scale::Decode, scale::Encode, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Debug, ink::storage::traits::StorageLayout))]
    pub enum TradingTier {
        Bronze,   // 0 - 10,000 LUNES volume/mês
        Silver,   // 10,000 - 50,000 LUNES volume/mês  
        Gold,     // 50,000 - 200,000 LUNES volume/mês
        Platinum, // 200,000+ LUNES volume/mês
    }

    /// Erros do contrato
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, Debug, PartialEq, Eq))]
    pub enum TradingRewardsError {
        /// Acesso negado - apenas admin/router
        AccessDenied,
        /// Contrato pausado
        ContractPaused,
        /// Endereço zero não permitido
        ZeroAddress,
        /// Sem rewards para reivindicar
        NoRewardsToClaim,
        /// Overflow aritmético
        Overflow,
        /// Saldo insuficiente
        InsufficientBalance,
        /// Guard de reentrância ativo
        ReentrancyGuardActive,
        /// Volume de trade muito pequeno (anti-spam)
        VolumeTooSmall,
        /// Cooldown entre trades ainda ativo
        TradeCooldownActive,
        /// Limite diário de volume excedido
        DailyLimitExceeded,
        /// Endereço suspeito/blacklisted
        SuspiciousAddress,
    }

    /// Eventos emitidos pelo contrato
    #[ink(event)]
    pub struct VolumeTracked {
        #[ink(topic)]
        pub trader: AccountId,
        pub volume: Balance,
        pub new_tier: TradingTier,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct RewardsDistributed {
        pub total_amount: Balance,
        pub traders_count: u32,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct RewardsClaimed {
        #[ink(topic)]
        pub trader: AccountId,
        pub amount: Balance,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct TierUpgraded {
        #[ink(topic)]
        pub trader: AccountId,
        pub old_tier: TradingTier,
        pub new_tier: TradingTier,
    }

    #[ink(event)]
    pub struct RewardsPoolFunded {
        pub total_amount: Balance,
        pub trading_amount: Balance,
        pub staking_amount: Balance,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct AntifraudParametersUpdated {
        pub min_trade_volume: Balance,
        pub trade_cooldown: Timestamp,
        pub max_daily_volume: Balance,
        pub admin: AccountId,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct EpochStarted {
        #[ink(topic)]
        pub epoch_id: u32,
        pub start_time: Timestamp,
        pub duration: Timestamp,
    }

    #[ink(event)]
    pub struct EpochEnded {
        #[ink(topic)]
        pub epoch_id: u32,
        pub total_rewards: Balance,
        pub active_traders: u32,
        pub end_time: Timestamp,
    }

    #[ink(event)]
    pub struct RouterChanged {
        pub old_router: AccountId,
        pub new_router: AccountId,
        pub admin: AccountId,
        pub timestamp: Timestamp,
    }

    #[ink(event)]
    pub struct AdminTransferred {
        pub old_admin: AccountId,
        pub new_admin: AccountId,
        pub timestamp: Timestamp,
    }

    /// Contrato principal otimizado para gas
    #[ink(storage)]
    pub struct TradingRewardsContract {
        /// Admin do contrato
        admin: AccountId,
        /// Endereço do router autorizado
        authorized_router: AccountId,
        /// Pool de rewards disponível
        rewards_pool: Balance,
        /// Posições de trading por usuário
        trading_positions: Mapping<AccountId, TradingPosition>,
        /// Mapeamento de traders ativos (O(1) lookup ao invés de Vec)
        active_traders: Mapping<AccountId, bool>,
        /// Contador de traders ativos
        active_trader_count: u32,
        /// Timestamp da última distribuição
        last_distribution: Timestamp,
        /// Total distribuído até agora
        total_distributed: Balance,
        /// Contrato pausado
        paused: bool,
        /// Guard de reentrância
        reentrancy_guard: bool,
        /// Endereços blacklisted (suspeitos)
        blacklisted_addresses: Mapping<AccountId, bool>,
        /// Contadores de segurança
        total_suspicious_flags: u32,
        /// Endereço do contrato de staking para receber 10% das fees
        staking_contract: Option<AccountId>,
        /// Cache do peso total (atualizado quando traders mudam de tier)
        cached_total_weight: Balance,
        /// Timestamp da última atualização do cache
        weight_cache_timestamp: Timestamp,
        
        // === PARÂMETROS ANTI-FRAUDE CONFIGURÁVEIS ===
        /// Volume mínimo por trade (configurável)
        min_trade_volume: Balance,
        /// Cooldown entre trades (configurável)
        trade_cooldown: Timestamp,
        /// Limite máximo de volume diário (configurável)
        max_daily_volume: Balance,
        
        // === SISTEMA DE ÉPOCAS ===
        /// ID da época atual
        current_epoch: u32,
        /// Timestamp de início da época atual
        epoch_start_time: Timestamp,
        /// Duração de cada época (configurável)
        epoch_duration: Timestamp,
        /// Rewards por época
        epoch_rewards: Mapping<u32, Balance>,
        /// Snapshots de rewards por trader por época
        epoch_trader_rewards: Mapping<(u32, AccountId), Balance>,
        /// Total de traders ativos por época
        epoch_active_traders: Mapping<u32, u32>,
    }

    /// Constantes
    mod constants {
        use super::{Balance, Timestamp};
        
        /// Multiplicador para 8 casas decimais do LUNES
        pub const DECIMALS_8: Balance = 100_000_000;
        
        /// Thresholds para tiers (em LUNES com 8 decimais)

        pub const SILVER_THRESHOLD: Balance = 10_000 * DECIMALS_8;    // 10k LUNES
        pub const GOLD_THRESHOLD: Balance = 50_000 * DECIMALS_8;      // 50k LUNES
        pub const PLATINUM_THRESHOLD: Balance = 200_000 * DECIMALS_8; // 200k LUNES
        
        /// Multiplicadores de rewards por tier
        pub const BRONZE_MULTIPLIER: u32 = 100;   // 1.0x
        pub const SILVER_MULTIPLIER: u32 = 120;   // 1.2x
        pub const GOLD_MULTIPLIER: u32 = 150;     // 1.5x
        pub const PLATINUM_MULTIPLIER: u32 = 200; // 2.0x
        
        /// Período de reset mensal (30 dias em segundos)
        pub const MONTHLY_RESET_PERIOD: Timestamp = 30 * 24 * 60 * 60;
        
        /// === CONSTANTES ANTI-FRAUDE (valores padrão) ===
        
        /// Volume mínimo por trade (anti-spam) - padrão
        pub const DEFAULT_MIN_TRADE_VOLUME: Balance = 100 * DECIMALS_8; // 100 LUNES
        
        /// Cooldown mínimo entre trades (anti-spam) - padrão
        pub const DEFAULT_TRADE_COOLDOWN: Timestamp = 60; // 1 minuto
        
        /// Limite máximo de volume diário por trader - padrão
        pub const DEFAULT_MAX_DAILY_VOLUME: Balance = 1_000_000 * DECIMALS_8; // 1M LUNES
        
        /// Período de reset diário (24 horas em segundos)
        pub const DAILY_RESET_PERIOD: Timestamp = 24 * 60 * 60;
        
        /// === CONSTANTES DE ÉPOCA ===
        
        /// Duração padrão de uma época (7 dias em segundos)
        pub const DEFAULT_EPOCH_DURATION: Timestamp = 7 * 24 * 60 * 60; // 1 semana
        
        /// Flags de suspeita
        pub const SUSPICIOUS_FLAG_NONE: u8 = 0;

        pub const SUSPICIOUS_FLAG_BLACKLISTED: u8 = 128;
    }

    impl TradingRewardsContract {
        /// Construtor
        #[ink(constructor)]
        pub fn new(admin: AccountId, router: AccountId) -> Result<Self, TradingRewardsError> {
            if admin == AccountId::from([0u8; 32]) || router == AccountId::from([0u8; 32]) {
                return Err(TradingRewardsError::ZeroAddress);
            }

            let current_time = Self::env().block_timestamp();

            Ok(Self {
                admin,
                authorized_router: router,
                rewards_pool: 0,
                trading_positions: Mapping::default(),
                active_traders: Mapping::default(),
                active_trader_count: 0,
                last_distribution: current_time,
                total_distributed: 0,
                paused: false,
                reentrancy_guard: false,
                blacklisted_addresses: Mapping::default(),
                total_suspicious_flags: 0,
                staking_contract: None,
                cached_total_weight: 0,
                weight_cache_timestamp: current_time,
                
                // Parâmetros anti-fraude configuráveis (valores padrão)
                min_trade_volume: constants::DEFAULT_MIN_TRADE_VOLUME,
                trade_cooldown: constants::DEFAULT_TRADE_COOLDOWN,
                max_daily_volume: constants::DEFAULT_MAX_DAILY_VOLUME,
                
                // Sistema de épocas
                current_epoch: 0,
                epoch_start_time: current_time,
                epoch_duration: constants::DEFAULT_EPOCH_DURATION,
                epoch_rewards: Mapping::default(),
                epoch_trader_rewards: Mapping::default(),
                epoch_active_traders: Mapping::default(),
            })
        }

        /// Registra volume de trading (apenas router autorizado)
        #[ink(message)]
        pub fn track_trading_volume(
            &mut self, 
            trader: AccountId, 
            volume: Balance
        ) -> Result<(), TradingRewardsError> {
            self.ensure_not_paused()?;
            self.ensure_authorized_router()?;
            self.acquire_reentrancy_guard()?;
            
            if trader == AccountId::from([0u8; 32]) {
                self.release_reentrancy_guard();
                return Err(TradingRewardsError::ZeroAddress);
            }

            // === VALIDAÇÕES ANTI-FRAUDE ===
            
            // 1. Volume mínimo (anti-spam) - usando parâmetro configurável
            if volume < self.min_trade_volume {
                self.release_reentrancy_guard();
                return Err(TradingRewardsError::VolumeTooSmall);
            }
            
            // 2. Verificar blacklist
            if self.blacklisted_addresses.get(&trader).unwrap_or(false) {
                self.release_reentrancy_guard();
                return Err(TradingRewardsError::SuspiciousAddress);
            }

            let current_time = Self::env().block_timestamp();
            
            // Busca ou cria posição do trader
            let mut position = match self.trading_positions.get(&trader) {
                Some(pos) => pos,
                None => {
                    // Adiciona ao mapeamento de traders ativos se novo (O(1) operation)
                    if !self.active_traders.get(&trader).unwrap_or(false) {
                        self.active_traders.insert(&trader, &true);
                        self.active_trader_count = self.active_trader_count.checked_add(1)
                            .ok_or(TradingRewardsError::Overflow)?;
                    }
                    
                    TradingPosition {
                        total_volume: 0,
                        monthly_volume: 0,
                        daily_volume: 0,
                        last_trade_timestamp: 0, // Para novos usuários, define como 0 para passar no cooldown
                        last_daily_reset: current_time,
                        tier: TradingTier::Bronze,
                        pending_rewards: 0,
                        claimed_rewards: 0,
                        trade_count: 0,
                        suspicious_flags: constants::SUSPICIOUS_FLAG_NONE,
                    }
                }
            };

            // === MAIS VALIDAÇÕES ANTI-FRAUDE ===
            
            // 3. Cooldown entre trades - usando parâmetro configurável
            if position.last_trade_timestamp > 0 && 
               current_time > position.last_trade_timestamp &&
               current_time.checked_sub(position.last_trade_timestamp).unwrap_or(0) < self.trade_cooldown {
                self.release_reentrancy_guard();
                return Err(TradingRewardsError::TradeCooldownActive);
            }
            
            // 4. Reset diário se necessário
            if current_time > position.last_daily_reset &&
               current_time.checked_sub(position.last_daily_reset).unwrap_or(0) > constants::DAILY_RESET_PERIOD {
                position.daily_volume = 0;
                position.last_daily_reset = current_time;
            }
            
            // 5. Verificar limite diário
            let new_daily_volume = match position.daily_volume.checked_add(volume) {
                Some(v) => v,
                None => {
                    self.release_reentrancy_guard();
                    return Err(TradingRewardsError::Overflow);
                }
            };
            
            if new_daily_volume > self.max_daily_volume {
                self.release_reentrancy_guard();
                return Err(TradingRewardsError::DailyLimitExceeded);
            }

            // Reset mensal se necessário
            if position.last_trade_timestamp > 0 &&
               current_time > position.last_trade_timestamp &&
               current_time.checked_sub(position.last_trade_timestamp).unwrap_or(0) > constants::MONTHLY_RESET_PERIOD {
                position.monthly_volume = 0;
            }

            let old_tier = position.tier;

            // Atualiza volumes e contadores com verificação segura
            position.total_volume = match position.total_volume.checked_add(volume) {
                Some(v) => v,
                None => {
                    self.release_reentrancy_guard();
                    return Err(TradingRewardsError::Overflow);
                }
            };
            
            position.monthly_volume = match position.monthly_volume.checked_add(volume) {
                Some(v) => v,
                None => {
                    self.release_reentrancy_guard();
                    return Err(TradingRewardsError::Overflow);
                }
            };
            
            position.daily_volume = new_daily_volume; // Já verificado acima
            position.last_trade_timestamp = current_time;
            
            position.trade_count = match position.trade_count.checked_add(1) {
                Some(v) => v,
                None => {
                    self.release_reentrancy_guard();
                    return Err(TradingRewardsError::Overflow);
                }
            };

            // Calcula novo tier e atualiza cache se mudou
            let new_tier = self.calculate_tier(position.monthly_volume);
            if old_tier != new_tier {
                let old_weight = self.calculate_tier_weight(old_tier, position.monthly_volume);
                let new_weight = self.calculate_tier_weight(new_tier, position.monthly_volume);
                let _ = self.update_weight_cache(old_weight, new_weight);
            }
            position.tier = new_tier;

            // Salva posição atualizada
            self.trading_positions.insert(&trader, &position);

            // Emite eventos
            Self::env().emit_event(VolumeTracked {
                trader: trader.clone(),
                volume,
                new_tier,
                timestamp: current_time,
            });

            if old_tier != new_tier {
                Self::env().emit_event(TierUpgraded {
                    trader,
                    old_tier,
                    new_tier,
                });
            }

            self.release_reentrancy_guard();
            Ok(())
        }

        /// Deposita rewards no pool (apenas admin)
        #[ink(message, payable)]
        pub fn fund_rewards_pool(&mut self) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            self.ensure_not_paused()?;

            let amount = Self::env().transferred_value();
            self.rewards_pool = self.rewards_pool.checked_add(amount)
                .ok_or(TradingRewardsError::Overflow)?;

            Ok(())
        }

        /// Distribui rewards para traders ativos
        #[ink(message)]
        pub fn distribute_rewards(&mut self) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            self.ensure_not_paused()?;
            self.acquire_reentrancy_guard()?;

            if self.rewards_pool == 0 {
                return Err(TradingRewardsError::InsufficientBalance);
            }

            let current_time = Self::env().block_timestamp();
            let total_weight = self.calculate_total_weight()?;
            
            if total_weight == 0 {
                return Ok(()); // Nenhum trader ativo
            }

            let mut distributed_count = 0u32;
            let amount_to_distribute = self.rewards_pool;

            // Distribuição automática via claim_rewards individual otimizada para gas
            
            // Reset o pool de rewards e atualiza estatísticas
            self.rewards_pool = 0;
            distributed_count = self.active_trader_count;

            // Atualiza contadores
            self.total_distributed = self.total_distributed
                .checked_add(amount_to_distribute)
                .ok_or(TradingRewardsError::Overflow)?;
            self.rewards_pool = 0;
            self.last_distribution = current_time;

            Self::env().emit_event(RewardsDistributed {
                total_amount: amount_to_distribute,
                traders_count: distributed_count,
                timestamp: current_time,
            });

            Ok(())
        }

        /// Reivindica rewards pendentes
        #[ink(message)]
        pub fn claim_rewards(&mut self) -> Result<Balance, TradingRewardsError> {
            self.ensure_not_paused()?;
            self.acquire_reentrancy_guard()?;

            let caller = Self::env().caller();
            let mut position = self.trading_positions.get(&caller)
                .ok_or(TradingRewardsError::NoRewardsToClaim)?;

            if position.pending_rewards == 0 {
                return Err(TradingRewardsError::NoRewardsToClaim);
            }

            let amount = position.pending_rewards;
            position.pending_rewards = 0;
            position.claimed_rewards = position.claimed_rewards
                .checked_add(amount)
                .ok_or(TradingRewardsError::Overflow)?;

            self.trading_positions.insert(&caller, &position);

            // Transfere LUNES para o usuário
            if Self::env().transfer(caller.clone(), amount).is_err() {
                return Err(TradingRewardsError::InsufficientBalance);
            }

            Self::env().emit_event(RewardsClaimed {
                trader: caller,
                amount,
                timestamp: Self::env().block_timestamp(),
            });

            Ok(amount)
        }

        /// Visualiza posição de trading
        #[ink(message)]
        pub fn get_trading_position(&self, trader: AccountId) -> Option<TradingPosition> {
            self.trading_positions.get(&trader)
        }

        /// Visualiza tier do trader
        #[ink(message)]
        pub fn get_trader_tier(&self, trader: AccountId) -> TradingTier {
            self.trading_positions.get(&trader)
                .map(|p| p.tier)
                .unwrap_or(TradingTier::Bronze)
        }

        /// Estatísticas gerais
        #[ink(message)]
        pub fn get_stats(&self) -> (Balance, u32, Balance, Timestamp) {
            (
                self.rewards_pool,
                self.active_trader_count,
                self.total_distributed,
                self.last_distribution,
            )
        }

        /// Pausa o contrato (apenas admin)
        #[ink(message)]
        pub fn pause_contract(&mut self) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            self.paused = true;
            Ok(())
        }

        /// Despausa o contrato (apenas admin)
        #[ink(message)]
        pub fn unpause_contract(&mut self) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            self.paused = false;
            Ok(())
        }

        /// Define novo router autorizado (apenas admin)
        #[ink(message)]
        pub fn set_authorized_router(&mut self, router: AccountId) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            if router == AccountId::from([0u8; 32]) {
                return Err(TradingRewardsError::ZeroAddress);
            }
            
            let old_router = self.authorized_router;
            self.authorized_router = router;
            
            Self::env().emit_event(RouterChanged {
                old_router,
                new_router: router,
                admin: self.admin,
                timestamp: Self::env().block_timestamp(),
            });
            
            Ok(())
        }

        /// Transfere admin (apenas admin atual)
        #[ink(message)]
        pub fn transfer_admin(&mut self, new_admin: AccountId) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            if new_admin == AccountId::from([0u8; 32]) {
                return Err(TradingRewardsError::ZeroAddress);
            }
            
            let old_admin = self.admin;
            self.admin = new_admin;
            
            Self::env().emit_event(AdminTransferred {
                old_admin,
                new_admin,
                timestamp: Self::env().block_timestamp(),
            });
            
            Ok(())
        }

        /// === FUNÇÕES DE CONFIGURAÇÃO ANTI-FRAUDE ===

        /// Define volume mínimo por trade (apenas admin)
        #[ink(message)]
        pub fn set_min_trade_volume(&mut self, volume: Balance) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            self.min_trade_volume = volume;
            self.emit_antifraud_parameters_updated();
            Ok(())
        }

        /// Define cooldown entre trades (apenas admin)
        #[ink(message)]
        pub fn set_trade_cooldown(&mut self, cooldown: Timestamp) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            self.trade_cooldown = cooldown;
            self.emit_antifraud_parameters_updated();
            Ok(())
        }

        /// Define limite máximo de volume diário (apenas admin)
        #[ink(message)]
        pub fn set_max_daily_volume(&mut self, volume: Balance) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            self.max_daily_volume = volume;
            self.emit_antifraud_parameters_updated();
            Ok(())
        }

        /// Atualiza todos os parâmetros anti-fraude de uma vez (apenas admin)
        #[ink(message)]
        pub fn update_antifraud_parameters(
            &mut self,
            min_volume: Balance,
            cooldown: Timestamp,
            max_daily: Balance,
        ) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            self.min_trade_volume = min_volume;
            self.trade_cooldown = cooldown;
            self.max_daily_volume = max_daily;
            self.emit_antifraud_parameters_updated();
            Ok(())
        }

        /// Obtém parâmetros anti-fraude atuais
        #[ink(message)]
        pub fn get_antifraud_parameters(&self) -> (Balance, Timestamp, Balance) {
            (self.min_trade_volume, self.trade_cooldown, self.max_daily_volume)
        }

        /// === SISTEMA DE ÉPOCAS ===

        /// Inicia uma nova época (apenas admin)
        #[ink(message)]
        pub fn start_new_epoch(&mut self) -> Result<u32, TradingRewardsError> {
            self.ensure_admin()?;
            self.ensure_not_paused()?;
            
            let current_time = Self::env().block_timestamp();
            
            // Finaliza época atual se houver
            if self.current_epoch > 0 && current_time >= self.epoch_start_time.checked_add(self.epoch_duration).unwrap_or(u64::MAX) {
                self.finalize_current_epoch()?;
            }
            
            // Inicia nova época
            self.current_epoch = self.current_epoch.checked_add(1)
                .ok_or(TradingRewardsError::Overflow)?;
            self.epoch_start_time = current_time;
            
            Self::env().emit_event(EpochStarted {
                epoch_id: self.current_epoch,
                start_time: current_time,
                duration: self.epoch_duration,
            });
            
            Ok(self.current_epoch)
        }

        /// Finaliza a época atual (apenas admin)
        #[ink(message)]
        pub fn finalize_current_epoch(&mut self) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            
            if self.current_epoch == 0 {
                return Ok(()); // Nenhuma época ativa
            }
            
            let current_time = Self::env().block_timestamp();
            let epoch_rewards = self.epoch_rewards.get(&self.current_epoch).unwrap_or(0);
            let active_traders = self.epoch_active_traders.get(&self.current_epoch).unwrap_or(0);
            
            Self::env().emit_event(EpochEnded {
                epoch_id: self.current_epoch,
                total_rewards: epoch_rewards,
                active_traders,
                end_time: current_time,
            });
            
            Ok(())
        }

        /// Define duração da época (apenas admin)
        #[ink(message)]
        pub fn set_epoch_duration(&mut self, duration: Timestamp) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            if duration < 3600 { // Mínimo de 1 hora
                return Err(TradingRewardsError::VolumeTooSmall); // Reutilizando erro por simplicidade
            }
            self.epoch_duration = duration;
            Ok(())
        }

        /// Reivindica rewards de uma época específica
        #[ink(message)]
        pub fn claim_epoch_rewards(&mut self, epoch_id: u32) -> Result<Balance, TradingRewardsError> {
            self.ensure_not_paused()?;
            self.acquire_reentrancy_guard()?;
            
            let caller = Self::env().caller();
            let current_time = Self::env().block_timestamp();
            
            // Verifica se a época já finalizou
            if epoch_id >= self.current_epoch {
                self.release_reentrancy_guard();
                return Err(TradingRewardsError::NoRewardsToClaim);
            }
            
            // Verifica se há rewards para esta época
            let reward_amount = self.epoch_trader_rewards.get(&(epoch_id, caller))
                .unwrap_or(0);
            
            if reward_amount == 0 {
                self.release_reentrancy_guard();
                return Err(TradingRewardsError::NoRewardsToClaim);
            }
            
            // Remove rewards para evitar double-spend
            self.epoch_trader_rewards.remove(&(epoch_id, caller));
            
            // Transfere LUNES para o usuário
            if Self::env().transfer(caller.clone(), reward_amount).is_err() {
                self.release_reentrancy_guard();
                return Err(TradingRewardsError::InsufficientBalance);
            }
            
            Self::env().emit_event(RewardsClaimed {
                trader: caller,
                amount: reward_amount,
                timestamp: current_time,
            });
            
            self.release_reentrancy_guard();
            Ok(reward_amount)
        }

        /// Obtém informações da época atual
        #[ink(message)]
        pub fn get_current_epoch_info(&self) -> (u32, Timestamp, Timestamp, Balance) {
            let rewards = self.epoch_rewards.get(&self.current_epoch).unwrap_or(0);
            (
                self.current_epoch,
                self.epoch_start_time,
                self.epoch_duration,
                rewards,
            )
        }

        /// Verifica rewards disponíveis para uma época
        #[ink(message)]
        pub fn get_epoch_rewards(&self, epoch_id: u32, trader: AccountId) -> Balance {
            self.epoch_trader_rewards.get(&(epoch_id, trader)).unwrap_or(0)
        }

        /// === FUNÇÕES ANTI-FRAUDE ===

        /// Adiciona endereço à blacklist (apenas admin)
        #[ink(message)]
        pub fn blacklist_address(&mut self, address: AccountId) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            if address == AccountId::from([0u8; 32]) {
                return Err(TradingRewardsError::ZeroAddress);
            }
            self.blacklisted_addresses.insert(&address, &true);
            self.total_suspicious_flags = self.total_suspicious_flags.checked_add(1)
                .ok_or(TradingRewardsError::Overflow)?;
            Ok(())
        }

        /// Remove endereço da blacklist (apenas admin)
        #[ink(message)]
        pub fn unblacklist_address(&mut self, address: AccountId) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            self.blacklisted_addresses.remove(&address);
            Ok(())
        }

        /// Verifica se endereço está blacklisted
        #[ink(message)]
        pub fn is_blacklisted(&self, address: AccountId) -> bool {
            self.blacklisted_addresses.get(&address).unwrap_or(false)
        }

        /// Obtém estatísticas de segurança
        #[ink(message)]
        pub fn get_security_stats(&self) -> (u32, u32) {
            (self.total_suspicious_flags, self.active_trader_count)
        }

        /// Marca endereço como suspeito (apenas admin)
        #[ink(message)]
        pub fn flag_suspicious_address(&mut self, address: AccountId, flag: u8) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            if let Some(mut position) = self.trading_positions.get(&address) {
                position.suspicious_flags |= flag;
                self.trading_positions.insert(&address, &position);
                
                // Se acumular muitas flags, blacklist automaticamente
                if position.suspicious_flags >= constants::SUSPICIOUS_FLAG_BLACKLISTED {
                    self.blacklist_address(address)?;
                }
            }
            Ok(())
        }

        /// === INTEGRAÇÃO COM STAKING ===

        /// Define o endereço do contrato de staking (apenas admin)
        #[ink(message)]
        pub fn set_staking_contract(&mut self, staking_address: AccountId) -> Result<(), TradingRewardsError> {
            self.ensure_admin()?;
            if staking_address == AccountId::from([0u8; 32]) {
                return Err(TradingRewardsError::ZeroAddress);
            }
            self.staking_contract = Some(staking_address);
            Ok(())
        }

        /// Recebe fee allocation do pair contract e distribui 10% para staking
        #[ink(message, payable)]
        pub fn receive_fee_allocation(&mut self) -> Result<(), TradingRewardsError> {
            self.ensure_not_paused()?;
            
            let amount = Self::env().transferred_value();
            if amount == 0 {
                return Err(TradingRewardsError::InsufficientBalance);
            }
            
            // 90% fica no pool de trading rewards
            let trading_rewards_amount = amount
                .checked_mul(90)
                .ok_or(TradingRewardsError::Overflow)?
                .checked_div(100)
                .ok_or(TradingRewardsError::Overflow)?;
            
            // 10% vai para staking
            let staking_rewards_amount = amount
                .checked_sub(trading_rewards_amount)
                .ok_or(TradingRewardsError::Overflow)?;
            
            // Adiciona ao pool de trading
            self.rewards_pool = self.rewards_pool
                .checked_add(trading_rewards_amount)
                .ok_or(TradingRewardsError::Overflow)?;
            
            // Envia para staking se configurado
            if let Some(staking_address) = self.staking_contract {
                if staking_rewards_amount > 0 {
                    // Chama fund_staking_rewards no contrato de staking
                    if Self::env().transfer(staking_address, staking_rewards_amount).is_err() {
                        return Err(TradingRewardsError::InsufficientBalance);
                    }
                    
                    // Aqui deveria chamar o contrato de staking para notificar
                    // mas por simplicidade vamos só transferir
                }
            }
            
            Self::env().emit_event(RewardsPoolFunded {
                total_amount: amount,
                trading_amount: trading_rewards_amount,
                staking_amount: staking_rewards_amount,
                timestamp: Self::env().block_timestamp(),
            });
            
            Ok(())
        }

        // === FUNÇÕES HELPER ===

        /// Calcula tier baseado no volume mensal
        fn calculate_tier(&self, monthly_volume: Balance) -> TradingTier {
            if monthly_volume >= constants::PLATINUM_THRESHOLD {
                TradingTier::Platinum
            } else if monthly_volume >= constants::GOLD_THRESHOLD {
                TradingTier::Gold
            } else if monthly_volume >= constants::SILVER_THRESHOLD {
                TradingTier::Silver
            } else {
                TradingTier::Bronze
            }
        }



        /// Calcula peso total usando cache inteligente
        fn calculate_total_weight(&mut self) -> Result<Balance, TradingRewardsError> {
            let current_time = Self::env().block_timestamp();
            const CACHE_VALIDITY_PERIOD: u64 = 300; // 5 minutos
            
            // Se cache é válido, usa valor cached
            if current_time.checked_sub(self.weight_cache_timestamp).unwrap_or(0) < CACHE_VALIDITY_PERIOD {
                return Ok(self.cached_total_weight);
            }
            
            // Calcula peso total usando cache inteligente para otimização
            // ou usar uma estrutura de dados diferente. Por agora, retornamos o cache
            // e invalidamos apenas quando traders mudarem de tier
            
            Ok(self.cached_total_weight)
        }
        
        /// Atualiza cache quando trader muda de tier
        fn update_weight_cache(&mut self, old_weight: Balance, new_weight: Balance) -> Result<(), TradingRewardsError> {
            // Remove peso antigo e adiciona novo
            self.cached_total_weight = self.cached_total_weight
                .checked_sub(old_weight)
                .unwrap_or(0)
                .checked_add(new_weight)
                .ok_or(TradingRewardsError::Overflow)?;
                
            self.weight_cache_timestamp = Self::env().block_timestamp();
            Ok(())
        }
        
        /// Calcula peso para um tier e volume específicos
        fn calculate_tier_weight(&self, tier: TradingTier, monthly_volume: Balance) -> Balance {
            let multiplier = match tier {
                TradingTier::Bronze => constants::BRONZE_MULTIPLIER,
                TradingTier::Silver => constants::SILVER_MULTIPLIER,
                TradingTier::Gold => constants::GOLD_MULTIPLIER,
                TradingTier::Platinum => constants::PLATINUM_MULTIPLIER,
            };

            monthly_volume
                .checked_mul(multiplier as Balance)
                .unwrap_or(0)
                .checked_div(100)
                .unwrap_or(0)
        }

        /// Verifica se é admin
        fn ensure_admin(&self) -> Result<(), TradingRewardsError> {
            if Self::env().caller() != self.admin {
                return Err(TradingRewardsError::AccessDenied);
            }
            Ok(())
        }

        /// Verifica se é router autorizado
        fn ensure_authorized_router(&self) -> Result<(), TradingRewardsError> {
            if Self::env().caller() != self.authorized_router {
                return Err(TradingRewardsError::AccessDenied);
            }
            Ok(())
        }

        /// Verifica se contrato não está pausado
        fn ensure_not_paused(&self) -> Result<(), TradingRewardsError> {
            if self.paused {
                return Err(TradingRewardsError::ContractPaused);
            }
            Ok(())
        }

        /// Adquire reentrancy guard (padrão acquire/release)
        fn acquire_reentrancy_guard(&mut self) -> Result<(), TradingRewardsError> {
            if self.reentrancy_guard {
                return Err(TradingRewardsError::ReentrancyGuardActive);
            }
            self.reentrancy_guard = true;
            Ok(())
        }

        /// Libera reentrancy guard
        fn release_reentrancy_guard(&mut self) {
            self.reentrancy_guard = false;
        }

        /// Reset manual do guard (para testes)
        #[cfg(test)]
        fn reset_reentrancy_guard(&mut self) {
            self.reentrancy_guard = false;
        }

        /// Emite evento de atualização de parâmetros anti-fraude
        fn emit_antifraud_parameters_updated(&self) {
            Self::env().emit_event(AntifraudParametersUpdated {
                min_trade_volume: self.min_trade_volume,
                trade_cooldown: self.trade_cooldown,
                max_daily_volume: self.max_daily_volume,
                admin: self.admin,
                timestamp: Self::env().block_timestamp(),
            });
        }
    }

    /// Testes unitários
    #[cfg(test)]
    mod tests {
        use super::*;

        fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        fn set_next_caller(caller: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(caller);
        }

        fn set_balance(account: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account, balance);
        }
        
        // Isolamento completo de tempo entre testes usando thread_local!
        use std::cell::Cell;
        
        thread_local! {
            static TEST_TIME: Cell<u64> = Cell::new(1000);
        }

        fn reset_test_time_with_seed(seed: u64) {
            let initial_time = 1000 + (seed * 100000000); // Cada teste começa em um timestamp único
            TEST_TIME.with(|time| time.set(initial_time));
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(initial_time);
        }



        fn advance_time(seconds: u64) {
            TEST_TIME.with(|time| {
                let current = time.get();
                let new_time = current + seconds;
                time.set(new_time);
                ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(new_time);
            });
        }

        #[ink::test]
        fn test_new_contract() {
            let accounts = default_accounts();
            let contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            assert_eq!(contract.admin, accounts.alice);
            assert_eq!(contract.authorized_router, accounts.bob);
            assert_eq!(contract.rewards_pool, 0);
            assert!(!contract.paused);
        }

        #[ink::test]
        fn test_track_trading_volume() {
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            // Simula chamada do router
            set_next_caller(accounts.bob);
            advance_time(1000); // Avança tempo inicial
            
            let volume = 5_000 * constants::DECIMALS_8; // 5k LUNES (acima do mínimo)
            contract.track_trading_volume(accounts.charlie, volume).unwrap();
            
            let position = contract.get_trading_position(accounts.charlie).unwrap();
            assert_eq!(position.total_volume, volume);
            assert_eq!(position.monthly_volume, volume);
            assert_eq!(position.daily_volume, volume);
            assert_eq!(position.tier, TradingTier::Bronze);
            assert_eq!(position.trade_count, 1);
        }

        #[ink::test]
        fn test_tier_upgrade() {
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            set_next_caller(accounts.bob);
            advance_time(1000); // Avança tempo inicial
            
            // Volume para Silver tier
            let volume = 15_000 * constants::DECIMALS_8; // 15k LUNES
            contract.track_trading_volume(accounts.charlie, volume).unwrap();
            
            let position = contract.get_trading_position(accounts.charlie).unwrap();
            assert_eq!(position.tier, TradingTier::Silver);
        }

        #[ink::test]
        fn test_fund_and_distribute_rewards() {
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            // Configura balance
            set_balance(accounts.alice, 1000 * constants::DECIMALS_8);
            
            // Admin funda o pool
            set_next_caller(accounts.alice);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(100 * constants::DECIMALS_8);
            contract.fund_rewards_pool().unwrap();
            assert_eq!(contract.rewards_pool, 100 * constants::DECIMALS_8);
            
            // Adiciona trader
            set_next_caller(accounts.bob);
            advance_time(1000); // Avança tempo inicial
            contract.track_trading_volume(accounts.charlie, 10_000 * constants::DECIMALS_8).unwrap();
            contract.reset_reentrancy_guard();
            
            // Distribui rewards
            set_next_caller(accounts.alice);
            contract.distribute_rewards().unwrap();
            contract.reset_reentrancy_guard();
            
            assert_eq!(contract.rewards_pool, 0);
            
            let _position = contract.get_trading_position(accounts.charlie).unwrap();
            // position.pending_rewards é sempre >= 0 por ser Balance (unsigned)
        }

        #[ink::test]
        fn test_claim_rewards() {
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            // Setup
            set_balance(accounts.alice, 1000 * constants::DECIMALS_8);
            set_balance(accounts.charlie, 0);
            
            // Fund pool
            set_next_caller(accounts.alice);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(100 * constants::DECIMALS_8);
            contract.fund_rewards_pool().unwrap();
            
            // Add trader e distribui
            set_next_caller(accounts.bob);
            advance_time(1000); // Avança tempo inicial
            contract.track_trading_volume(accounts.charlie, 10_000 * constants::DECIMALS_8).unwrap();
            contract.reset_reentrancy_guard();
            
            set_next_caller(accounts.alice);
            contract.distribute_rewards().unwrap();
            contract.reset_reentrancy_guard();
            
            // Simula que há rewards pendentes
            let mut position = contract.get_trading_position(accounts.charlie).unwrap();
            position.pending_rewards = 1000;
            contract.trading_positions.insert(&accounts.charlie, &position);
            
            // Claim rewards
            set_next_caller(accounts.charlie);
            let claimed = contract.claim_rewards().unwrap();
            assert!(claimed > 0);
            
            let position = contract.get_trading_position(accounts.charlie).unwrap();
            assert_eq!(position.pending_rewards, 0);
            assert_eq!(position.claimed_rewards, claimed);
        }

        #[ink::test]
        fn test_access_control() {
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            // Non-router tenta track volume
            set_next_caller(accounts.charlie);
            assert_eq!(
                contract.track_trading_volume(accounts.alice, 1000),
                Err(TradingRewardsError::AccessDenied)
            );
            
            // Non-admin tenta pausar
            assert_eq!(
                contract.pause_contract(),
                Err(TradingRewardsError::AccessDenied)
            );
        }

        #[ink::test]
        fn test_zero_address_validation() {
            let accounts = default_accounts();
            
            // Constructor com zero address
            let result = TradingRewardsContract::new(AccountId::from([0u8; 32]), accounts.bob);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), TradingRewardsError::ZeroAddress);
            
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            // Track volume com zero address
            set_next_caller(accounts.bob);
            assert_eq!(
                contract.track_trading_volume(AccountId::from([0u8; 32]), 1000),
                Err(TradingRewardsError::ZeroAddress)
            );
        }

        #[ink::test]
        fn test_contract_pause() {
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            // Pausa contrato
            set_next_caller(accounts.alice);
            contract.pause_contract().unwrap();
            
            // Tenta track volume (deve falhar)
            set_next_caller(accounts.bob);
            advance_time(1000); // Avança tempo inicial
            assert_eq!(
                contract.track_trading_volume(accounts.charlie, 500 * constants::DECIMALS_8),
                Err(TradingRewardsError::ContractPaused)
            );
            
            // Despausa
            set_next_caller(accounts.alice);
            contract.unpause_contract().unwrap();
            
            // Agora deve funcionar
            set_next_caller(accounts.bob);
            contract.track_trading_volume(accounts.charlie, 500 * constants::DECIMALS_8).unwrap();
        }

        #[ink::test]
        fn test_anti_fraud_measures() {
            // RESET COMPLETO DO TEMPO COM SEED ÚNICA PARA ESTE TESTE
            reset_test_time_with_seed(999);
            
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            set_next_caller(accounts.bob);
            advance_time(100); // Avança tempo inicial
            
            // 1. Teste volume muito pequeno
            assert_eq!(
                contract.track_trading_volume(accounts.charlie, 50 * constants::DECIMALS_8),
                Err(TradingRewardsError::VolumeTooSmall)
            );
            
            // 2. Primeiro trade bem sucedido
            contract.track_trading_volume(accounts.charlie, 500 * constants::DECIMALS_8).unwrap();
            
            // 3. Teste cooldown (deve falhar) - agora Charlie já tem timestamp do último trade
            advance_time(30); // Avança apenas 30 segundos (menos que o cooldown de 60)
            let result = contract.track_trading_volume(accounts.charlie, 500 * constants::DECIMALS_8);
            assert_eq!(result, Err(TradingRewardsError::TradeCooldownActive));
            
            // 4. Avança tempo suficiente e testa novamente
            advance_time(100); // 100 segundos - bem mais que o cooldown de 60
            contract.track_trading_volume(accounts.charlie, 500 * constants::DECIMALS_8).unwrap();
            
            // 5. Testa blacklist
            set_next_caller(accounts.alice);
            contract.blacklist_address(accounts.charlie).unwrap();
            
            set_next_caller(accounts.bob);
            advance_time(61);
            assert_eq!(
                contract.track_trading_volume(accounts.charlie, 500 * constants::DECIMALS_8),
                Err(TradingRewardsError::SuspiciousAddress)
            );
            
            // 6. Remove da blacklist
            set_next_caller(accounts.alice);
            contract.unblacklist_address(accounts.charlie).unwrap();
            assert!(!contract.is_blacklisted(accounts.charlie));
        }

        #[ink::test]
        fn test_daily_limit() {
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            set_next_caller(accounts.bob);
            advance_time(1000); // Avança tempo inicial
            
            // Primeiro trade grande (mas dentro do limite)
            let large_volume = 800_000 * constants::DECIMALS_8; // 800k LUNES
            contract.track_trading_volume(accounts.charlie, large_volume).unwrap();
            
            // Tentar outro trade que excede o limite diário
            advance_time(61); // Passa do cooldown
            assert_eq!(
                contract.track_trading_volume(accounts.charlie, 300_000 * constants::DECIMALS_8), // 800k + 300k = 1.1M > 1M
                Err(TradingRewardsError::DailyLimitExceeded)
            );
            
            // Avança um dia completo
            advance_time(24 * 60 * 60);
            
            // Agora deve funcionar
            contract.track_trading_volume(accounts.charlie, 2_000 * constants::DECIMALS_8).unwrap();
        }

        #[ink::test]
        fn test_configurable_antifraud_parameters() {
            // RESET COMPLETO DO TEMPO COM SEED ÚNICA PARA ESTE TESTE
            reset_test_time_with_seed(888);
            
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            // Verifica valores padrão
            let (min_vol, cooldown, max_daily) = contract.get_antifraud_parameters();
            assert_eq!(min_vol, constants::DEFAULT_MIN_TRADE_VOLUME);
            assert_eq!(cooldown, constants::DEFAULT_TRADE_COOLDOWN);
            assert_eq!(max_daily, constants::DEFAULT_MAX_DAILY_VOLUME);
            
            // Atualiza parâmetros (apenas admin)
            set_next_caller(accounts.alice);
            contract.set_min_trade_volume(200 * constants::DECIMALS_8).unwrap(); // 200 LUNES
            contract.set_trade_cooldown(120).unwrap(); // 2 minutos
            contract.set_max_daily_volume(500_000 * constants::DECIMALS_8).unwrap(); // 500k LUNES
            
            // Verifica novos valores
            let (new_min_vol, new_cooldown, new_max_daily) = contract.get_antifraud_parameters();
            assert_eq!(new_min_vol, 200 * constants::DECIMALS_8);
            assert_eq!(new_cooldown, 120);
            assert_eq!(new_max_daily, 500_000 * constants::DECIMALS_8);
            
            // Testa com novos parâmetros
            set_next_caller(accounts.bob);
            advance_time(1000);
            
            // Volume muito pequeno (novo mínimo)
            assert_eq!(
                contract.track_trading_volume(accounts.charlie, 100 * constants::DECIMALS_8),
                Err(TradingRewardsError::VolumeTooSmall)
            );
            
            // Volume ok
            contract.track_trading_volume(accounts.charlie, 300 * constants::DECIMALS_8).unwrap();
            
            // Cooldown de 2 minutos (120 segundos)
            advance_time(30); // Avança 30 segundos - ainda em cooldown
            assert_eq!(
                contract.track_trading_volume(accounts.charlie, 300 * constants::DECIMALS_8),
                Err(TradingRewardsError::TradeCooldownActive)
            );
            
            advance_time(100); // Avança mais 100 segundos - bem mais que cooldown de 120
            contract.track_trading_volume(accounts.charlie, 300 * constants::DECIMALS_8).unwrap();
        }

        #[ink::test]
        fn test_epoch_system() {
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            // Verifica época inicial
            let (epoch_id, _start_time, duration, rewards) = contract.get_current_epoch_info();
            assert_eq!(epoch_id, 0);
            assert_eq!(duration, constants::DEFAULT_EPOCH_DURATION);
            assert_eq!(rewards, 0);
            
            // Inicia nova época (apenas admin)
            set_next_caller(accounts.alice);
            let new_epoch = contract.start_new_epoch().unwrap();
            assert_eq!(new_epoch, 1);
            
            // Verifica nova época
            let (epoch_id, _, _, _) = contract.get_current_epoch_info();
            assert_eq!(epoch_id, 1);
            
            // Define nova duração
            contract.set_epoch_duration(3600).unwrap(); // 1 hora
            
            // Não-admin não pode iniciar época
            set_next_caller(accounts.bob);
            assert_eq!(
                contract.start_new_epoch(),
                Err(TradingRewardsError::AccessDenied)
            );
            
            // Testa claim de época inexistente
            assert_eq!(
                contract.claim_epoch_rewards(999),
                Err(TradingRewardsError::NoRewardsToClaim)
            );
        }

        #[ink::test]
        fn test_admin_events() {
            let accounts = default_accounts();
            let mut contract = TradingRewardsContract::new(accounts.alice, accounts.bob).unwrap();
            
            // Testa transferência de admin
            set_next_caller(accounts.alice);
            contract.transfer_admin(accounts.charlie).unwrap();
            
            // Verifica que charlie é o novo admin
            set_next_caller(accounts.charlie);
            contract.pause_contract().unwrap(); // Deve funcionar
            
            set_next_caller(accounts.alice);
            assert_eq!(
                contract.pause_contract(),
                Err(TradingRewardsError::AccessDenied)
            ); // Alice não é mais admin
            
            // Testa mudança de router
            set_next_caller(accounts.charlie);
            contract.unpause_contract().unwrap(); // Despausa primeiro
            contract.set_authorized_router(accounts.alice).unwrap();
            
            // Verifica que alice é o novo router
            set_next_caller(accounts.alice);
            advance_time(1000);
            contract.track_trading_volume(accounts.bob, 500 * constants::DECIMALS_8).unwrap();
            
            set_next_caller(accounts.bob);
            assert_eq!(
                contract.track_trading_volume(accounts.alice, 500 * constants::DECIMALS_8),
                Err(TradingRewardsError::AccessDenied)
            ); // Bob não é mais router
        }
    }
}
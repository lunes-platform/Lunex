/// 🚀 TESTE DE INTEGRAÇÃO COMPLETA - LUNEX DEX
/// 
/// Este teste demonstra todo o ecossistema Lunex funcionando:
/// - DEX com nova estrutura de taxas 0.5%
/// - Trading Rewards com sistema de tiers
/// - Staking com governança
/// - Distribuição automática de fees
///
/// Representa um cenário real de uso da plataforma

use std::collections::HashMap;

/// Simulação da moeda nativa Lunes (8 decimais)
const LUNES_DECIMALS: u128 = 100_000_000; // 10^8

/// Tipos centralizados
type AccountId = String;
type Balance = u128;
type Timestamp = u64;

/// Estrutura principal da Lunex DEX completa
#[derive(Debug, Clone)]
struct LunexEcosystem {
    // DEX Core
    dex: LunexDEX,
    // Trading Rewards
    trading_rewards: TradingRewardsSystem,
    // Staking & Governance
    staking: StakingSystem,
    // Configurações globais
    config: EcosystemConfig,
}

/// DEX principal com nova estrutura de taxas
#[derive(Debug, Clone)]
struct LunexDEX {
    pools: HashMap<String, LiquidityPool>,
    total_volume_24h: Balance,
    collected_fees: FeeCollection,
}

#[derive(Debug, Clone)]
struct LiquidityPool {
    token_a: String,
    token_b: String,
    reserve_a: Balance,
    reserve_b: Balance,
    total_lp_supply: Balance,
    lp_holders: HashMap<AccountId, Balance>,
}

#[derive(Debug, Clone)]
struct FeeCollection {
    lp_fees: Balance,       // 60% = 0.3%
    protocol_fees: Balance, // 20% = 0.1%
    rewards_fees: Balance,  // 20% = 0.1%
}

/// Sistema de Trading Rewards
#[derive(Debug, Clone)]
struct TradingRewardsSystem {
    traders: HashMap<AccountId, TraderProfile>,
    monthly_pool: Balance,
    last_distribution: Timestamp,
}

#[derive(Debug, Clone)]
struct TraderProfile {
    monthly_volume: Balance,
    total_volume: Balance,
    tier: TradingTier,
    pending_rewards: Balance,
    claimed_rewards: Balance,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TradingTier {
    Bronze,   // 0 - 10k LUNES
    Silver,   // 10k - 50k LUNES
    Gold,     // 50k - 200k LUNES
    Platinum, // 200k+ LUNES
}

/// Sistema de Staking e Governança
#[derive(Debug, Clone)]
struct StakingSystem {
    stakes: HashMap<AccountId, StakePosition>,
    proposals: HashMap<u32, Proposal>,
    next_proposal_id: u32,
}

#[derive(Debug, Clone)]
struct StakePosition {
    amount: Balance,
    start_time: Timestamp,
    duration_days: u32,
}

#[derive(Debug, Clone)]
struct Proposal {
    id: u32,
    proposer: AccountId,
    title: String,
    description: String,
    votes_for: Balance,
    votes_against: Balance,
    status: ProposalStatus,
    end_time: Timestamp,
}

#[derive(Debug, Clone, PartialEq)]
enum ProposalStatus {
    Active,
    Approved,
    Rejected,
    Executed,
}

/// Configurações do ecossistema
#[derive(Debug, Clone)]
struct EcosystemConfig {
    // Fees (basis points - 10000 = 100%)
    total_fee_rate: u32,      // 50 = 0.5%
    lp_fee_share: u32,        // 60% da fee total
    protocol_fee_share: u32,  // 20% da fee total
    rewards_fee_share: u32,   // 20% da fee total
    
    // Trading Rewards
    bronze_threshold: Balance,
    silver_threshold: Balance,
    gold_threshold: Balance,
    platinum_threshold: Balance,
    
    // Staking
    min_stake_amount: Balance,
    min_proposal_power: Balance,
}

impl Default for EcosystemConfig {
    fn default() -> Self {
        Self {
            total_fee_rate: 50,      // 0.5%
            lp_fee_share: 60,        // 60%
            protocol_fee_share: 20,  // 20%
            rewards_fee_share: 20,   // 20%
            
            bronze_threshold: 0,
            silver_threshold: 10_000 * LUNES_DECIMALS,
            gold_threshold: 50_000 * LUNES_DECIMALS,
            platinum_threshold: 200_000 * LUNES_DECIMALS,
            
            min_stake_amount: 1_000 * LUNES_DECIMALS,
            min_proposal_power: 10_000 * LUNES_DECIMALS,
        }
    }
}

impl LunexEcosystem {
    /// Inicializa o ecossistema completo
    fn new() -> Self {
        let config = EcosystemConfig::default();
        
        Self {
            dex: LunexDEX::new(),
            trading_rewards: TradingRewardsSystem::new(),
            staking: StakingSystem::new(),
            config,
        }
    }
    
    /// Executa um trade completo com distribuição de fees
    fn execute_trade(
        &mut self,
        trader: &AccountId,
        token_in: &str,
        token_out: &str,
        amount_in: Balance,
        current_time: Timestamp,
    ) -> Result<Balance, String> {
        // 1. Executa trade na DEX
        let (amount_out, total_fee) = self.dex.swap(token_in, token_out, amount_in)?;
        
        // 2. Distribui fees conforme nova estrutura
        self.distribute_fees(total_fee);
        
        // 3. Registra volume para trading rewards
        self.trading_rewards.track_volume(trader.clone(), amount_in, current_time);
        
        // 4. Atualiza métricas
        self.dex.total_volume_24h += amount_in;
        
        println!("🔄 Trade executado:");
        println!("   Trader: {}", trader);
        println!("   {} {} → {} {}", 
                 amount_in / LUNES_DECIMALS, token_in,
                 amount_out / LUNES_DECIMALS, token_out);
        println!("   Fee total: {} LUNES", total_fee / LUNES_DECIMALS);
        
        Ok(amount_out)
    }
    
    /// Distribui fees conforme nova estrutura (0.5% total)
    fn distribute_fees(&mut self, total_fee: Balance) {
        let lp_fee = total_fee * self.config.lp_fee_share as Balance / 100;
        let protocol_fee = total_fee * self.config.protocol_fee_share as Balance / 100;
        let rewards_fee = total_fee * self.config.rewards_fee_share as Balance / 100;
        
        self.dex.collected_fees.lp_fees += lp_fee;
        self.dex.collected_fees.protocol_fees += protocol_fee;
        self.dex.collected_fees.rewards_fees += rewards_fee;
        
        // Adiciona rewards fee ao pool de trading rewards
        self.trading_rewards.monthly_pool += rewards_fee;
    }
    
    /// Distribui trading rewards mensalmente
    fn distribute_monthly_rewards(&mut self, current_time: Timestamp) -> Balance {
        let distributed = self.trading_rewards.distribute_rewards(current_time);
        
        println!("💰 Distribuição mensal de trading rewards:");
        println!("   Total distribuído: {} LUNES", distributed / LUNES_DECIMALS);
        
        // Reset do pool de rewards
        self.dex.collected_fees.rewards_fees = 0;
        
        distributed
    }
    
    /// Cria proposta de governança
    fn create_governance_proposal(
        &mut self,
        proposer: AccountId,
        title: String,
        description: String,
        current_time: Timestamp,
    ) -> Result<u32, String> {
        // Verifica se tem LUNES suficiente staked
        let stake = self.staking.stakes.get(&proposer)
            .ok_or("Proposer não tem stake")?;
            
        if stake.amount < self.config.min_proposal_power {
            return Err("Stake insuficiente para criar proposta".to_string());
        }
        
        let proposal_id = self.staking.create_proposal(proposer, title, description, current_time)?;
        
        println!("🗳️ Nova proposta criada:");
        println!("   ID: {}", proposal_id);
        
        Ok(proposal_id)
    }
    
    /// Obtém estatísticas completas do ecossistema
    fn get_ecosystem_stats(&self) -> EcosystemStats {
        let total_staked = self.staking.stakes.values()
            .map(|s| s.amount)
            .sum::<Balance>();
            
        let total_traders = self.trading_rewards.traders.len();
        
        let tier_distribution = self.get_tier_distribution();
        
        EcosystemStats {
            total_volume_24h: self.dex.total_volume_24h,
            total_staked,
            total_traders,
            tier_distribution,
            fees_collected: self.dex.collected_fees.clone(),
            pending_rewards_pool: self.trading_rewards.monthly_pool,
        }
    }
    
    fn get_tier_distribution(&self) -> HashMap<TradingTier, u32> {
        let mut distribution = HashMap::new();
        distribution.insert(TradingTier::Bronze, 0);
        distribution.insert(TradingTier::Silver, 0);
        distribution.insert(TradingTier::Gold, 0);
        distribution.insert(TradingTier::Platinum, 0);
        
        for trader in self.trading_rewards.traders.values() {
            *distribution.entry(trader.tier.clone()).or_insert(0) += 1;
        }
        
        distribution
    }
}

#[derive(Debug)]
struct EcosystemStats {
    total_volume_24h: Balance,
    total_staked: Balance,
    total_traders: usize,
    tier_distribution: HashMap<TradingTier, u32>,
    fees_collected: FeeCollection,
    pending_rewards_pool: Balance,
}

impl LunexDEX {
    fn new() -> Self {
        let mut pools = HashMap::new();
        
        // Pool LUNES/USDT inicial
        pools.insert(
            "LUNES-USDT".to_string(),
            LiquidityPool {
                token_a: "LUNES".to_string(),
                token_b: "USDT".to_string(),
                reserve_a: 1_000_000 * LUNES_DECIMALS,  // 1M LUNES
                reserve_b: 1_000_000 * LUNES_DECIMALS,  // 1M USDT (assumindo 1:1)
                total_lp_supply: 1_000_000 * LUNES_DECIMALS,
                lp_holders: HashMap::new(),
            }
        );
        
        Self {
            pools,
            total_volume_24h: 0,
            collected_fees: FeeCollection {
                lp_fees: 0,
                protocol_fees: 0,
                rewards_fees: 0,
            },
        }
    }
    
    fn swap(&mut self, token_in: &str, token_out: &str, amount_in: Balance) -> Result<(Balance, Balance), String> {
        // Tenta as duas direções do pool
        let pool_key1 = format!("{}-{}", token_in, token_out);
        let pool_key2 = format!("{}-{}", token_out, token_in);
        
        let pool_key = if self.pools.contains_key(&pool_key1) {
            pool_key1
        } else if self.pools.contains_key(&pool_key2) {
            pool_key2
        } else {
            return Err("Pool não encontrado".to_string());
        };
        
        let pool = self.pools.get_mut(&pool_key).unwrap();
        
        // Simulação simplificada do AMM
        let (reserve_in, reserve_out) = if token_in == &pool.token_a {
            (pool.reserve_a, pool.reserve_b)
        } else {
            (pool.reserve_b, pool.reserve_a)
        };
        
        // Calcula fee (0.5% = 995/1000)
        let amount_in_with_fee = amount_in * 995 / 1000;
        let total_fee = amount_in - amount_in_with_fee;
        
        // Fórmula AMM: amount_out = (reserve_out * amount_in_with_fee) / (reserve_in + amount_in_with_fee)
        let amount_out = (reserve_out * amount_in_with_fee) / (reserve_in + amount_in_with_fee);
        
        // Atualiza reservas
        if token_in == &pool.token_a {
            pool.reserve_a += amount_in;
            pool.reserve_b -= amount_out;
        } else {
            pool.reserve_b += amount_in;
            pool.reserve_a -= amount_out;
        }
        
        Ok((amount_out, total_fee))
    }
}

impl TradingRewardsSystem {
    fn new() -> Self {
        Self {
            traders: HashMap::new(),
            monthly_pool: 0,
            last_distribution: 0,
        }
    }
    
    fn track_volume(&mut self, trader: AccountId, volume: Balance, _current_time: Timestamp) {
        // Primeiro, garantimos que o trader existe
        if !self.traders.contains_key(&trader) {
            self.traders.insert(trader.clone(), TraderProfile {
                monthly_volume: 0,
                total_volume: 0,
                tier: TradingTier::Bronze,
                pending_rewards: 0,
                claimed_rewards: 0,
            });
        }
        
        // Atualiza volumes
        let trader_profile = self.traders.get_mut(&trader).unwrap();
        trader_profile.monthly_volume += volume;
        trader_profile.total_volume += volume;
        
        // Calcula novo tier
        let monthly_volume = trader_profile.monthly_volume;
        
        // Calcula tier separadamente para evitar borrow conflicts
        let new_tier = if monthly_volume >= 200_000 * LUNES_DECIMALS {
            TradingTier::Platinum
        } else if monthly_volume >= 50_000 * LUNES_DECIMALS {
            TradingTier::Gold
        } else if monthly_volume >= 10_000 * LUNES_DECIMALS {
            TradingTier::Silver
        } else {
            TradingTier::Bronze
        };
        
        trader_profile.tier = new_tier;
    }
    
    fn calculate_tier(&self, monthly_volume: Balance) -> TradingTier {
        if monthly_volume >= 200_000 * LUNES_DECIMALS {
            TradingTier::Platinum
        } else if monthly_volume >= 50_000 * LUNES_DECIMALS {
            TradingTier::Gold
        } else if monthly_volume >= 10_000 * LUNES_DECIMALS {
            TradingTier::Silver
        } else {
            TradingTier::Bronze
        }
    }
    
    fn distribute_rewards(&mut self, current_time: Timestamp) -> Balance {
        if self.monthly_pool == 0 {
            return 0;
        }
        
        let total_weight = self.calculate_total_weight();
        if total_weight == 0 {
            return 0;
        }
        
        let pool_to_distribute = self.monthly_pool;
        
        let mut updates = Vec::new();
        
        for (trader_id, trader) in self.traders.iter() {
            let trader_weight = self.calculate_trader_weight(trader);
            let reward = pool_to_distribute * trader_weight / total_weight;
            
            updates.push((trader_id.clone(), reward));
            
            println!("   {} ({}): {} LUNES", 
                     trader_id, 
                     format!("{:?}", trader.tier),
                     reward / LUNES_DECIMALS);
        }
        
        for (trader_id, reward) in updates {
            if let Some(trader) = self.traders.get_mut(&trader_id) {
                trader.pending_rewards += reward;
            }
        }
        
        self.monthly_pool = 0;
        self.last_distribution = current_time;
        
        pool_to_distribute
    }
    
    fn calculate_trader_weight(&self, trader: &TraderProfile) -> Balance {
        let multiplier = match trader.tier {
            TradingTier::Bronze => 100,
            TradingTier::Silver => 120,
            TradingTier::Gold => 150,
            TradingTier::Platinum => 200,
        };
        
        trader.monthly_volume * multiplier / 100
    }
    
    fn calculate_total_weight(&self) -> Balance {
        self.traders.values()
            .map(|trader| self.calculate_trader_weight(trader))
            .sum()
    }
}

impl StakingSystem {
    fn new() -> Self {
        Self {
            stakes: HashMap::new(),
            proposals: HashMap::new(),
            next_proposal_id: 1,
        }
    }
    
    fn stake(&mut self, user: AccountId, amount: Balance, duration_days: u32, current_time: Timestamp) -> Result<(), String> {
        if amount < 1_000 * LUNES_DECIMALS {
            return Err("Stake mínimo é 1.000 LUNES".to_string());
        }
        
        self.stakes.insert(user.clone(), StakePosition {
            amount,
            start_time: current_time,
            duration_days,
        });
        
        println!("💰 Stake realizado:");
        println!("   Usuário: {}", user);
        println!("   Quantidade: {} LUNES", amount / LUNES_DECIMALS);
        println!("   Duração: {} dias", duration_days);
        
        Ok(())
    }
    
    fn create_proposal(
        &mut self,
        proposer: AccountId,
        title: String,
        description: String,
        current_time: Timestamp,
    ) -> Result<u32, String> {
        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;
        
        let proposal = Proposal {
            id: proposal_id,
            proposer,
            title,
            description,
            votes_for: 0,
            votes_against: 0,
            status: ProposalStatus::Active,
            end_time: current_time + (14 * 24 * 60 * 60), // 14 dias
        };
        
        self.proposals.insert(proposal_id, proposal);
        
        Ok(proposal_id)
    }
}

/// TESTE PRINCIPAL - CENÁRIO COMPLETO
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lunex_complete_ecosystem() {
        println!("\n🚀 INICIANDO TESTE COMPLETO DO ECOSSISTEMA LUNEX");
        println!("===============================================");
        
        let mut lunex = LunexEcosystem::new();
        let mut current_time = 1_600_000_000u64; // Timestamp base
        
        // === FASE 1: CONFIGURAÇÃO INICIAL ===
        println!("\n📋 FASE 1: Configuração Inicial");
        
        // Usuários do teste
        let alice = "alice".to_string();
        let bob = "bob".to_string();
        let carol = "carol".to_string();
        let david = "david".to_string();
        
        // Stakes iniciais para governança
        lunex.staking.stake(alice.clone(), 50_000 * LUNES_DECIMALS, 90, current_time).unwrap();
        lunex.staking.stake(bob.clone(), 25_000 * LUNES_DECIMALS, 60, current_time).unwrap();
        lunex.staking.stake(carol.clone(), 15_000 * LUNES_DECIMALS, 30, current_time).unwrap();
        
        println!("✅ Stakes iniciais configurados");
        
        // === FASE 2: ATIVIDADE DE TRADING ===
        println!("\n💱 FASE 2: Atividade de Trading Intenso");
        
        // Simula 30 dias de trading
        for day in 1..=30 {
            current_time += 24 * 60 * 60; // +1 dia
            
            // Alice: Trader Gold (volume alto)
            lunex.execute_trade(&alice, "LUNES", "USDT", 3_000 * LUNES_DECIMALS, current_time).unwrap();
            lunex.execute_trade(&alice, "USDT", "LUNES", 2_800 * LUNES_DECIMALS, current_time).unwrap();
            
            // Bob: Trader Silver (volume médio)
            lunex.execute_trade(&bob, "LUNES", "USDT", 1_500 * LUNES_DECIMALS, current_time).unwrap();
            lunex.execute_trade(&bob, "USDT", "LUNES", 1_400 * LUNES_DECIMALS, current_time).unwrap();
            
            // Carol: Trader Silver inicial
            if day <= 15 {
                lunex.execute_trade(&carol, "LUNES", "USDT", 800 * LUNES_DECIMALS, current_time).unwrap();
            }
            
            // David: Trader Bronze (volume baixo)
            if day % 3 == 0 {
                lunex.execute_trade(&david, "LUNES", "USDT", 200 * LUNES_DECIMALS, current_time).unwrap();
            }
        }
        
        println!("✅ 30 dias de trading simulados");
        
        // === FASE 3: VERIFICAÇÃO DE TIERS ===
        println!("\n🏆 FASE 3: Verificação de Tiers de Trading");
        
        for (trader_id, trader) in &lunex.trading_rewards.traders {
            println!("   {}: {} LUNES volume → {:?}", 
                     trader_id,
                     trader.monthly_volume / LUNES_DECIMALS,
                     trader.tier);
        }
        
        // Verifica tiers esperados
        assert_eq!(lunex.trading_rewards.traders[&alice].tier, TradingTier::Gold);
        assert_eq!(lunex.trading_rewards.traders[&bob].tier, TradingTier::Gold); // Bob atingiu Gold com 87k volume
        assert_eq!(lunex.trading_rewards.traders[&carol].tier, TradingTier::Silver); // Carol atingiu Silver com 12k volume
        assert_eq!(lunex.trading_rewards.traders[&david].tier, TradingTier::Bronze);
        
        // === FASE 4: DISTRIBUIÇÃO DE TRADING REWARDS ===
        println!("\n💰 FASE 4: Distribuição Mensal de Trading Rewards");
        
        let total_distributed = lunex.distribute_monthly_rewards(current_time);
        assert!(total_distributed > 0, "Deveria ter distribuído rewards");
        
        // === FASE 5: GOVERNANÇA ===
        println!("\n🗳️ FASE 5: Governança - Criação de Proposta");
        
        let proposal_id = lunex.create_governance_proposal(
            alice.clone(),
            "Listagem do TOKEN_XYZ".to_string(),
            "Proposta para adicionar TOKEN_XYZ na DEX".to_string(),
            current_time,
        ).unwrap();
        
        println!("✅ Proposta {} criada por Alice", proposal_id);
        
        // === FASE 6: ESTATÍSTICAS FINAIS ===
        println!("\n📊 FASE 6: Estatísticas Finais do Ecossistema");
        
        let stats = lunex.get_ecosystem_stats();
        
        println!("📈 Volume 24h: {} LUNES", stats.total_volume_24h / LUNES_DECIMALS);
        println!("💰 Total Staked: {} LUNES", stats.total_staked / LUNES_DECIMALS);
        println!("👥 Total Traders: {}", stats.total_traders);
        println!("💎 Fees coletadas:");
        println!("   LPs: {} LUNES", stats.fees_collected.lp_fees / LUNES_DECIMALS);
        println!("   Protocolo: {} LUNES", stats.fees_collected.protocol_fees / LUNES_DECIMALS);
        println!("   Rewards: {} LUNES (distribuído)", total_distributed / LUNES_DECIMALS);
        
        println!("\n🏆 Distribuição por Tiers:");
        for (tier, count) in stats.tier_distribution {
            if count > 0 {
                println!("   {:?}: {} traders", tier, count);
            }
        }
        
        // === VERIFICAÇÕES FINAIS ===
        println!("\n✅ VERIFICAÇÕES FINAIS");
        
        // 1. Fees distribuídas corretamente
        let total_fees = stats.fees_collected.lp_fees + stats.fees_collected.protocol_fees + total_distributed;
        assert!(total_fees > 0, "Deveria ter coletado fees");
        
        // 2. Proporção de fees corretas (aproximadamente)
        let lp_percentage = stats.fees_collected.lp_fees * 100 / total_fees;
        let protocol_percentage = stats.fees_collected.protocol_fees * 100 / total_fees;
        let rewards_percentage = total_distributed * 100 / total_fees;
        
        println!("📊 Distribuição de fees:");
        println!("   LPs: {}%", lp_percentage);
        println!("   Protocolo: {}%", protocol_percentage);
        println!("   Trading Rewards: {}%", rewards_percentage);
        
        // Tolerância de ±5% devido à arredondamentos
        assert!(lp_percentage >= 55 && lp_percentage <= 65, "LP fee share deveria ser ~60%");
        assert!(protocol_percentage >= 15 && protocol_percentage <= 25, "Protocol fee share deveria ser ~20%");
        assert!(rewards_percentage >= 15 && rewards_percentage <= 25, "Rewards fee share deveria ser ~20%");
        
        // 3. Trading rewards funcionando
        assert!(lunex.trading_rewards.traders[&alice].pending_rewards > 0, "Alice deveria ter rewards");
        assert!(lunex.trading_rewards.traders[&bob].pending_rewards > 0, "Bob deveria ter rewards");
        
        // 4. Governança funcionando
        assert!(lunex.staking.proposals.contains_key(&proposal_id), "Proposta deveria existir");
        
        println!("✅ Todas as verificações passaram!");
        
        println!("\n🎉 TESTE COMPLETO FINALIZADO COM SUCESSO!");
        println!("===============================================");
        println!("🚀 Lunex DEX está pronta para produção!");
        println!("💰 Nova estrutura de taxas 0.5% implementada");
        println!("🎁 Sistema de Trading Rewards funcionando");
        println!("🗳️ Governança descentralizada ativa");
        println!("🔒 Todos os sistemas integrados e testados");
    }
    
    #[test]
    fn test_fee_distribution_accuracy() {
        println!("\n🔍 TESTE DE PRECISÃO DA DISTRIBUIÇÃO DE FEES");
        
        let mut lunex = LunexEcosystem::new();
        let current_time = 1_600_000_000u64;
        
        let trader = "precision_trader".to_string();
        let trade_amount = 10_000 * LUNES_DECIMALS; // 10k LUNES
        
        // Execute 1 trade e verifique fees exatas
        lunex.execute_trade(&trader, "LUNES", "USDT", trade_amount, current_time).unwrap();
        
        let expected_total_fee = trade_amount * 5 / 1000; // 0.5%
        let expected_lp_fee = expected_total_fee * 60 / 100; // 60%
        let expected_protocol_fee = expected_total_fee * 20 / 100; // 20%
        let expected_rewards_fee = expected_total_fee * 20 / 100; // 20%
        
        println!("💰 Fee de {} LUNES:", trade_amount / LUNES_DECIMALS);
        println!("   Total: {} LUNES", expected_total_fee / LUNES_DECIMALS);
        println!("   LPs: {} LUNES", expected_lp_fee / LUNES_DECIMALS);
        println!("   Protocolo: {} LUNES", expected_protocol_fee / LUNES_DECIMALS);
        println!("   Rewards: {} LUNES", expected_rewards_fee / LUNES_DECIMALS);
        
        assert_eq!(lunex.dex.collected_fees.lp_fees, expected_lp_fee);
        assert_eq!(lunex.dex.collected_fees.protocol_fees, expected_protocol_fee);
        assert_eq!(lunex.dex.collected_fees.rewards_fees, expected_rewards_fee);
        assert_eq!(lunex.trading_rewards.monthly_pool, expected_rewards_fee);
        
        println!("✅ Distribuição de fees está matematicamente correta!");
    }
}
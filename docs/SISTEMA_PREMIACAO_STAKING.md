# 🏆 **SISTEMA DE PREMIAÇÃO PARA STAKING - LUNEX DEX**

## 🎯 **Visão Geral**

O novo sistema de premiação para staking incentivará os usuários a manter LUNES em stake por períodos mais longos e participar ativamente da governança, criando mais valor para o ecossistema.

---

## 💎 **TIPOS DE PREMIAÇÃO**

### **1. 📈 Recompensas Base (Existente - Melhorado)**

#### **Taxa Anual por Duração:**
```
🔸 7-30 dias:   8% APY   (Bronze Staker)
🔸 31-90 dias:  10% APY  (Silver Staker) 
🔸 91-180 dias: 12% APY  (Gold Staker)
🔸 181+ dias:   15% APY  (Platinum Staker)
```

#### **Multiplicadores de Quantidade:**
```
🔸 1k-10k LUNES:    1.0x Base
🔸 10k-50k LUNES:   1.1x Base  (+10%)
🔸 50k-200k LUNES:  1.2x Base  (+20%)
🔸 200k+ LUNES:     1.3x Base  (+30%)
```

### **2. 🎁 Bônus de Trading Rewards**

#### **Stakers recebem % das Trading Rewards:**
```
🔸 Bronze Staker:   5% das Trading Rewards
🔸 Silver Staker:   10% das Trading Rewards
🔸 Gold Staker:     15% das Trading Rewards
🔸 Platinum Staker: 20% das Trading Rewards
```

#### **Distribuição Proporcional:**
- **Base:** Volume de stake + duração
- **Multiplicador:** Tier do staker
- **Pool:** 10% das Trading Rewards vão para stakers

### **3. 🗳️ Bônus de Governança**

#### **Participação Ativa:**
```
🔸 Criar proposta aprovada:    1000 LUNES bonus
🔸 Votar em 80% das propostas: 200 LUNES bonus/mês
🔸 Proposta implementada:      5000 LUNES bonus
```

#### **Early Adopter Bonus:**
```
🔸 Top 100 primeiros stakers:  +50% rewards por 3 meses
🔸 Top 500 primeiros stakers:  +25% rewards por 2 meses
🔸 Top 1000 primeiros stakers: +10% rewards por 1 mês
```

### **4. 🎪 Eventos Especiais**

#### **Campanhas Sazonais:**
```
🔸 Lançamento da DEX:     Double rewards por 30 dias
🔸 Listagem de novo token: +500 LUNES para voters
🔸 Milestone de volume:    Jackpot proporcional
🔸 Aniversário da rede:    NFT exclusivo + bonus
```

---

## ⚡ **IMPLEMENTAÇÃO TÉCNICA**

### **Modificações no Contrato de Staking:**

#### **1. Novos Campos no Storage:**
```rust
/// Sistema de premiação expandido
pub struct StakingContract {
    // ... campos existentes ...
    
    /// Pool de bônus de trading rewards para stakers
    pub trading_rewards_pool: Balance,
    
    /// Referência ao contrato de trading rewards
    pub trading_rewards_contract: Option<AccountId>,
    
    /// Multiplicadores por tier
    pub tier_multipliers: Mapping<StakingTier, u32>,
    
    /// Bônus de governança acumulados
    pub governance_bonuses: Mapping<AccountId, Balance>,
    
    /// Histórico de participação em votações
    pub voting_participation: Mapping<AccountId, u32>,
    
    /// Campanhas ativas
    pub active_campaigns: Mapping<u32, Campaign>,
    
    /// Early adopter tracking
    pub early_adopters: Mapping<AccountId, EarlyAdopterTier>,
}
```

#### **2. Novas Estruturas:**
```rust
/// Tiers de staking baseados em duração
#[derive(scale::Decode, scale::Encode, Clone, Copy, PartialEq, Eq)]
pub enum StakingTier {
    Bronze,   // 7-30 dias
    Silver,   // 31-90 dias  
    Gold,     // 91-180 dias
    Platinum, // 181+ dias
}

/// Informações de campanha
#[derive(scale::Decode, scale::Encode, Clone)]
pub struct Campaign {
    pub name: String,
    pub bonus_rate: u32,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub active: bool,
}

/// Tier de early adopter
#[derive(scale::Decode, scale::Encode, Clone, Copy)]
pub enum EarlyAdopterTier {
    None,
    Top1000,  // +10% por 1 mês
    Top500,   // +25% por 2 meses  
    Top100,   // +50% por 3 meses
}
```

### **3. Novas Funções:**

#### **Integração com Trading Rewards:**
```rust
/// Recebe trading rewards do contrato de rewards
#[ink(message, payable)]
pub fn fund_staking_rewards(&mut self) -> Result<(), StakingError> {
    // Apenas trading rewards contract pode chamar
    self.ensure_authorized_trading_contract()?;
    
    let amount = self.env().transferred_value();
    self.trading_rewards_pool = self.trading_rewards_pool
        .checked_add(amount)
        .ok_or(StakingError::Overflow)?;
    
    Ok(())
}

/// Distribui trading rewards para stakers
#[ink(message)]
pub fn distribute_trading_rewards(&mut self) -> Result<(), StakingError> {
    self.ensure_admin()?;
    
    if self.trading_rewards_pool == 0 {
        return Ok(());
    }
    
    let total_weight = self.calculate_total_staking_weight()?;
    if total_weight == 0 {
        return Ok(());
    }
    
    // Distribui proporcionalmente
    for i in 0..self.staker_index {
        if let Some(staker) = self.staker_addresses.get(&i) {
            if let Some(mut stake) = self.stakes.get(&staker) {
                if stake.active {
                    let weight = self.calculate_staker_weight(&stake);
                    let reward = self.trading_rewards_pool
                        .checked_mul(weight)
                        .ok_or(StakingError::Overflow)?
                        .checked_div(total_weight)
                        .ok_or(StakingError::Overflow)?;
                    
                    stake.pending_rewards = stake.pending_rewards
                        .checked_add(reward)
                        .ok_or(StakingError::Overflow)?;
                    
                    self.stakes.insert(&staker, &stake);
                }
            }
        }
    }
    
    self.trading_rewards_pool = 0;
    Ok(())
}
```

#### **Sistema de Tiers:**
```rust
/// Calcula tier baseado na duração
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

/// Calcula peso do staker para distribuição
fn calculate_staker_weight(&self, stake: &StakePosition) -> Balance {
    let tier = self.calculate_staking_tier(stake.duration);
    let tier_multiplier = self.tier_multipliers.get(&tier).unwrap_or(100);
    
    // Peso = quantidade * multiplicador_tier * multiplicador_quantidade
    let quantity_multiplier = self.get_quantity_multiplier(stake.amount);
    
    stake.amount
        .checked_mul(tier_multiplier as Balance)
        .unwrap_or(0)
        .checked_mul(quantity_multiplier as Balance)
        .unwrap_or(0)
        .checked_div(10000) // Normalizar basis points
        .unwrap_or(0)
}
```

#### **Bônus de Governança:**
```rust
/// Registra participação em votação
pub fn record_vote_participation(&mut self, voter: AccountId) -> Result<(), StakingError> {
    // Apenas contrato de governança pode chamar
    self.ensure_authorized_governance()?;
    
    let current_participation = self.voting_participation.get(&voter).unwrap_or(0);
    self.voting_participation.insert(&voter, &(current_participation + 1));
    
    // Bônus por participação ativa (80% das votações no mês)
    if current_participation + 1 >= 8 { // Assumindo ~10 votações/mês
        let bonus = 200 * constants::DECIMALS_8; // 200 LUNES
        let current_bonus = self.governance_bonuses.get(&voter).unwrap_or(0);
        self.governance_bonuses.insert(&voter, &(current_bonus + bonus));
    }
    
    Ok(())
}

/// Paga bônus de proposta aprovada
pub fn reward_approved_proposal(&mut self, proposer: AccountId) -> Result<(), StakingError> {
    self.ensure_authorized_governance()?;
    
    let bonus = 1000 * constants::DECIMALS_8; // 1000 LUNES
    let current_bonus = self.governance_bonuses.get(&proposer).unwrap_or(0);
    self.governance_bonuses.insert(&proposer, &(current_bonus + bonus));
    
    Ok(())
}
```

---

## 📊 **NOVA ESTRUTURA DE REWARDS**

### **Distribuição das Trading Rewards (Atualizada):**

```
🔸 60% para LPs (Provedores de Liquidez)     [MANTIDO]
🔸 15% para Protocol/Desenvolvimento         [REDUZIDO de 20%]
🔸 15% para Trading Rewards                  [REDUZIDO de 20%]
🔸 10% para Staking Rewards                  [NOVO]
```

### **Impacto na Tokenomics:**

#### **Antes:**
```
Taxa total: 0.5%
├── 60% LPs (0.3%)
├── 20% Protocol (0.1%)  
└── 20% Trading (0.1%)
```

#### **Depois:**
```
Taxa total: 0.5%
├── 60% LPs (0.3%)
├── 15% Protocol (0.075%)
├── 15% Trading (0.075%)
└── 10% Staking (0.05%)
```

---

## 🚀 **INCENTIVOS E BENEFÍCIOS**

### **Para os Usuários:**
1. **🔒 Múltiplas fontes de renda** - Base APY + Trading rewards + Governance
2. **⏰ Rewards crescentes** - Quanto mais tempo, maior o retorno
3. **🏛️ Poder de governança** - Voz na evolução da plataforma
4. **🎁 Eventos exclusivos** - Campanhas e bônus especiais

### **Para o Protocolo:**
1. **🔐 Maior estabilidade** - LUNES locked por períodos longos
2. **📊 Governança ativa** - Comunidade engajada nas decisões
3. **💰 Redução da pressão de venda** - Incentivos para hold
4. **🌱 Crescimento sustentável** - Recompensas financiadas pelo próprio volume

---

## 📈 **PROJEÇÕES DE IMPACTO**

### **Cenário Conservador (6 meses):**
```
🔸 Total Staked: 50M LUNES (25% do supply)
🔸 Stakers ativos: 5,000 usuários
🔸 Recompensas distribuídas: 2M LUNES
🔸 Participação governança: 60%
```

### **Cenário Otimista (12 meses):**
```
🔸 Total Staked: 120M LUNES (60% do supply)
🔸 Stakers ativos: 15,000 usuários  
🔸 Recompensas distribuídas: 8M LUNES
🔸 Participação governança: 80%
```

---

## 🛡️ **MEDIDAS DE SEGURANÇA**

### **Proteções Anti-Gaming:**
1. **⏱️ Lock periods** - Prevenção de stake/unstake rápido
2. **📊 Weight normalization** - Limite de influência por wallet
3. **🔍 Governance monitoring** - Detecção de coordenação maliciosa
4. **⚖️ Penalty system** - Punições por early unstaking

### **Sustentabilidade:**
1. **💧 Gradual emission** - Recompensas distribuídas ao longo do tempo
2. **📉 Decreasing rates** - APY reduz conforme total staked aumenta
3. **🔄 Pool rebalancing** - Ajuste automático de distribuição
4. **🛑 Emergency controls** - Pausar sistema em caso de bugs

---

## 🎯 **CRONOGRAMA DE IMPLEMENTAÇÃO**

### **Fase 1 (1-2 semanas):**
- ✅ Modificar contrato de staking
- ✅ Implementar sistema de tiers
- ✅ Adicionar integração com trading rewards
- ✅ Testes unitários completos

### **Fase 2 (1 semana):**
- 🔄 Atualizar contrato de trading rewards
- 🔄 Implementar nova distribuição de fees
- 🔄 Testes de integração E2E

### **Fase 3 (1 semana):**
- 🔄 Sistema de governança expandido
- 🔄 Bônus de participação
- 🔄 Interface para campanhas

### **Fase 4 (1 semana):**
- 🔄 Testes de segurança
- 🔄 Auditoria do sistema completo
- 🔄 Deploy e migração

---

## 💡 **VANTAGENS COMPETITIVAS**

### **Único no Mercado:**
1. **🔄 Multi-layered rewards** - Staking + Trading + Governance
2. **⚡ Dynamic tiers** - Rewards que evoluem com comprometimento
3. **🏛️ Governance-to-earn** - Pagar para participar da governança
4. **🎪 Gamified experience** - Eventos, achievements, progressão

### **Sustentabilidade Econômica:**
1. **💰 Self-funded** - Recompensas vêm das próprias taxas
2. **⚖️ Balanced incentives** - Beneficia stakers sem prejudicar traders
3. **📈 Growth-aligned** - Mais volume = mais rewards
4. **🔒 Capital efficiency** - LUNES locked gera valor real

---

## 🎉 **CONCLUSÃO**

O novo sistema de premiação para staking transformará a Lunex DEX no protocolo DeFi mais atrativo da rede Lunes, criando um ciclo virtuoso de:

**STAKE → GOVERN → EARN → STAKE MAIS**

**🚀 Ready to implement and revolutionize staking rewards!**
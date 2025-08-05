# 🗳️ **SISTEMA DE GOVERNANÇA DE TAXAS - LUNEX DEX**

## 📋 **RESUMO**

A Lunex DEX implementa um sistema democrático e flexível onde a própria comunidade pode ajustar a taxa de criação de propostas através de votação. Isso garante que o sistema evolua conforme as necessidades da comunidade.

---

## ⚖️ **CONFIGURAÇÃO INICIAL**

### **Taxa Padrão:**
- **1,000 LUNES** (100,000,000,000 unidades com 8 decimais)
- **Reembolsável** se a proposta for aprovada
- **Distribuída** se a proposta for rejeitada

### **Limites de Taxa:**
- **Mínimo:** > 0 LUNES 
- **Máximo:** 100,000 LUNES (para evitar barreiras excessivas)

---

## 🏛️ **COMO FUNCIONA**

### **1. Proposta de Mudança de Taxa**

Qualquer usuário com **voting power suficiente** (≥ 10,000 LUNES stakados) pode propor uma nova taxa:

```rust
staking.propose_fee_change(
    new_fee: Balance,        // Nova taxa em unidades (ex: 50_000_000_000 = 500 LUNES)
    justification: String    // Razão para a mudança
) -> Result<u32, StakingError>
```

**Requisitos:**
- Pagar a taxa atual (1,000 LUNES inicialmente)
- Ter ≥ 10,000 LUNES stakados
- Fornecer justificativa

**Exemplo:**
```javascript
// Propor redução para 500 LUNES
await staking.propose_fee_change(
    50_000_000_000,  // 500 LUNES
    "Reduzir barreira de entrada para pequenos projetos"
);
```

### **2. Votação**

- **Duração:** 7 dias
- **Poder de Voto:** Proporcional ao valor stakado
- **Quórum:** Não há mínimo (seguindo padrão de outras propostas)

### **3. Execução**

Após o período de votação:

**Se APROVADA:**
- Taxa é alterada para o novo valor
- Taxa paga é **reembolsada** ao proponente
- Evento `ProposalFeeChanged` é emitido

**Se REJEITADA:**
- Taxa permanece inalterada
- Taxa paga é **distribuída**:
  - 50% → Tesouraria do projeto
  - 20% → Equipe (owner)
  - 20% → Pool de Trading Rewards
  - 10% → Pool de Staking Rewards

---

## 📊 **EVENTOS E MONITORAMENTO**

### **Eventos Emitidos:**

```rust
// Quando uma proposta de mudança é criada
FeeChangeProposed {
    proposal_id: u32,
    proposer: AccountId,
    current_fee: Balance,
    proposed_fee: Balance,
    justification: String,
    voting_deadline: Timestamp,
}

// Quando a taxa é efetivamente alterada
ProposalFeeChanged {
    proposal_id: u32,
    old_fee: Balance,
    new_fee: Balance,
    changed_by: AccountId,
    timestamp: Timestamp,
}
```

### **Consulta da Taxa Atual:**

```rust
staking.get_current_proposal_fee() -> Balance
```

---

## 🔍 **IDENTIFICAÇÃO DE PROPOSTAS**

**Propostas de mudança de taxa são identificadas por:**
- `token_address` = `0x0000000000000000000000000000000000000000` (zero address)
- `new_fee_amount` = `Some(nova_taxa)`
- `name` = `"MUDANCA_TAXA_PROPOSTA"`

---

## 💡 **CASOS DE USO**

### **Cenário 1: Barreira de Entrada Alta**
Se 1,000 LUNES estiver muito caro, a comunidade pode votar para reduzir para 500 LUNES.

### **Cenário 2: Spam de Propostas**
Se houver muitas propostas de baixa qualidade, a comunidade pode votar para aumentar para 2,000 LUNES.

### **Cenário 3: Crescimento do Ecossistema**
Conforme LUNES valoriza, a taxa pode ser ajustada proporcionalmente.

---

## 🛡️ **MEDIDAS DE SEGURANÇA**

### **Anti-Spam:**
- Taxa mínima sempre aplicada
- Voting power necessário
- Justificativa obrigatória

### **Limites Razoáveis:**
- Taxa máxima de 100,000 LUNES
- Taxa mínima > 0

### **Transparência:**
- Todas as mudanças são registradas em eventos
- Histórico completo de propostas
- Justificativas públicas

---

## 📋 **COMANDOS PRÁTICOS**

### **Via Interface Web (Polkadot.js Apps):**
1. Conectar wallet
2. Ir para Contracts → Staking
3. Chamar `propose_fee_change`
4. Aguardar período de votação
5. Executar proposta

### **Via Script:**
```typescript
// Verificar taxa atual
const currentFee = await staking.query.getCurrentProposalFee();

// Propor nova taxa
const { result } = await staking.tx.proposeFeeChange(
    newFee,
    "Justificativa aqui",
    { value: currentFee }
);

// Votar
await staking.tx.vote(proposalId, true); // true = a favor

// Executar após deadline
await staking.tx.executeProposal(proposalId);
```

---

## 📈 **IMPACTO NO ECOSISTEMA**

### **Benefícios:**
- **Democracia:** Comunidade controla as regras
- **Flexibilidade:** Adaptação às condições de mercado
- **Sustentabilidade:** Taxa adequada garante qualidade das propostas
- **Revenue:** Taxas rejeitadas financiam o desenvolvimento

### **Considerações:**
- **Participação:** Requer engajamento da comunidade
- **Timing:** Mudanças levam tempo (7 dias de votação)
- **Consenso:** Precisa de maioria para mudanças

---

## 🎯 **PRÓXIMOS PASSOS**

1. **Deploy:** Sistema já implementado no contrato Staking
2. **Testes:** Testes automatizados passando
3. **Interface:** Integrar com frontend para facilitar uso
4. **Documentação:** Guias para usuários finais
5. **Governança:** Primeiras propostas pela comunidade

---

## 💻 **EXEMPLO COMPLETO DE USO**

```bash
# 1. Verificar taxa atual
npm run admin-list-token check-fee

# 2. Propor nova taxa
npm run governance propose-fee-change 50000000000 "Reduzir para 500 LUNES"

# 3. Votar na proposta
npm run governance vote 123 true

# 4. Executar após deadline
npm run governance execute 123

# 5. Verificar nova taxa
npm run admin-list-token check-fee
```

---

## 🏆 **RESULTADO**

A Lunex DEX agora possui um sistema de governança **completamente democrático** onde:

✅ **Comunidade controla as taxas**  
✅ **Sistema adaptável e flexível**  
✅ **Transparente e auditável**  
✅ **Sustentável financeiramente**  
✅ **Testado e funcional**  

**A plataforma evolui com suas necessidades!** 🚀
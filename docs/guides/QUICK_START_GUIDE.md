# ⚡ **LUNEX DEX - GUIA RÁPIDO DE COMANDOS**

## 🚀 **SETUP INICIAL**

```bash
# 1. Instalar dependências
npm install

# 2. Configurar ambiente Rust
npm run setup:dev

# 3. Compilar todos os contratos
npm run compile:all

# 4. Executar testes
npm run test:unit
npm run test:security
```

---

## 🌐 **DEPLOY NO LUNES BLOCKCHAIN**

### **🧪 TESTNET**

```bash
# Deploy completo na testnet (dry run)
npm run deploy:dry-run

# Deploy real na testnet
npm run deploy:testnet "//Alice"

# Deploy com seed customizada
npm run deploy:lunes testnet "your twelve word seed phrase here"
```

### **🏭 MAINNET**

```bash
# Deploy na mainnet (ATENÇÃO: custos reais!)
npm run deploy:mainnet "your twelve word seed phrase here"
```

---

## 💎 **LISTAGEM DE TOKENS**

### **🔧 VIA ADMIN (Team do Projeto)**

#### **Configuração para Admin Listing:**
```bash
# Copiar exemplo de configuração
cp examples/admin-tokens.json my-admin-tokens.json

# Editar com tokens reais do ecossistema Lunes
nano my-admin-tokens.json
```

#### **Comandos de Admin:**
```bash
# Listar tokens iniciais (batch) - LANÇAMENTO
npm run admin-list-token list examples/admin-tokens.json

# Listar token individual
npm run admin-list-token list-single 5ABC...TOKEN_ADDR "USDT Stablecoin"

# Remover token problemático (emergência)
npm run admin-list-token delist 5BAD...TOKEN_ADDR "Token com problemas"

# Verificar se token está listado
npm run admin-list-token check 5ABC...TOKEN_ADDR

# Ver estatísticas de listagem
npm run admin-list-token stats
```

### **🗳️ VIA GOVERNANÇA (Comunidade)**

#### **1. Preparar Configuração**
```bash
# Copiar exemplo de configuração
cp examples/token-listing-config.json my-token-config.json

# Editar com informações do seu token
nano my-token-config.json
```

### **2. Exemplo de Configuração**
```json
{
  "network": "testnet",
  "proposerSeed": "//Alice",
  "stakingContract": "5GHU...ADDRESS_FROM_DEPLOY",
  "factoryContract": "5FHU...ADDRESS_FROM_DEPLOY",
  "routerContract": "5EHU...ADDRESS_FROM_DEPLOY",
  "token": {
    "address": "5DHU...YOUR_TOKEN_ADDRESS",
    "name": "My Amazing Token",
    "symbol": "MAT",
    "decimals": 8,
    "description": "Descrição do token...",
    "website": "https://mytoken.com"
  },
  "initialLiquidity": {
    "tokenAmount": "1000000000000000",
    "lunesAmount": "10000000000000"
  }
}
```

### **3. Executar Listagem**
```bash
# Criar proposta de listagem
npm run list-token my-token-config.json

# Verificar status da proposta
npm run check-proposal <proposal_id>

# Executar proposta aprovada
npm run execute-proposal <proposal_id>

# Adicionar liquidez inicial
npm run add-liquidity <token_address> <token_amount> <lunes_amount>
```

---

## 🧪 **TESTES E VALIDAÇÃO**

```bash
# Testes unitários
npm run test:unit

# Testes de integração
npm run test:integration

# Testes end-to-end
npm run test:e2e

# Testes de segurança
npm run test:security

# Stress tests
npm run test:stress

# Lint e formatação
npm run lint:fix
```

---

## 📊 **MONITORAMENTO**

### **Via Polkadot.js Apps**
1. Acesse: https://polkadot.js.org/apps
2. Configure endpoint: `wss://ws-test.lunes.io` (testnet) ou `wss://ws.lunes.io` (mainnet)
3. Contracts → Upload & Deploy

### **Block Explorer**
- Testnet: `https://explorer-test.lunes.io`
- Mainnet: `https://explorer.lunes.io`

---

## 🔧 **TROUBLESHOOTING**

### **Erros Comuns**

#### **"Out of Gas"**
```bash
# Aumentar gas limit no script
# Editar: scripts/deploy-lunes.ts
# GAS_LIMITS.contract_name = new BN('2000000000000')
```

#### **"Insufficient Balance"**
```bash
# Verificar balance na rede
# Necessário ~100,000 LUNES para deploy completo
# Use faucet: https://faucet-test.lunes.io
```

#### **"Contract Not Found"**
```bash
# Verificar se compilação foi executada
npm run compile:all

# Verificar se artefatos foram gerados
find . -name "*.contract" -type f
```

#### **"Network Connection Failed"**
```bash
# Testar endpoints alternativos
# Testnet: wss://ws-test.lunes.io
# Mainnet: wss://ws-lunes-main-01.lunes.io
#         wss://ws-lunes-main-02.lunes.io
```

---

## 📚 **DOCUMENTAÇÃO COMPLETA**

- **Deploy Completo:** [README_DEPLOY_LUNES.md](./README_DEPLOY_LUNES.md)
- **Auditoria de Segurança:** [AUDITORIA_SEGURANCA_E_GAS_COMPLETA.md](./AUDITORIA_SEGURANCA_E_GAS_COMPLETA.md)
- **Relatório Final:** [RELATORIO_FINAL_SEGURANCA_E_GAS.md](./RELATORIO_FINAL_SEGURANCA_E_GAS.md)
- **Features:** [LUNEX_DEX_FEATURES.md](./LUNEX_DEX_FEATURES.md)

---

## 🆘 **SUPORTE RÁPIDO**

### **Comandos de Debug**
```bash
# Verificar status da rede
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
     wss://ws.lunes.io

# Verificar balance
# Via Polkadot.js Developer Console

# Logs detalhados
export DEBUG=true
npm run deploy:testnet "//Alice"
```

### **Reset Completo**
```bash
# Limpar tudo e começar do zero
cargo clean
rm -rf target/
rm -rf node_modules/
npm install
npm run setup:dev
npm run compile:all
```

---

## 🎯 **FLUXO COMPLETO DE PRODUÇÃO**

```bash
# 1. Setup
git clone <repo>
cd Lunex
npm install
npm run setup:dev

# 2. Build
npm run compile:all

# 3. Test
npm run test:unit
npm run test:security

# 4. Deploy Testnet
npm run deploy:testnet "your_seed_here"

# 5. Listar Token
cp examples/token-listing-config.json my-config.json
# Editar my-config.json
npm run list-token my-config.json

# 6. Aguardar Votação (7 dias)
npm run check-proposal <proposal_id>

# 7. Executar Proposta
npm run execute-proposal <proposal_id>

# 8. Adicionar Liquidez
npm run add-liquidity <token> <amount> <lunes>

# 9. Deploy Mainnet (quando pronto)
npm run deploy:mainnet "your_production_seed"

# 10. Anunciar Launch! 🚀
```

---

**🌟 PRONTO PARA REVOLUCIONAR O DEFI NO LUNES! 🚀**
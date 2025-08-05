# 🔍 **GUIA DE VERIFICAÇÃO DE DEPLOYMENT - LUNEX DEX**

Este guia explica como usar o script de verificação de deployment para garantir que todos os contratos da Lunex DEX foram implantados corretamente na rede Lunes.

## 📋 **Pré-requisitos**

1. **Node.js** >= 16.0.0
2. **Yarn** ou **npm**
3. Contratos já implantados na rede Lunes
4. Arquivo de configuração de deployment configurado

## 🚀 **Uso Básico**

### Verificar Deployment na Testnet

```bash
npm run verify:testnet
```

### Verificar Deployment na Mainnet

```bash
npm run verify:mainnet
```

### Verificar Rede Específica

```bash
npm run verify:deployment [network]
```

Onde `network` pode ser:
- `testnet` - Rede de teste Lunes
- `mainnet` - Rede principal Lunes

## ⚙️ **Configuração**

### 1. Arquivo de Configuração de Deployment

Crie um arquivo de configuração em `deployment/{network}.json` baseado no exemplo:

```json
{
  "network": "testnet",
  "deployedAt": "2024-01-15T10:30:00Z",
  "deployer": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "contracts": {
    "factory": {
      "name": "factory",
      "address": "5D5PhZQNJzcJXVBxwJxZcsutjKPqUPydrvpu6HeiBfMae2Qu",
      "abi": null
    },
    "router": {
      "name": "router", 
      "address": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
      "abi": null
    },
    "staking": {
      "name": "staking",
      "address": "5GKoR4ckjqvbpPPgNDQD2AjAGGLdS1VZvh8JDJLGaKVyX7qK",
      "abi": null
    },
    "rewards": {
      "name": "rewards",
      "address": "5Dp6EHYLr8JFrSECdPKwE7cjr9Mw8zUTZZzVhZjXZjPj9qXX",
      "abi": null
    },
    "psp22": {
      "name": "psp22",
      "address": "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy",
      "abi": null
    },
    "wnative": {
      "name": "wnative",
      "address": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
      "abi": null
    }
  },
  "expectedConfigurations": {
    "factory": {
      "feeToSetter": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    },
    "router": {
      "factory": "5D5PhZQNJzcJXVBxwJxZcsutjKPqUPydrvpu6HeiBfMae2Qu",
      "wnative": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y"
    },
    "staking": {
      "owner": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
      "treasury": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    },
    "rewards": {
      "admin": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
      "router": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
    }
  }
}
```

### 2. ABIs dos Contratos

O script automaticamente carrega as ABIs dos contratos do diretório `target/ink/`:

```
target/ink/
├── factory/factory.json
├── router/router.json  
├── staking/staking.json
├── rewards/rewards.json
├── psp22/psp22.json
└── wnative/wnative.json
```

Certifique-se de que os contratos foram compilados com `npm run compile:all`.

## 🔍 **O Que o Script Verifica**

### 1. 📋 Existência dos Contratos
- ✅ Verifica se cada endereço contém código de contrato
- ✅ Exibe o code hash de cada contrato
- ❌ Identifica endereços sem código

### 2. ⚙️ Configurações dos Contratos
- **Factory**: Fee to setter configurado corretamente
- **Router**: Factory e WNative configurados corretamente  
- **Staking**: Owner e treasury configurados corretamente
- **Trading Rewards**: Admin e router autorizado configurados corretamente

### 3. 🔗 Integrações Entre Contratos
- ✅ Staking ↔ Trading Rewards connection
- ✅ Trading Rewards ↔ Router authorization
- ✅ Router ↔ Factory integration
- ✅ Router ↔ WNative integration

### 4. 🔐 Permissões e Segurança
- ✅ Status de pausa dos contratos
- ✅ Owners e admins corretos
- ✅ Endereços autorizados

### 5. 🧪 Funcionalidades Básicas
- ✅ Informações básicas dos contratos
- ✅ Estatísticas de uso (pares criados, stakes ativos, etc.)
- ✅ Queries básicas funcionando

## 📊 **Interpretando os Resultados**

### ✅ Sucesso
```
🔍 === VERIFICAÇÃO DE DEPLOYMENT ===

✅ TODOS OS CONTRATOS ESTÃO FUNCIONANDO CORRETAMENTE!
```

### ❌ Problemas Encontrados
```
📋 1. Verificando existência dos contratos...

🔍 Verificando FACTORY...
   📍 Endereço: 5D5PhZQNJzcJXVBxwJxZcsutjKPqUPydrvpu6HeiBfMae2Qu
   ❌ Nenhum código encontrado no endereço!

❌ ALGUNS PROBLEMAS FORAM ENCONTRADOS. VERIFIQUE OS LOGS ACIMA.
```

## 🛠️ **Troubleshooting**

### Erro: "Arquivo de configuração não encontrado"
**Solução**: Crie o arquivo `deployment/{network}.json` baseado no exemplo.

### Erro: "ABI não encontrada"
**Solução**: Execute `npm run compile:all` para compilar todos os contratos.

### Erro: "Conexão com a rede falhou"
**Solução**: 
- Verifique sua conexão com a internet
- Verifique se o endpoint da rede Lunes está funcionando
- Tente novamente em alguns minutos

### Erro: "Query failed"
**Solução**:
- Verifique se o endereço do contrato está correto
- Verifique se o contrato foi implantado corretamente
- Verifique se a ABI está atualizada

### Configuração Incorreta
**Solução**:
- Verifique os endereços no arquivo de configuração
- Execute o script de deployment novamente se necessário
- Configure as integrações entre contratos manualmente

## 🔄 **Automação**

### CI/CD Integration

Adicione ao seu workflow de CI/CD:

```yaml
- name: Verify Deployment
  run: |
    npm install
    npm run compile:all
    npm run verify:testnet
```

### Monitoring

Execute periodicamente para monitorar a saúde dos contratos:

```bash
# Cron job example - rodar a cada hora
0 * * * * cd /path/to/lunex && npm run verify:mainnet >> /var/log/lunex-verification.log 2>&1
```

## 📞 **Suporte**

Se encontrar problemas:

1. Verifique os logs detalhados
2. Confirme que todos os contratos foram implantados
3. Verifique a configuração de rede
4. Execute novamente após alguns minutos

Para suporte adicional, consulte a documentação principal do projeto ou abra uma issue no repositório.

---

**💡 Dica**: Execute a verificação sempre após fazer deploy ou atualizações nos contratos para garantir que tudo está funcionando corretamente!
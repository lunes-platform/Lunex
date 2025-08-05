# Guia de Lançamento e Liquidez Gênese da Lunex DEX

**Versão 1.0.0**  
**Ink! Version:** 5.1.1  
**Rede Alvo:** Lunes Network (`wss://ws.lunes.io`)  
**Última Atualização:** Agosto 2024

Este guia fornece o passo a passo técnico para realizar o deploy completo dos contratos da Lunex DEX e para popular a DEX com a liquidez inicial, focando na moeda nativa LUNES (via WLUNES) e em um token `PSP22` parceiro (ex: UP Token).

**📋 Pré-requisitos Técnicos:**
- Rust toolchain: `nightly` (atualizado)
- `cargo-contract`: Versão compatível com ink! 5.1.1
- `typechain-compiler`: Para geração de artifacts TypeScript
- Rede Lunes configurada e acessível

Este processo é tipicamente executado pelo time de desenvolvimento principal em um ambiente de testnet antes de ser replicado na mainnet.

---

## 🚀 Fase 1: Deploy dos Contratos Core

Nesta fase, colocaremos toda a infraestrutura de contratos no ar. Execute cada passo na ordem e **anote o `AccountId` (endereço do contrato) de cada deploy**.

### **Passo 1.1: Deploy do Wnative (Adaptador LUNES)**
- **Contrato:** `wnative_contract`
- **Por que primeiro?** O `Router` depende dele.
- **Comando (Exemplo):**
  ```bash
  # Navegue até a pasta do contrato e compile
  cargo contract build --manifest-path uniswap-v2/contracts/wnative/Cargo.toml --release
  # Faça o deploy e anote o endereço
  # (Use sua ferramenta de deploy preferida, como o Contracts UI)
  ```
- **✅ Resultado:** `WNATIVE_ADDRESS`

### **Passo 1.2: Deploy do Factory**
- **Contrato:** `factory_contract`
- **Parâmetros do Construtor:**
  - `fee_to_setter`: O endereço da sua conta de admin/tesouraria.
  - `pair_code_hash`: O hash do Wasm do contrato `pair_contract`. Você obtém este hash ao fazer o upload do `pair_contract.wasm` no Contracts UI ou via `cargo-contract`.
- **✅ Resultado:** `FACTORY_ADDRESS`

### **Passo 1.3: Deploy do Router**
- **Contrato:** `router_contract`
- **Parâmetros do Construtor:**
  - `factory`: O `FACTORY_ADDRESS` do passo 1.2.
  - `wnative`: O `WNATIVE_ADDRESS` do passo 1.1.
- **✅ Resultado:** `ROUTER_ADDRESS`

### **Passo 1.4: Deploy do Staking**
- **Contrato:** `staking_contract`
- **Parâmetros do Construtor:**
  - `treasury_address`: O endereço da sua conta de admin/tesouraria.
- **✅ Resultado:** `STAKING_ADDRESS`

### **Passo 1.5 (Opcional): Deploy do Token Parceiro**
- **Contrato:** `seu_token_psp22` (ex: UP Token)
- **Ação:** Faça o deploy do contrato do token `PSP22`.
- **✅ Resultado:** `PARTNER_TOKEN_ADDRESS`

**Ao final desta fase, você deve ter todos os endereços dos contratos core anotados.**

---

## 🎯 Fase 2: Configuração e Listagem Gênese

Agora que os contratos estão no ar, vamos conectá-los e listar os primeiros tokens. **Todas as ações a seguir devem ser executadas pela conta de admin/owner.**

### **Passo 2.1: Listar WLUNES na DEX**
- **Ação:** Chamar a função `admin_list_token` no contrato `Staking`.
- **Destino da Chamada:** `STAKING_ADDRESS`
- **Parâmetros:**
  - `token_address`: O `WNATIVE_ADDRESS` (do passo 1.1).
  - `reason`: `"Lunes Nativo (WLUNES)"`.
- **Resultado:** O `WLUNES` agora é um ativo reconhecido e permitido pela DEX.

### **Passo 2.2: Listar Token Parceiro na DEX**
- **Ação:** Chamar a função `admin_list_token` no contrato `Staking`.
- **Destino da Chamada:** `STAKING_ADDRESS`
- **Parâmetros:**
  - `token_address`: O `PARTNER_TOKEN_ADDRESS` (do passo 1.5).
  - `reason`: `"Token Parceiro Gênese: NOME_DO_TOKEN"`.
- **Resultado:** O token parceiro agora é um ativo reconhecido e permitido.

---

## 💧 Fase 3: Injeção da Liquidez Inicial (Seeding)

Com os tokens listados, o mercado precisa de liquidez para que as trocas possam ocorrer.

### **Passo 3.1: "Embrulhar" LUNES em WLUNES**
- **Ação:** Para obter `WLUNES`, você precisa enviar LUNES nativos para o contrato `Wnative`.
- **Contrato:** `wnative_contract`
- **Função:** `deposit`
- **Valor Enviado:** A quantidade de LUNES que você deseja para a liquidez inicial (ex: 1,000,000 LUNES).
- **Resultado:** Sua conta de admin agora possui `1,000,000` tokens `WLUNES`.

### **Passo 3.2: Aprovar o Router a Gastar Seus Tokens**
- Antes que o `Router` possa pegar seus tokens para criar o par, você precisa dar permissão a ele.
- **Ação 1: Aprovar WLUNES**
  - **Contrato:** `wnative_contract`
  - **Função:** `psp22::approve`
  - **Parâmetros:**
    - `spender`: O `ROUTER_ADDRESS`.
    - `value`: A quantidade de `WLUNES` que você vai usar na liquidez (ex: `1,000,000`).
- **Ação 2: Aprovar Token Parceiro**
  - **Contrato:** `seu_token_psp22`
  - **Função:** `psp22::approve`
  - **Parâmetros:**
    - `spender`: O `ROUTER_ADDRESS`.
    - `value`: A quantidade do token parceiro para a liquidez (ex: `50,000,000`).

### **Passo 3.3: Criar o Primeiro Par de Liquidez**
- **Ação:** Esta é a chamada que efetivamente cria o mercado `WLUNES / TOKEN_PARCEIRO`.
- **Contrato:** `router_contract`
- **Função:** `add_liquidity`
- **Parâmetros:**
  - `token_a`: `WNATIVE_ADDRESS`.
  - `token_b`: `PARTNER_TOKEN_ADDRESS`.
  - `amount_a_desired`: Quantidade de `WLUNES` (ex: `1,000,000`).
  - `amount_b_desired`: Quantidade do token parceiro (ex: `50,000,000`).
  - `amount_a_min`: Para a primeira liquidez, pode ser o mesmo que o `desired` ou um pouco menos para segurança.
  - `amount_b_min`: Mesmo que acima.
  - `to`: O endereço que receberá os LP Tokens (geralmente a conta admin/tesouraria).
  - `deadline`: Um timestamp no futuro (ex: `agora + 10 minutos`).

---

## ✅ Verificação Final

Se todos os passos foram executados com sucesso:
- O par `WLUNES / TOKEN_PARCEIRO` foi criado e tem um endereço.
- O par possui uma reserva inicial de liquidez.
- **A DEX está oficialmente aberta para negociação neste par!**

Qualquer usuário agora pode fazer swaps entre `WLUNES` e o token parceiro. Repita as Fases 2 e 3 para cada novo token que queira adicionar no lançamento.

#!/usr/bin/env ts-node

/**
 * 🚀 LUNEX DEX - SCRIPT DE DEPLOY AUTOMATIZADO PARA LUNES BLOCKCHAIN
 * 
 * Este script automatiza o deploy completo da Lunex DEX no blockchain Lunes
 * Inclui deploy de todos os contratos e configuração das integrações
 */

import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { ContractPromise, CodePromise } from '@polkadot/api-contract';
import { BN } from '@polkadot/util';
import * as fs from 'fs';
import * as path from 'path';

// 🌐 CONFIGURAÇÃO DE REDE
const NETWORKS = {
  testnet: {
    endpoint: 'wss://ws-test.lunes.io',
    name: 'Lunes Testnet'
  },
  mainnet: {
    endpoint: 'wss://ws.lunes.io',
    name: 'Lunes Mainnet'
  }
};

// 💰 CONFIGURAÇÃO DE GAS E DEPOSITS
const GAS_LIMITS = {
  wnative: new BN('1000000000000'),      // 10,000 LUNES
  factory: new BN('1200000000000'),      // 12,000 LUNES  
  staking: new BN('1100000000000'),      // 11,000 LUNES
  rewards: new BN('900000000000'),       // 9,000 LUNES
  router: new BN('1300000000000'),       // 13,000 LUNES
};

const STORAGE_DEPOSITS = {
  wnative: new BN('1000000000000'),      // 10,000 LUNES
  factory: new BN('1500000000000'),      // 15,000 LUNES
  staking: new BN('1200000000000'),      // 12,000 LUNES  
  rewards: new BN('1000000000000'),      // 10,000 LUNES
  router: new BN('1800000000000'),       // 18,000 LUNES
};

// 📁 CAMINHOS DOS CONTRATOS
const CONTRACT_PATHS = {
  wnative: './uniswap-v2/contracts/wnative/target/ink/wnative_contract.contract',
  factory: './uniswap-v2/contracts/factory/target/ink/factory_contract.contract',
  staking: './uniswap-v2/contracts/staking/target/ink/staking_contract.contract',
  rewards: './uniswap-v2/contracts/rewards/target/ink/trading_rewards_contract.contract',
  router: './uniswap-v2/contracts/router/target/ink/router_contract.contract',
  pair: './uniswap-v2/contracts/pair/target/ink/pair_contract.contract'
};

// 🔑 TIPOS
interface DeployedContract {
  address: string;
  contract: ContractPromise;
  txHash: string;
  deployBlock: number;
}

interface DeployConfig {
  network: 'testnet' | 'mainnet';
  adminSeed: string;
  treasuryAddress?: string;
  skipVerification?: boolean;
  dryRun?: boolean;
  initialTokens?: Array<{
    address: string;
    reason: string;
  }>;
}

class LunexDeployer {
  private api!: ApiPromise;
  private keyring: Keyring;
  private adminAccount: any;
  private deployedContracts: Map<string, DeployedContract> = new Map();

  constructor(private config: DeployConfig) {
    this.keyring = new Keyring({ type: 'sr25519' });
    this.adminAccount = this.keyring.addFromUri(config.adminSeed);
  }

  /**
   * 🚀 Inicializar conexão com a rede Lunes
   */
  async initialize(): Promise<void> {
    console.log(`🌐 Conectando à ${NETWORKS[this.config.network].name}...`);
    
    const provider = new WsProvider(NETWORKS[this.config.network].endpoint);
    this.api = await ApiPromise.create({ provider });
    
    await this.api.isReady;
    console.log(`✅ Conectado à ${NETWORKS[this.config.network].name}`);
    
    // Verificar balance do admin
    const balance = await this.api.query.system.account(this.adminAccount.address);
    const freeBalance = balance.data.free.toBN();
    const requiredBalance = new BN('100000000000000'); // 1,000,000 LUNES
    
    console.log(`💰 Balance Admin: ${freeBalance.div(new BN('100000000')).toString()} LUNES`);
    
    if (freeBalance.lt(requiredBalance)) {
      throw new Error(`❌ Balance insuficiente! Necessário: ${requiredBalance.div(new BN('100000000')).toString()} LUNES`);
    }
  }

  /**
   * 📦 Carregar metadata do contrato
   */
  private loadContractMetadata(contractPath: string): any {
    const fullPath = path.resolve(contractPath);
    if (!fs.existsSync(fullPath)) {
      throw new Error(`❌ Contrato não encontrado: ${fullPath}`);
    }
    
    const contractData = JSON.parse(fs.readFileSync(fullPath, 'utf8'));
    return contractData;
  }

  /**
   * 🏗️ Deploy de um contrato
   */
  private async deployContract(
    name: string,
    contractPath: string,
    constructorName: string,
    args: any[] = [],
    gasLimit?: BN,
    storageDepositLimit?: BN
  ): Promise<DeployedContract> {
    console.log(`📦 Fazendo deploy do ${name}...`);

    try {
      const contractData = this.loadContractMetadata(contractPath);
      const code = new CodePromise(this.api, contractData, contractData.source.wasm);

      const gasLimitToUse = gasLimit || GAS_LIMITS[name as keyof typeof GAS_LIMITS] || new BN('1000000000000');
      const storageDepositToUse = storageDepositLimit || STORAGE_DEPOSITS[name as keyof typeof STORAGE_DEPOSITS] || new BN('1000000000000');

      if (this.config.dryRun) {
        console.log(`🧪 DRY RUN - ${name}: Gas: ${gasLimitToUse.toString()}, Storage: ${storageDepositToUse.toString()}`);
        return {
          address: 'DRY_RUN_ADDRESS',
          contract: {} as ContractPromise,
          txHash: 'DRY_RUN_HASH',
          deployBlock: 0
        };
      }

      // Estimar gas primeiro
      const { gasRequired } = await code.tx[constructorName]({
        gasLimit: gasLimitToUse,
        storageDepositLimit: storageDepositToUse,
      }, ...args).dryRun(this.adminAccount);

      console.log(`⛽ Gas estimado: ${gasRequired.toString()}`);

      // Deploy real
      const tx = code.tx[constructorName]({
        gasLimit: gasLimitToUse,
        storageDepositLimit: storageDepositToUse,
      }, ...args);

      return new Promise((resolve, reject) => {
        let contractAddress = '';
        let deployBlock = 0;
        let txHash = '';

        tx.signAndSend(this.adminAccount, ({ events = [], status, txHash: hash }) => {
          txHash = hash.toString();
          
          if (status.isInBlock) {
            console.log(`📋 ${name} incluído no bloco: ${status.asInBlock}`);
            deployBlock = parseInt(status.asInBlock.toString());
            
            events.forEach(({ event: { data, method, section } }) => {
              if (section === 'contracts' && method === 'Instantiated') {
                contractAddress = data[1].toString();
                console.log(`✅ ${name} deployado em: ${contractAddress}`);
              }
            });
          } else if (status.isFinalized) {
            if (contractAddress) {
              const contract = new ContractPromise(this.api, contractData, contractAddress);
              
              const deployedContract: DeployedContract = {
                address: contractAddress,
                contract,
                txHash,
                deployBlock
              };
              
              this.deployedContracts.set(name, deployedContract);
              console.log(`🎉 ${name} finalizado com sucesso!`);
              resolve(deployedContract);
            } else {
              reject(new Error(`❌ Falha no deploy do ${name}: endereço não encontrado`));
            }
          } else if (status.isError) {
            reject(new Error(`❌ Falha no deploy do ${name}: ${status.toString()}`));
          }
        }).catch(reject);
      });

    } catch (error) {
      console.error(`❌ Erro no deploy do ${name}:`, error);
      throw error;
    }
  }

  /**
   * 📋 Configurar tokens iniciais via admin listing
   */
  private async configureInitialTokens(): Promise<void> {
    if (!this.config.initialTokens || this.config.initialTokens.length === 0) {
      console.log(`⏭️ Nenhum token inicial especificado, pulando...`);
      return;
    }

    console.log(`📋 Configurando ${this.config.initialTokens.length} tokens iniciais...`);

    try {
      const staking = this.deployedContracts.get('staking')!;

      // Usar batch listing se múltiplos tokens
      if (this.config.initialTokens.length > 1) {
        const tokensArray = this.config.initialTokens.map(token => [token.address, token.reason]);
        
        const batchListTx = await staking.contract.tx.adminBatchListTokens(
          { gasLimit: new BN('500000000000') }, // 5,000 LUNES
          tokensArray
        );
        
        await this.signAndWaitForFinalization(batchListTx, 'adminBatchListTokens');
        console.log(`✅ ${this.config.initialTokens.length} tokens listados via batch!`);
        
      } else {
        // Lista único token
        const token = this.config.initialTokens[0];
        const listTx = await staking.contract.tx.adminListToken(
          { gasLimit: new BN('200000000000') }, // 2,000 LUNES
          token.address,
          token.reason
        );
        
        await this.signAndWaitForFinalization(listTx, 'adminListToken');
        console.log(`✅ Token ${token.address} listado com sucesso!`);
      }

      // Verificar se os tokens foram realmente listados
      for (const token of this.config.initialTokens) {
        const isListed = await staking.contract.query.isProjectApproved(
          this.adminAccount.address,
          {},
          token.address
        );
        
        if (isListed.output?.toJSON()) {
          console.log(`✅ Token ${token.address} confirmado como listado`);
        } else {
          console.error(`❌ Erro: Token ${token.address} não foi listado corretamente`);
        }
      }

    } catch (error) {
      console.error(`❌ Erro ao configurar tokens iniciais:`, error);
      throw error;
    }
  }

  /**
   * 🔧 Configurar integrações entre contratos
   */
  private async configureIntegrations(): Promise<void> {
    console.log(`🔗 Configurando integrações entre contratos...`);

    try {
      const factory = this.deployedContracts.get('factory')!;
      const router = this.deployedContracts.get('router')!;
      const staking = this.deployedContracts.get('staking')!;
      const rewards = this.deployedContracts.get('rewards')!;

      // 1. Configurar router autorizado no trading rewards
      console.log(`⚙️ Configurando router autorizado...`);
      const setRouterTx = await rewards.contract.tx.setAuthorizedRouter(
        { gasLimit: new BN('100000000000') },
        router.address
      );
      await this.signAndWaitForFinalization(setRouterTx, 'setAuthorizedRouter');

      // 2. Conectar staking ao trading rewards
      console.log(`⚙️ Conectando staking ao trading rewards...`);
      const setStakingTx = await staking.contract.tx.setTradingRewardsContract(
        { gasLimit: new BN('100000000000') },
        rewards.address
      );
      await this.signAndWaitForFinalization(setStakingTx, 'setTradingRewardsContract');

      // 3. Conectar trading rewards ao staking
      console.log(`⚙️ Conectando trading rewards ao staking...`);
      const setStakingInRewardsTx = await rewards.contract.tx.setStakingContract(
        { gasLimit: new BN('100000000000') },
        staking.address
      );
      await this.signAndWaitForFinalization(setStakingInRewardsTx, 'setStakingContract');

      console.log(`✅ Integrações configuradas com sucesso!`);

    } catch (error) {
      console.error(`❌ Erro ao configurar integrações:`, error);
      throw error;
    }
  }

  /**
   * ✍️ Assinar e aguardar finalização da transação
   */
  private async signAndWaitForFinalization(tx: any, operation: string): Promise<void> {
    return new Promise((resolve, reject) => {
      tx.signAndSend(this.adminAccount, ({ status, dispatchError }: any) => {
        if (status.isInBlock) {
          console.log(`📋 ${operation} incluído no bloco`);
        } else if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(`❌ ${operation} falhou: ${dispatchError.toString()}`));
          } else {
            console.log(`✅ ${operation} finalizado com sucesso`);
            resolve();
          }
        }
      }).catch(reject);
    });
  }

  /**
   * 💾 Salvar informações de deploy
   */
  private async saveDeploymentInfo(): Promise<void> {
    const deploymentInfo = {
      network: this.config.network,
      timestamp: new Date().toISOString(),
      deployedBy: this.adminAccount.address,
      contracts: {} as any
    };

    for (const [name, contract] of this.deployedContracts) {
      deploymentInfo.contracts[name] = {
        address: contract.address,
        txHash: contract.txHash,
        deployBlock: contract.deployBlock
      };
    }

    const filename = `deployment-${this.config.network}-${Date.now()}.json`;
    fs.writeFileSync(filename, JSON.stringify(deploymentInfo, null, 2));
    console.log(`💾 Informações de deploy salvas em: ${filename}`);
  }

  /**
   * 🧪 Executar testes básicos pós-deploy
   */
  private async runBasicTests(): Promise<void> {
    if (this.config.skipVerification) {
      console.log(`⏭️ Pulando testes (skipVerification=true)`);
      return;
    }

    console.log(`🧪 Executando testes básicos...`);

    try {
      // Testar WNative
      const wnative = this.deployedContracts.get('wnative')!;
      const nameResult = await wnative.contract.query.name(this.adminAccount.address, {});
      console.log(`✅ WNative name: ${nameResult.output?.toString()}`);

      // Testar Factory
      const factory = this.deployedContracts.get('factory')!;
      const feeToSetter = await factory.contract.query.feeToSetter(this.adminAccount.address, {});
      console.log(`✅ Factory fee_to_setter: ${feeToSetter.output?.toString()}`);

      // Testar Staking
      const staking = this.deployedContracts.get('staking')!;
      const stakingStats = await staking.contract.query.getStats(this.adminAccount.address, {});
      console.log(`✅ Staking stats: ${stakingStats.output?.toString()}`);

      console.log(`✅ Todos os testes básicos passaram!`);

    } catch (error) {
      console.error(`❌ Erro nos testes básicos:`, error);
      throw error;
    }
  }

  /**
   * 🚀 Executar deploy completo
   */
  async deployAll(): Promise<void> {
    try {
      console.log(`🚀 Iniciando deploy completo da Lunex DEX...`);
      console.log(`📡 Rede: ${NETWORKS[this.config.network].name}`);
      console.log(`👤 Admin: ${this.adminAccount.address}`);
      console.log(`🧪 Dry Run: ${this.config.dryRun ? 'SIM' : 'NÃO'}`);
      
      await this.initialize();

      // Deploy order é crítico!
      console.log(`\n📦 FASE 1: Deploy dos contratos base...`);
      
      // 1. WNative (wrapper para LUNES)
      await this.deployContract('wnative', CONTRACT_PATHS.wnative, 'new');
      
      // 2. Factory (para criar pares)
      await this.deployContract('factory', CONTRACT_PATHS.factory, 'new', [this.adminAccount.address]);
      
      // 3. Staking (governança e rewards)
      const treasuryAddress = this.config.treasuryAddress || this.adminAccount.address;
      console.log(`🏦 Endereço de Tesouraria: ${treasuryAddress}`);
      await this.deployContract('staking', CONTRACT_PATHS.staking, 'new', [treasuryAddress]);

      console.log(`\n📦 FASE 2: Deploy dos contratos de integração...`);
      
      // 4. Router (precisa do factory e wnative)
      const factoryAddress = this.deployedContracts.get('factory')!.address;
      const wnativeAddress = this.deployedContracts.get('wnative')!.address;
      await this.deployContract('router', CONTRACT_PATHS.router, 'new', [factoryAddress, wnativeAddress]);
      
      // 5. Trading Rewards (precisa do router)
      const routerAddress = this.deployedContracts.get('router')!.address;
      await this.deployContract('rewards', CONTRACT_PATHS.rewards, 'new', [this.adminAccount.address, routerAddress]);

      console.log(`\n🔗 FASE 3: Configuração das integrações...`);
      if (!this.config.dryRun) {
        await this.configureIntegrations();
      }

      console.log(`\n📋 FASE 3.1: Configuração de tokens iniciais...`);
      if (!this.config.dryRun) {
        await this.configureInitialTokens();
      }

      console.log(`\n🧪 FASE 4: Testes básicos...`);
      if (!this.config.dryRun) {
        await this.runBasicTests();
      }

      console.log(`\n💾 FASE 5: Salvando informações...`);
      if (!this.config.dryRun) {
        await this.saveDeploymentInfo();
      }

      console.log(`\n🎉 DEPLOY COMPLETO DA LUNEX DEX FINALIZADO COM SUCESSO! 🎉`);
      console.log(`\n📋 RESUMO DOS CONTRATOS DEPLOYADOS:`);
      
      for (const [name, contract] of this.deployedContracts) {
        console.log(`   ${name.toUpperCase()}: ${contract.address}`);
      }

      if (this.config.network === 'testnet') {
        console.log(`\n🧪 Testnet deploy completo! Próximos passos:`);
        console.log(`   1. Testar funcionalidades via Polkadot.js`);
        console.log(`   2. Executar stress tests`);
        console.log(`   3. Fazer audit final`);
        console.log(`   4. Deploy na mainnet`);
      } else {
        console.log(`\n🏭 Mainnet deploy completo! Próximos passos:`);
        console.log(`   1. Configurar monitoring`);
        console.log(`   2. Anunciar para a comunidade`);
        console.log(`   3. Começar programa de incentivos`);
        console.log(`   4. Listagem dos primeiros tokens`);
      }

    } catch (error) {
      console.error(`💥 Erro durante o deploy:`, error);
      throw error;
    } finally {
      await this.api?.disconnect();
    }
  }
}

// 🎯 FUNÇÃO PRINCIPAL
async function main() {
  const args = process.argv.slice(2);
  
  if (args.length < 2) {
    console.log(`
🚀 LUNEX DEX DEPLOY SCRIPT

Uso: npm run deploy:lunes <network> <admin_seed> [options]

Parâmetros:
  network     testnet | mainnet
  admin_seed  Seed phrase da conta admin

Opções:
  --dry-run           Simula deploy sem executar
  --skip-verification Pula testes pós-deploy

Exemplos:
  npm run deploy:lunes testnet "//Alice" --dry-run
  npm run deploy:lunes mainnet "bottom drive obey lake curtain smoke basket hold race lonely fit walk" 
    `);
    process.exit(1);
  }

  const network = args[0] as 'testnet' | 'mainnet';
  const adminSeed = args[1];
  const dryRun = args.includes('--dry-run');
  const skipVerification = args.includes('--skip-verification');

  if (!['testnet', 'mainnet'].includes(network)) {
    console.error(`❌ Rede inválida: ${network}. Use 'testnet' ou 'mainnet'`);
    process.exit(1);
  }

  if (network === 'mainnet' && !dryRun) {
    console.log(`⚠️  ATENÇÃO: Deploy na MAINNET com valores reais!`);
    console.log(`💰 Certifique-se de ter pelo menos 100,000 LUNES para fees e deposits`);
    console.log(`🔐 Verifique se a seed está segura e é a conta correta`);
    
    // Aguardar confirmação em mainnet
    const readline = require('readline').createInterface({
      input: process.stdin,
      output: process.stdout
    });
    
    const answer = await new Promise(resolve => {
      readline.question('Digite "CONFIRMO" para continuar: ', resolve);
    });
    
    readline.close();
    
    if (answer !== 'CONFIRMO') {
      console.log(`❌ Deploy cancelado pelo usuário`);
      process.exit(1);
    }
  }

  const config: DeployConfig = {
    network,
    adminSeed,
    dryRun,
    skipVerification
  };

  const deployer = new LunexDeployer(config);
  await deployer.deployAll();
}

// 🚀 EXECUTAR
if (require.main === module) {
  main().catch(error => {
    console.error('💥 Erro fatal:', error);
    process.exit(1);
  });
}

export { LunexDeployer, DeployConfig };
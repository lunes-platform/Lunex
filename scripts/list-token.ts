#!/usr/bin/env ts-node

/**
 * 💎 LUNEX DEX - SCRIPT DE LISTAGEM DE TOKENS
 * 
 * Este script automatiza o processo de listagem de novos tokens na Lunex DEX
 * Inclui proposta de governança, votação e execução
 */

import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { ContractPromise } from '@polkadot/api-contract';
import { BN } from '@polkadot/util';
import * as fs from 'fs';

// 🌐 CONFIGURAÇÃO DE REDE
const NETWORKS = {
  testnet: 'wss://ws-test.lunes.io',
  mainnet: 'wss://ws.lunes.io'
};

// 📋 CONFIGURAÇÃO DE LISTAGEM
const LISTING_CONFIG = {
  MIN_PROPOSAL_POWER: new BN('1000000000000'),    // 10,000 LUNES (8 decimais)
  PROPOSAL_FEE: new BN('100000000000'),           // 1,000 LUNES 
  IMPLEMENTATION_FEE: new BN('500000000000'),     // 5,000 LUNES
  MIN_LIQUIDITY: new BN('1000000000000'),         // 10,000 LUNES
  VOTING_PERIOD: 7 * 24 * 60 * 60,               // 7 dias em segundos
  MIN_QUORUM: new BN('100000000000000'),          // 1,000,000 LUNES
};

// 🎯 TIPOS
interface TokenInfo {
  address: string;
  name: string;
  symbol: string;
  decimals: number;
  description: string;
  website?: string;
  whitepaper?: string;
  audit?: string;
}

interface ListingConfig {
  network: 'testnet' | 'mainnet';
  proposerSeed: string;
  stakingContract: string;
  factoryContract: string;
  routerContract: string;
}

class TokenLister {
  private api!: ApiPromise;
  private keyring: Keyring;
  private proposerAccount: any;
  private stakingContract!: ContractPromise;
  private factoryContract!: ContractPromise;
  private routerContract!: ContractPromise;

  constructor(private config: ListingConfig) {
    this.keyring = new Keyring({ type: 'sr25519' });
    this.proposerAccount = this.keyring.addFromUri(config.proposerSeed);
  }

  /**
   * 🚀 Inicializar conexões
   */
  async initialize(): Promise<void> {
    console.log(`🌐 Conectando à rede...`);
    
    const provider = new WsProvider(NETWORKS[this.config.network]);
    this.api = await ApiPromise.create({ provider });
    await this.api.isReady;
    
    // Carregar metadatas dos contratos (assumindo que existem)
    const stakingMetadata = this.loadContractMetadata('./deployments/staking-metadata.json');
    const factoryMetadata = this.loadContractMetadata('./deployments/factory-metadata.json');
    const routerMetadata = this.loadContractMetadata('./deployments/router-metadata.json');
    
    this.stakingContract = new ContractPromise(this.api, stakingMetadata, this.config.stakingContract);
    this.factoryContract = new ContractPromise(this.api, factoryMetadata, this.config.factoryContract);
    this.routerContract = new ContractPromise(this.api, routerMetadata, this.config.routerContract);
    
    console.log(`✅ Conectado e contratos carregados`);
  }

  /**
   * 📋 Validar token para listagem
   */
  async validateToken(tokenInfo: TokenInfo): Promise<boolean> {
    console.log(`🔍 Validando token ${tokenInfo.symbol}...`);

    try {
      // 1. Verificar se é contrato PSP22 válido
      const isValidPSP22 = await this.validatePSP22Contract(tokenInfo.address);
      if (!isValidPSP22) {
        console.error(`❌ Token não é PSP22 válido`);
        return false;
      }

      // 2. Verificar se já está listado
      const isAlreadyListed = await this.isTokenListed(tokenInfo.address);
      if (isAlreadyListed) {
        console.error(`❌ Token já está listado`);
        return false;
      }

      // 3. Verificar poder de voto do proposer
      const votingPower = await this.getVotingPower(this.proposerAccount.address);
      if (votingPower.lt(LISTING_CONFIG.MIN_PROPOSAL_POWER)) {
        console.error(`❌ Poder de voto insuficiente. Necessário: ${LISTING_CONFIG.MIN_PROPOSAL_POWER.div(new BN('100000000')).toString()} LUNES`);
        return false;
      }

      // 4. Verificar balance para taxa de proposta
      const balance = await this.api.query.system.account(this.proposerAccount.address);
      const freeBalance = balance.data.free.toBN();
      if (freeBalance.lt(LISTING_CONFIG.PROPOSAL_FEE.add(LISTING_CONFIG.IMPLEMENTATION_FEE))) {
        console.error(`❌ Balance insuficiente para taxas`);
        return false;
      }

      console.log(`✅ Token validado com sucesso`);
      return true;

    } catch (error) {
      console.error(`❌ Erro na validação:`, error);
      return false;
    }
  }

  /**
   * 🏛️ Criar proposta de listagem
   */
  async createListingProposal(tokenInfo: TokenInfo): Promise<string> {
    console.log(`📝 Criando proposta de listagem para ${tokenInfo.symbol}...`);

    const title = `LIST_${tokenInfo.symbol.toUpperCase()}`;
    const description = `Listar ${tokenInfo.name} (${tokenInfo.symbol}) na Lunex DEX\n\n` +
                       `Endereço: ${tokenInfo.address}\n` +
                       `Decimais: ${tokenInfo.decimals}\n` +
                       `Descrição: ${tokenInfo.description}\n` +
                       (tokenInfo.website ? `Website: ${tokenInfo.website}\n` : '') +
                       (tokenInfo.whitepaper ? `Whitepaper: ${tokenInfo.whitepaper}\n` : '') +
                       (tokenInfo.audit ? `Auditoria: ${tokenInfo.audit}\n` : '');

    const tx = this.stakingContract.tx.createProposal(
      {
        gasLimit: new BN('500000000000'), // 5,000 LUNES
        storageDepositLimit: new BN('100000000000'), // 1,000 LUNES
        value: LISTING_CONFIG.PROPOSAL_FEE
      },
      title,
      description,
      tokenInfo.address,
      LISTING_CONFIG.VOTING_PERIOD
    );

    return new Promise((resolve, reject) => {
      let proposalId = '';
      
      tx.signAndSend(this.proposerAccount, ({ events = [], status }) => {
        if (status.isInBlock) {
          events.forEach(({ event: { data, method, section } }) => {
            if (section === 'contracts' && method === 'ContractEmitted') {
              // Decodificar evento para pegar proposal ID
              try {
                const decoded = this.stakingContract.abi.decodeEvent(data[1] as any);
                if (decoded.event.identifier === 'ProposalCreated') {
                  proposalId = decoded.args[0].toString();
                  console.log(`📋 Proposta criada com ID: ${proposalId}`);
                }
              } catch (e) {
                // Ignore decode errors
              }
            }
          });
        } else if (status.isFinalized) {
          if (proposalId) {
            console.log(`✅ Proposta ${proposalId} criada com sucesso`);
            resolve(proposalId);
          } else {
            reject(new Error('❌ Falha ao criar proposta'));
          }
        }
      }).catch(reject);
    });
  }

  /**
   * 🗳️ Verificar status da votação
   */
  async checkVotingStatus(proposalId: string): Promise<any> {
    console.log(`🗳️ Verificando status da proposta ${proposalId}...`);

    const proposalDetails = await this.stakingContract.query.getProposalDetails(
      this.proposerAccount.address,
      { gasLimit: new BN('100000000000') },
      proposalId
    );

    if (proposalDetails.output) {
      const details = proposalDetails.output.toJSON();
      console.log(`📊 Status da votação:`, details);
      return details;
    }

    return null;
  }

  /**
   * ⚡ Executar proposta aprovada
   */
  async executeProposal(proposalId: string): Promise<void> {
    console.log(`⚡ Executando proposta ${proposalId}...`);

    const tx = this.stakingContract.tx.executeProposal(
      {
        gasLimit: new BN('800000000000'), // 8,000 LUNES
        storageDepositLimit: new BN('200000000000'), // 2,000 LUNES
        value: LISTING_CONFIG.IMPLEMENTATION_FEE
      },
      proposalId
    );

    return new Promise((resolve, reject) => {
      tx.signAndSend(this.proposerAccount, ({ events = [], status }) => {
        if (status.isInBlock) {
          console.log(`📋 Execução incluída no bloco`);
        } else if (status.isFinalized) {
          let success = false;
          events.forEach(({ event: { data, method, section } }) => {
            if (section === 'contracts' && method === 'ContractEmitted') {
              try {
                const decoded = this.stakingContract.abi.decodeEvent(data[1] as any);
                if (decoded.event.identifier === 'ProposalExecuted') {
                  success = true;
                  console.log(`✅ Proposta executada com sucesso`);
                }
              } catch (e) {
                // Ignore decode errors
              }
            }
          });
          
          if (success) {
            resolve();
          } else {
            reject(new Error('❌ Falha na execução da proposta'));
          }
        }
      }).catch(reject);
    });
  }

  /**
   * 💧 Criar pool de liquidez inicial
   */
  async createInitialLiquidity(
    tokenAddress: string,
    tokenAmount: BN,
    lunesAmount: BN
  ): Promise<void> {
    console.log(`💧 Criando liquidez inicial...`);

    // Primeiro aprovar tokens
    const tokenMetadata = this.loadContractMetadata('./deployments/psp22-metadata.json');
    const tokenContract = new ContractPromise(this.api, tokenMetadata, tokenAddress);

    console.log(`1. Aprovando tokens...`);
    const approveTx = tokenContract.tx.approve(
      { gasLimit: new BN('200000000000') },
      this.config.routerContract,
      tokenAmount
    );

    await this.signAndWaitForFinalization(approveTx, 'approve');

    // Adicionar liquidez
    console.log(`2. Adicionando liquidez...`);
    const addLiquidityTx = this.routerContract.tx.addLiquidityLunes(
      {
        gasLimit: new BN('800000000000'),
        value: lunesAmount
      },
      tokenAddress,
      tokenAmount,
      tokenAmount.mul(new BN(95)).div(new BN(100)), // 5% slippage
      lunesAmount.mul(new BN(95)).div(new BN(100)),  // 5% slippage
      this.proposerAccount.address,
      Math.floor(Date.now() / 1000) + 3600 // 1 hora deadline
    );

    await this.signAndWaitForFinalization(addLiquidityTx, 'addLiquidityLunes');
    console.log(`✅ Liquidez inicial criada com sucesso`);
  }

  /**
   * 📊 Gerar relatório de listagem
   */
  async generateListingReport(tokenInfo: TokenInfo, proposalId: string): Promise<void> {
    const report = {
      timestamp: new Date().toISOString(),
      network: this.config.network,
      token: tokenInfo,
      proposalId,
      proposer: this.proposerAccount.address,
      status: 'listed',
      fees: {
        proposal: LISTING_CONFIG.PROPOSAL_FEE.toString(),
        implementation: LISTING_CONFIG.IMPLEMENTATION_FEE.toString()
      }
    };

    const filename = `token-listing-${tokenInfo.symbol.toLowerCase()}-${Date.now()}.json`;
    fs.writeFileSync(filename, JSON.stringify(report, null, 2));
    console.log(`📊 Relatório salvo em: ${filename}`);
  }

  // ===============================
  // FUNÇÕES AUXILIARES
  // ===============================

  private loadContractMetadata(path: string): any {
    return JSON.parse(fs.readFileSync(path, 'utf8'));
  }

  private async validatePSP22Contract(address: string): Promise<boolean> {
    try {
      // Carregar metadata PSP22 padrão
      const psp22Metadata = this.loadContractMetadata('./deployments/psp22-metadata.json');
      const contract = new ContractPromise(this.api, psp22Metadata, address);

      // Testar métodos PSP22 básicos
      const nameResult = await contract.query.name(this.proposerAccount.address, {});
      const symbolResult = await contract.query.symbol(this.proposerAccount.address, {});
      const decimalsResult = await contract.query.decimals(this.proposerAccount.address, {});

      return nameResult.output && symbolResult.output && decimalsResult.output;
    } catch {
      return false;
    }
  }

  private async isTokenListed(tokenAddress: string): Promise<boolean> {
    try {
      const result = await this.stakingContract.query.isProjectApproved(
        this.proposerAccount.address,
        { gasLimit: new BN('100000000000') },
        tokenAddress
      );
      return result.output?.toJSON() || false;
    } catch {
      return false;
    }
  }

  private async getVotingPower(address: string): Promise<BN> {
    try {
      const result = await this.stakingContract.query.getVotingPower(
        this.proposerAccount.address,
        { gasLimit: new BN('100000000000') },
        address
      );
      return new BN(result.output?.toString() || '0');
    } catch {
      return new BN('0');
    }
  }

  private async signAndWaitForFinalization(tx: any, operation: string): Promise<void> {
    return new Promise((resolve, reject) => {
      tx.signAndSend(this.proposerAccount, ({ status, dispatchError }: any) => {
        if (status.isFinalized) {
          if (dispatchError) {
            reject(new Error(`❌ ${operation} falhou: ${dispatchError.toString()}`));
          } else {
            resolve();
          }
        }
      }).catch(reject);
    });
  }

  /**
   * 🚀 Processo completo de listagem
   */
  async listToken(
    tokenInfo: TokenInfo,
    initialLiquidity?: { tokenAmount: string, lunesAmount: string }
  ): Promise<void> {
    try {
      console.log(`🚀 Iniciando processo de listagem do ${tokenInfo.symbol}...`);

      await this.initialize();

      // 1. Validar token
      const isValid = await this.validateToken(tokenInfo);
      if (!isValid) {
        throw new Error('❌ Token não passou na validação');
      }

      // 2. Criar proposta
      const proposalId = await this.createListingProposal(tokenInfo);
      console.log(`📝 Proposta criada: ${proposalId}`);
      console.log(`🗳️ Período de votação: ${LISTING_CONFIG.VOTING_PERIOD / (24 * 60 * 60)} dias`);
      console.log(`⏰ A comunidade pode votar agora!`);

      // 3. Aguardar votação (em um processo real, isso seria feito separadamente)
      console.log(`⏳ Para verificar o status da votação, use:`);
      console.log(`   npm run check-proposal ${proposalId}`);
      console.log(`⏳ Para executar proposta aprovada, use:`);
      console.log(`   npm run execute-proposal ${proposalId}`);

      // 4. Se liquidez inicial foi especificada, mostrar instruções
      if (initialLiquidity) {
        console.log(`\n💧 Após aprovação da proposta, adicione liquidez inicial:`);
        console.log(`   Token Amount: ${initialLiquidity.tokenAmount}`);
        console.log(`   LUNES Amount: ${initialLiquidity.lunesAmount}`);
        console.log(`   Use: npm run add-liquidity ${tokenInfo.address} ${initialLiquidity.tokenAmount} ${initialLiquidity.lunesAmount}`);
      }

      // 5. Gerar relatório
      await this.generateListingReport(tokenInfo, proposalId);

      console.log(`\n🎉 PROCESSO DE LISTAGEM INICIADO COM SUCESSO! 🎉`);
      console.log(`📋 Próximos passos:`);
      console.log(`   1. Aguardar período de votação (${LISTING_CONFIG.VOTING_PERIOD / (24 * 60 * 60)} dias)`);
      console.log(`   2. Promover a proposta na comunidade`);
      console.log(`   3. Executar proposta se aprovada`);
      console.log(`   4. Adicionar liquidez inicial`);
      console.log(`   5. Anunciar listagem para traders`);

    } catch (error) {
      console.error(`💥 Erro no processo de listagem:`, error);
      throw error;
    } finally {
      await this.api?.disconnect();
    }
  }
}

// 🎯 COMANDOS PRINCIPAIS
async function main() {
  const command = process.argv[2];
  
  if (!command) {
    console.log(`
💎 LUNEX DEX TOKEN LISTING TOOL

Comandos disponíveis:

  list-token <config.json>         - Iniciar processo de listagem
  check-proposal <proposal_id>     - Verificar status da proposta
  execute-proposal <proposal_id>   - Executar proposta aprovada
  add-liquidity <token> <amount> <lunes> - Adicionar liquidez inicial

Exemplo de config.json:
{
  "network": "testnet",
  "proposerSeed": "//Alice", 
  "stakingContract": "5GH...",
  "factoryContract": "5FH...",
  "routerContract": "5EH...",
  "token": {
    "address": "5DH...",
    "name": "Example Token",
    "symbol": "EXT",
    "decimals": 8,
    "description": "Token de exemplo para testes",
    "website": "https://example.com"
  },
  "initialLiquidity": {
    "tokenAmount": "1000000000000000",
    "lunesAmount": "1000000000000"
  }
}
    `);
    process.exit(1);
  }

  try {
    switch (command) {
      case 'list-token':
        const configPath = process.argv[3];
        if (!configPath) {
          console.error('❌ Especifique o arquivo de configuração');
          process.exit(1);
        }
        
        const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
        const lister = new TokenLister(config);
        await lister.listToken(config.token, config.initialLiquidity);
        break;

      case 'check-proposal':
        // Implementar check de proposta
        console.log('🔍 Verificando proposta...');
        break;

      case 'execute-proposal':
        // Implementar execução de proposta
        console.log('⚡ Executando proposta...');
        break;

      case 'add-liquidity':
        // Implementar adição de liquidez
        console.log('💧 Adicionando liquidez...');
        break;

      default:
        console.error(`❌ Comando inválido: ${command}`);
        process.exit(1);
    }
  } catch (error) {
    console.error('💥 Erro:', error);
    process.exit(1);
  }
}

// 🚀 EXECUTAR
if (require.main === module) {
  main();
}

export { TokenLister, TokenInfo, ListingConfig };
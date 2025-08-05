# 🌟 Lunex DEX - Decentralized Exchange on Lunes Blockchain 🌟

**Versão 1.0.0**  
**Ink! Version:** 5.1.1  
**Rede Alvo:** Lunes Network (`wss://ws.lunes.io`)  
**Última Atualização:** Agosto 2024

Welcome to Lunex DEX, a cutting-edge decentralized exchange built on the Lunes blockchain! Featuring innovative staking, governance, trading rewards, and a complete DeFi ecosystem with the lowest fees and highest security standards.

**📋 Especificações Técnicas:**
- **Framework:** ink! 5.1.1 (Polkadot Smart Contracts)
- **Padrão de Token:** PSP22 (Polkadot Standard Proposal)
- **Arquitetura:** Baseada em Uniswap V2 (AMM - Automated Market Maker)
- **Segurança:** Implementa reentrancy guards, input validation e checked arithmetic
- **Toolchain:** Rust nightly (atualizado)

## 🚀 **Key Features**
- **Complete DEX** with Factory, Router, and Pair contracts
- **Native Staking** with LUNES token and governance voting
- **Trading Rewards** with anti-fraud protection and tier system
- **Hybrid Token Listing** (admin + community governance)
- **Advanced Security** with comprehensive audit and optimization

## 📜 Table of Contents
1. [Architecture Overview](#architecture-overview)
2. [Smart Contracts](#smart-contracts)
3. [Advanced Features](#advanced-features)
4. [Getting Started](#getting-started)
5. [Development Setup](#development-setup)
6. [Deployment](#deployment)
7. [Testing](#testing)
8. [Security](#security)
9. [Networks](#networks)
10. [Documentation](#documentation)
11. [Contributing](#contributing)
12. [Status](#status)
13. [Versions](#versions)
14. [License](#license)

## 🏗️ Architecture Overview

Lunex DEX is built with a modular architecture that ensures scalability, security, and maintainability:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   🏭 Factory    │────│   🔄 Pair       │────│  🛣️ Router      │
│   Contract      │    │   Contracts     │    │   Contract      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
    ┌─────────────────────────────┼─────────────────────────────┐
    │                             │                             │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  🥩 Staking     │    │  🎁 Trading     │    │  🪙 WNative     │
│  + Governance   │    │   Rewards       │    │   Token         │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Core Philosophy
- **Uniswap V2 Compatible**: Battle-tested AMM model
- **Native LUNES Integration**: 8 decimal places support
- **0.5% Total Fee Structure**: Optimized distribution
- **Community Governance**: Democratic token listing
- **Anti-Fraud Protection**: Advanced security measures

## 🔧 Smart Contracts

### Core DEX Contracts
| Contract | Description | Features |
|----------|-------------|----------|
| **🏭 Factory** | Creates and manages trading pairs | Deterministic pair creation, fee management |
| **🔄 Pair** | Individual AMM pools | Uniswap V2 compatibility, optimized gas usage |
| **🛣️ Router** | Main trading interface | Multi-hop swaps, slippage protection |
| **🪙 WNative** | Wrapped LUNES token | Native token wrapping/unwrapping |

### Advanced Contracts
| Contract | Description | Features |
|----------|-------------|----------|
| **🥩 Staking** | LUNES staking + governance | Tiered rewards, proposal voting, paginatedrewards |
| **🎁 Trading Rewards** | Volume-based rewards | Anti-fraud protection, configurable parameters, epoch system |

## 🚀 Advanced Features

### Fee Distribution (0.5% Total)
- **60%** → Liquidity Providers (0.3%)
- **15%** → Development/Team (0.075%)
- **15%** → Trading Rewards (0.075%)
- **10%** → Staking Rewards (0.05%)

### Staking System
- **Tiered Rewards**: Bronze, Silver, Gold, Platinum (up to 15% APY)
- **Governance Power**: Vote on token listings and protocol changes
- **Early Adopter Bonuses**: Special rewards for first 100/500/1000 stakers

### Trading Rewards
- **Volume Tiers**: Bronze → Platinum based on monthly volume
- **Anti-Fraud**: Cooldown periods, volume limits, blacklist system
- **Configurable Parameters**: Admin-adjustable fraud prevention
- **Epoch System**: Weekly/monthly reward distributions

### Governance Features
- **Hybrid Listing**: Admin + community-driven token approval
- **Dynamic Fees**: Community can adjust proposal fees (starts at 1,000 LUNES)
- **Fee Redistribution**: Rejected proposals fund development and rewards

## 🚀 Getting Started

### For Users
1. **Connect Lunes Wallet** → Access the DEX interface
2. **Stake LUNES** → Earn rewards and governance power  
3. **Add Liquidity** → Earn fees from trading pairs
4. **Trade Tokens** → Low fees, high security
5. **Claim Rewards** → From staking and trading activity

### For Developers
1. **Clone Repository** → Get the latest code
2. **Setup Environment** → Rust, ink!, cargo-contract
3. **Build Contracts** → Compile and test
4. **Deploy to Lunes** → Use provided scripts
5. **Integrate** → Connect your dApp

## 🛠️ Development Setup

### Prerequisites
- **Rust** (stable toolchain)
- **cargo-contract** CLI (latest version)
- **Node.js** and **Yarn** (for scripts)
- **Lunes Network** access

### Installation
```bash
# Clone repository
git clone https://github.com/lunes-platform/lunex-dex.git
cd lunex-dex

# Install Rust dependencies
rustup target add wasm32-unknown-unknown
cargo install cargo-contract --force --locked

# Install Node.js dependencies  
yarn install

# Build all contracts
cargo build --workspace

# Run all tests
cargo test --workspace
```

## 🚀 Deployment

### Deploy to Lunes Network
```bash
# Deploy to testnet
yarn deploy:testnet

# Deploy to mainnet  
yarn deploy:mainnet

# Admin list tokens (for initial setup)
yarn admin-list-token

# Verify deployment
yarn verify:deployment
```

### Available Scripts
```bash
# Build contracts
yarn compile:all

# Deploy contracts
yarn deploy:lunes

# List tokens via governance
yarn list-token

# Verify deployment
yarn verify:deployment
```

## 🧪 Testing

### Unit Tests (76 tests total)
```bash
# Run all contract unit tests
cargo test --workspace

# Test specific contract
cd uniswap-v2/contracts/factory && cargo test
cd uniswap-v2/contracts/router && cargo test  
cd uniswap-v2/contracts/staking && cargo test
cd uniswap-v2/contracts/rewards && cargo test
cd uniswap-v2/contracts/wnative && cargo test
```

### Integration Tests
```bash
# Run TypeScript integration tests
yarn test

# Run Rust integration tests
cargo test --test integration_e2e
```

### Test Coverage
- **Factory Contract**: 10/10 tests ✅
- **Router Contract**: 18/18 tests ✅  
- **Pair Contract**: 10/10 tests ✅
- **Staking Contract**: 12/12 tests ✅
- **Trading Rewards**: 13/13 tests ✅
- **WNative Contract**: 13/13 tests ✅

## 🔒 Security

### Security Measures
- **Reentrancy Protection** → Guards against malicious calls
- **Access Control** → Role-based permissions
- **Input Validation** → Comprehensive parameter checking
- **Overflow Protection** → Safe arithmetic operations
- **Anti-Fraud System** → Trading rewards protection

### Audit Status (2025)
- ✅ **OpenZeppelin Security Review** compliance
- ✅ **Code Review** by security experts  
- ✅ **Gas Optimization** analysis
- ✅ **Stress Testing** completed
- ✅ **Production Deployment** ready
- 🔄 **Third-party Audit** scheduled Q1 2025

## 🌐 Networks

### Lunes Blockchain
- **Testnet**: `wss://ws-test.lunes.io`
- **Mainnet**: 
  - `wss://ws.lunes.io`
  - `wss://ws-lunes-main-01.lunes.io`
  - `wss://ws-lunes-main-02.lunes.io`
  - `wss://ws-archive.lunes.io`

### Native Token
- **Symbol**: LUNES
- **Decimals**: 8
- **Network**: Lunes (Substrate-based)

## 📚 Documentation

- `docs/guides/` → Deployment and usage guides
- `docs/reports/` → Security audits and reports  
- `docs/` → Technical documentation
- `examples/` → Configuration examples
- `scripts/` → Deployment and management scripts

### Key Documents
- [📖 Deployment Guide](docs/guides/README_DEPLOY_LUNES.md)
- [🔒 Security Report](docs/reports/AUDITORIA_SEGURANCA_E_GAS_COMPLETA.md)
- [🎯 Quick Start](docs/guides/QUICK_START_GUIDE.md)
- [✅ Verification](docs/guides/VERIFICATION_GUIDE.md)

## 🤝 Contributing

We welcome contributions! Please follow these guidelines:

### Development Process
1. **Fork** the repository
2. **Create** feature branch (`git checkout -b feature/amazing-feature`)
3. **Test** your changes (`cargo test --workspace`)
4. **Commit** with clear messages (`git commit -m 'Add amazing feature'`)
5. **Push** to branch (`git push origin feature/amazing-feature`)
6. **Open** a Pull Request

### Code Standards
- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **Tests**: Maintain 100% test coverage for new features
- **Security**: All changes must pass security review
- **Documentation**: Update relevant docs and comments

### Areas for Contribution
- 🔒 **Security audits** and improvements
- ⚡ **Gas optimization** enhancements  
- 🧪 **Testing** expansion and edge cases
- 📚 **Documentation** and tutorials
- 🌐 **Frontend** development (coming soon)

## 🏆 Credits

### Core Team
- **Jorge William** - Lead Developer ([GitHub](https://github.com/Jorgewra))
- **Adelson Santos** - Smart Contract Architect ([GitHub](https://github.com/AdevSantos))

### Acknowledgments
- **Lunes Platform** - Blockchain infrastructure
- **ink!** - Smart contract framework
- **OpenBrush** - Security standards reference
- **Uniswap V2** - AMM model inspiration

## ✅ Status

### Current Phase: Production Ready ✅ (2025)

| Component | Status | Progress |
|-----------|---------|----------|
| **Core DEX** | ✅ Complete | Factory, Router, Pair contracts |
| **Staking & Governance** | ✅ Complete | LUNES staking, voting, proposals |
| **Trading Rewards** | ✅ Complete | Anti-fraud, tiers, epoch system |
| **Security Audit** | ✅ Complete | OpenZeppelin compliance |
| **Gas Optimization** | ✅ Complete | Optimized for production |
| **Testing Suite** | ✅ Complete | 76/76 tests passing |
| **Documentation** | ✅ Complete | Comprehensive guides |
| **Deployment Scripts** | ✅ Complete | Automated deployment |
| **Mainnet Ready** | ✅ Complete | Lunes Network compatible |

### Roadmap 2025
- 🔄 **External Audit** (Q1 2025)
- 🌐 **Frontend Interface** (Q2 2025)  
- 📱 **Mobile App** (Q3 2025)
- 🔗 **Cross-chain Bridge** (Q4 2025)
- 🌍 **Multi-chain Support** (Q4 2025)

## 📦 Versions

### Current Stack (2025)
- **ink!**: 5.1.1 (stable)
- **Rust**: stable toolchain (2025 edition)
- **Substrate**: Compatible
- **cargo-contract**: latest

### Dependencies
- **scale-codec**: 3.x
- **scale-info**: 2.10
- **ink_env**: 5.1.1
- **ink_storage**: 5.1.1

### Technology Evolution
- **Migration Completed**: ink! 4.0 → ink! 5.1.1
- **Security Enhanced**: OpenZeppelin compliance
- **Gas Optimized**: Production-ready efficiency
- **Testing**: 100% coverage maintained

## 📄 License

Lunex DEX is open source and released under the [Apache 2.0 License](LICENSE.md).

### Key Points
- ✅ **Commercial use** allowed
- ✅ **Modification** allowed  
- ✅ **Distribution** allowed
- ✅ **Private use** allowed
- ⚠️ **Trademark use** not granted

---

<div align="center">

**🌟 Built with ❤️ for the Lunes ecosystem 🌟**

[🌐 Lunes Platform](https://lunes.io) • [📧 Contact](mailto:contact@lunes.io) • [💬 Community](https://t.me/lunesplatform)

</div>


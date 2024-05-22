# Lunex - A DEX on the Lunes Blockchain

Welcome to Lunex, the decentralized exchange (DEX) built on the Lunes blockchain! Our mission is to provide a seamless, secure, and cost-effective platform for trading digital assets. With low fees and a focus on liquidity pools, Lunex is your go-to solution for decentralized trading on the Lunes network.

## Table of Contents
1. [Introduction](#introduction)
2. [Features](#features)
3. [Getting Started](#getting-started)
4. [Liquidity Pools](#liquidity-pools)
5. [Smart Contracts](#smart-contracts)
6. [Low Fees](#low-fees)
7. [Contributing](#contributing)
8. [License](#license)

## Introduction
Lunex is a cutting-edge DEX specifically designed for the Lunes blockchain. By leveraging the power of smart contracts developed with !INK 4.x, Lunex ensures a robust and secure trading environment. Whether you are a seasoned trader or new to the world of decentralized finance (DeFi), Lunex offers a user-friendly platform with the lowest fees in the market.

## Features
- **Decentralized Trading**: Trade directly from your wallet without the need for intermediaries.
- **Liquidity Pools**: Provide liquidity and earn rewards through our innovative liquidity pool system.
- **Low Fees**: Enjoy some of the lowest transaction fees in the DeFi space.
- **Secure Smart Contracts**: Built with !INK 4.x, ensuring security and efficiency.
- **User-Friendly Interface**: Easy-to-navigate platform for all levels of users.

## Getting Started
1. **Create a Lunes Wallet**: If you don't have one already, create a Lunes wallet to start trading.
2. **Connect Your Wallet**: Connect your Lunes wallet to the Lunex DEX.
3. **Add Liquidity**: Deposit your tokens into our liquidity pools to start earning rewards.
4. **Start Trading**: Begin trading your favorite tokens with low fees and high security.

## Liquidity Pools
Lunex offers a variety of liquidity pools that allow users to provide liquidity and earn rewards. By adding liquidity to a pool, you facilitate smoother trading operations on our DEX and earn a portion of the transaction fees.

### How to Add Liquidity
1. Navigate to the **Liquidity Pools** section on the Lunex platform.
2. Select the pool you want to provide liquidity to.
3. Deposit the required token pair into the pool.
4. Start earning rewards based on the trading activity in that pool.

## Smart Contracts
Our smart contracts are developed using !INK 4.x, a robust and secure framework for blockchain applications. This ensures that all transactions on Lunex are executed efficiently and securely, providing users with peace of mind.

## Low Fees
One of Lunex's standout features is our exceptionally low fees. We believe that trading should be accessible to everyone, which is why we've minimized transaction costs without compromising on security or performance.

## Contributing
We welcome contributions from the community! Whether you are a developer, designer, or simply passionate about DeFi, your input is valuable to us.

### How to Contribute
1. Fork the repository.
2. Create a new branch (`git checkout -b feature/YourFeature`).
3. Commit your changes (`git commit -m 'Add some feature'`).
4. Push to the branch (`git push origin feature/YourFeature`).
5. Open a pull request.

## 🏗️ How to use - Contracts
##### 💫 Build
Use these [instructions](https://use.ink/getting-started/setup) to set up your ink!/Rust environment    
Run this command in the contract folder:

```sh
cargo contract build
```

##### 💫 Run unit test

```sh
cargo test
```
##### 💫 Deploy
First start your local node.  
Deploy using polkadot JS. Instructions on [Astar docs](https://docs.astar.network/docs/wasm/sc-dev/polkadotjs-ui)

##### 💫 Run integration test
First start your local node. Recommended [swanky-node](https://github.com/AstarNetwork/swanky-node) v0.13.0

```sh
yarn
yarn compile
yarn test:typechain
```

### Status
- :white_check_mark: contracts
- :white_large_square: integration tests
- :white_large_square: UI (Julho 2024)
- :white_large_square: integration mainet (Agosto 2024)
- :white_large_square: Audit

### Versions
[ink! 4.0.0](https://github.com/paritytech/ink/tree/v4.0.0)   
[openbrush 3.0.0](https://github.com/727-Ventures/openbrush-contracts/tree/3.0.0)

### License
Apache 2.0

---


## License
Lunex is open-source and available under the Apache 2.0.

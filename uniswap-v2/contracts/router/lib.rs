#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![warn(clippy::arithmetic_side_effects)]
#[ink::contract]
pub mod router_contract {
    use psp22::PSP22Error;
    use ink::prelude::vec::Vec;
    use ink::prelude::vec;

    // ========================================
    // ROUTER CONTRACT - DEX OPERATIONS COORDINATOR
    // ========================================
    // 
    // Este contrato coordena operações complexas do DEX:
    // - Add/Remove Liquidity: Gerencia tokens e LP tokens
    // - Swaps: Coordena trocas através de múltiplos pares
    // - Slippage Protection: Validações min/max amounts
    // - Multi-hop: Swaps através de múltiplos pares
    // 
    // ## Segurança:
    // - Deadline verification para prevenir transações antigas
    // - Slippage protection em todas operações
    // - Safe arithmetic em todos os cálculos
    // - Input validation rigorosa

    // ========================================
    // EVENTOS (PARA INDEXADORES E UIS)
    // ========================================

    /// Emitido quando liquidez é adicionada
    #[ink(event)]
    pub struct LiquidityAdded {
        #[ink(topic)]
        pub token_a: AccountId,
        #[ink(topic)]
        pub token_b: AccountId,
        pub amount_a: Balance,
        pub amount_b: Balance,
        pub liquidity: Balance,
        #[ink(topic)]
        pub to: AccountId,
    }

    /// Emitido quando liquidez é removida
    #[ink(event)]
    pub struct LiquidityRemoved {
        #[ink(topic)]
        pub token_a: AccountId,
        #[ink(topic)]
        pub token_b: AccountId,
        pub amount_a: Balance,
        pub amount_b: Balance,
        pub liquidity: Balance,
        #[ink(topic)]
        pub to: AccountId,
    }

    /// Emitido quando swap é realizado
    #[ink(event)]
    pub struct Swap {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_in: Balance,
        pub amount_out: Balance,
        pub path: Vec<AccountId>,
        #[ink(topic)]
        pub to: AccountId,
    }

    // ========================================
    // ERROS ESPECÍFICOS DO ROUTER CONTRACT
    // ========================================

    /// Erros que podem ocorrer nas operações do Router
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    #[allow(clippy::cast_possible_truncation)]
    pub enum RouterError {
        /// Deadline da transação expirou
        Expired,
        /// Amount insuficiente de output
        InsufficientAAmount,
        /// Amount insuficiente de B amount
        InsufficientBAmount,
        /// Output amount insuficiente no swap
        InsufficientOutputAmount,
        /// Liquidez insuficiente
        InsufficientLiquidity,
        /// Path de swap inválido
        InvalidPath,
        /// Token addresses iguais
        IdenticalAddresses,
        /// Token address zero
        ZeroAddress,
        /// Excessive input amount (slippage muito alto)
        ExcessiveInputAmount,
        /// Pair não existe
        PairNotExists,
        /// Erro no token PSP22 subjacente
        PSP22(PSP22Error),
    }

    impl From<PSP22Error> for RouterError {
        fn from(error: PSP22Error) -> Self {
            RouterError::PSP22(error)
        }
    }

    // ========================================
    // CONSTANTES DO PROTOCOLO ROUTER
    // ========================================
    mod constants {
        /// Minimum liquidity para cálculos (mesmo valor do Pair)
        pub const MINIMUM_LIQUIDITY: u128 = 100;
        
        /// Fee para swaps (0.3% = 997/1000)
        pub const FEE_DENOMINATOR: u128 = 1000;
        pub const FEE_NUMERATOR: u128 = 997;
    }

    // ========================================
    // STORAGE DO ROUTER CONTRACT
    // ========================================

    /// Storage principal do Router Contract
    #[ink(storage)]
    pub struct RouterContract {
        /// Endereço do Factory Contract
        factory: AccountId,
        /// Endereço do WNative Contract
        wnative: AccountId,
    }

    impl RouterContract {
        /// Construtor do Router Contract
        #[ink(constructor)]
        pub fn new(factory: AccountId, wnative: AccountId) -> Self {
            Self { factory, wnative }
        }

        // ========================================
        // QUERIES (READ-ONLY)
        // ========================================

        /// Retorna o endereço do Factory
        #[ink(message)]
        pub fn factory(&self) -> AccountId {
            self.factory
        }

        /// Retorna o endereço do WNative
        #[ink(message)]
        pub fn wnative(&self) -> AccountId {
            self.wnative
        }

        // ========================================
        // OPERAÇÕES DE LIQUIDEZ
        // ========================================

        /// Adiciona liquidez a um par de tokens
        /// 
        /// # Parâmetros
        /// - `token_a`: Primeiro token do par
        /// - `token_b`: Segundo token do par  
        /// - `amount_a_desired`: Amount desejado do token A
        /// - `amount_b_desired`: Amount desejado do token B
        /// - `amount_a_min`: Amount mínimo do token A (slippage protection)
        /// - `amount_b_min`: Amount mínimo do token B (slippage protection)
        /// - `to`: Destinatário dos LP tokens
        /// - `deadline`: Timestamp limite para execução
        #[ink(message)]
        pub fn add_liquidity(
            &mut self,
            token_a: AccountId,
            token_b: AccountId,
            amount_a_desired: Balance,
            amount_b_desired: Balance,
            amount_a_min: Balance,
            amount_b_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(Balance, Balance, Balance), RouterError> {
            // Validações iniciais
            self.ensure_deadline(deadline)?;
            self.validate_addresses(token_a, token_b)?;
            
            // Para TDD, implementação simplificada
            // Em produção, coordenaria com Factory e Pair
            let amount_a = amount_a_desired;
            let amount_b = amount_b_desired;
            
            // Validar slippage protection
            if amount_a < amount_a_min {
                return Err(RouterError::InsufficientAAmount);
            }
            if amount_b < amount_b_min {
                return Err(RouterError::InsufficientBAmount);
            }
            
            // Calcular liquidez usando fórmula simplificada para TDD
            let liquidity = self.calculate_liquidity(amount_a, amount_b)?;
            
            // Emitir evento
            self.env().emit_event(LiquidityAdded {
                token_a,
                token_b,
                amount_a,
                amount_b,
                liquidity,
                to,
            });
            
            Ok((amount_a, amount_b, liquidity))
        }

        /// Remove liquidez de um par de tokens
        #[ink(message)]
        pub fn remove_liquidity(
            &mut self,
            token_a: AccountId,
            token_b: AccountId,
            liquidity: Balance,
            amount_a_min: Balance,
            amount_b_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(Balance, Balance), RouterError> {
            // Validações iniciais
            self.ensure_deadline(deadline)?;
            self.validate_addresses(token_a, token_b)?;
            
            if liquidity == 0 {
                return Err(RouterError::InsufficientLiquidity);
            }
            
            // Para TDD, implementação simplificada
            // Calcular amounts proporcionais à liquidez
            let amount_a = liquidity / 2; // Simplificado para TDD
            let amount_b = liquidity / 2;
            
            // Validar slippage protection
            if amount_a < amount_a_min {
                return Err(RouterError::InsufficientAAmount);
            }
            if amount_b < amount_b_min {
                return Err(RouterError::InsufficientBAmount);
            }
            
            // Emitir evento
            self.env().emit_event(LiquidityRemoved {
                token_a,
                token_b,
                amount_a,
                amount_b,
                liquidity,
                to,
            });
            
            Ok((amount_a, amount_b))
        }

        // ========================================
        // OPERAÇÕES DE SWAP
        // ========================================

        /// Swap com input amount exato
        #[ink(message)]
        pub fn swap_exact_tokens_for_tokens(
            &mut self,
            amount_in: Balance,
            amount_out_min: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            // Validações iniciais
            self.ensure_deadline(deadline)?;
            self.validate_path(&path)?;
            
            if amount_in == 0 {
                return Err(RouterError::InsufficientOutputAmount);
            }
            
            // Para TDD, implementação simplificada do swap
            // Em produção, calcularia através de múltiplos pares
            let amount_out = self.calculate_output_amount(amount_in, &path)?;
            
            // Validar slippage protection
            if amount_out < amount_out_min {
                return Err(RouterError::InsufficientOutputAmount);
            }
            
            // Retornar amounts array (input + output)
            let amounts = vec![amount_in, amount_out];
            
            // Emitir evento
            self.env().emit_event(Swap {
                sender: self.env().caller(),
                amount_in,
                amount_out,
                path: path.clone(),
                to,
            });
            
            Ok(amounts)
        }

        /// Swap com output amount exato
        #[ink(message)]
        pub fn swap_tokens_for_exact_tokens(
            &mut self,
            amount_out: Balance,
            amount_in_max: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            // Validações iniciais
            self.ensure_deadline(deadline)?;
            self.validate_path(&path)?;
            
            if amount_out == 0 {
                return Err(RouterError::InsufficientOutputAmount);
            }
            
            // Para TDD, implementação simplificada do swap reverso
            let amount_in = self.calculate_input_amount(amount_out, &path)?;
            
            // Validar slippage protection
            if amount_in > amount_in_max {
                return Err(RouterError::ExcessiveInputAmount);
            }
            
            // Retornar amounts array (input + output)
            let amounts = vec![amount_in, amount_out];
            
            // Emitir evento
            self.env().emit_event(Swap {
                sender: self.env().caller(),
                amount_in,
                amount_out,
                path: path.clone(),
                to,
            });
            
            Ok(amounts)
        }

        // ========================================
        // FUNÇÕES INTERNAS (VALIDAÇÕES E CÁLCULOS)
        // ========================================

        /// Valida se o deadline não expirou
        fn ensure_deadline(&self, deadline: u64) -> Result<(), RouterError> {
            let current_time = self.env().block_timestamp();
            if current_time > deadline {
                return Err(RouterError::Expired);
            }
            Ok(())
        }

        /// Valida endereços dos tokens
        fn validate_addresses(&self, token_a: AccountId, token_b: AccountId) -> Result<(), RouterError> {
            let zero_address = AccountId::from([0u8; 32]);
            if token_a == zero_address || token_b == zero_address {
                return Err(RouterError::ZeroAddress);
            }
            if token_a == token_b {
                return Err(RouterError::IdenticalAddresses);
            }
            Ok(())
        }

        /// Valida path de swap
        fn validate_path(&self, path: &Vec<AccountId>) -> Result<(), RouterError> {
            if path.len() < 2 {
                return Err(RouterError::InvalidPath);
            }
            
            let zero_address = AccountId::from([0u8; 32]);
            for token in path {
                if *token == zero_address {
                    return Err(RouterError::ZeroAddress);
                }
            }
            
            Ok(())
        }

        /// Calcula liquidez para add_liquidity (implementação simplificada para TDD)
        fn calculate_liquidity(&self, amount_a: Balance, amount_b: Balance) -> Result<Balance, RouterError> {
            if amount_a == 0 || amount_b == 0 {
                return Err(RouterError::InsufficientLiquidity);
            }
            
            // Fórmula simplificada para TDD: geometric mean
            let liquidity = self.sqrt(amount_a.checked_mul(amount_b).ok_or(RouterError::InsufficientLiquidity)?);
            
            if liquidity <= constants::MINIMUM_LIQUIDITY {
                return Err(RouterError::InsufficientLiquidity);
            }
            
            Ok(liquidity)
        }

        /// Calcula amount de output para swap (implementação simplificada para TDD)
        fn calculate_output_amount(&self, amount_in: Balance, path: &Vec<AccountId>) -> Result<Balance, RouterError> {
            if amount_in == 0 || path.len() < 2 {
                return Err(RouterError::InsufficientOutputAmount);
            }
            
            // Implementação simplificada para TDD
            // Em produção, usaria as reserves dos pares e fórmula AMM
            let amount_with_fee = amount_in
                .checked_mul(constants::FEE_NUMERATOR)
                .ok_or(RouterError::InsufficientOutputAmount)?
                .checked_div(constants::FEE_DENOMINATOR)
                .ok_or(RouterError::InsufficientOutputAmount)?;
            
            // Simular taxa de câmbio 1:1 para TDD
            Ok(amount_with_fee)
        }

        /// Calcula amount de input para swap reverso (implementação simplificada para TDD)
        fn calculate_input_amount(&self, amount_out: Balance, path: &Vec<AccountId>) -> Result<Balance, RouterError> {
            if amount_out == 0 || path.len() < 2 {
                return Err(RouterError::ExcessiveInputAmount);
            }
            
            // Implementação simplificada para TDD - swap reverso
            let amount_in = amount_out
                .checked_mul(constants::FEE_DENOMINATOR)
                .ok_or(RouterError::ExcessiveInputAmount)?
                .checked_div(constants::FEE_NUMERATOR)
                .ok_or(RouterError::ExcessiveInputAmount)?;
            
            Ok(amount_in)
        }

        /// Implementação da raiz quadrada (Babylonian method)
        fn sqrt(&self, value: Balance) -> Balance {
            if value == 0 {
                return 0;
            }
            
            let mut x = value;
            let mut y = value.checked_add(1).and_then(|sum| sum.checked_div(2)).unwrap_or(1);
            
            while y < x {
                x = y;
                y = value.checked_div(x).and_then(|div| div.checked_add(x)).and_then(|sum| sum.checked_div(2)).unwrap_or(x);
            }
            
            x
        }
    }

    // ========================================
    // TESTES UNITÁRIOS TDD
    // ========================================

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::DefaultEnvironment;

        fn default_accounts() -> ink::env::test::DefaultAccounts<DefaultEnvironment> {
            ink::env::test::default_accounts::<DefaultEnvironment>()
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<DefaultEnvironment>(sender);
        }

        fn set_timestamp(timestamp: u64) {
            ink::env::test::set_block_timestamp::<DefaultEnvironment>(timestamp);
        }

        // ========================================
        // TESTES BÁSICOS DE INICIALIZAÇÃO
        // ========================================

        #[ink::test]
        fn test_new_router_initializes_correctly() {
            let accounts = default_accounts();
            let router = RouterContract::new(accounts.bob, accounts.charlie);
            
            // GREEN: Factory e WNative devem estar configurados corretamente
            assert_eq!(router.factory(), accounts.bob);
            assert_eq!(router.wnative(), accounts.charlie);
        }

        // ========================================
        // TESTES DE VALIDAÇÃO DE DEADLINE
        // ========================================

        #[ink::test]
        fn test_expired_deadline_fails() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000); // Current time
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            // RED: Deadline no passado deve falhar
            let result = router.add_liquidity(
                accounts.django, // token_a
                accounts.eve,    // token_b
                100,             // amount_a_desired
                100,             // amount_b_desired
                90,              // amount_a_min
                90,              // amount_b_min
                accounts.alice,  // to
                500,             // deadline (no passado)
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::Expired);
        }

        // ========================================
        // TESTES DE ADD LIQUIDITY
        // ========================================

        #[ink::test]
        fn test_add_liquidity_success() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            // GREEN: Add liquidity com parâmetros válidos deve funcionar
            let result = router.add_liquidity(
                accounts.django, // token_a
                accounts.eve,    // token_b
                100,             // amount_a_desired
                200,             // amount_b_desired
                90,              // amount_a_min
                180,             // amount_b_min
                accounts.alice,  // to
                2000,            // deadline (futuro)
            );
            
            assert!(result.is_ok());
            let (amount_a, amount_b, liquidity) = result.unwrap();
            assert_eq!(amount_a, 100);
            assert_eq!(amount_b, 200);
            assert!(liquidity > 0);
        }

        #[ink::test]
        fn test_add_liquidity_insufficient_a_amount() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            // RED: amount_a_min muito alto deve falhar
            let result = router.add_liquidity(
                accounts.django, // token_a
                accounts.eve,    // token_b
                100,             // amount_a_desired
                200,             // amount_b_desired
                150,             // amount_a_min (muito alto)
                180,             // amount_b_min
                accounts.alice,  // to
                2000,            // deadline
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::InsufficientAAmount);
        }

        #[ink::test]
        fn test_add_liquidity_identical_addresses() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            // RED: Tokens idênticos devem falhar
            let result = router.add_liquidity(
                accounts.django, // token_a
                accounts.django, // token_b (igual ao A)
                100,             // amount_a_desired
                200,             // amount_b_desired
                90,              // amount_a_min
                180,             // amount_b_min
                accounts.alice,  // to
                2000,            // deadline
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::IdenticalAddresses);
        }

        // ========================================
        // TESTES DE REMOVE LIQUIDITY
        // ========================================

        #[ink::test]
        fn test_remove_liquidity_success() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            // GREEN: Remove liquidity com parâmetros válidos deve funcionar
            let result = router.remove_liquidity(
                accounts.django, // token_a
                accounts.eve,    // token_b
                200,             // liquidity
                90,              // amount_a_min
                90,              // amount_b_min
                accounts.alice,  // to
                2000,            // deadline
            );
            
            assert!(result.is_ok());
            let (amount_a, amount_b) = result.unwrap();
            assert_eq!(amount_a, 100); // liquidity / 2
            assert_eq!(amount_b, 100); // liquidity / 2
        }

        #[ink::test]
        fn test_remove_liquidity_zero_liquidity() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            // RED: Zero liquidity deve falhar
            let result = router.remove_liquidity(
                accounts.django, // token_a
                accounts.eve,    // token_b
                0,               // liquidity (zero)
                90,              // amount_a_min
                90,              // amount_b_min
                accounts.alice,  // to
                2000,            // deadline
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::InsufficientLiquidity);
        }

        #[ink::test]
        fn test_remove_liquidity_insufficient_b_amount() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            // RED: amount_b_min muito alto deve falhar
            let result = router.remove_liquidity(
                accounts.django, // token_a
                accounts.eve,    // token_b
                200,             // liquidity
                90,              // amount_a_min
                150,             // amount_b_min (muito alto)
                accounts.alice,  // to
                2000,            // deadline
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::InsufficientBAmount);
        }

        // ========================================
        // TESTES DE SWAP EXACT TOKENS FOR TOKENS
        // ========================================

        #[ink::test]
        fn test_swap_exact_tokens_success() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            let path = vec![accounts.django, accounts.eve];
            
            // GREEN: Swap com parâmetros válidos deve funcionar
            let result = router.swap_exact_tokens_for_tokens(
                100,            // amount_in
                90,             // amount_out_min
                path,           // path
                accounts.alice, // to
                2000,           // deadline
            );
            
            assert!(result.is_ok());
            let amounts = result.unwrap();
            assert_eq!(amounts.len(), 2);
            assert_eq!(amounts[0], 100); // amount_in
            assert!(amounts[1] >= 90);   // amount_out >= min
        }

        #[ink::test]
        fn test_swap_exact_tokens_zero_input() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            let path = vec![accounts.django, accounts.eve];
            
            // RED: Zero input deve falhar
            let result = router.swap_exact_tokens_for_tokens(
                0,              // amount_in (zero)
                90,             // amount_out_min
                path,           // path
                accounts.alice, // to
                2000,           // deadline
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::InsufficientOutputAmount);
        }

        #[ink::test]
        fn test_swap_exact_tokens_insufficient_output() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            let path = vec![accounts.django, accounts.eve];
            
            // RED: amount_out_min muito alto deve falhar (slippage protection)
            let result = router.swap_exact_tokens_for_tokens(
                100,            // amount_in
                150,            // amount_out_min (muito alto)
                path,           // path
                accounts.alice, // to
                2000,           // deadline
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::InsufficientOutputAmount);
        }

        #[ink::test]
        fn test_swap_exact_tokens_invalid_path() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            let path = vec![accounts.django]; // Path muito curto
            
            // RED: Path inválido deve falhar
            let result = router.swap_exact_tokens_for_tokens(
                100,            // amount_in
                90,             // amount_out_min
                path,           // path (inválido)
                accounts.alice, // to
                2000,           // deadline
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::InvalidPath);
        }

        // ========================================
        // TESTES DE SWAP TOKENS FOR EXACT TOKENS
        // ========================================

        #[ink::test]
        fn test_swap_tokens_for_exact_success() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            let path = vec![accounts.django, accounts.eve];
            
            // GREEN: Swap reverso com parâmetros válidos deve funcionar
            let result = router.swap_tokens_for_exact_tokens(
                100,            // amount_out
                110,            // amount_in_max
                path,           // path
                accounts.alice, // to
                2000,           // deadline
            );
            
            assert!(result.is_ok());
            let amounts = result.unwrap();
            assert_eq!(amounts.len(), 2);
            assert!(amounts[0] <= 110); // amount_in <= max
            assert_eq!(amounts[1], 100); // amount_out exato
        }

        #[ink::test]
        fn test_swap_tokens_for_exact_zero_output() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            let path = vec![accounts.django, accounts.eve];
            
            // RED: Zero output deve falhar
            let result = router.swap_tokens_for_exact_tokens(
                0,              // amount_out (zero)
                110,            // amount_in_max
                path,           // path
                accounts.alice, // to
                2000,           // deadline
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::InsufficientOutputAmount);
        }

        #[ink::test]
        fn test_swap_tokens_for_exact_excessive_input() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            let path = vec![accounts.django, accounts.eve];
            
            // RED: amount_in_max muito baixo deve falhar (slippage protection)
            let result = router.swap_tokens_for_exact_tokens(
                100,            // amount_out
                90,             // amount_in_max (muito baixo)
                path,           // path
                accounts.alice, // to
                2000,           // deadline
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::ExcessiveInputAmount);
        }

        // ========================================
        // TESTES DE VALIDAÇÃO DE PATH E EDGE CASES
        // ========================================

        #[ink::test]
        fn test_path_with_zero_address() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_timestamp(1000);
            
            let mut router = RouterContract::new(accounts.bob, accounts.charlie);
            
            let zero_address = AccountId::from([0u8; 32]);
            let path = vec![accounts.django, zero_address]; // Zero address no path
            
            // RED: Path com zero address deve falhar
            let result = router.swap_exact_tokens_for_tokens(
                100,            // amount_in
                90,             // amount_out_min
                path,           // path (com zero address)
                accounts.alice, // to
                2000,           // deadline
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::ZeroAddress);
        }

        #[ink::test]
        fn test_calculate_liquidity_edge_cases() {
            let accounts = default_accounts();
            let router = RouterContract::new(accounts.bob, accounts.charlie);
            
            // RED: Zero amounts devem falhar
            let result = router.calculate_liquidity(0, 100);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::InsufficientLiquidity);
            
            let result = router.calculate_liquidity(100, 0);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RouterError::InsufficientLiquidity);
            
            // GREEN: Amounts válidos devem funcionar
            let result = router.calculate_liquidity(100, 200);
            assert!(result.is_ok());
            assert!(result.unwrap() > constants::MINIMUM_LIQUIDITY);
        }

        #[ink::test]
        fn test_sqrt_function() {
            let accounts = default_accounts();
            let router = RouterContract::new(accounts.bob, accounts.charlie);
            
            // GREEN: Testes da função sqrt
            assert_eq!(router.sqrt(0), 0);
            assert_eq!(router.sqrt(1), 1);
            assert_eq!(router.sqrt(4), 2);
            assert_eq!(router.sqrt(9), 3);
            assert_eq!(router.sqrt(16), 4);
            assert_eq!(router.sqrt(100), 10);
            
            // Teste com números não quadrados perfeitos
            let result = router.sqrt(8);
            assert!(result >= 2 && result <= 3); // sqrt(8) ≈ 2.83
        }
    }
}
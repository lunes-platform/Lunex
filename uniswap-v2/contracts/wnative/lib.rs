#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod wnative_contract {
    use ink::prelude::string::String;
    use ink::storage::Mapping;

    // ========================================
    // WNATIVE CONTRACT - WRAPPED NATIVE TOKEN
    // ========================================
    // 
    // Este contrato implementa um "Wrapped Native Token" (similar ao WETH no Ethereum).
    // Permite converter o token nativo da blockchain em um token PSP22 e vice-versa.
    // 
    // ## Funcionalidades Principais:
    // - **Deposit**: Recebe token nativo e emite WNATIVE tokens equivalentes
    // - **Withdraw**: Queima WNATIVE tokens e envia token nativo de volta
    // - **PSP22-like**: Implementação simplificada de funcionalidades PSP22
    // - **1:1 Backing**: Cada WNATIVE token é garantido por 1 token nativo
    // 
    // ## Segurança:
    // - Validação de balances antes de withdraw
    // - Proteção contra overflow em deposits
    // - Transfer failures são tratados adequadamente
    // - Maintain 1:1 reserve ratio sempre

    // ========================================
    // EVENTOS (PARA INDEXADORES E UIS)
    // ========================================

    /// Emitido quando tokens nativos são depositados e WNATIVE é mintado
    #[ink(event)]
    pub struct Deposit {
        #[ink(topic)]
        pub dst: AccountId,
        /// Quantidade de tokens nativos depositados
        pub wad: Balance,
    }

    /// Emitido quando WNATIVE é queimado e tokens nativos são sacados
    #[ink(event)]
    pub struct Withdrawal {
        #[ink(topic)]
        pub src: AccountId,
        /// Quantidade de tokens nativos sacados
        pub wad: Balance,
    }

    /// Emitido quando transfer acontece (PSP22-like)
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        pub from: Option<AccountId>,
        #[ink(topic)]
        pub to: Option<AccountId>,
        pub value: Balance,
    }

    /// Emitido quando aprovação acontece (PSP22-like)
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        pub owner: AccountId,
        #[ink(topic)]
        pub spender: AccountId,
        pub value: Balance,
    }

    // ========================================
    // ERROS ESPECÍFICOS DO WNATIVE CONTRACT
    // ========================================

    /// Erros que podem ocorrer nas operações do WNative
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum WnativeError {
        /// Balance insuficiente para withdraw ou transfer
        InsufficientBalance,
        /// Allowance insuficiente para transferFrom
        InsufficientAllowance,
        /// Transfer de token nativo falhou
        TransferFailed,
        /// Quantidade zero para deposit/withdraw
        ZeroAmount,
        /// Self transfer não permitido
        SelfTransfer,
        /// Arithmetic overflow
        Overflow,
    }

    // ========================================
    // STORAGE DO WNATIVE CONTRACT
    // ========================================

    /// Storage principal do WNative Contract
    #[ink(storage)]
    pub struct WnativeContract {
        /// Total supply de WNATIVE tokens
        total_supply: Balance,
        /// Balances dos usuários
        balances: Mapping<AccountId, Balance>,
        /// Allowances para transferFrom
        allowances: Mapping<(AccountId, AccountId), Balance>,
        /// Token metadata
        name: Option<String>,
        symbol: Option<String>,
        decimals: u8,
    }

    impl WnativeContract {
        /// Construtor do WNative Contract
        #[ink(constructor)]
        pub fn new(name: Option<String>, symbol: Option<String>, decimals: u8) -> Self {
            Self {
                total_supply: 0,
                balances: Mapping::new(),
                allowances: Mapping::new(),
                name,
                symbol,
                decimals,
            }
        }

        // ========================================
        // PSP22-LIKE QUERIES (READ-ONLY)
        // ========================================

        /// Retorna o total supply de tokens
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Retorna o balance de um endereço
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or(0)
        }

        /// Retorna a allowance entre owner e spender
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or(0)
        }

        /// Retorna o nome do token
        #[ink(message)]
        pub fn token_name(&self) -> Option<String> {
            self.name.clone()
        }

        /// Retorna o símbolo do token
        #[ink(message)]
        pub fn token_symbol(&self) -> Option<String> {
            self.symbol.clone()
        }

        /// Retorna os decimais do token
        #[ink(message)]
        pub fn token_decimals(&self) -> u8 {
            self.decimals
        }

        // ========================================
        // PSP22-LIKE OPERATIONS
        // ========================================

        /// Transfer tokens para outro endereço
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<(), WnativeError> {
            let from = self.env().caller();
            self._transfer(from, to, value)
        }

        /// Transfer tokens de from para to (requer allowance)
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<(), WnativeError> {
            let spender = self.env().caller();
            
            // Verificar allowance
            let current_allowance = self.allowance(from, spender);
            if current_allowance < value {
                return Err(WnativeError::InsufficientAllowance);
            }
            
            // Decrementar allowance
            let new_allowance = current_allowance.checked_sub(value).ok_or(WnativeError::InsufficientAllowance)?;
            self.allowances.insert((from, spender), &new_allowance);
            
            // Fazer transfer
            self._transfer(from, to, value)
        }

        /// Aprovar spender para gastar tokens
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), WnativeError> {
            let owner = self.env().caller();
            
            if owner == spender {
                return Err(WnativeError::SelfTransfer);
            }
            
            self.allowances.insert((owner, spender), &value);
            
            // Emitir evento
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            
            Ok(())
        }

        // ========================================
        // OPERAÇÕES PRINCIPAIS (WRAP/UNWRAP)
        // ========================================

        /// Deposita tokens nativos e recebe WNATIVE tokens
        /// 
        /// A quantidade de tokens nativos enviados na transação (transferred_value)
        /// será convertida 1:1 em WNATIVE tokens para o caller.
        #[ink(message, payable)]
        pub fn deposit(&mut self) -> Result<(), WnativeError> {
            let caller = self.env().caller();
            let amount = self.env().transferred_value();
            
            // Validações
            if amount == 0 {
                return Err(WnativeError::ZeroAmount);
            }
            
            // Mint WNATIVE tokens 1:1 com native tokens depositados
            self._mint(caller, amount)?;
            
            // Emitir evento
            self.env().emit_event(Deposit {
                dst: caller,
                wad: amount,
            });
            
            Ok(())
        }

        /// Queima WNATIVE tokens e envia tokens nativos de volta
        /// 
        /// # Parâmetros
        /// - `amount`: Quantidade de WNATIVE tokens a queimar
        #[ink(message)]
        pub fn withdraw(&mut self, amount: Balance) -> Result<(), WnativeError> {
            let caller = self.env().caller();
            
            // Validações
            if amount == 0 {
                return Err(WnativeError::ZeroAmount);
            }
            
            // Verificar se o caller tem balance suficiente
            if self.balance_of(caller) < amount {
                return Err(WnativeError::InsufficientBalance);
            }
            
            // Queimar WNATIVE tokens primeiro
            self._burn(caller, amount)?;
            
            // Transferir tokens nativos de volta
            self.env()
                .transfer(caller, amount)
                .map_err(|_| WnativeError::TransferFailed)?;
            
            // Emitir evento
            self.env().emit_event(Withdrawal {
                src: caller,
                wad: amount,
            });
            
            Ok(())
        }

        // ========================================
        // QUERIES AUXILIARES
        // ========================================

        /// Retorna o balance de tokens nativos do contrato
        /// (deve ser igual ao total_supply de WNATIVE tokens)
        #[ink(message)]
        pub fn native_balance(&self) -> Balance {
            self.env().balance()
        }

        /// Verifica se o contrato está "saudável" (1:1 backing)
        #[ink(message)]
        pub fn is_healthy(&self) -> bool {
            // Em um contrato saudável: native_balance >= total_supply
            // (pode ser > por conta de donations acidentais)
            self.env().balance() >= self.total_supply
        }

        // ========================================
        // FUNÇÕES INTERNAS
        // ========================================

        /// Transfer interno entre endereços
        fn _transfer(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<(), WnativeError> {
            if from == to {
                return Err(WnativeError::SelfTransfer);
            }
            
            if value == 0 {
                return Ok(()); // Transfer de 0 é válido mas não faz nada
            }
            
            // Verificar balance do from
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(WnativeError::InsufficientBalance);
            }
            
            // Atualizar balances
            let new_from_balance = from_balance.checked_sub(value).ok_or(WnativeError::InsufficientBalance)?;
            self.balances.insert(from, &new_from_balance);
            let to_balance = self.balance_of(to);
            let new_to_balance = to_balance.checked_add(value).ok_or(WnativeError::Overflow)?;
            self.balances.insert(to, &new_to_balance);
            
            // Emitir evento
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });
            
            Ok(())
        }

        /// Mint tokens para um endereço
        fn _mint(&mut self, to: AccountId, value: Balance) -> Result<(), WnativeError> {
            if value == 0 {
                return Ok(());
            }
            
            // Atualizar total supply
            self.total_supply = self.total_supply.checked_add(value).ok_or(WnativeError::Overflow)?;
            
            // Atualizar balance
            let to_balance = self.balance_of(to);
            let new_to_balance = to_balance.checked_add(value).ok_or(WnativeError::Overflow)?;
            self.balances.insert(to, &new_to_balance);
            
            // Emitir evento
            self.env().emit_event(Transfer {
                from: None,
                to: Some(to),
                value,
            });
            
            Ok(())
        }

        /// Burn tokens de um endereço
        fn _burn(&mut self, from: AccountId, value: Balance) -> Result<(), WnativeError> {
            if value == 0 {
                return Ok(());
            }
            
            // Atualizar total supply
            self.total_supply = self.total_supply.checked_sub(value).ok_or(WnativeError::InsufficientBalance)?;
            
            // Atualizar balance
            let from_balance = self.balance_of(from);
            let new_from_balance = from_balance.checked_sub(value).ok_or(WnativeError::InsufficientBalance)?;
            self.balances.insert(from, &new_from_balance);
            
            // Emitir evento
            self.env().emit_event(Transfer {
                from: Some(from),
                to: None,
                value,
            });
            
            Ok(())
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

        fn set_balance(account: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<DefaultEnvironment>(account, balance);
        }

        fn set_value_transferred(value: Balance) {
            ink::env::test::set_value_transferred::<DefaultEnvironment>(value);
        }

        // ========================================
        // TESTES BÁSICOS DE INICIALIZAÇÃO
        // ========================================

        #[ink::test]
        fn test_new_wnative_initializes_correctly() {
            let wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // GREEN: Metadata deve estar configurado corretamente
            assert_eq!(wnative.token_name(), Some("Wrapped Native".to_string()));
            assert_eq!(wnative.token_symbol(), Some("WNATIVE".to_string()));
            assert_eq!(wnative.token_decimals(), 18);
            
            // GREEN: Supply inicial deve ser zero
            assert_eq!(wnative.total_supply(), 0);
        }

        // ========================================
        // TESTES DE DEPOSIT
        // ========================================

        #[ink::test]
        fn test_deposit_success() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(accounts.alice, 1_000_000); // Minimum balance requirement
            set_value_transferred(100); // Alice envia 100 tokens nativos
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // GREEN: Deposit deve funcionar
            let result = wnative.deposit();
            assert!(result.is_ok());
            
            // GREEN: Alice deve ter 100 WNATIVE tokens
            assert_eq!(wnative.balance_of(accounts.alice), 100);
            
            // GREEN: Total supply deve ser 100
            assert_eq!(wnative.total_supply(), 100);
        }

        #[ink::test]
        fn test_deposit_zero_amount() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_value_transferred(0); // Zero tokens enviados
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // RED: Deposit com zero amount deve falhar
            let result = wnative.deposit();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), WnativeError::ZeroAmount);
        }

        #[ink::test]
        fn test_multiple_deposits() {
            let accounts = default_accounts();
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // GREEN: Alice deposita 100
            set_sender(accounts.alice);
            set_value_transferred(100);
            assert!(wnative.deposit().is_ok());
            
            // GREEN: Bob deposita 200
            set_sender(accounts.bob);
            set_value_transferred(200);
            assert!(wnative.deposit().is_ok());
            
            // GREEN: Verificar balances
            assert_eq!(wnative.balance_of(accounts.alice), 100);
            assert_eq!(wnative.balance_of(accounts.bob), 200);
            assert_eq!(wnative.total_supply(), 300);
        }

        // ========================================
        // TESTES DE WITHDRAW
        // ========================================

        #[ink::test]
        fn test_withdraw_success() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(accounts.alice, 1_000_000); // Minimum balance requirement
            set_value_transferred(200);
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // Setup: Alice deposita primeiro
            assert!(wnative.deposit().is_ok());
            assert_eq!(wnative.balance_of(accounts.alice), 200);
            
            // GREEN: Withdraw deve funcionar
            let result = wnative.withdraw(100);
            assert!(result.is_ok());
            
            // GREEN: Alice deve ter 100 WNATIVE tokens restantes
            assert_eq!(wnative.balance_of(accounts.alice), 100);
            
            // GREEN: Total supply deve ser 100
            assert_eq!(wnative.total_supply(), 100);
        }

        #[ink::test]
        fn test_withdraw_zero_amount() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // RED: Withdraw com zero amount deve falhar
            let result = wnative.withdraw(0);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), WnativeError::ZeroAmount);
        }

        #[ink::test]
        fn test_withdraw_insufficient_balance() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_value_transferred(100);
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // Setup: Alice deposita 100
            assert!(wnative.deposit().is_ok());
            
            // RED: Tentar withdraw mais do que tem deve falhar
            let result = wnative.withdraw(200);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), WnativeError::InsufficientBalance);
        }

        // ========================================
        // TESTES DE PSP22-LIKE FUNCTIONS
        // ========================================

        #[ink::test]
        fn test_transfer_success() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_value_transferred(100);
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // Setup: Alice deposita 100
            assert!(wnative.deposit().is_ok());
            
            // GREEN: Transfer deve funcionar
            let result = wnative.transfer(accounts.bob, 50);
            assert!(result.is_ok());
            
            // GREEN: Verificar balances
            assert_eq!(wnative.balance_of(accounts.alice), 50);
            assert_eq!(wnative.balance_of(accounts.bob), 50);
        }

        #[ink::test]
        fn test_transfer_insufficient_balance() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_value_transferred(100);
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // Setup: Alice deposita 100
            assert!(wnative.deposit().is_ok());
            
            // RED: Transfer mais do que tem deve falhar
            let result = wnative.transfer(accounts.bob, 150);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), WnativeError::InsufficientBalance);
        }

        #[ink::test]
        fn test_approve_and_transfer_from() {
            let accounts = default_accounts();
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // Setup: Alice deposita 100
            set_sender(accounts.alice);
            set_value_transferred(100);
            assert!(wnative.deposit().is_ok());
            
            // GREEN: Alice aprova Bob para gastar 50
            let result = wnative.approve(accounts.bob, 50);
            assert!(result.is_ok());
            assert_eq!(wnative.allowance(accounts.alice, accounts.bob), 50);
            
            // GREEN: Bob transfere 30 de Alice para Charlie
            set_sender(accounts.bob);
            let result = wnative.transfer_from(accounts.alice, accounts.charlie, 30);
            assert!(result.is_ok());
            
            // GREEN: Verificar balances e allowance
            assert_eq!(wnative.balance_of(accounts.alice), 70);
            assert_eq!(wnative.balance_of(accounts.charlie), 30);
            assert_eq!(wnative.allowance(accounts.alice, accounts.bob), 20); // 50 - 30
        }

        #[ink::test]
        fn test_transfer_from_insufficient_allowance() {
            let accounts = default_accounts();
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // Setup: Alice deposita 100
            set_sender(accounts.alice);
            set_value_transferred(100);
            assert!(wnative.deposit().is_ok());
            
            // Setup: Alice aprova Bob para gastar apenas 30
            assert!(wnative.approve(accounts.bob, 30).is_ok());
            
            // RED: Bob tenta transferir mais do que tem allowance
            set_sender(accounts.bob);
            let result = wnative.transfer_from(accounts.alice, accounts.charlie, 50);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), WnativeError::InsufficientAllowance);
        }

        // ========================================
        // TESTE DE CICLO COMPLETO
        // ========================================

        #[ink::test]
        fn test_full_cycle_deposit_transfer_withdraw() {
            let accounts = default_accounts();
            
            let mut wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // GREEN: Alice deposita 500
            set_sender(accounts.alice);
            set_value_transferred(500);
            assert!(wnative.deposit().is_ok());
            
            // GREEN: Alice transfere 200 para Bob
            assert!(wnative.transfer(accounts.bob, 200).is_ok());
            assert_eq!(wnative.balance_of(accounts.alice), 300);
            assert_eq!(wnative.balance_of(accounts.bob), 200);
            
            // GREEN: Bob withdraws 100
            set_sender(accounts.bob);
            assert!(wnative.withdraw(100).is_ok());
            assert_eq!(wnative.balance_of(accounts.bob), 100);
            assert_eq!(wnative.total_supply(), 400);
            
            // GREEN: Alice withdraws restante
            set_sender(accounts.alice);
            assert!(wnative.withdraw(300).is_ok());
            assert_eq!(wnative.balance_of(accounts.alice), 0);
            assert_eq!(wnative.total_supply(), 100); // Só Bob restante
        }

        // ========================================
        // TESTES DE HEALTH CHECK
        // ========================================

        #[ink::test]
        fn test_is_healthy() {
            let _accounts = default_accounts();
            
            let wnative = WnativeContract::new(
                Some("Wrapped Native".to_string()),
                Some("WNATIVE".to_string()),
                18,
            );
            
            // GREEN: Contrato vazio deve ser saudável
            assert!(wnative.is_healthy());
            
            // GREEN: Deve retornar balance nativo (pode ser > 0 no ambiente de teste)
            let _native_balance = wnative.native_balance();
            // native_balance é sempre >= 0 por ser Balance (unsigned)
        }
    }
}
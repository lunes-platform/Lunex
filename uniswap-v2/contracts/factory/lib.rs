#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod factory {
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

    // Eventos do contrato
    #[ink(event)]
    pub struct PairCreated {
        #[ink(topic)]
        pub token_0: AccountId,
        #[ink(topic)]
        pub token_1: AccountId,
        pub pair: AccountId,
        pub length: u64,
    }

    // Erros personalizados com documentação detalhada
    /// Erros que podem ocorrer nas operações da Factory
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum FactoryError {
        /// Tentativa de criar par com tokens idênticos
        IdenticalAddresses,
        /// Par já existe para estes tokens
        PairExists,
        /// Chamador não é o fee_to_setter autorizado
        CallerIsNotFeeSetter,
        /// Endereço zero não é permitido para tokens
        ZeroAddress,
    }

    /// Constantes do contrato
    mod constants {
        /// Endereço zero (usado para validações)
        pub const ZERO_ADDRESS: [u8; 32] = [0u8; 32];
    }

    /// Estrutura principal do contrato
    #[ink(storage)]
    pub struct FactoryContract {
        /// Fee recipient (rarely accessed - optimized with Lazy)
        fee_to: ink::storage::Lazy<AccountId>,
        /// Fee setter (admin field, rarely accessed - optimized with Lazy)
        fee_to_setter: AccountId,
        /// Token pair mapping (frequently accessed - kept as direct Mapping)
        get_pair: Mapping<(AccountId, AccountId), AccountId>,
        /// All pairs list (frequently accessed for indexing - kept as Vec)
        all_pairs: Vec<AccountId>,
        /// Pair contract code hash (rarely accessed - optimized with Lazy)
        pair_contract_code_hash: ink::storage::Lazy<Hash>,
    }

    impl Default for FactoryContract {
        fn default() -> Self {
            Self {
                fee_to: ink::storage::Lazy::new(),
                fee_to_setter: AccountId::from([0u8; 32]),
                get_pair: Mapping::default(),
                all_pairs: Vec::new(),
                pair_contract_code_hash: ink::storage::Lazy::new(),
            }
        }
    }

    impl FactoryContract {
        /// Constructor do contrato
        /// 
        /// # Parâmetros
        /// * `fee_to_setter` - Endereço autorizado a definir o fee_to
        /// * `pair_code_hash` - Hash do código dos contratos de par
        /// 
        /// # Validações
        /// * fee_to_setter não pode ser endereço zero
        #[ink(constructor)]
        pub fn new(fee_to_setter: AccountId, pair_code_hash: Hash) -> Self {
            // Validação defensiva no constructor
            assert!(
                fee_to_setter != AccountId::from(constants::ZERO_ADDRESS),
                "fee_to_setter cannot be zero address"
            );
            
            let mut instance = Self {
                fee_to: ink::storage::Lazy::new(),
                fee_to_setter,
                get_pair: Mapping::default(),
                all_pairs: Vec::new(),
                pair_contract_code_hash: ink::storage::Lazy::new(),
            };
            
            // Initialize Lazy fields for gas optimization
            instance.fee_to.set(&AccountId::from(constants::ZERO_ADDRESS));
            instance.pair_contract_code_hash.set(&pair_code_hash);
            
            instance
        }

        // ========================================
        // FUNÇÕES INTERNAS (LÓGICA MODULARIZADA)
        // ========================================

        /// Valida que o chamador é o fee_to_setter autorizado
        fn ensure_caller_is_fee_setter(&self) -> Result<(), FactoryError> {
            if self.env().caller() != self.fee_to_setter {
                return Err(FactoryError::CallerIsNotFeeSetter);
            }
            Ok(())
        }

        /// Valida entrada para criação de par
        fn validate_pair_creation(&self, token_a: AccountId, token_b: AccountId) -> Result<(), FactoryError> {
            // Fail fast: tokens não podem ser endereço zero (verificar primeiro)
            if token_a == AccountId::from(constants::ZERO_ADDRESS) || 
               token_b == AccountId::from(constants::ZERO_ADDRESS) {
                return Err(FactoryError::ZeroAddress);
            }

            // Fail fast: tokens não podem ser idênticos
            if token_a == token_b {
                return Err(FactoryError::IdenticalAddresses);
            }

            Ok(())
        }

        /// Ordena tokens para garantir consistência (token_0 < token_1)
        fn sort_tokens(&self, token_a: AccountId, token_b: AccountId) -> (AccountId, AccountId) {
            if token_a < token_b {
                (token_a, token_b)
            } else {
                (token_b, token_a)
            }
        }

        /// Gera endereço determinístico do par usando hash dos tokens
        fn generate_pair_address(&self, token_0: AccountId, token_1: AccountId) -> AccountId {
            let mut salt_input = Vec::new();
            salt_input.extend_from_slice(token_0.as_ref());
            salt_input.extend_from_slice(token_1.as_ref());
            
            let mut output = <ink::env::hash::Blake2x256 as ink::env::hash::HashOutput>::Type::default();
            ink::env::hash_bytes::<ink::env::hash::Blake2x256>(&salt_input, &mut output);
            AccountId::from(output)
        }

        /// Registra par nos mappings bidirecionais
        fn register_pair(&mut self, token_0: AccountId, token_1: AccountId, pair_address: AccountId) {
            self.get_pair.insert((token_0, token_1), &pair_address);
            self.get_pair.insert((token_1, token_0), &pair_address);
            self.all_pairs.push(pair_address);
        }

        // ========================================
        // FUNÇÕES PÚBLICAS (INTERFACE)
        // ========================================

        /// Retorna endereço do par no índice especificado
        /// 
        /// # Parâmetros
        /// * `pid` - Índice do par (0-based)
        /// 
        /// # Retorna
        /// * `Some(AccountId)` - Endereço do par se existir
        /// * `None` - Se índice inválido
        #[ink(message)]
        pub fn all_pairs(&self, pid: u64) -> Option<AccountId> {
            usize::try_from(pid)
                .ok()
                .and_then(|index| self.all_pairs.get(index).copied())
        }

        /// Retorna quantidade total de pares criados
        /// 
        /// # Retorna
        /// * `u64` - Número total de pares registrados na factory
        #[ink(message)]
        pub fn all_pairs_length(&self) -> u64 {
            self.all_pairs.len() as u64
        }

        /// Retorna endereço atual do fee_to (receptor de taxas)
        /// 
        /// # Retorna
        /// * `AccountId` - Endereço que recebe taxas ou endereço zero se desabilitado
        #[ink(message)]
        pub fn fee_to(&self) -> AccountId {
            self.fee_to.get().unwrap_or(AccountId::from(constants::ZERO_ADDRESS))
        }

        /// Retorna endereço do fee_to_setter (autorizado a definir taxas)
        /// 
        /// # Retorna
        /// * `AccountId` - Endereço autorizado a modificar configurações de taxa
        #[ink(message)]
        pub fn fee_to_setter(&self) -> AccountId {
            self.fee_to_setter
        }

        /// Retorna endereço do par para dois tokens (ordenação automática)
        /// 
        /// # Parâmetros
        /// * `token_a` - Primeiro token
        /// * `token_b` - Segundo token
        /// 
        /// # Retorna
        /// * `Some(AccountId)` - Endereço do par se existir
        /// * `None` - Se par não foi criado ainda
        #[ink(message)]
        pub fn get_pair(&self, token_a: AccountId, token_b: AccountId) -> Option<AccountId> {
            self.get_pair.get((token_a, token_b))
        }

        /// Retorna hash do código dos contratos de par
        /// 
        /// # Retorna
        /// * `Hash` - Hash usado para deterministic deployment dos pares
        #[ink(message)]
        pub fn pair_contract_code_hash(&self) -> Hash {
            self.pair_contract_code_hash.get().unwrap_or(Hash::default())
        }

        /// Cria um novo par de tokens
        /// 
        /// # Parâmetros
        /// * `token_a` - Primeiro token do par
        /// * `token_b` - Segundo token do par
        /// 
        /// # Retorna
        /// * `Ok(AccountId)` - Endereço do novo par criado
        /// * `Err(FactoryError)` - Erro específico da operação
        /// 
        /// # Validações
        /// * Tokens não podem ser idênticos
        /// * Tokens não podem ser endereço zero
        /// * Par não pode já existir
        #[ink(message)]
        pub fn create_pair(
            &mut self,
            token_a: AccountId,
            token_b: AccountId,
        ) -> Result<AccountId, FactoryError> {
            // Fail fast: validações de entrada
            self.validate_pair_creation(token_a, token_b)?;

            // Ordenar tokens para consistência
            let (token_0, token_1) = self.sort_tokens(token_a, token_b);

            // Fail fast: verificar se par já existe
            if self.get_pair.get((token_0, token_1)).is_some() {
                return Err(FactoryError::PairExists);
            }

            // Gerar endereço determinístico do par
            let pair_address = self.generate_pair_address(token_0, token_1);

            // Registrar o novo par
            self.register_pair(token_0, token_1, pair_address);

            // Emitir evento para indexadores/UIs
            self.env().emit_event(PairCreated {
                token_0,
                token_1,
                pair: pair_address,
                length: self.all_pairs.len() as u64,
            });

            Ok(pair_address)
        }

        /// Define novo endereço fee_to
        /// 
        /// # Parâmetros
        /// * `fee_to` - Novo endereço que receberá as taxas
        /// 
        /// # Controle de Acesso
        /// * Apenas o fee_to_setter pode chamar esta função
        #[ink(message)]
        pub fn set_fee_to(&mut self, fee_to: AccountId) -> Result<(), FactoryError> {
            // Validação de acesso centralizada
            self.ensure_caller_is_fee_setter()?;
            
            self.fee_to.set(&fee_to);
            Ok(())
        }

        /// Define novo endereço fee_to_setter (transferência de propriedade)
        /// 
        /// # Parâmetros
        /// * `fee_to_setter` - Novo endereço autorizado a definir taxas
        /// 
        /// # Controle de Acesso
        /// * Apenas o fee_to_setter atual pode chamar esta função
        /// 
        /// # Validações
        /// * Novo fee_to_setter não pode ser endereço zero
        #[ink(message)]
        pub fn set_fee_to_setter(&mut self, fee_to_setter: AccountId) -> Result<(), FactoryError> {
            // Validação de acesso centralizada
            self.ensure_caller_is_fee_setter()?;
            
            // Validação defensiva: não permitir endereço zero
            if fee_to_setter == AccountId::from(constants::ZERO_ADDRESS) {
                return Err(FactoryError::ZeroAddress);
            }
            
            self.fee_to_setter = fee_to_setter;
            Ok(())
        }
    }

    /// Testes unitários
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        fn set_sender(sender: AccountId) {
            test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }

        #[ink::test]
        fn test_new_factory_initializes_correctly() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            
            let factory = FactoryContract::new(accounts.bob, Hash::default());
            
            assert_eq!(factory.fee_to_setter(), accounts.bob);
            assert_eq!(factory.pair_contract_code_hash(), Hash::default());
            assert_eq!(factory.all_pairs_length(), 0);
        }

        #[ink::test]  
        fn test_create_pair_with_valid_tokens() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            
            let mut factory = FactoryContract::new(accounts.bob, Hash::default());
            let token_a = accounts.charlie;
            let token_b = accounts.django;
            
            // RED: Este teste deve falhar inicialmente se create_pair não estiver implementado corretamente
            let result = factory.create_pair(token_a, token_b);
            
            // GREEN: Verificar se o par foi criado com sucesso
            assert!(result.is_ok());
            let pair_address = result.unwrap();
            
            // Verificar se o par foi registrado
            assert_eq!(factory.all_pairs_length(), 1);
            assert_eq!(factory.get_pair(token_a, token_b), Some(pair_address));
            assert_eq!(factory.get_pair(token_b, token_a), Some(pair_address));
        }

        #[ink::test]
        fn test_create_pair_with_identical_tokens_fails() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            
            let mut factory = FactoryContract::new(accounts.bob, Hash::default());
            let token = accounts.charlie;
            
            // RED: Este teste deve passar, verificando que tokens idênticos falham
            let result = factory.create_pair(token, token);
            
            // GREEN: Deve falhar com IdenticalAddresses
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), FactoryError::IdenticalAddresses);
        }

        #[ink::test]
        fn test_create_pair_duplicate_fails() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            
            let mut factory = FactoryContract::new(accounts.bob, Hash::default());
            let token_a = accounts.charlie;
            let token_b = accounts.django;
            
            // Criar o primeiro par
            let result1 = factory.create_pair(token_a, token_b);
            assert!(result1.is_ok());
            
            // RED: Tentar criar o mesmo par novamente deve falhar
            let result2 = factory.create_pair(token_a, token_b);
            assert!(result2.is_err());
            assert_eq!(result2.unwrap_err(), FactoryError::PairExists);
            
            // GREEN: Também deve falhar na ordem inversa
            let result3 = factory.create_pair(token_b, token_a);
            assert!(result3.is_err());
            assert_eq!(result3.unwrap_err(), FactoryError::PairExists);
        }

        #[ink::test]
        fn test_set_fee_to_only_by_setter() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            
            let mut factory = FactoryContract::new(accounts.bob, Hash::default());
            
            // RED: Alice não é fee_to_setter, deve falhar
            let result = factory.set_fee_to(accounts.charlie);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), FactoryError::CallerIsNotFeeSetter);
            
            // GREEN: Bob é fee_to_setter, deve passar
            set_sender(accounts.bob);
            let result = factory.set_fee_to(accounts.charlie);
            assert!(result.is_ok());
            assert_eq!(factory.fee_to(), accounts.charlie);
        }

        #[ink::test]
        fn test_set_fee_to_setter_only_by_current_setter() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            
            let mut factory = FactoryContract::new(accounts.bob, Hash::default());
            
            // RED: Alice não é fee_to_setter atual, deve falhar
            let result = factory.set_fee_to_setter(accounts.charlie);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), FactoryError::CallerIsNotFeeSetter);
            
            // GREEN: Bob é fee_to_setter atual, deve passar
            set_sender(accounts.bob);
            let result = factory.set_fee_to_setter(accounts.charlie);
            assert!(result.is_ok());
            assert_eq!(factory.fee_to_setter(), accounts.charlie);
        }

        // ========================================
        // TESTES ADICIONAIS PARA NOVAS VALIDAÇÕES
        // ========================================

        #[ink::test]
        fn test_create_pair_with_zero_address_fails() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            
            let mut factory = FactoryContract::new(accounts.bob, Hash::default());
            let zero_address = AccountId::from([0u8; 32]);
            let valid_token = accounts.charlie;
            
            // RED: token_a é endereço zero, deve falhar
            let result = factory.create_pair(zero_address, valid_token);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), FactoryError::ZeroAddress);
            
            // RED: token_b é endereço zero, deve falhar
            let result = factory.create_pair(valid_token, zero_address);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), FactoryError::ZeroAddress);
            
            // RED: ambos tokens são endereço zero, deve falhar
            let result = factory.create_pair(zero_address, zero_address);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), FactoryError::ZeroAddress);
        }

        #[ink::test]
        fn test_set_fee_to_setter_zero_address_fails() {
            let accounts = default_accounts();
            set_sender(accounts.bob);
            
            let mut factory = FactoryContract::new(accounts.bob, Hash::default());
            let zero_address = AccountId::from([0u8; 32]);
            
            // RED: Tentar definir fee_to_setter como endereço zero deve falhar
            let result = factory.set_fee_to_setter(zero_address);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), FactoryError::ZeroAddress);
            
            // GREEN: O fee_to_setter original deve permanecer inalterado
            assert_eq!(factory.fee_to_setter(), accounts.bob);
        }

        #[ink::test]
        #[should_panic(expected = "fee_to_setter cannot be zero address")]
        fn test_constructor_with_zero_fee_to_setter_panics() {
            let zero_address = AccountId::from([0u8; 32]);
            
            // RED: Constructor com fee_to_setter zero deve causar panic
            let _factory = FactoryContract::new(zero_address, Hash::default());
        }

        #[ink::test]
        fn test_pair_address_deterministic_and_token_order() {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            
            let mut factory = FactoryContract::new(accounts.bob, Hash::default());
            
            let token_a = accounts.charlie;
            let token_b = accounts.django;
            let token_c = accounts.eve;
            
            // GREEN: Primeiro par criado normalmente
            let pair_ab = factory.create_pair(token_a, token_b).unwrap();
            
            // GREEN: Verificar que get_pair funciona em ambas as direções
            assert_eq!(factory.get_pair(token_a, token_b), Some(pair_ab));
            assert_eq!(factory.get_pair(token_b, token_a), Some(pair_ab));
            
            // GREEN: Segundo par com tokens diferentes deve ter endereço diferente
            let pair_ac = factory.create_pair(token_a, token_c).unwrap();
            
            assert_ne!(pair_ab, pair_ac, "Pares diferentes devem ter endereços diferentes");
            
            // GREEN: Verificar que ambos os pares estão registrados corretamente
            assert_eq!(factory.all_pairs_length(), 2);
            assert_eq!(factory.all_pairs(0), Some(pair_ab));
            assert_eq!(factory.all_pairs(1), Some(pair_ac));
        }
    }
}
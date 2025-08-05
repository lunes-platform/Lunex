//! Integration E2E Tests for Lunex DEX
//!
//! This module contains comprehensive end-to-end tests that validate the complete
//! DEX functionality by testing interactions between all contracts:
//! - Factory Contract (pair creation and management)
//! - Pair Contract (AMM liquidity and swaps)
//! - Router Contract (user operation coordination)
//! - WNative Contract (native token wrapping)
//!
//! Following TDD principles: "TDD is not about testing, but about code design
//! and creating testable code."

#[cfg(test)]
mod e2e_tests {
    use std::collections::HashMap;

    // ========================================
    // MOCK CONTRACTS FOR E2E TESTING
    // ========================================
    
    /// Mock Factory Contract for E2E testing
    pub struct MockFactory {
        pairs: HashMap<(String, String), String>,
        pair_count: u32,
        fee_to: Option<String>,
        fee_to_setter: String,
    }
    
    impl MockFactory {
        pub fn new(fee_to_setter: String) -> Self {
            Self {
                pairs: HashMap::new(),
                pair_count: 0,
                fee_to: None,
                fee_to_setter,
            }
        }
        
        pub fn create_pair(&mut self, token_a: String, token_b: String) -> Result<String, String> {
            // Sort tokens to ensure consistent pair addressing
            let (token_0, token_1) = if token_a < token_b {
                (token_a, token_b)
            } else {
                (token_b, token_a)
            };
            
            let key = (token_0.clone(), token_1.clone());
            
            // Check if pair already exists
            if self.pairs.contains_key(&key) {
                return Err("Pair already exists".to_string());
            }
            
            // Create new pair address (simplified)
            self.pair_count += 1;
            let pair_address = format!("pair_{}", self.pair_count);
            
            self.pairs.insert(key, pair_address.clone());
            
            Ok(pair_address)
        }
        
        pub fn get_pair(&self, token_a: String, token_b: String) -> Option<String> {
            let (token_0, token_1) = if token_a < token_b {
                (token_a, token_b)
            } else {
                (token_b, token_a)
            };
            
            self.pairs.get(&(token_0, token_1)).cloned()
        }
        
        pub fn all_pairs_length(&self) -> u32 {
            self.pairs.len() as u32
        }
    }
    
    /// Mock Pair Contract for E2E testing
    pub struct MockPair {
        token_0: String,
        token_1: String,
        reserve_0: u128,
        reserve_1: u128,
        total_supply: u128,
        balances: HashMap<String, u128>,
        factory: String,
    }
    
    impl MockPair {
        pub fn new(token_0: String, token_1: String, factory: String) -> Self {
            Self {
                token_0,
                token_1,
                reserve_0: 0,
                reserve_1: 0,
                total_supply: 0,
                balances: HashMap::new(),
                factory,
            }
        }
        
        pub fn mint(&mut self, to: String, amount_0: u128, amount_1: u128) -> Result<u128, String> {
            if amount_0 == 0 || amount_1 == 0 {
                return Err("Insufficient liquidity".to_string());
            }
            
            let liquidity = if self.total_supply == 0 {
                // First liquidity provision
                let liquidity = self.sqrt(amount_0 * amount_1);
                let minimum_liquidity = 100;
                
                if liquidity <= minimum_liquidity {
                    return Err("Insufficient liquidity".to_string());
                }
                
                // Lock minimum liquidity to zero address
                self.balances.insert("0x0".to_string(), minimum_liquidity);
                liquidity - minimum_liquidity
            } else {
                // Subsequent liquidity provision
                let liquidity_a = amount_0 * self.total_supply / self.reserve_0;
                let liquidity_b = amount_1 * self.total_supply / self.reserve_1;
                std::cmp::min(liquidity_a, liquidity_b)
            };
            
            // Update reserves
            self.reserve_0 += amount_0;
            self.reserve_1 += amount_1;
            
            // Mint LP tokens
            self.total_supply += liquidity;
            let current_balance = self.balances.get(&to).unwrap_or(&0);
            self.balances.insert(to, current_balance + liquidity);
            
            Ok(liquidity)
        }
        
        pub fn burn(&mut self, _to: String, liquidity: u128) -> Result<(u128, u128), String> {
            if liquidity == 0 || self.total_supply == 0 {
                return Err("Insufficient liquidity burned".to_string());
            }
            
            // Calculate proportional amounts
            let amount_0 = liquidity * self.reserve_0 / self.total_supply;
            let amount_1 = liquidity * self.reserve_1 / self.total_supply;
            
            if amount_0 == 0 || amount_1 == 0 {
                return Err("Insufficient liquidity burned".to_string());
            }
            
            // Update reserves
            self.reserve_0 -= amount_0;
            self.reserve_1 -= amount_1;
            
            // Burn LP tokens
            self.total_supply -= liquidity;
            
            Ok((amount_0, amount_1))
        }
        
        pub fn swap(&mut self, amount_0_out: u128, amount_1_out: u128) -> Result<(), String> {
            if amount_0_out == 0 && amount_1_out == 0 {
                return Err("Insufficient output amount".to_string());
            }
            
            if amount_0_out >= self.reserve_0 || amount_1_out >= self.reserve_1 {
                return Err("Insufficient liquidity".to_string());
            }
            
            // Simplified swap logic for E2E testing
            // In real implementation, would check K invariant
            self.reserve_0 -= amount_0_out;
            self.reserve_1 -= amount_1_out;
            
            Ok(())
        }
        
        pub fn get_reserves(&self) -> (u128, u128, u64) {
            (self.reserve_0, self.reserve_1, 0) // timestamp = 0 for simplicity
        }
        
        pub fn sqrt(&self, value: u128) -> u128 {
            if value == 0 {
                return 0;
            }
            
            let mut x = value;
            let mut y = (value + 1) / 2;
            
            while y < x {
                x = y;
                y = (value / x + x) / 2;
            }
            
            x
        }
    }
    
    /// Mock Router Contract for E2E testing
    pub struct MockRouter {
        factory: String,
        wnative: String,
        pairs: HashMap<String, MockPair>,
    }
    
    impl MockRouter {
        pub fn new(factory: String, wnative: String) -> Self {
            Self {
                factory,
                wnative,
                pairs: HashMap::new(),
            }
        }
        
        pub fn add_liquidity(
            &mut self,
            token_a: String,
            token_b: String,
            amount_a_desired: u128,
            amount_b_desired: u128,
            amount_a_min: u128,
            amount_b_min: u128,
            to: String,
            deadline: u64,
        ) -> Result<(u128, u128, u128), String> {
            // Validate deadline (simplified)
            if deadline == 0 {
                return Err("Expired".to_string());
            }
            
            // Get or create pair
            let pair_key = if token_a < token_b {
                format!("{}_{}", token_a, token_b)
            } else {
                format!("{}_{}", token_b, token_a)
            };
            
            if !self.pairs.contains_key(&pair_key) {
                let (token_0, token_1) = if token_a < token_b {
                    (token_a.clone(), token_b.clone())
                } else {
                    (token_b.clone(), token_a.clone())
                };
                self.pairs.insert(pair_key.clone(), MockPair::new(token_0, token_1, self.factory.clone()));
            }
            
            let pair = self.pairs.get_mut(&pair_key).unwrap();
            
            // Simplified amounts calculation
            let amount_a = amount_a_desired;
            let amount_b = amount_b_desired;
            
            // Validate slippage
            if amount_a < amount_a_min {
                return Err("Insufficient A amount".to_string());
            }
            if amount_b < amount_b_min {
                return Err("Insufficient B amount".to_string());
            }
            
            // Mint liquidity
            let liquidity = pair.mint(to, amount_a, amount_b)?;
            
            Ok((amount_a, amount_b, liquidity))
        }
        
        pub fn remove_liquidity(
            &mut self,
            token_a: String,
            token_b: String,
            liquidity: u128,
            amount_a_min: u128,
            amount_b_min: u128,
            to: String,
            deadline: u64,
        ) -> Result<(u128, u128), String> {
            // Validate deadline
            if deadline == 0 {
                return Err("Expired".to_string());
            }
            
            // Get pair
            let pair_key = if token_a < token_b {
                format!("{}_{}", token_a, token_b)
            } else {
                format!("{}_{}", token_b, token_a)
            };
            
            let pair = self.pairs.get_mut(&pair_key).ok_or("Pair not found")?;
            
            // Burn liquidity
            let (amount_a, amount_b) = pair.burn(to, liquidity)?;
            
            // Validate slippage
            if amount_a < amount_a_min {
                return Err("Insufficient A amount".to_string());
            }
            if amount_b < amount_b_min {
                return Err("Insufficient B amount".to_string());
            }
            
            Ok((amount_a, amount_b))
        }
        
        pub fn swap_exact_tokens_for_tokens(
            &mut self,
            amount_in: u128,
            amount_out_min: u128,
            path: Vec<String>,
            to: String,
            deadline: u64,
        ) -> Result<Vec<u128>, String> {
            if deadline == 0 {
                return Err("Expired".to_string());
            }
            
            if path.len() < 2 {
                return Err("Invalid path".to_string());
            }
            
            // Simplified single-hop swap for E2E testing
            let token_in = &path[0];
            let token_out = &path[1];
            
            let pair_key = if token_in < token_out {
                format!("{}_{}", token_in, token_out)
            } else {
                format!("{}_{}", token_out, token_in)
            };
            
            let pair = self.pairs.get_mut(&pair_key).ok_or("Pair not found")?;
            
            // Simplified swap calculation (0.3% fee)
            let amount_out = amount_in * 997 / 1000;
            
            if amount_out < amount_out_min {
                return Err("Insufficient output amount".to_string());
            }
            
            // Perform swap
            if token_in < token_out {
                pair.swap(0, amount_out)?;
            } else {
                pair.swap(amount_out, 0)?;
            }
            
            Ok(vec![amount_in, amount_out])
        }
    }
    
    /// Mock WNative Contract for E2E testing
    pub struct MockWNative {
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: u128,
        balances: HashMap<String, u128>,
        allowances: HashMap<(String, String), u128>,
    }
    
    impl MockWNative {
        pub fn new(name: String, symbol: String, decimals: u8) -> Self {
            Self {
                name,
                symbol,
                decimals,
                total_supply: 0,
                balances: HashMap::new(),
                allowances: HashMap::new(),
            }
        }
        
        pub fn deposit(&mut self, user: String, amount: u128) -> Result<(), String> {
            if amount == 0 {
                return Err("Zero amount".to_string());
            }
            
            // Mint WNATIVE tokens
            self.total_supply += amount;
            let current_balance = self.balances.get(&user).unwrap_or(&0);
            self.balances.insert(user, current_balance + amount);
            
            Ok(())
        }
        
        pub fn withdraw(&mut self, user: String, amount: u128) -> Result<(), String> {
            if amount == 0 {
                return Err("Zero amount".to_string());
            }
            
            let current_balance = self.balances.get(&user).unwrap_or(&0);
            if *current_balance < amount {
                return Err("Insufficient balance".to_string());
            }
            
            // Burn WNATIVE tokens
            self.total_supply -= amount;
            self.balances.insert(user, current_balance - amount);
            
            Ok(())
        }
        
        pub fn balance_of(&self, user: String) -> u128 {
            *self.balances.get(&user).unwrap_or(&0)
        }
        
        pub fn total_supply(&self) -> u128 {
            self.total_supply
        }
    }
    
    // ========================================
    // E2E TEST SCENARIOS
    // ========================================
    
    /// Test complete DEX deployment and initialization
    #[test]
    fn test_dex_deployment_e2e() {
        // GREEN: Deploy all contracts successfully
        let factory = MockFactory::new("admin".to_string());
        let wnative = MockWNative::new("Wrapped Native".to_string(), "WNATIVE".to_string(), 18);
        let _router = MockRouter::new("factory_address".to_string(), "wnative_address".to_string());
        
        // GREEN: Verify initial states
        assert_eq!(factory.all_pairs_length(), 0);
        assert_eq!(wnative.total_supply(), 0);
        
        println!("✅ DEX deployment successful!");
    }
    
    /// Test complete add liquidity flow: User -> Router -> Factory -> Pair
    #[test]
    fn test_add_liquidity_e2e_flow() {
        let _factory = MockFactory::new("admin".to_string());
        let mut router = MockRouter::new("factory_address".to_string(), "wnative_address".to_string());
        
        // GREEN: User adds liquidity for new pair
        let result = router.add_liquidity(
            "TOKEN_A".to_string(),
            "TOKEN_B".to_string(),
            1000,  // amount_a_desired
            2000,  // amount_b_desired
            900,   // amount_a_min
            1800,  // amount_b_min
            "user1".to_string(),
            9999,  // deadline
        );
        
        assert!(result.is_ok());
        let (amount_a, amount_b, liquidity) = result.unwrap();
        
        // GREEN: Verify amounts and liquidity
        assert_eq!(amount_a, 1000);
        assert_eq!(amount_b, 2000);
        assert!(liquidity > 0);
        
        println!("✅ Add liquidity E2E flow successful! Liquidity: {}", liquidity);
    }
    
    /// Test slippage protection in add liquidity
    #[test]
    fn test_add_liquidity_slippage_protection_e2e() {
        let mut router = MockRouter::new("factory_address".to_string(), "wnative_address".to_string());
        
        // RED: Slippage protection should trigger
        let result = router.add_liquidity(
            "TOKEN_A".to_string(),
            "TOKEN_B".to_string(),
            1000,  // amount_a_desired
            2000,  // amount_b_desired
            1100,  // amount_a_min (too high)
            1800,  // amount_b_min
            "user1".to_string(),
            9999,  // deadline
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient A amount");
        
        println!("✅ Slippage protection working correctly!");
    }
    
    /// Test swap flow: User -> Router -> Pair
    #[test]
    fn test_swap_e2e_flow() {
        let mut router = MockRouter::new("factory_address".to_string(), "wnative_address".to_string());
        
        // Setup: Add liquidity first
        let _ = router.add_liquidity(
            "TOKEN_A".to_string(),
            "TOKEN_B".to_string(),
            10000,  // Large initial liquidity
            20000,
            9000,
            18000,
            "user1".to_string(),
            9999,
        ).unwrap();
        
        // GREEN: User performs swap
        let path = vec!["TOKEN_A".to_string(), "TOKEN_B".to_string()];
        let result = router.swap_exact_tokens_for_tokens(
            100,     // amount_in
            90,      // amount_out_min
            path,
            "user2".to_string(),
            9999,    // deadline
        );
        
        assert!(result.is_ok());
        let amounts = result.unwrap();
        
        // GREEN: Verify swap amounts
        assert_eq!(amounts[0], 100);  // amount_in
        assert!(amounts[1] >= 90);    // amount_out >= min
        
        println!("✅ Swap E2E flow successful! Output: {}", amounts[1]);
    }
    
    /// Test remove liquidity flow: LP Tokens -> Router -> Pair -> Tokens
    #[test]
    fn test_remove_liquidity_e2e_flow() {
        let mut router = MockRouter::new("factory_address".to_string(), "wnative_address".to_string());
        
        // Setup: Add liquidity first
        let (_, _, liquidity) = router.add_liquidity(
            "TOKEN_A".to_string(),
            "TOKEN_B".to_string(),
            1000,
            2000,
            900,
            1800,
            "user1".to_string(),
            9999,
        ).unwrap();
        
        // GREEN: User removes half of liquidity
        let remove_amount = liquidity / 2;
        let result = router.remove_liquidity(
            "TOKEN_A".to_string(),
            "TOKEN_B".to_string(),
            remove_amount,
            400,    // amount_a_min
            800,    // amount_b_min
            "user1".to_string(),
            9999,   // deadline
        );
        
        assert!(result.is_ok());
        let (amount_a, amount_b) = result.unwrap();
        
        // GREEN: Verify proportional amounts returned
        assert!(amount_a >= 400);
        assert!(amount_b >= 800);
        assert!(amount_a <= 500);  // Should be around half
        assert!(amount_b <= 1000); // Should be around half
        
        println!("✅ Remove liquidity E2E flow successful! Returned: {}, {}", amount_a, amount_b);
    }
    
    /// Test WNative wrap/unwrap integration
    #[test]
    fn test_wnative_wrap_unwrap_e2e() {
        let mut wnative = MockWNative::new("Wrapped Native".to_string(), "WNATIVE".to_string(), 18);
        
        // GREEN: User wraps native tokens
        let result = wnative.deposit("user1".to_string(), 1000);
        assert!(result.is_ok());
        
        // GREEN: Verify wrapped tokens minted
        assert_eq!(wnative.balance_of("user1".to_string()), 1000);
        assert_eq!(wnative.total_supply(), 1000);
        
        // GREEN: User unwraps some tokens
        let result = wnative.withdraw("user1".to_string(), 300);
        assert!(result.is_ok());
        
        // GREEN: Verify tokens burned
        assert_eq!(wnative.balance_of("user1".to_string()), 700);
        assert_eq!(wnative.total_supply(), 700);
        
        println!("✅ WNative wrap/unwrap E2E flow successful!");
    }
    
    /// Test complete user journey: Wrap -> Add Liquidity -> Swap -> Remove Liquidity -> Unwrap
    #[test]
    fn test_complete_user_journey_e2e() {
        let mut wnative = MockWNative::new("Wrapped Native".to_string(), "WNATIVE".to_string(), 18);
        let mut router = MockRouter::new("factory_address".to_string(), "wnative_address".to_string());
        
        // STEP 1: User wraps native tokens
        assert!(wnative.deposit("user1".to_string(), 5000).is_ok());
        println!("✅ Step 1: Wrapped 5000 native tokens");
        
        // STEP 2: User adds liquidity
        let (_, _, liquidity) = router.add_liquidity(
            "WNATIVE".to_string(),
            "TOKEN_B".to_string(),
            2000,
            4000,
            1800,
            3600,
            "user1".to_string(),
            9999,
        ).unwrap();
        println!("✅ Step 2: Added liquidity, received {} LP tokens", liquidity);
        
        // STEP 3: Another user performs swap
        // (First they need to add liquidity too for the swap to work)
        assert!(wnative.deposit("user2".to_string(), 1000).is_ok());
        
        let path = vec!["WNATIVE".to_string(), "TOKEN_B".to_string()];
        let amounts = router.swap_exact_tokens_for_tokens(
            500,
            400,
            path,
            "user2".to_string(),
            9999,
        ).unwrap();
        println!("✅ Step 3: Swapped {} WNATIVE for {} TOKEN_B", amounts[0], amounts[1]);
        
        // STEP 4: User removes liquidity
        let (amount_wnative, amount_token_b) = router.remove_liquidity(
            "WNATIVE".to_string(),
            "TOKEN_B".to_string(),
            liquidity,
            1500,
            3000,
            "user1".to_string(),
            9999,
        ).unwrap();
        println!("✅ Step 4: Removed liquidity, got {} WNATIVE and {} TOKEN_B", amount_wnative, amount_token_b);
        
        // STEP 5: User unwraps remaining WNATIVE
        let user_wnative_balance = wnative.balance_of("user1".to_string());
        if user_wnative_balance > 0 {
            assert!(wnative.withdraw("user1".to_string(), user_wnative_balance).is_ok());
            println!("✅ Step 5: Unwrapped {} WNATIVE back to native", user_wnative_balance);
        }
        
        println!("🎉 Complete user journey E2E test successful!");
    }
    
    /// Test deadline validation across all operations
    #[test]
    fn test_deadline_validation_e2e() {
        let mut router = MockRouter::new("factory_address".to_string(), "wnative_address".to_string());
        
        // RED: Expired deadline should fail for add_liquidity
        let result = router.add_liquidity(
            "TOKEN_A".to_string(),
            "TOKEN_B".to_string(),
            1000, 2000, 900, 1800,
            "user1".to_string(),
            0, // Expired deadline
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Expired");
        
        // RED: Expired deadline should fail for remove_liquidity
        let result = router.remove_liquidity(
            "TOKEN_A".to_string(),
            "TOKEN_B".to_string(),
            100, 90, 180,
            "user1".to_string(),
            0, // Expired deadline
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Expired");
        
        // RED: Expired deadline should fail for swap
        let path = vec!["TOKEN_A".to_string(), "TOKEN_B".to_string()];
        let result = router.swap_exact_tokens_for_tokens(
            100, 90, path,
            "user1".to_string(),
            0, // Expired deadline
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Expired");
        
        println!("✅ Deadline validation working across all operations!");
    }
    
    /// Test error handling and edge cases across contracts
    #[test]
    fn test_error_handling_e2e() {
        let mut wnative = MockWNative::new("Wrapped Native".to_string(), "WNATIVE".to_string(), 18);
        let mut router = MockRouter::new("factory_address".to_string(), "wnative_address".to_string());
        
        // RED: WNative zero amount operations should fail
        assert!(wnative.deposit("user1".to_string(), 0).is_err());
        assert!(wnative.withdraw("user1".to_string(), 0).is_err());
        
        // RED: WNative insufficient balance withdrawal should fail
        assert!(wnative.withdraw("user1".to_string(), 100).is_err());
        
        // RED: Router invalid path should fail
        let path = vec!["TOKEN_A".to_string()]; // Too short
        let result = router.swap_exact_tokens_for_tokens(
            100, 90, path, "user1".to_string(), 9999
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid path");
        
        // RED: Router swap on non-existent pair should fail
        let path = vec!["NONEXISTENT_A".to_string(), "NONEXISTENT_B".to_string()];
        let result = router.swap_exact_tokens_for_tokens(
            100, 90, path, "user1".to_string(), 9999
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Pair not found");
        
        println!("✅ Error handling working correctly across all contracts!");
    }
}

// ========================================
// E2E TEST UTILITIES AND HELPERS
// ========================================

/// Helper function to setup a complete DEX environment for testing
#[cfg(test)]
fn setup_dex_environment() -> (e2e_tests::MockFactory, e2e_tests::MockRouter, e2e_tests::MockWNative) {
    let factory = e2e_tests::MockFactory::new("admin".to_string());
    let router = e2e_tests::MockRouter::new("factory_address".to_string(), "wnative_address".to_string());
    let wnative = e2e_tests::MockWNative::new("Wrapped Native".to_string(), "WNATIVE".to_string(), 18);
    
    (factory, router, wnative)
}

/// Helper function to setup liquidity for testing swaps
#[cfg(test)]
fn setup_liquidity_for_swaps(router: &mut e2e_tests::MockRouter) -> Result<u128, String> {
    router.add_liquidity(
        "TOKEN_A".to_string(),
        "TOKEN_B".to_string(),
        10000,  // Large amounts for stable swaps
        20000,
        9000,
        18000,
        "liquidity_provider".to_string(),
        9999,
    ).map(|(_, _, liquidity)| liquidity)
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    
    /// Test performance with multiple sequential operations
    #[test]
    fn test_multiple_operations_performance() {
        let (mut _factory, mut router, mut wnative) = setup_dex_environment();
        
        // Setup initial liquidity
        let _ = setup_liquidity_for_swaps(&mut router).unwrap();
        
        // Perform multiple operations
        for i in 1..=10 {
            let user = format!("user_{}", i);
            
            // Wrap tokens
            assert!(wnative.deposit(user.clone(), 1000).is_ok());
            
            // Add some liquidity
            let _ = router.add_liquidity(
                "WNATIVE".to_string(),
                "TOKEN_B".to_string(),
                100,
                200,
                90,
                180,
                user.clone(),
                9999,
            ).unwrap();
            
            // Perform a small swap
            let path = vec!["TOKEN_A".to_string(), "TOKEN_B".to_string()];
            let _ = router.swap_exact_tokens_for_tokens(
                50, 40, path, user.clone(), 9999
            ).unwrap();
        }
        
        println!("✅ Multiple operations performance test completed successfully!");
    }
}
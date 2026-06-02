// Unit tests for the oracle contract (Issue #101 - extracted from lib.rs)

#[cfg(test)]
mod oracle_tests {
    use super::*;
    use crate::propchain_oracle::PropertyValuationOracle;
    use ink::env::{test, DefaultEnvironment};

    fn setup_oracle() -> PropertyValuationOracle {
        let accounts = test::default_accounts::<DefaultEnvironment>();
        test::set_caller::<DefaultEnvironment>(accounts.alice);
        PropertyValuationOracle::new(accounts.alice)
    }

    #[ink::test]
    fn test_new_oracle_works() {
        let oracle = setup_oracle();
        assert_eq!(oracle.active_sources.len(), 0);
        assert_eq!(oracle.min_sources_required, 2);
    }

    #[ink::test]
    fn test_add_oracle_source_works() {
        let mut oracle = setup_oracle();
        let accounts = test::default_accounts::<DefaultEnvironment>();

        let source = OracleSource {
            id: "chainlink_feed".to_string(),
            source_type: OracleSourceType::Chainlink,
            address: accounts.bob,
            is_active: true,
            weight: 50,
            last_updated: ink::env::block_timestamp::<DefaultEnvironment>(),
        };

        assert!(oracle.add_oracle_source(source).is_ok());
        assert_eq!(oracle.active_sources.len(), 1);
        assert_eq!(oracle.active_sources[0], "chainlink_feed");
    }

    #[ink::test]
    fn test_unauthorized_add_source_fails() {
        let mut oracle = setup_oracle();
        let accounts = test::default_accounts::<DefaultEnvironment>();

        test::set_caller::<DefaultEnvironment>(accounts.bob);

        let source = OracleSource {
            id: "chainlink_feed".to_string(),
            source_type: OracleSourceType::Chainlink,
            address: accounts.bob,
            is_active: true,
            weight: 50,
            last_updated: ink::env::block_timestamp::<DefaultEnvironment>(),
        };

        assert_eq!(
            oracle.add_oracle_source(source),
            Err(OracleError::Unauthorized)
        );
    }

    #[ink::test]
    fn test_update_property_valuation_works() {
        let mut oracle = setup_oracle();

        let valuation = PropertyValuation {
            property_id: 1,
            valuation: 500000,
            confidence_score: 85,
            sources_used: 3,
            last_updated: ink::env::block_timestamp::<DefaultEnvironment>(),
            valuation_method: ValuationMethod::MarketData,
        };

        assert!(oracle
            .update_property_valuation(1, valuation.clone())
            .is_ok());

        let retrieved = oracle.get_property_valuation(1);
        assert!(retrieved.is_ok());
        assert_eq!(
            retrieved.expect("Valuation should exist after update"),
            valuation
        );
    }

    #[ink::test]
    fn test_get_nonexistent_valuation_fails() {
        let oracle = setup_oracle();
        assert_eq!(
            oracle.get_property_valuation(999),
            Err(OracleError::PropertyNotFound)
        );
    }

    #[ink::test]
    fn test_set_price_alert_works() {
        let mut oracle = setup_oracle();
        let accounts = test::default_accounts::<DefaultEnvironment>();

        assert!(oracle.set_price_alert(1, 5, accounts.bob).is_ok());

        let alerts = oracle.price_alerts.get(&1).unwrap_or_default();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].threshold_percentage, 5);
        assert_eq!(alerts[0].alert_address, accounts.bob);
    }

    #[ink::test]
    fn test_calculate_percentage_change() {
        let oracle = setup_oracle();

        assert_eq!(oracle.calculate_percentage_change(100, 110), 10);
        assert_eq!(oracle.calculate_percentage_change(100, 80), 20);
        assert_eq!(oracle.calculate_percentage_change(100, 100), 0);
        assert_eq!(oracle.calculate_percentage_change(0, 100), 0);
    }

    #[ink::test]
    fn test_aggregate_prices_works() {
        let mut oracle = setup_oracle();
        let accounts = test::default_accounts::<DefaultEnvironment>();

        for (id, weight) in &[("source1", 50u32), ("source2", 50u32), ("source3", 50u32)] {
            oracle
                .add_oracle_source(OracleSource {
                    id: id.to_string(),
                    source_type: OracleSourceType::Manual,
                    address: accounts.bob,
                    is_active: true,
                    weight: *weight,
                    last_updated: ink::env::block_timestamp::<DefaultEnvironment>(),
                })
                .expect("Oracle source registration should succeed in test");
        }

        let prices = vec![
            PriceData {
                price: 100,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source1".to_string(),
            },
            PriceData {
                price: 105,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source2".to_string(),
            },
            PriceData {
                price: 98,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source3".to_string(),
            },
        ];

        let result = oracle.aggregate_prices(&prices);
        assert!(result.is_ok());

        let aggregated = result.expect("Price aggregation should succeed in test");
        assert!((98..=105).contains(&aggregated));
    }

    #[ink::test]
    fn test_filter_outliers_works() {
        let oracle = setup_oracle();

        let prices = vec![
            PriceData {
                price: 98,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source1".to_string(),
            },
            PriceData {
                price: 99,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source2".to_string(),
            },
            PriceData {
                price: 100,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source3".to_string(),
            },
            PriceData {
                price: 101,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source4".to_string(),
            },
            PriceData {
                price: 102,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source5".to_string(),
            },
            PriceData {
                price: 1000,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source6".to_string(),
            },
        ];

        let filtered = oracle.filter_outliers(&prices);
        assert_eq!(filtered.len(), 5);
        assert!(filtered.iter().all(|p| p.price < 200));
    }

    #[ink::test]
    fn test_calculate_confidence_score() {
        let oracle = setup_oracle();

        let prices = vec![
            PriceData {
                price: 100,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source1".to_string(),
            },
            PriceData {
                price: 102,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source2".to_string(),
            },
            PriceData {
                price: 98,
                timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
                source: "source3".to_string(),
            },
        ];

        let score = oracle.calculate_confidence_score(&prices);
        assert!(score.is_ok());

        let score = score.expect("Confidence score calculation should succeed in test");
        assert!(score > 50);
    }

    #[ink::test]
    fn test_set_location_adjustment_works() {
        let mut oracle = setup_oracle();

        let adjustment = LocationAdjustment {
            location_code: "NYC_MANHATTAN".to_string(),
            adjustment_percentage: 15,
            last_updated: ink::env::block_timestamp::<DefaultEnvironment>(),
            confidence_score: 90,
        };

        assert!(oracle.set_location_adjustment(adjustment.clone()).is_ok());

        let stored = oracle.location_adjustments.get(&adjustment.location_code);
        assert!(stored.is_some());
        assert_eq!(
            stored.expect("Location adjustment should exist after setting"),
            adjustment
        );
    }

    #[ink::test]
    fn test_get_comparable_properties_works() {
        let oracle = setup_oracle();

        let comparables = oracle.get_comparable_properties(1, 10);
        assert_eq!(comparables.len(), 0);
    }

    #[ink::test]
    fn test_get_historical_valuations_works() {
        let oracle = setup_oracle();

        let history = oracle.get_historical_valuations(1, 10);
        assert_eq!(history.len(), 0);
    }

    #[ink::test]
    fn test_insufficient_sources_error() {
        let oracle = setup_oracle();

        let prices = vec![PriceData {
            price: 100,
            timestamp: ink::env::block_timestamp::<DefaultEnvironment>(),
            source: "source1".to_string(),
        }];

        let result = oracle.aggregate_prices(&prices);
        assert_eq!(result, Err(OracleError::InsufficientSources));
    }

    #[ink::test]
    fn test_source_reputation_works() {
        let mut oracle = setup_oracle();
        let source_id = "source1".to_string();

        assert!(oracle
            .update_source_reputation(source_id.clone(), true)
            .is_ok());
        assert_eq!(
            oracle
                .source_reputations
                .get(&source_id)
                .expect("Source reputation should exist after update"),
            510
        );

        assert!(oracle
            .update_source_reputation(source_id.clone(), false)
            .is_ok());
        assert_eq!(
            oracle
                .source_reputations
                .get(&source_id)
                .expect("Source reputation should exist after update"),
            460
        );
    }

    #[ink::test]
    fn test_slashing_works() {
        let mut oracle = setup_oracle();
        let source_id = "source1".to_string();

        oracle.source_stakes.insert(&source_id, &1000);
        assert!(oracle.slash_source(source_id.clone(), 100).is_ok());

        assert_eq!(
            oracle
                .source_stakes
                .get(&source_id)
                .expect("Source stake should exist after slashing"),
            900
        );
        assert!(
            oracle
                .source_reputations
                .get(&source_id)
                .expect("Source reputation should exist after slashing")
                < 500
        );
    }

    #[ink::test]
    fn test_anomaly_detection_works() {
        let mut oracle = setup_oracle();
        let property_id = 1;

        let valuation = PropertyValuation {
            property_id,
            valuation: 100000,
            confidence_score: 90,
            sources_used: 3,
            last_updated: 0,
            valuation_method: ValuationMethod::Automated,
        };

        oracle.property_valuations.insert(&property_id, &valuation);

        assert!(!oracle.is_anomaly(property_id, 105000));
        assert!(oracle.is_anomaly(property_id, 130000));
    }

    #[ink::test]
    fn test_batch_request_works() {
        let mut oracle = setup_oracle();
        let result = oracle.batch_request_valuations(vec![1, 2, 3]).unwrap();
        assert_eq!(result.successes.len(), 3);
        assert!(result.failures.is_empty());

        assert!(oracle.pending_requests.get(&1).is_some());
        assert!(oracle.pending_requests.get(&2).is_some());
        assert!(oracle.pending_requests.get(&3).is_some());
    }
}

// =========================================================================
// AUTO-SLASH TESTS (Issue #497)
// =========================================================================

#[cfg(test)]
mod auto_slash_tests {
    use super::*;
    use crate::propchain_oracle::PropertyValuationOracle;
    use ink::env::{test, DefaultEnvironment};

    fn setup() -> PropertyValuationOracle {
        let accounts = test::default_accounts::<DefaultEnvironment>();
        test::set_caller::<DefaultEnvironment>(accounts.alice);
        PropertyValuationOracle::new(accounts.alice)
    }

    fn add_source(oracle: &mut PropertyValuationOracle, id: &str) {
        let accounts = test::default_accounts::<DefaultEnvironment>();
        oracle
            .add_oracle_source(OracleSource {
                id: id.to_string(),
                source_type: OracleSourceType::Manual,
                address: accounts.bob,
                is_active: true,
                weight: 50,
                last_updated: 0,
            })
            .unwrap();
        // Give the source some stake
        oracle.source_stakes.insert(&id.to_string(), &1_000_000);
        // Give it a reputation
        oracle
            .source_reputations
            .insert(&id.to_string(), &500u32);
    }

    #[ink::test]
    fn test_auto_slash_config_defaults() {
        let oracle = setup();
        let (on_s, secs, on_d, bps, on_m, cnt) = oracle.get_auto_slash_config();
        assert!(!on_s);
        assert_eq!(secs, 3600);
        assert!(!on_d);
        assert_eq!(bps, 2000);
        assert!(!on_m);
        assert_eq!(cnt, 3);
    }

    #[ink::test]
    fn test_set_auto_slash_config() {
        let mut oracle = setup();
        assert!(oracle
            .set_auto_slash_config(true, 1800, true, 1500, true, 5)
            .is_ok());
        let (on_s, secs, on_d, bps, on_m, cnt) = oracle.get_auto_slash_config();
        assert!(on_s);
        assert_eq!(secs, 1800);
        assert!(on_d);
        assert_eq!(bps, 1500);
        assert!(on_m);
        assert_eq!(cnt, 5);
    }

    #[ink::test]
    fn test_set_auto_slash_config_zero_threshold_fails() {
        let mut oracle = setup();
        assert_eq!(
            oracle.set_auto_slash_config(true, 0, false, 1000, false, 3),
            Err(OracleError::InvalidParameters)
        );
    }

    #[ink::test]
    fn test_auto_slash_on_staleness() {
        let mut oracle = setup();
        add_source(&mut oracle, "stale_src");

        // Enable staleness auto-slash with a very short threshold (1 second)
        oracle
            .set_auto_slash_config(true, 1, false, 2000, false, 3)
            .unwrap();

        // Record the source as having reported at time 0
        oracle
            .source_last_report_time
            .insert(&"stale_src".to_string(), &0u64);

        // Set block timestamp to 100 (> staleness threshold of 1)
        test::set_block_timestamp::<DefaultEnvironment>(100);

        let stake_before = oracle
            .source_stakes
            .get(&"stale_src".to_string())
            .unwrap_or(0);

        // Trigger auto-slash via run_auto_slash_checks
        oracle.run_auto_slash_checks(500_000);

        let stake_after = oracle
            .source_stakes
            .get(&"stale_src".to_string())
            .unwrap_or(0);

        // Stake should have decreased
        assert!(stake_after < stake_before, "Stake should be slashed for staleness");
    }

    #[ink::test]
    fn test_auto_slash_on_missed_updates() {
        let mut oracle = setup();
        add_source(&mut oracle, "lazy_src");

        // Enable missed-updates auto-slash with threshold of 2
        oracle
            .set_auto_slash_config(false, 3600, false, 2000, true, 2)
            .unwrap();

        // Set missed update counter to 3 (above threshold)
        oracle
            .source_missed_updates
            .insert(&"lazy_src".to_string(), &3u32);

        let stake_before = oracle
            .source_stakes
            .get(&"lazy_src".to_string())
            .unwrap_or(0);

        oracle.run_auto_slash_checks(500_000);

        let stake_after = oracle
            .source_stakes
            .get(&"lazy_src".to_string())
            .unwrap_or(0);

        assert!(stake_after < stake_before, "Stake should be slashed for missed updates");

        // Counter should be reset after slash
        assert_eq!(
            oracle
                .source_missed_updates
                .get(&"lazy_src".to_string())
                .unwrap_or(99),
            0
        );
    }

    #[ink::test]
    fn test_auto_slash_respects_disabled_flags() {
        let mut oracle = setup();
        add_source(&mut oracle, "fine_src");

        // All auto-slash disabled
        oracle
            .set_auto_slash_config(false, 1, false, 1, false, 1)
            .unwrap();

        // Give the source a stale last-report time and high missed count
        oracle
            .source_last_report_time
            .insert(&"fine_src".to_string(), &0u64);
        oracle
            .source_missed_updates
            .insert(&"fine_src".to_string(), &100u32);
        test::set_block_timestamp::<DefaultEnvironment>(99_999);

        let stake_before = oracle
            .source_stakes
            .get(&"fine_src".to_string())
            .unwrap_or(0);

        oracle.run_auto_slash_checks(500_000);

        let stake_after = oracle
            .source_stakes
            .get(&"fine_src".to_string())
            .unwrap_or(0);

        // No slashing should have occurred
        assert_eq!(stake_before, stake_after, "No slash when all flags disabled");
    }

    #[ink::test]
    fn test_source_last_report_time_initially_zero() {
        let oracle = setup();
        assert_eq!(
            oracle.get_source_last_report_time("nonexistent".to_string()),
            0
        );
    }

    #[ink::test]
    fn test_source_missed_updates_initially_zero() {
        let oracle = setup();
        assert_eq!(
            oracle.get_source_missed_updates("nonexistent".to_string()),
            0
        );
    }
}

// =========================================================================
// MULTI-SIG ORACLE SOURCE MANAGEMENT TESTS (Issue #495)
// =========================================================================

#[cfg(test)]
mod oracle_source_multisig_tests {
    use super::*;
    use crate::propchain_oracle::PropertyValuationOracle;
    use ink::env::{test, DefaultEnvironment};

    fn setup_with_signers() -> PropertyValuationOracle {
        let accounts = test::default_accounts::<DefaultEnvironment>();
        test::set_caller::<DefaultEnvironment>(accounts.alice);
        let mut oracle = PropertyValuationOracle::new(accounts.alice);
        // Register two signers
        oracle.add_multisig_signer(accounts.alice).unwrap();
        oracle.add_multisig_signer(accounts.bob).unwrap();
        // Require 2 approvals
        oracle.set_multisig_threshold(2).unwrap();
        oracle
    }

    fn sample_source(id: &str) -> OracleSource {
        let accounts = test::default_accounts::<DefaultEnvironment>();
        OracleSource {
            id: id.to_string(),
            source_type: OracleSourceType::Chainlink,
            address: accounts.charlie,
            is_active: true,
            weight: 50,
            last_updated: 0,
        }
    }

    #[ink::test]
    fn test_single_admin_cannot_add_source_without_multisig() {
        let mut oracle = setup_with_signers();
        // Propose as alice (1 of 2 approvals) — should NOT be executed yet
        let proposal_id = oracle
            .propose_add_oracle_source(sample_source("chainlink_1"))
            .unwrap();

        // Source should not be active yet
        assert!(
            !oracle.active_sources.contains(&"chainlink_1".to_string()),
            "Source should not be added until threshold reached"
        );

        let prop = oracle.get_source_proposal(proposal_id).unwrap();
        assert!(!prop.executed);
        assert_eq!(prop.approvals.len(), 1);
    }

    #[ink::test]
    fn test_multisig_approval_executes_source_addition() {
        let mut oracle = setup_with_signers();
        let accounts = test::default_accounts::<DefaultEnvironment>();

        // Alice proposes
        let proposal_id = oracle
            .propose_add_oracle_source(sample_source("chainlink_2"))
            .unwrap();

        // Bob approves — threshold reached
        test::set_caller::<DefaultEnvironment>(accounts.bob);
        let executed = oracle.approve_source_proposal(proposal_id).unwrap();
        assert!(executed, "Proposal should execute when threshold reached");

        // Source should now be active
        assert!(
            oracle.active_sources.contains(&"chainlink_2".to_string()),
            "Source should be added after threshold reached"
        );

        let prop = oracle.get_source_proposal(proposal_id).unwrap();
        assert!(prop.executed);
    }

    #[ink::test]
    fn test_multisig_approval_executes_source_removal() {
        let mut oracle = setup_with_signers();
        let accounts = test::default_accounts::<DefaultEnvironment>();

        // First add the source directly (admin bypass with no signers)
        oracle
            .add_oracle_source(sample_source("pyth_1"))
            .unwrap();
        assert!(oracle.active_sources.contains(&"pyth_1".to_string()));

        // Propose removal
        let proposal_id = oracle
            .propose_remove_oracle_source("pyth_1".to_string())
            .unwrap();

        // Bob approves
        test::set_caller::<DefaultEnvironment>(accounts.bob);
        let executed = oracle.approve_source_proposal(proposal_id).unwrap();
        assert!(executed);

        // Source should be gone
        assert!(
            !oracle.active_sources.contains(&"pyth_1".to_string()),
            "Source should be removed after threshold reached"
        );
    }

    #[ink::test]
    fn test_non_signer_cannot_propose() {
        let mut oracle = setup_with_signers();
        let accounts = test::default_accounts::<DefaultEnvironment>();
        test::set_caller::<DefaultEnvironment>(accounts.django);
        assert_eq!(
            oracle.propose_add_oracle_source(sample_source("bad_src")),
            Err(OracleError::Unauthorized)
        );
    }

    #[ink::test]
    fn test_double_approval_fails() {
        let mut oracle = setup_with_signers();
        let proposal_id = oracle
            .propose_add_oracle_source(sample_source("chainlink_3"))
            .unwrap();

        // Alice tries to approve again
        assert_eq!(
            oracle.approve_source_proposal(proposal_id),
            Err(OracleError::AlreadyExists)
        );
    }

    #[ink::test]
    fn test_approve_executed_proposal_fails() {
        let mut oracle = setup_with_signers();
        let accounts = test::default_accounts::<DefaultEnvironment>();

        let proposal_id = oracle
            .propose_add_oracle_source(sample_source("chainlink_4"))
            .unwrap();

        test::set_caller::<DefaultEnvironment>(accounts.bob);
        oracle.approve_source_proposal(proposal_id).unwrap();

        // Try to approve again after execution
        assert_eq!(
            oracle.approve_source_proposal(proposal_id),
            Err(OracleError::AlreadyExists)
        );
    }

    #[ink::test]
    fn test_no_signers_add_source_immediately() {
        let accounts = test::default_accounts::<DefaultEnvironment>();
        test::set_caller::<DefaultEnvironment>(accounts.alice);
        let mut oracle = PropertyValuationOracle::new(accounts.alice);
        // No signers registered → immediate execution

        oracle
            .propose_add_oracle_source(sample_source("instant_src"))
            .unwrap();

        assert!(
            oracle.active_sources.contains(&"instant_src".to_string()),
            "Source added immediately when no signers configured"
        );
    }
}

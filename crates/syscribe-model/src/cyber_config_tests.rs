//! Unit tests for [`super`] (`cyber_config`), GH #222.

use super::*;

#[test]
fn default_reproduces_the_historical_rank_sum_for_every_cell() {
    let c = CyberConfig::default();
    assert!(c.is_default());
    for i in 0..4u8 {
        for f in 0..4u8 {
            let r = c.risk_cell(i, f);
            assert_eq!(r.level, level_from_score(i + f));
            assert_eq!(r.value, None);
        }
    }
    for (rank, lvl) in [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High, RiskLevel::Critical]
        .into_iter()
        .enumerate()
    {
        // no table, so any vector falls through to the 1:1 risk -> CAL map
        assert_eq!(c.cal_for(2, Some("network"), lvl), rank as u8 + 1);
    }
}

#[test]
fn absent_or_cyber_free_toml_is_default() {
    assert!(CyberConfig::from_toml_str("").is_default());
    assert!(CyberConfig::from_toml_str("[ids]\nmax_digits = 5\n").is_default());
    assert!(CyberConfig::from_toml_str("not toml at all ===").is_default());
    assert!(CyberConfig::from_toml_str("[cyber]\nmethod = \"simple\"\n").is_default());
}

#[test]
fn annex_example_tables_have_the_expected_shape() {
    let c = CyberConfig::from_toml_str("[cyber]\nmethod = \"annex\"\n");
    assert_eq!(c.method(), RiskMethod::Annex);
    assert!(!c.is_default());
    assert!(c.defects().is_empty());
    assert_eq!(c.risk_cell(0, 3), Risk { level: RiskLevel::Low, value: Some(1) });
    assert_eq!(c.risk_cell(3, 3), Risk { level: RiskLevel::Critical, value: Some(5) });
    assert_eq!(c.risk_cell(2, 2).value, Some(3));
    // monotone in both axes
    for i in 0..4u8 {
        for f in 0..3u8 {
            assert!(c.risk_cell(i, f).value <= c.risk_cell(i, f + 1).value);
        }
    }
    for f in 0..4u8 {
        for i in 0..3u8 {
            assert!(c.risk_cell(i, f).value <= c.risk_cell(i + 1, f).value);
        }
    }
    // CAL grows with exposure (network >= physical)
    for i in 0..4u8 {
        assert!(
            c.cal_for(i, Some("network"), RiskLevel::Low)
                >= c.cal_for(i, Some("physical"), RiskLevel::Low)
        );
    }
    assert_eq!(c.cal_for(3, Some("network"), RiskLevel::Low), 4);
    // no vector -> falls back to the risk map
    assert_eq!(c.cal_for(3, None, RiskLevel::Medium), 2);
}

#[test]
fn attack_potential_sum_thresholds() {
    let ap = AttackPotential::example();
    assert_eq!(ap.rank_for_points(0), 3);
    assert_eq!(ap.rank_for_points(13), 3);
    assert_eq!(ap.rank_for_points(14), 2);
    assert_eq!(ap.rank_for_points(19), 2);
    assert_eq!(ap.rank_for_points(20), 1);
    assert_eq!(ap.rank_for_points(24), 1);
    assert_eq!(ap.rank_for_points(25), 0);
}

#[test]
fn thresholds_must_be_strictly_increasing() {
    let c = CyberConfig::from_toml_str(
        "[cyber.attack_potential.thresholds]\nhigh_max = 5\nmedium_max = 5\nlow_max = 9\n",
    );
    assert!(c.is_default());
    assert_eq!(c.defects().len(), 1, "{:?}", c.defects());
}

#[test]
fn camel_and_snake_case_keys_are_both_accepted() {
    let a = CyberConfig::from_toml_str("[cyber.attack_potential.elapsed_time]\nfast = 1\n");
    let b = CyberConfig::from_toml_str("[cyber.attackPotential.elapsedTime]\nfast = 1\n");
    assert_eq!(a, b);
    assert!(a.defects().is_empty());
}

#[test]
fn non_table_cyber_is_a_defect() {
    let c = CyberConfig::from_toml_str("cyber = 3\n");
    assert!(c.is_default());
    assert_eq!(c.defects().len(), 1);
}

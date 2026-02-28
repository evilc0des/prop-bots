use crate::profiles::PropFirmProfile;
use chrono::Utc;
use propbot_core::*;
use rust_decimal::Decimal;
use tracing::{info, warn};

/// Prop firm risk manager that enforces evaluation/funded account rules.
pub struct PropFirmRiskManager {
    profile: PropFirmProfile,
    halted: bool,
    violations: Vec<RiskViolation>,
    /// Total unrealized + realized PnL for the current day.
    daily_pnl: Decimal,
    /// Current equity (tracked for drawdown).
    current_equity: Decimal,
    /// High water mark for trailing drawdown.
    high_water_mark: Decimal,
    /// Starting balance (the drawdown reference for non-trailing).
    initial_balance: Decimal,
    /// Total open position size across all instruments.
    total_position_size: Decimal,
}

impl PropFirmRiskManager {
    pub fn new(profile: PropFirmProfile) -> Self {
        let initial = profile.initial_balance;
        Self {
            profile,
            halted: false,
            violations: Vec::new(),
            daily_pnl: Decimal::ZERO,
            current_equity: initial,
            high_water_mark: initial,
            initial_balance: initial,
            total_position_size: Decimal::ZERO,
        }
    }

    pub fn profile(&self) -> &PropFirmProfile {
        &self.profile
    }

    /// Check the daily loss limit.
    fn check_daily_loss(&self) -> Option<RiskViolation> {
        let daily_loss = -self.daily_pnl;
        if daily_loss >= self.profile.daily_loss_limit {
            Some(RiskViolation {
                rule: "daily_loss_limit".to_string(),
                message: format!(
                    "Daily loss limit breached: ${:.2} loss >= ${:.2} limit",
                    daily_loss, self.profile.daily_loss_limit
                ),
                current_value: daily_loss.to_string(),
                threshold: self.profile.daily_loss_limit.to_string(),
                severity: RiskSeverity::Breach,
            })
        } else {
            let threshold = self.profile.daily_loss_limit * self.profile.auto_flatten_threshold;
            if daily_loss >= threshold {
                Some(RiskViolation {
                    rule: "daily_loss_limit".to_string(),
                    message: format!(
                        "Approaching daily loss limit: ${:.2} loss / ${:.2} limit",
                        daily_loss, self.profile.daily_loss_limit
                    ),
                    current_value: daily_loss.to_string(),
                    threshold: self.profile.daily_loss_limit.to_string(),
                    severity: RiskSeverity::Warning,
                })
            } else {
                None
            }
        }
    }

    /// Check the max drawdown.
    fn check_drawdown(&self) -> Option<RiskViolation> {
        let drawdown = if self.profile.trailing_drawdown {
            self.high_water_mark - self.current_equity
        } else {
            self.initial_balance - self.current_equity
        };

        if drawdown >= self.profile.max_drawdown {
            Some(RiskViolation {
                rule: "max_drawdown".to_string(),
                message: format!(
                    "Max drawdown breached: ${:.2} drawdown >= ${:.2} limit ({})",
                    drawdown,
                    self.profile.max_drawdown,
                    if self.profile.trailing_drawdown {
                        "trailing"
                    } else {
                        "fixed"
                    }
                ),
                current_value: drawdown.to_string(),
                threshold: self.profile.max_drawdown.to_string(),
                severity: RiskSeverity::Breach,
            })
        } else {
            let threshold = self.profile.max_drawdown * self.profile.auto_flatten_threshold;
            if drawdown >= threshold {
                Some(RiskViolation {
                    rule: "max_drawdown".to_string(),
                    message: format!(
                        "Approaching max drawdown: ${:.2} drawdown / ${:.2} limit",
                        drawdown, self.profile.max_drawdown
                    ),
                    current_value: drawdown.to_string(),
                    threshold: self.profile.max_drawdown.to_string(),
                    severity: RiskSeverity::Warning,
                })
            } else {
                None
            }
        }
    }

    /// Check position size constraints.
    fn check_position_size(&self, new_order_qty: Decimal) -> Option<RiskViolation> {
        if let Some(max_size) = self.profile.max_position_size {
            let projected = self.total_position_size + new_order_qty;
            if projected > max_size {
                return Some(RiskViolation {
                    rule: "max_position_size".to_string(),
                    message: format!(
                        "Position size would exceed limit: {} + {} = {} > {} max",
                        self.total_position_size, new_order_qty, projected, max_size
                    ),
                    current_value: projected.to_string(),
                    threshold: max_size.to_string(),
                    severity: RiskSeverity::Critical,
                });
            }
        }
        None
    }

    /// Check if we're within allowed trading hours.
    fn check_trading_hours(&self) -> Option<RiskViolation> {
        if let (Some(start), Some(end)) = (
            self.profile.trading_start_utc,
            self.profile.trading_end_utc,
        ) {
            let now = Utc::now().time();
            if now < start || now > end {
                return Some(RiskViolation {
                    rule: "trading_hours".to_string(),
                    message: format!(
                        "Outside trading hours: {} not in {}-{}",
                        now.format("%H:%M"),
                        start.format("%H:%M"),
                        end.format("%H:%M"),
                    ),
                    current_value: now.to_string(),
                    threshold: format!("{}-{}", start, end),
                    severity: RiskSeverity::Critical,
                });
            }
        }
        None
    }
}

impl RiskManager for PropFirmRiskManager {
    fn evaluate_order(&self, order: &Order, _account: &AccountState) -> RiskDecision {
        // Check if trading is halted
        if self.halted {
            return RiskDecision::Rejected("Trading is halted due to risk breach".to_string());
        }

        // Check trading hours
        if let Some(violation) = self.check_trading_hours() {
            if violation.severity == RiskSeverity::Critical || violation.severity == RiskSeverity::Breach {
                return RiskDecision::Rejected(violation.message);
            }
        }

        // Check daily loss limit
        if let Some(violation) = self.check_daily_loss() {
            if violation.severity == RiskSeverity::Breach {
                return RiskDecision::Rejected(violation.message);
            }
        }

        // Check drawdown
        if let Some(violation) = self.check_drawdown() {
            if violation.severity == RiskSeverity::Breach {
                return RiskDecision::Rejected(violation.message);
            }
        }

        // Check position size (only for new entries, not closes)
        if let Some(violation) = self.check_position_size(order.quantity) {
            return RiskDecision::Rejected(violation.message);
        }

        RiskDecision::Approved
    }

    fn update_account(&mut self, account: &AccountState) {
        self.current_equity = account.equity;
        self.daily_pnl = account.daily_pnl;
        self.total_position_size = Decimal::from(account.open_positions);

        // Update high water mark for trailing drawdown
        if account.equity > self.high_water_mark {
            self.high_water_mark = account.equity;
        }

        // Check for breaches and halt if needed
        self.violations.clear();

        if let Some(v) = self.check_daily_loss() {
            if v.severity == RiskSeverity::Breach {
                warn!(rule = %v.rule, "Risk breach: {}", v.message);
                self.halted = true;
            }
            self.violations.push(v);
        }

        if let Some(v) = self.check_drawdown() {
            if v.severity == RiskSeverity::Breach {
                warn!(rule = %v.rule, "Risk breach: {}", v.message);
                self.halted = true;
            }
            self.violations.push(v);
        }
    }

    fn reset_daily(&mut self) {
        self.daily_pnl = Decimal::ZERO;
        // Only un-halt if the daily loss was the cause (drawdown breach is permanent)
        if let Some(v) = self.check_drawdown() {
            if v.severity != RiskSeverity::Breach {
                self.halted = false;
            }
        } else {
            self.halted = false;
        }
        self.violations.clear();
        info!("Daily risk counters reset");
    }

    fn should_halt(&self) -> bool {
        self.halted
    }

    fn active_violations(&self) -> Vec<RiskViolation> {
        self.violations.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_daily_loss_breach() {
        let profile = PropFirmProfile::topstep_50k();
        let mut risk = PropFirmRiskManager::new(profile);

        let mut account = AccountState::new(dec!(50000));
        account.daily_pnl = dec!(-1000); // Hit the daily loss limit
        account.equity = dec!(49000);
        risk.update_account(&account);

        assert!(risk.should_halt());
    }

    #[test]
    fn test_order_rejected_when_halted() {
        let profile = PropFirmProfile::topstep_50k();
        let mut risk = PropFirmRiskManager::new(profile);
        risk.halted = true;

        let order = Order::market("ES", Side::Buy, dec!(1));
        let account = AccountState::new(dec!(50000));
        let decision = risk.evaluate_order(&order, &account);

        match decision {
            RiskDecision::Rejected(msg) => assert!(msg.contains("halted")),
            _ => panic!("Expected rejection"),
        }
    }

    #[test]
    fn test_position_size_limit() {
        let profile = PropFirmProfile::topstep_50k(); // max 5 contracts
        let mut risk = PropFirmRiskManager::new(profile);

        let mut account = AccountState::new(dec!(50000));
        account.open_positions = 4;
        risk.update_account(&account);

        // Trying to add 2 more when we have 4 → total 6 > 5
        let order = Order::market("ES", Side::Buy, dec!(2));
        let decision = risk.evaluate_order(&order, &account);

        match decision {
            RiskDecision::Rejected(msg) => assert!(msg.contains("exceed")),
            _ => panic!("Expected rejection"),
        }
    }
}

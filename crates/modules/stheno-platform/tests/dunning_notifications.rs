//! P30-12: every dunning rung has email and SMS notification intents.

use stheno_platform::PlatformService;

#[test]
fn each_dunning_rung_has_email_and_sms_templates() {
    for (status, template) in [
        ("past_due", "billing_past_due"),
        ("grace", "billing_grace"),
        ("suspended", "billing_suspended"),
    ] {
        let notifications = PlatformService::dunning_notifications(status)
            .unwrap_or_else(|e| panic!("{status} should have notifications: {e}"));

        assert_eq!(notifications.len(), 2);
        assert!(
            notifications
                .iter()
                .any(|n| n.channel == "email" && n.template == template),
            "{status} needs an email template"
        );
        assert!(
            notifications
                .iter()
                .any(|n| n.channel == "sms" && n.template == template),
            "{status} needs an SMS template"
        );
    }
}

#[test]
fn active_and_cancelled_do_not_emit_dunning_notifications() {
    for status in ["active", "cancelled"] {
        assert!(PlatformService::dunning_notifications(status)
            .expect("known non-dunning status")
            .is_empty());
    }
}

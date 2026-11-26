//! Identity and Auth event subscribers delivering transactional email via NotificationService.
//!
//! Handles:
//! - `auth.identity_created` -> Welcome / Account Activation email
//! - `auth.password_reset_requested` -> Password Reset Link & Token email
//! - `user.invite_created` -> Team Member / Staff Invitation email
//! - `order.placed` -> Customer Order Confirmation transactional email

use std::sync::Arc;

use async_trait::async_trait;
use stheno_events::Event;
use stheno_notification::{NotificationService, SendNotificationInput};
use stheno_types::Result;

use crate::EventSubscriber;

/// Handles `auth.identity_created` by sending a welcome email via the configured email provider (Resend).
pub struct IdentityCreatedSubscriber {
    notification_service: Arc<NotificationService>,
    store_name: String,
}

impl IdentityCreatedSubscriber {
    pub fn new(
        notification_service: Arc<NotificationService>,
        store_name: impl Into<String>,
    ) -> Self {
        Self {
            notification_service,
            store_name: store_name.into(),
        }
    }
}

#[async_trait]
impl EventSubscriber for IdentityCreatedSubscriber {
    fn event_name(&self) -> &'static str {
        "auth.identity_created"
    }

    async fn handle(&self, event: &Event) -> Result<()> {
        let payload = &event.payload;
        let email = match payload.get("entity_id").and_then(|v| v.as_str()) {
            Some(e) if e.contains('@') => e,
            _ => return Ok(()),
        };

        let actor_type = payload
            .get("actor_type")
            .and_then(|v| v.as_str())
            .unwrap_or("customer");

        let (subject, body) = if actor_type == "user" {
            (
                format!("Welcome to {} — Merchant Workspace", self.store_name),
                format!(
                    "<div style=\"background-color: #FCFBF9; padding: 40px 16px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;\">\
                        <div style=\"max-width: 560px; margin: 0 auto; background-color: #FFFFFF; border: 1px solid #E6E0D8; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 12px rgba(24, 24, 27, 0.04);\">\
                            <div style=\"height: 4px; background-color: #C2410C;\"></div>\
                            <div style=\"padding: 36px 32px;\">\
                                <div style=\"font-size: 13px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: #EA580C; margin-bottom: 8px;\">Merchant Platform</div>\
                                <h1 style=\"font-size: 24px; font-weight: 700; color: #18181B; margin: 0 0 16px 0; line-height: 1.3;\">Welcome to {}</h1>\
                                <p style=\"font-size: 15px; line-height: 1.6; color: #5F6368; margin: 0 0 24px 0;\">Your operator account for <strong style=\"color: #18181B;\">{}</strong> is now active. Vendryx delivers truth-first commerce infrastructure engineered for high-concurrency throughput and verifiable consistency.</p>\
                                <div style=\"background-color: #F7F3ED; border: 1px solid #E6E0D8; border-radius: 8px; padding: 18px 20px; margin-bottom: 28px;\">\
                                    <div style=\"font-size: 13px; font-weight: 600; color: #18181B; margin-bottom: 8px;\">Your Workspace Capabilities:</div>\
                                    <ul style=\"margin: 0; padding-left: 18px; font-size: 13px; line-height: 1.6; color: #5F6368;\">\
                                        <li>Multi-rail payment settlement & automated reconciliation</li>\
                                        <li>High-speed catalogue management and stock reservations</li>\
                                        <li>Courier integration with automated status tracking</li>\
                                    </ul>\
                                </div>\
                                <div style=\"margin: 28px 0;\">\
                                    <a href=\"/login\" style=\"background-color: #C2410C; color: #FFFFFF; padding: 12px 26px; border-radius: 8px; text-decoration: none; font-weight: 600; font-size: 14px; display: inline-block;\">Sign In to Dashboard</a>\
                                </div>\
                                <div style=\"border-top: 1px solid #E6E0D8; margin-top: 32px; padding-top: 20px;\">\
                                    <p style=\"font-size: 12px; color: #7C818A; margin: 0; line-height: 1.5;\">{} by Syed Systems · Truth before beauty.<br/>High-performance commerce engineering.</p>\
                                </div>\
                            </div>\
                        </div>\
                    </div>",
                    self.store_name, email, self.store_name
                ),
            )
        } else {
            (
                format!("Welcome to {} — Your account is ready", self.store_name),
                format!(
                    "<div style=\"background-color: #FCFBF9; padding: 40px 16px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;\">\
                        <div style=\"max-width: 560px; margin: 0 auto; background-color: #FFFFFF; border: 1px solid #E6E0D8; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 12px rgba(24, 24, 27, 0.04);\">\
                            <div style=\"height: 4px; background-color: #C2410C;\"></div>\
                            <div style=\"padding: 36px 32px;\">\
                                <div style=\"font-size: 13px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: #EA580C; margin-bottom: 8px;\">Customer Account</div>\
                                <h1 style=\"font-size: 24px; font-weight: 700; color: #18181B; margin: 0 0 16px 0; line-height: 1.3;\">Welcome to {}</h1>\
                                <p style=\"font-size: 15px; line-height: 1.6; color: #5F6368; margin: 0 0 24px 0;\">Your customer account for <strong style=\"color: #18181B;\">{}</strong> is ready. You now have access to express checkout, seamless order tracking, and purchase history.</p>\
                                <div style=\"background-color: #F7F3ED; border: 1px solid #E6E0D8; border-radius: 8px; padding: 18px 20px; margin-bottom: 28px;\">\
                                    <div style=\"font-size: 13px; font-weight: 600; color: #18181B; margin-bottom: 8px;\">Account Benefits:</div>\
                                    <ul style=\"margin: 0; padding-left: 18px; font-size: 13px; line-height: 1.6; color: #5F6368;\">\
                                        <li>Instant order lookup and real-time delivery status</li>\
                                        <li>Saved addresses for friction-free checkout</li>\
                                        <li>Verified digital receipts and purchase confirmations</li>\
                                    </ul>\
                                </div>\
                                <div style=\"margin: 28px 0;\">\
                                    <a href=\"/account\" style=\"background-color: #C2410C; color: #FFFFFF; padding: 12px 26px; border-radius: 8px; text-decoration: none; font-weight: 600; font-size: 14px; display: inline-block;\">View Your Account</a>\
                                </div>\
                                <div style=\"border-top: 1px solid #E6E0D8; margin-top: 32px; padding-top: 20px;\">\
                                    <p style=\"font-size: 12px; color: #7C818A; margin: 0; line-height: 1.5;\">{} · Modern high-performance commerce powered by Syed Systems.</p>\
                                </div>\
                            </div>\
                        </div>\
                    </div>",
                    self.store_name, email, self.store_name
                ),
            )
        };

        self.notification_service
            .enqueue_notification_idempotent_for_tenant(
                SendNotificationInput {
                    to: email.to_string(),
                    channel: "email".to_string(),
                    template: None,
                    subject: Some(subject),
                    body: Some(body),
                    data: Some(payload.clone()),
                    trigger_type: Some("auth.identity_created".to_string()),
                    resource_id: payload
                        .get("identity_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    resource_type: Some("auth_identity".to_string()),
                    receiver_id: Some(email.to_string()),
                },
                &format!("identity-created/{}", event.id),
                &event.tenant_id,
            )
            .await?;

        Ok(())
    }
}

/// Handles `auth.password_reset_requested` by delivering a single-use token email.
pub struct PasswordResetSubscriber {
    notification_service: Arc<NotificationService>,
    store_name: String,
}

impl PasswordResetSubscriber {
    pub fn new(
        notification_service: Arc<NotificationService>,
        store_name: impl Into<String>,
    ) -> Self {
        Self {
            notification_service,
            store_name: store_name.into(),
        }
    }
}

#[async_trait]
impl EventSubscriber for PasswordResetSubscriber {
    fn event_name(&self) -> &'static str {
        "auth.password_reset_requested"
    }

    async fn handle(&self, event: &Event) -> Result<()> {
        let payload = &event.payload;
        let email = match payload.get("entity_id").and_then(|v| v.as_str()) {
            Some(e) if e.contains('@') => e,
            _ => return Ok(()),
        };

        let token = payload
            .get("reset_token")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let subject = format!("Reset your {} password", self.store_name);
        let body = format!(
            "<div style=\"background-color: #FCFBF9; padding: 40px 16px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;\">\
                <div style=\"max-width: 560px; margin: 0 auto; background-color: #FFFFFF; border: 1px solid #E6E0D8; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 12px rgba(24, 24, 27, 0.04);\">\
                    <div style=\"height: 4px; background-color: #C2410C;\"></div>\
                    <div style=\"padding: 36px 32px;\">\
                        <div style=\"font-size: 13px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: #EA580C; margin-bottom: 8px;\">Security Verification</div>\
                        <h1 style=\"font-size: 22px; font-weight: 700; color: #18181B; margin: 0 0 16px 0;\">Password Reset Request</h1>\
                        <p style=\"font-size: 15px; line-height: 1.6; color: #5F6368; margin: 0 0 20px 0;\">A password reset was requested for <strong style=\"color: #18181B;\">{}</strong>. Enter the single-use verification code below to establish your new password. This code expires in 2 hours.</p>\
                        <div style=\"margin: 24px 0; background-color: #F7F3ED; border: 1px solid #E6E0D8; border-radius: 8px; padding: 20px; text-align: center;\">\
                            <div style=\"font-size: 12px; text-transform: uppercase; letter-spacing: 0.1em; color: #7C818A; margin-bottom: 6px;\">Verification Code</div>\
                            <code style=\"font-size: 24px; font-weight: 700; letter-spacing: 4px; color: #18181B; font-family: ui-monospace, Menlo, Consolas, monospace;\">{}</code>\
                        </div>\
                        <p style=\"font-size: 13px; color: #7C818A; margin: 0 0 24px 0; line-height: 1.5;\">If you did not initiate this request, you can safely ignore this email. Your existing password and account credentials remain intact.</p>\
                        <div style=\"border-top: 1px solid #E6E0D8; margin-top: 28px; padding-top: 18px;\">\
                            <p style=\"font-size: 12px; color: #7C818A; margin: 0;\">{} Security by Syed Systems</p>\
                        </div>\
                    </div>\
                </div>\
            </div>",
            email, token, self.store_name
        );

        let _ = self
            .notification_service
            .send_notification(SendNotificationInput {
                to: email.to_string(),
                channel: "email".to_string(),
                template: None,
                subject: Some(subject),
                body: Some(body),
                data: Some(payload.clone()),
                trigger_type: Some("auth.password_reset_requested".to_string()),
                resource_id: payload
                    .get("identity_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                resource_type: Some("auth_identity".to_string()),
                receiver_id: Some(email.to_string()),
            })
            .await?;

        Ok(())
    }
}

/// Handles `user.invite_created` by sending a staff invitation email.
pub struct UserInviteSubscriber {
    notification_service: Arc<NotificationService>,
    store_name: String,
}

impl UserInviteSubscriber {
    pub fn new(
        notification_service: Arc<NotificationService>,
        store_name: impl Into<String>,
    ) -> Self {
        Self {
            notification_service,
            store_name: store_name.into(),
        }
    }
}

#[async_trait]
impl EventSubscriber for UserInviteSubscriber {
    fn event_name(&self) -> &'static str {
        "user.invite_created"
    }

    async fn handle(&self, event: &Event) -> Result<()> {
        let payload = &event.payload;
        let email = match payload.get("email").and_then(|v| v.as_str()) {
            Some(e) if e.contains('@') => e,
            _ => return Ok(()),
        };

        let role = payload
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("staff");
        let token = payload
            .get("token")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let subject = format!("You've been invited to join {} Admin", self.store_name);
        let body = format!(
            "<div style=\"background-color: #FCFBF9; padding: 40px 16px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;\">\
                <div style=\"max-width: 560px; margin: 0 auto; background-color: #FFFFFF; border: 1px solid #E6E0D8; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 12px rgba(24, 24, 27, 0.04);\">\
                    <div style=\"height: 4px; background-color: #C2410C;\"></div>\
                    <div style=\"padding: 36px 32px;\">\
                        <div style=\"font-size: 13px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: #EA580C; margin-bottom: 8px;\">Team Invitation</div>\
                        <h1 style=\"font-size: 22px; font-weight: 700; color: #18181B; margin: 0 0 16px 0;\">Join the {} Team</h1>\
                        <p style=\"font-size: 15px; line-height: 1.6; color: #5F6368; margin: 0 0 20px 0;\">You have been invited to join the merchant operations workspace with role <strong style=\"color: #18181B;\">{}</strong>.</p>\
                        <div style=\"margin: 28px 0;\">\
                            <a href=\"/invite/accept?token={}\" style=\"background-color: #C2410C; color: #FFFFFF; padding: 12px 26px; border-radius: 8px; text-decoration: none; font-weight: 600; font-size: 14px; display: inline-block;\">Accept Invitation</a>\
                        </div>\
                        <div style=\"background-color: #F7F3ED; border: 1px solid #E6E0D8; border-radius: 8px; padding: 14px 16px; margin-bottom: 24px;\">\
                            <p style=\"font-size: 13px; color: #5F6368; margin: 0;\">Invitation token: <code style=\"font-weight: 600; color: #18181B; font-family: ui-monospace, Menlo, monospace;\">{}</code> (Valid for 7 days)</p>\
                        </div>\
                        <div style=\"border-top: 1px solid #E6E0D8; margin-top: 28px; padding-top: 18px;\">\
                            <p style=\"font-size: 12px; color: #7C818A; margin: 0;\">{} Admin by Syed Systems</p>\
                        </div>\
                    </div>\
                </div>\
            </div>",
            self.store_name, role, token, token, self.store_name
        );

        self.notification_service
            .enqueue_notification_idempotent_for_tenant(
                SendNotificationInput {
                    to: email.to_string(),
                    channel: "email".to_string(),
                    template: None,
                    subject: Some(subject),
                    body: Some(body),
                    data: Some(payload.clone()),
                    trigger_type: Some("user.invite_created".to_string()),
                    resource_id: payload
                        .get("invite_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    resource_type: Some("user_invite".to_string()),
                    receiver_id: Some(email.to_string()),
                },
                &format!("user-invite/{}", event.id),
                &event.tenant_id,
            )
            .await?;

        Ok(())
    }
}

/// Handles `order.placed` by delivering a customer order confirmation email.
pub struct OrderConfirmationEmailSubscriber {
    notification_service: Arc<NotificationService>,
    store_name: String,
}

impl OrderConfirmationEmailSubscriber {
    pub fn new(
        notification_service: Arc<NotificationService>,
        store_name: impl Into<String>,
    ) -> Self {
        Self {
            notification_service,
            store_name: store_name.into(),
        }
    }
}

#[async_trait]
impl EventSubscriber for OrderConfirmationEmailSubscriber {
    fn event_name(&self) -> &'static str {
        "order.placed"
    }

    async fn handle(&self, event: &Event) -> Result<()> {
        let payload = &event.payload;

        let email = payload
            .get("email")
            .or_else(|| payload.get("customer").and_then(|c| c.get("email")))
            .and_then(|v| v.as_str());

        let email = match email {
            Some(e) if e.contains('@') => e,
            _ => return Ok(()),
        };

        let order_id = payload
            .get("order_id")
            .or_else(|| payload.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let total = payload
            .get("total")
            .or_else(|| payload.get("total_amount"))
            .and_then(|v| v.as_str())
            .unwrap_or("$0.00");

        let subject = format!("Order Confirmation #{order_id}: Thank you for your purchase");
        let body = format!(
            "<div style=\"background-color: #FCFBF9; padding: 40px 16px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;\">\
                <div style=\"max-width: 560px; margin: 0 auto; background-color: #FFFFFF; border: 1px solid #E6E0D8; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 12px rgba(24, 24, 27, 0.04);\">\
                    <div style=\"height: 4px; background-color: #C2410C;\"></div>\
                    <div style=\"padding: 36px 32px;\">\
                        <div style=\"font-size: 13px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: #EA580C; margin-bottom: 8px;\">Order Confirmed</div>\
                        <h1 style=\"font-size: 22px; font-weight: 700; color: #18181B; margin: 0 0 16px 0;\">Thank you for your order</h1>\
                        <p style=\"font-size: 15px; line-height: 1.6; color: #5F6368; margin: 0 0 20px 0;\">We have received your order <strong style=\"color: #18181B;\">#{}</strong> and our team is preparing it for fulfillment and dispatch.</p>\
                        <div style=\"background-color: #F7F3ED; border: 1px solid #E6E0D8; border-radius: 8px; padding: 18px 20px; margin-bottom: 24px;\">\
                            <div style=\"display: flex; justify-content: space-between; margin-bottom: 8px;\">\
                                <span style=\"font-size: 14px; color: #5F6368;\">Order Reference</span>\
                                <strong style=\"font-size: 14px; color: #18181B;\">#{}</strong>\
                            </div>\
                            <div style=\"display: flex; justify-content: space-between;\">\
                                <span style=\"font-size: 14px; color: #5F6368;\">Total Amount</span>\
                                <strong style=\"font-size: 14px; color: #18181B;\">{}</strong>\
                            </div>\
                        </div>\
                        <p style=\"font-size: 13px; color: #7C818A; line-height: 1.5; margin: 0 0 24px 0;\">You will receive delivery and courier tracking updates via email as soon as your parcel is dispatched.</p>\
                        <div style=\"border-top: 1px solid #E6E0D8; margin-top: 28px; padding-top: 18px;\">\
                            <p style=\"font-size: 12px; color: #7C818A; margin: 0;\">{} · High-performance commerce by Syed Systems.</p>\
                        </div>\
                    </div>\
                </div>\
            </div>",
            order_id, order_id, total, self.store_name
        );

        let _ = self
            .notification_service
            .enqueue_notification_idempotent_for_tenant(
                SendNotificationInput {
                    to: email.to_string(),
                    channel: "email".to_string(),
                    template: None,
                    subject: Some(subject),
                    body: Some(body),
                    data: Some(payload.clone()),
                    trigger_type: Some("order.placed".to_string()),
                    resource_id: Some(order_id.to_string()),
                    resource_type: Some("order".to_string()),
                    receiver_id: Some(email.to_string()),
                },
                &format!("order-confirmation/{}", event.id),
                &event.tenant_id,
            )
            .await?;

        Ok(())
    }
}

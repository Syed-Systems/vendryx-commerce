use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::{
    CreateTemplateInput, NotificationDTO, NotificationDeliveryDTO, NotificationTemplateDTO,
    SendNotificationInput,
};
use crate::models::{Notification, NotificationDelivery, NotificationTemplate};
use crate::provider::NotificationProvider;

#[derive(Default)]
struct ProviderRegistry(HashMap<String, Arc<dyn NotificationProvider>>);

#[derive(Clone)]
pub struct NotificationService {
    providers: Arc<RwLock<ProviderRegistry>>,
    templates: PgCrudRepository<NotificationTemplate>,
    notifications: PgCrudRepository<Notification>,
    deliveries: PgCrudRepository<NotificationDelivery>,
}

impl NotificationService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            providers: Arc::new(RwLock::new(ProviderRegistry(HashMap::new()))),
            templates: PgCrudRepository::new(pool.clone()),
            notifications: PgCrudRepository::new(pool.clone()),
            deliveries: PgCrudRepository::new(pool),
        }
    }

    pub fn register_provider(&self, provider: Arc<dyn NotificationProvider>) {
        let mut reg = self.providers.write().unwrap();
        reg.0.insert(provider.provider_id().to_string(), provider);
    }

    fn find_provider_for_channel(&self, channel: &str) -> Result<Arc<dyn NotificationProvider>> {
        let reg = self.providers.read().unwrap();
        for provider in reg.0.values() {
            if provider.supported_channels().contains(&channel) {
                return Ok(provider.clone());
            }
        }
        Err(Error::NotFound {
            entity: "notification_provider_for_channel",
            id: channel.to_string(),
        })
    }

    // ==========================================
    // 1. Templates
    // ==========================================

    pub async fn create_template(
        &self,
        input: CreateTemplateInput,
    ) -> Result<NotificationTemplateDTO> {
        let id = Id::new();
        let now = Utc::now();

        let template = NotificationTemplate {
            id,
            name: input.name,
            channel: input.channel,
            subject_template: input.subject_template,
            body_template: input.body_template,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.templates.create(None, template).await?;

        Ok(NotificationTemplateDTO {
            id: created.id.to_string(),
            name: created.name,
            channel: created.channel,
            subject_template: created.subject_template,
            body_template: created.body_template,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    pub async fn get_template_by_name(&self, name: &str) -> Result<NotificationTemplateDTO> {
        let filter = FilterSpec::new()
            .with_predicate("name", FilterOp::Eq(name.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.templates.list(None, &filter).await?;
        let t = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "notification_template",
            id: name.to_string(),
        })?;

        Ok(NotificationTemplateDTO {
            id: t.id.to_string(),
            name: t.name,
            channel: t.channel,
            subject_template: t.subject_template,
            body_template: t.body_template,
            created_at: t.created_at,
            updated_at: t.updated_at,
        })
    }

    fn render_text(template: &str, data: Option<&serde_json::Value>) -> String {
        let mut rendered = template.to_string();
        if let Some(serde_json::Value::Object(map)) = data {
            for (k, v) in map {
                let placeholder = format!("{{{{{}}}}}", k);
                let val_str = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                rendered = rendered.replace(&placeholder, &val_str);
            }
        }
        rendered
    }

    // ==========================================
    // 2. Notifications & Delivery
    // ==========================================

    pub async fn send_notification(&self, input: SendNotificationInput) -> Result<NotificationDTO> {
        let provider = self.find_provider_for_channel(&input.channel)?;

        // Render subject and body
        let (subject, body) = if let Some(ref template_name) = input.template {
            let tmpl = self.get_template_by_name(template_name).await?;
            let subj = tmpl
                .subject_template
                .as_ref()
                .map(|s| Self::render_text(s, input.data.as_ref()));
            let b = Self::render_text(&tmpl.body_template, input.data.as_ref());
            (subj, b)
        } else {
            (
                input.subject.clone(),
                input.body.clone().unwrap_or_default(),
            )
        };

        let notification_id = Id::new();
        let now = Utc::now();

        // Attempt dispatch via provider
        let send_res = provider
            .send(
                &input.to,
                &input.channel,
                subject.as_deref(),
                &body,
                input.data.as_ref(),
            )
            .await;

        let (status, error_msg, meta) = match send_res {
            Ok(res) => ("sent".to_string(), None, res.metadata),
            Err(e) => ("failed".to_string(), Some(e.to_string()), None),
        };

        let notification = Notification {
            id: notification_id,
            to: input.to,
            channel: input.channel,
            template: input.template,
            data: input.data,
            trigger_type: input.trigger_type,
            resource_id: input.resource_id,
            resource_type: input.resource_type,
            receiver_id: input.receiver_id,
            status: status.clone(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let delivery = NotificationDelivery {
            id: Id::new(),
            notification_id: notification_id.to_string(),
            provider_id: provider.provider_id().to_string(),
            delivered_at: if status == "sent" {
                Some(Utc::now())
            } else {
                None
            },
            error_message: error_msg,
            metadata: meta,
            created_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.notifications.create(None, notification).await?;
        self.deliveries.create(None, delivery).await?;

        Ok(NotificationDTO {
            id: created.id.to_string(),
            to: created.to,
            channel: created.channel,
            template: created.template,
            data: created.data,
            trigger_type: created.trigger_type,
            resource_id: created.resource_id,
            resource_type: created.resource_type,
            receiver_id: created.receiver_id,
            status: created.status,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    pub async fn get_notification(&self, id: &str) -> Result<NotificationDTO> {
        let n = self.notifications.retrieve(None, id).await?;

        Ok(NotificationDTO {
            id: n.id.to_string(),
            to: n.to,
            channel: n.channel,
            template: n.template,
            data: n.data,
            trigger_type: n.trigger_type,
            resource_id: n.resource_id,
            resource_type: n.resource_type,
            receiver_id: n.receiver_id,
            status: n.status,
            created_at: n.created_at,
            updated_at: n.updated_at,
        })
    }

    pub async fn list_deliveries(
        &self,
        notification_id: &str,
    ) -> Result<Vec<NotificationDeliveryDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("notification_id", FilterOp::Eq(notification_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.deliveries.list(None, &filter).await?;
        Ok(list
            .into_iter()
            .map(|d| NotificationDeliveryDTO {
                id: d.id.to_string(),
                notification_id: d.notification_id,
                provider_id: d.provider_id,
                delivered_at: d.delivered_at,
                error_message: d.error_message,
                metadata: d.metadata,
                created_at: d.created_at,
            })
            .collect())
    }
}

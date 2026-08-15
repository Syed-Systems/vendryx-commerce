use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::Utc;
use stheno_types::{Error, Id, Result};

use crate::dto::{
    CreateTemplateInput, NotificationDTO, NotificationDeliveryDTO, NotificationTemplateDTO,
    SendNotificationInput,
};
use crate::models::{Notification, NotificationDelivery, NotificationTemplate};
use crate::provider::NotificationProvider;

pub struct NotificationService {
    providers: RwLock<HashMap<String, Arc<dyn NotificationProvider>>>,
    templates: RwLock<HashMap<String, NotificationTemplate>>,
    notifications: RwLock<HashMap<String, Notification>>,
    deliveries: RwLock<Vec<NotificationDelivery>>,
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            templates: RwLock::new(HashMap::new()),
            notifications: RwLock::new(HashMap::new()),
            deliveries: RwLock::new(Vec::new()),
        }
    }

    pub fn register_provider(&self, provider: Arc<dyn NotificationProvider>) {
        let mut map = self.providers.write().unwrap();
        map.insert(provider.provider_id().to_string(), provider);
    }

    fn find_provider_for_channel(&self, channel: &str) -> Result<Arc<dyn NotificationProvider>> {
        let map = self.providers.read().unwrap();
        for provider in map.values() {
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
        };

        let dto = NotificationTemplateDTO {
            id: id.to_string(),
            name: template.name.clone(),
            channel: template.channel.clone(),
            subject_template: template.subject_template.clone(),
            body_template: template.body_template.clone(),
            created_at: template.created_at,
            updated_at: template.updated_at,
        };

        {
            let mut map = self.templates.write().unwrap();
            map.insert(template.name.clone(), template);
        }

        Ok(dto)
    }

    pub async fn get_template_by_name(&self, name: &str) -> Result<NotificationTemplateDTO> {
        let map = self.templates.read().unwrap();
        let t = map.get(name).ok_or_else(|| Error::NotFound {
            entity: "notification_template",
            id: name.to_string(),
        })?;

        Ok(NotificationTemplateDTO {
            id: t.id.to_string(),
            name: t.name.clone(),
            channel: t.channel.clone(),
            subject_template: t.subject_template.clone(),
            body_template: t.body_template.clone(),
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
        };

        let dto = NotificationDTO {
            id: notification_id.to_string(),
            to: notification.to.clone(),
            channel: notification.channel.clone(),
            template: notification.template.clone(),
            data: notification.data.clone(),
            trigger_type: notification.trigger_type.clone(),
            resource_id: notification.resource_id.clone(),
            resource_type: notification.resource_type.clone(),
            receiver_id: notification.receiver_id.clone(),
            status: notification.status.clone(),
            created_at: notification.created_at,
            updated_at: notification.updated_at,
        };

        {
            let mut n_map = self.notifications.write().unwrap();
            n_map.insert(notification_id.to_string(), notification);
        }

        {
            let mut d_list = self.deliveries.write().unwrap();
            d_list.push(delivery);
        }

        Ok(dto)
    }

    pub async fn get_notification(&self, id: &str) -> Result<NotificationDTO> {
        let map = self.notifications.read().unwrap();
        let n = map.get(id).ok_or_else(|| Error::NotFound {
            entity: "notification",
            id: id.to_string(),
        })?;

        Ok(NotificationDTO {
            id: n.id.to_string(),
            to: n.to.clone(),
            channel: n.channel.clone(),
            template: n.template.clone(),
            data: n.data.clone(),
            trigger_type: n.trigger_type.clone(),
            resource_id: n.resource_id.clone(),
            resource_type: n.resource_type.clone(),
            receiver_id: n.receiver_id.clone(),
            status: n.status.clone(),
            created_at: n.created_at,
            updated_at: n.updated_at,
        })
    }

    pub async fn list_deliveries(
        &self,
        notification_id: &str,
    ) -> Result<Vec<NotificationDeliveryDTO>> {
        let list = self.deliveries.read().unwrap();
        let filtered: Vec<NotificationDeliveryDTO> = list
            .iter()
            .filter(|d| d.notification_id == notification_id)
            .map(|d| NotificationDeliveryDTO {
                id: d.id.to_string(),
                notification_id: d.notification_id.clone(),
                provider_id: d.provider_id.clone(),
                delivered_at: d.delivered_at,
                error_message: d.error_message.clone(),
                metadata: d.metadata.clone(),
                created_at: d.created_at,
            })
            .collect();
        Ok(filtered)
    }
}

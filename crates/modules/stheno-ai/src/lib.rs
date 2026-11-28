//! Tenant-owned static knowledge for the social commerce assistant (ADR-032).
//!
//! This module stores merchant-authored policy and FAQ material only. Conversation
//! messages, customer records, live inventory, and prices do not belong in this corpus.

mod configuration;
mod conversation;
mod dto;
mod events;
mod llm;
mod models;
mod offers;
mod service;
mod store_intelligence;
mod store_intelligence_review;
mod store_intelligence_sources;
mod tenant_data;

pub use configuration::{
    normalize_search_console_property, AiConfigurationService, AiConfigurationStatus,
    AiProviderRuntimeConfig, GoogleSearchConsoleRuntimeConfig, MetaRuntimeConfig,
    SaveAiConfiguration, SourceConnectionAudit, WhatsAppSenderRuntimeConfig,
};
pub use conversation::{ConversationMemoryService, StoredConversationTurn};
pub use dto::{CreateKnowledgeEntry, KnowledgeEntry, KnowledgeSearchHit, UpdateKnowledgeEntry};
pub use events::WebhookEventService;
pub use llm::{
    AiChatMessage, AiChatRequest, AiChatResponse, AiToolCall, AiToolDefinition, AiUsage,
    ChatCompletionProvider,
};
pub use offers::{ChatOffer, ChatOfferQuote, ChatOfferService, SaveChatOffer};
pub use service::KnowledgeService;
pub use store_intelligence::{
    build_store_intelligence_report, rank_store_options, CategoryTrend, KnowledgeEvidenceReference,
    KnowledgeEvidenceSnippet, ProductTrend, RankedStoreOption, SignalConfidence,
    StoreCategoryObservation, StoreIntelligenceCitation, StoreIntelligenceRanking,
    StoreIntelligenceReport, StoreOptionRecommendation, StoreOrderObservation, TrendDirection,
};
pub use store_intelligence_review::{
    StoreIntelligenceReview, StoreIntelligenceReviewAudit, StoreIntelligenceReviewService,
};
pub use store_intelligence_sources::{
    aggregate_compatible_source_observations, compare_adjacent_source_observations,
    compare_adjacent_source_snapshots, CompatibleSourceMetricAggregate, MetricCoverage,
    SourceMetricObservation, StoreIntelligenceSourceChange, StoreIntelligenceSourceService,
    StoreIntelligenceSourceSnapshot,
};
pub use tenant_data::StoreIntelligenceTenantDataService;

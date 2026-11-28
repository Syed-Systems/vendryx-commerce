-- ADR-032 phase 32.6: owner-authorized per-variant chat prices and bound short-lived quotes.
CREATE TABLE ai.chat_offer (
    id                  text          PRIMARY KEY,
    tenant_id           text          NOT NULL DEFAULT current_setting('app.tenant_id', true),
    variant_id          text          NOT NULL,
    currency_code       text          NOT NULL CHECK (currency_code ~ '^[A-Z]{3}$'),
    chat_unit_price     numeric(20,6) NOT NULL CHECK (chat_unit_price > 0),
    max_quantity        integer       NOT NULL DEFAULT 1 CHECK (max_quantity BETWEEN 1 AND 1000),
    starts_at           timestamptz,
    ends_at             timestamptz,
    enabled             boolean       NOT NULL DEFAULT false,
    version             integer       NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at          timestamptz   NOT NULL DEFAULT now(),
    updated_at          timestamptz   NOT NULL DEFAULT now(),
    deleted_at          timestamptz,
    CHECK (starts_at IS NULL OR ends_at IS NULL OR starts_at < ends_at)
);

ALTER TABLE ai.chat_offer ADD CONSTRAINT ai_chat_offer_tenant_id_id_unique UNIQUE (tenant_id, id);

-- serves: resolve one active chat offer by published variant, currency, and validity window
CREATE UNIQUE INDEX ai_chat_offer_one_enabled_variant_currency
    ON ai.chat_offer (tenant_id, variant_id, currency_code)
    WHERE enabled AND deleted_at IS NULL;
CREATE INDEX ai_chat_offer_tenant_updated
    ON ai.chat_offer (tenant_id, updated_at DESC, id)
    WHERE deleted_at IS NULL;

CREATE TABLE ai.chat_offer_quote (
    id                  text          PRIMARY KEY,
    tenant_id           text          NOT NULL DEFAULT current_setting('app.tenant_id', true),
    offer_id            text          NOT NULL,
    offer_version       integer       NOT NULL CHECK (offer_version > 0),
    conversation_key    text          NOT NULL,
    variant_id          text          NOT NULL,
    quantity            integer       NOT NULL CHECK (quantity BETWEEN 1 AND 1000),
    currency_code       text          NOT NULL CHECK (currency_code ~ '^[A-Z]{3}$'),
    region_id           text,
    live_unit_price     numeric(20,6) NOT NULL CHECK (live_unit_price > 0),
    chat_unit_price     numeric(20,6) NOT NULL CHECK (chat_unit_price > 0 AND chat_unit_price < live_unit_price),
    issued_at           timestamptz   NOT NULL DEFAULT now(),
    expires_at          timestamptz   NOT NULL,
    claimed_event_id    text,
    CHECK (expires_at > issued_at)
);

-- Keep quote and audit ownership aligned even if application scoping regresses.
ALTER TABLE ai.chat_offer_quote
    ADD CONSTRAINT ai_chat_offer_quote_offer_tenant_fk
    FOREIGN KEY (tenant_id, offer_id) REFERENCES ai.chat_offer (tenant_id, id);

CREATE TABLE ai.chat_offer_audit (
    id              text        PRIMARY KEY,
    tenant_id       text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    actor_id        text        NOT NULL,
    offer_id        text        NOT NULL,
    action          text        NOT NULL CHECK (action IN ('saved', 'disabled')),
    offer_version   integer     NOT NULL CHECK (offer_version > 0),
    created_at      timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY (tenant_id, offer_id) REFERENCES ai.chat_offer (tenant_id, id)
);

-- serves: retrieve owner changes to chat prices by tenant and time
CREATE INDEX ai_chat_offer_audit_tenant_created
    ON ai.chat_offer_audit (tenant_id, created_at DESC);

-- serves: validate a short-lived quote by tenant, conversation, line, and event before draft creation
CREATE INDEX ai_chat_offer_quote_expiry
    ON ai.chat_offer_quote (tenant_id, expires_at);
CREATE INDEX ai_chat_offer_quote_conversation
    ON ai.chat_offer_quote (tenant_id, conversation_key, issued_at DESC);
-- serves: locate all consumed quotes for one event during staff draft confirmation
CREATE INDEX ai_chat_offer_quote_claimed_event
    ON ai.chat_offer_quote (tenant_id, claimed_event_id)
    WHERE claimed_event_id IS NOT NULL;

ALTER TABLE ai.chat_offer ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.chat_offer FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ai.chat_offer
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
ALTER TABLE ai.chat_offer_quote ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.chat_offer_quote FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ai.chat_offer_quote
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
ALTER TABLE ai.chat_offer_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.chat_offer_audit FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ai.chat_offer_audit
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
GRANT SELECT, INSERT, UPDATE, DELETE ON ai.chat_offer, ai.chat_offer_quote TO stheno_app;
GRANT SELECT, INSERT ON ai.chat_offer_audit TO stheno_app;

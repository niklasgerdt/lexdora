-- DORA EU (Regulation (EU) 2022/2554) Database Schema (PostgreSQL)
-- Purpose: Provide a comprehensive, normalized schema to support governance,
-- ICT asset/inventory, risk management, incident management & reporting,
-- resilience testing, third‑party risk, information sharing, compliance/audit,
-- metrics, and workflow as required for digital operational resilience.
-- Notes:
-- - Target database: PostgreSQL 13+
-- - Adjust ownership, tablespaces, and extensions as needed.
-- - This file creates a dedicated schema `dora` and all objects under it.
-- - This schema is intentionally generic and can be integrated with existing IAM.

-- Recommended extensions
CREATE EXTENSION IF NOT EXISTS pgcrypto; -- for gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS citext;   -- case-insensitive text for emails

CREATE SCHEMA IF NOT EXISTS dora;
SET search_path = dora, public;

-- =============================
-- Section 0: Enumerated types
-- =============================

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'severity_level') THEN
        CREATE TYPE severity_level AS ENUM ('low', 'medium', 'high', 'critical');
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'likelihood_level') THEN
        CREATE TYPE likelihood_level AS ENUM ('rare', 'unlikely', 'possible', 'likely', 'almost_certain');
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'status_type') THEN
        CREATE TYPE status_type AS ENUM ('draft', 'planned', 'in_progress', 'blocked', 'completed', 'closed', 'cancelled');
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'incident_severity') THEN
        CREATE TYPE incident_severity AS ENUM ('minor', 'moderate', 'major', 'severe');
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'incident_type') THEN
        CREATE TYPE incident_type AS ENUM (
            'service_disruption', 'security_breach', 'data_breach', 'degradation', 'malware', 'ddos', 'fraud', 'other'
        );
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'control_type') THEN
        CREATE TYPE control_type AS ENUM ('preventive', 'detective', 'corrective');
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'test_type') THEN
        CREATE TYPE test_type AS ENUM ('tabletop', 'technical', 'red_team', 'scenario', 'backup_restore', 'dr_failover', 'other');
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'report_type') THEN
        CREATE TYPE report_type AS ENUM ('initial', 'intermediate', 'final', 'ad_hoc');
    END IF;
END $$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'criticality_level') THEN
        CREATE TYPE criticality_level AS ENUM ('non_critical', 'important', 'critical');
    END IF;
END $$;

-- =============================
-- Section 1: Governance & Org
-- =============================

CREATE TABLE IF NOT EXISTS organizations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL UNIQUE,
    legal_entity_id TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS business_units (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    description    TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, name)
);

CREATE TABLE IF NOT EXISTS roles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE IF NOT EXISTS permissions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id       UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email           CITEXT NOT NULL UNIQUE,
    full_name       TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE IF NOT EXISTS regulators (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    jurisdiction TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS regulatory_obligations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code            TEXT NOT NULL, -- e.g., DORA-ART17-1
    title           TEXT NOT NULL,
    description     TEXT,
    regulator_id    UUID REFERENCES regulators(id) ON DELETE SET NULL,
    effective_date  DATE,
    UNIQUE (code)
);

-- =============================
-- Section 2: Asset Inventory
-- =============================

CREATE TABLE IF NOT EXISTS ict_assets (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id  UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name             TEXT NOT NULL,
    asset_type       TEXT NOT NULL, -- server, app, db, network_device, cloud_service
    owner_user_id    UUID REFERENCES users(id) ON DELETE SET NULL,
    business_unit_id UUID REFERENCES business_units(id) ON DELETE SET NULL,
    criticality      criticality_level NOT NULL DEFAULT 'non_critical',
    is_important     BOOLEAN NOT NULL DEFAULT false, -- DORA: important function support
    description      TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, name)
);

CREATE TABLE IF NOT EXISTS data_assets (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id  UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name             TEXT NOT NULL,
    data_category    TEXT NOT NULL, -- PII, payment, log, secret, etc.
    classification   TEXT NOT NULL, -- public, internal, confidential, secret
    owner_user_id    UUID REFERENCES users(id) ON DELETE SET NULL,
    business_unit_id UUID REFERENCES business_units(id) ON DELETE SET NULL,
    criticality      criticality_level NOT NULL DEFAULT 'non_critical',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, name)
);

CREATE TABLE IF NOT EXISTS services (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id  UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name             TEXT NOT NULL,
    description      TEXT,
    is_important     BOOLEAN NOT NULL DEFAULT false, -- Important/critical functions
    owner_user_id    UUID REFERENCES users(id) ON DELETE SET NULL,
    business_unit_id UUID REFERENCES business_units(id) ON DELETE SET NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (organization_id, name)
);

CREATE TABLE IF NOT EXISTS configurations (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ict_asset_id UUID NOT NULL REFERENCES ict_assets(id) ON DELETE CASCADE,
    key          TEXT NOT NULL,
    value        TEXT NOT NULL,
    version      TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (ict_asset_id, key)
);

CREATE TABLE IF NOT EXISTS asset_relationships (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_asset_id UUID NOT NULL REFERENCES ict_assets(id) ON DELETE CASCADE,
    child_asset_id  UUID NOT NULL REFERENCES ict_assets(id) ON DELETE CASCADE,
    relation_type   TEXT NOT NULL, -- depends_on, hosts, connects_to, etc.
    UNIQUE (parent_asset_id, child_asset_id, relation_type),
    CHECK (parent_asset_id <> child_asset_id)
);

-- =============================
-- Section 3: Risk Management
-- =============================

CREATE TABLE IF NOT EXISTS control_library (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code          TEXT NOT NULL UNIQUE, -- e.g., CTRL-IA-001
    title         TEXT NOT NULL,
    description   TEXT,
    control_type  control_type NOT NULL,
    source        TEXT -- policy/standard mapping
);

CREATE TABLE IF NOT EXISTS risk_register (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id  UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    title            TEXT NOT NULL,
    description      TEXT,
    inherent_impact  severity_level NOT NULL DEFAULT 'medium',
    inherent_likelihood likelihood_level NOT NULL DEFAULT 'possible',
    residual_impact  severity_level,
    residual_likelihood likelihood_level,
    status           status_type NOT NULL DEFAULT 'draft',
    owner_user_id    UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS risk_assets (
    risk_id     UUID NOT NULL REFERENCES risk_register(id) ON DELETE CASCADE,
    ict_asset_id UUID REFERENCES ict_assets(id) ON DELETE CASCADE,
    data_asset_id UUID REFERENCES data_assets(id) ON DELETE CASCADE,
    service_id   UUID REFERENCES services(id) ON DELETE CASCADE,
    PRIMARY KEY (risk_id, ict_asset_id, data_asset_id, service_id)
);

CREATE TABLE IF NOT EXISTS asset_controls (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    control_id   UUID NOT NULL REFERENCES control_library(id) ON DELETE CASCADE,
    ict_asset_id UUID REFERENCES ict_assets(id) ON DELETE CASCADE,
    data_asset_id UUID REFERENCES data_assets(id) ON DELETE CASCADE,
    service_id   UUID REFERENCES services(id) ON DELETE CASCADE,
    implemented  BOOLEAN NOT NULL DEFAULT false,
    last_reviewed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS control_tests (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    control_id    UUID NOT NULL REFERENCES control_library(id) ON DELETE CASCADE,
    test_date     DATE NOT NULL,
    tester_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    result        TEXT NOT NULL, -- pass/fail/details
    evidence_link TEXT
);

CREATE TABLE IF NOT EXISTS risk_assessments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    risk_id         UUID NOT NULL REFERENCES risk_register(id) ON DELETE CASCADE,
    assessment_date DATE NOT NULL,
    assessed_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    impact          severity_level NOT NULL,
    likelihood      likelihood_level NOT NULL,
    notes           TEXT
);

CREATE TABLE IF NOT EXISTS risk_treatments (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    risk_id      UUID NOT NULL REFERENCES risk_register(id) ON DELETE CASCADE,
    strategy     TEXT NOT NULL, -- avoid, mitigate, transfer, accept
    plan         TEXT,
    due_date     DATE,
    status       status_type NOT NULL DEFAULT 'planned',
    owner_user_id UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS risk_exceptions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    control_id   UUID REFERENCES control_library(id) ON DELETE SET NULL,
    description  TEXT NOT NULL,
    approved_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    valid_until  DATE,
    status       status_type NOT NULL DEFAULT 'in_progress'
);

-- =============================
-- Section 4: Incident Management & Reporting
-- =============================

CREATE TABLE IF NOT EXISTS incidents (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id    UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    title              TEXT NOT NULL,
    description        TEXT,
    type               incident_type NOT NULL,
    severity           incident_severity NOT NULL,
    detected_at        TIMESTAMPTZ NOT NULL,
    started_at         TIMESTAMPTZ,
    contained_at       TIMESTAMPTZ,
    recovered_at       TIMESTAMPTZ,
    status             status_type NOT NULL DEFAULT 'in_progress',
    is_major           BOOLEAN NOT NULL DEFAULT false, -- as per DORA classification
    affected_customers INTEGER DEFAULT 0,
    created_by         UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS incident_assets (
    incident_id  UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    ict_asset_id UUID REFERENCES ict_assets(id) ON DELETE SET NULL,
    data_asset_id UUID REFERENCES data_assets(id) ON DELETE SET NULL,
    service_id   UUID REFERENCES services(id) ON DELETE SET NULL,
    PRIMARY KEY (incident_id, ict_asset_id, data_asset_id, service_id)
);

CREATE TABLE IF NOT EXISTS incident_impacts (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id   UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    impact_area   TEXT NOT NULL, -- availability, integrity, confidentiality, continuity, other
    description   TEXT,
    impact_level  severity_level NOT NULL
);

CREATE TABLE IF NOT EXISTS incident_root_causes (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id   UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    category      TEXT NOT NULL, -- human_error, third_party, vulnerability, change, capacity, etc.
    description   TEXT
);

CREATE TABLE IF NOT EXISTS corrective_actions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id   UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    action        TEXT NOT NULL,
    owner_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    due_date      DATE,
    status        status_type NOT NULL DEFAULT 'planned',
    completed_at  TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS incident_evidence (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id   UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    label         TEXT NOT NULL,
    link          TEXT NOT NULL,
    added_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    added_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS communications (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id   UUID REFERENCES incidents(id) ON DELETE CASCADE,
    audience      TEXT NOT NULL, -- internal, customers, media, regulator, tpp
    channel       TEXT NOT NULL, -- email, phone, portal
    content       TEXT NOT NULL,
    sent_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_by       UUID REFERENCES users(id) ON DELETE SET NULL
);

-- DORA authority reporting (initial/intermediate/final within deadlines)
CREATE TABLE IF NOT EXISTS authority_reports (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id      UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    regulator_id     UUID REFERENCES regulators(id) ON DELETE SET NULL,
    report_type      report_type NOT NULL,
    report_reference TEXT, -- regulator-assigned id
    submitted_at     TIMESTAMPTZ,
    due_at           TIMESTAMPTZ, -- computed by policy per DORA deadlines
    content          TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS post_incident_reviews (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id   UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    conducted_at  DATE NOT NULL,
    led_by        UUID REFERENCES users(id) ON DELETE SET NULL,
    summary       TEXT,
    lessons_learned TEXT
);

-- =============================
-- Section 5: Vulnerabilities & Findings
-- =============================

CREATE TABLE IF NOT EXISTS vulnerability_scans (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    tool_name       TEXT NOT NULL,
    executed_at     TIMESTAMPTZ NOT NULL,
    scope           TEXT,
    executed_by     UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS vulnerabilities (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_id       UUID REFERENCES vulnerability_scans(id) ON DELETE SET NULL,
    cve_id        TEXT,
    title         TEXT NOT NULL,
    description   TEXT,
    severity      severity_level NOT NULL,
    vector        TEXT
);

CREATE TABLE IF NOT EXISTS findings (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source        TEXT NOT NULL, -- scan, audit, test
    title         TEXT NOT NULL,
    description   TEXT,
    severity      severity_level NOT NULL,
    ict_asset_id  UUID REFERENCES ict_assets(id) ON DELETE SET NULL,
    data_asset_id UUID REFERENCES data_assets(id) ON DELETE SET NULL,
    service_id    UUID REFERENCES services(id) ON DELETE SET NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    status        status_type NOT NULL DEFAULT 'in_progress'
);

CREATE TABLE IF NOT EXISTS remediation_tasks (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    finding_id    UUID REFERENCES findings(id) ON DELETE CASCADE,
    title         TEXT NOT NULL,
    owner_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    due_date      DATE,
    status        status_type NOT NULL DEFAULT 'planned',
    completed_at  TIMESTAMPTZ
);

-- =============================
-- Section 6: Continuity & Resilience Testing
-- =============================

CREATE TABLE IF NOT EXISTS bcp_plans (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    version         TEXT,
    approved_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_bcp_plans_unique ON bcp_plans (organization_id, name, COALESCE(version, ''));

CREATE TABLE IF NOT EXISTS dr_plans (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    version         TEXT,
    rto_minutes     INTEGER,
    rpo_minutes     INTEGER,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_dr_plans_unique ON dr_plans (organization_id, name, COALESCE(version, ''));

CREATE TABLE IF NOT EXISTS resilience_tests (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    test_type       test_type NOT NULL,
    scenario        TEXT,
    planned_date    DATE,
    executed_date   DATE,
    status          status_type NOT NULL DEFAULT 'planned',
    lead_user_id    UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS test_results (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    test_id         UUID NOT NULL REFERENCES resilience_tests(id) ON DELETE CASCADE,
    outcome         TEXT NOT NULL, -- pass/fail/partial
    issues_found    INTEGER NOT NULL DEFAULT 0,
    summary         TEXT,
    evidence_link   TEXT
);

CREATE TABLE IF NOT EXISTS lessons_learned (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_type  TEXT NOT NULL, -- test or incident
    source_id    UUID NOT NULL, -- references resilience_tests.id or incidents.id
    description  TEXT NOT NULL,
    action_taken TEXT
);

-- =============================
-- Section 7: Third‑Party Risk Management (TPRM)
-- =============================

CREATE TABLE IF NOT EXISTS third_parties (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    country         TEXT,
    criticality     criticality_level NOT NULL DEFAULT 'non_critical',
    is_important    BOOLEAN NOT NULL DEFAULT false, -- supports important function
    UNIQUE (organization_id, name)
);

CREATE TABLE IF NOT EXISTS tpp_services (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    third_party_id UUID NOT NULL REFERENCES third_parties(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    description    TEXT
);

CREATE TABLE IF NOT EXISTS tpp_contracts (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    third_party_id UUID NOT NULL REFERENCES third_parties(id) ON DELETE CASCADE,
    service_id     UUID REFERENCES tpp_services(id) ON DELETE SET NULL,
    start_date     DATE NOT NULL,
    end_date       DATE,
    notice_period_days INTEGER,
    exit_strategy  TEXT
);

CREATE TABLE IF NOT EXISTS tpp_risks (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    third_party_id UUID NOT NULL REFERENCES third_parties(id) ON DELETE CASCADE,
    title          TEXT NOT NULL,
    description    TEXT,
    impact         severity_level NOT NULL,
    likelihood     likelihood_level NOT NULL,
    status         status_type NOT NULL DEFAULT 'in_progress'
);

CREATE TABLE IF NOT EXISTS tpp_assessments (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    third_party_id UUID NOT NULL REFERENCES third_parties(id) ON DELETE CASCADE,
    assessment_date DATE NOT NULL,
    assessed_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    methodology    TEXT,
    overall_result TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tpp_incidents (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    third_party_id UUID NOT NULL REFERENCES third_parties(id) ON DELETE CASCADE,
    incident_id    UUID REFERENCES incidents(id) ON DELETE SET NULL,
    description    TEXT,
    severity       incident_severity NOT NULL,
    reported_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS subcontractors (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    third_party_id   UUID NOT NULL REFERENCES third_parties(id) ON DELETE CASCADE,
    name             TEXT NOT NULL,
    country          TEXT,
    criticality      criticality_level NOT NULL DEFAULT 'non_critical'
);

CREATE TABLE IF NOT EXISTS concentration_risk (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    theme           TEXT NOT NULL, -- e.g., cloud provider region, single vendor
    description     TEXT,
    assessed_at     DATE NOT NULL,
    risk_level      severity_level NOT NULL
);

-- =============================
-- Section 8: Information Sharing
-- =============================

CREATE TABLE IF NOT EXISTS intel_feeds (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    provider        TEXT,
    tlp             TEXT, -- traffic light protocol
    UNIQUE (organization_id, name)
);

CREATE TABLE IF NOT EXISTS shared_indicators (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feed_id     UUID REFERENCES intel_feeds(id) ON DELETE SET NULL,
    indicator   TEXT NOT NULL,
    type        TEXT NOT NULL, -- ip, domain, hash, url, yara
    tlp         TEXT,
    first_seen  TIMESTAMPTZ,
    last_seen   TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS sharing_agreements (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    partner_name    TEXT NOT NULL,
    scope           TEXT,
    tlp_default     TEXT,
    signed_at       DATE
);

CREATE TABLE IF NOT EXISTS shared_incidents (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_org      TEXT NOT NULL,
    title           TEXT NOT NULL,
    description     TEXT,
    tlp             TEXT,
    shared_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- =============================
-- Section 9: Compliance, Audit, Evidence, Training
-- =============================

CREATE TABLE IF NOT EXISTS policies (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    version         TEXT,
    approved_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at     TIMESTAMPTZ
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_policies_unique ON policies (organization_id, name, COALESCE(version, ''));

CREATE TABLE IF NOT EXISTS standards (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    version         TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_standards_unique ON standards (organization_id, name, COALESCE(version, ''));

CREATE TABLE IF NOT EXISTS procedures (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    version         TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_procedures_unique ON procedures (organization_id, name, COALESCE(version, ''));

CREATE TABLE IF NOT EXISTS trainings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    description     TEXT,
    required_for_role UUID REFERENCES roles(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS user_trainings (
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    training_id   UUID NOT NULL REFERENCES trainings(id) ON DELETE CASCADE,
    completed_at  TIMESTAMPTZ,
    PRIMARY KEY (user_id, training_id)
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    actor_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action        TEXT NOT NULL,
    entity_type   TEXT NOT NULL,
    entity_id     UUID,
    at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    details       JSONB
);

CREATE TABLE IF NOT EXISTS controls_evidence (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    control_id    UUID NOT NULL REFERENCES control_library(id) ON DELETE CASCADE,
    label         TEXT NOT NULL,
    link          TEXT NOT NULL,
    collected_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS attestations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    period_start    DATE NOT NULL,
    period_end      DATE NOT NULL,
    signed_by       UUID REFERENCES users(id) ON DELETE SET NULL,
    signed_at       TIMESTAMPTZ
);

-- =============================
-- Section 10: Metrics (KPIs/KRIs) & Alerts
-- =============================

CREATE TABLE IF NOT EXISTS metrics_catalog (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    key             TEXT NOT NULL, -- e.g., incidents.mttr_hours
    name            TEXT NOT NULL,
    description     TEXT,
    unit            TEXT,
    UNIQUE (organization_id, key)
);

CREATE TABLE IF NOT EXISTS metric_values (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_id    UUID NOT NULL REFERENCES metrics_catalog(id) ON DELETE CASCADE,
    ts           TIMESTAMPTZ NOT NULL,
    value        NUMERIC NOT NULL,
    dimensions   JSONB -- optional labels
);

CREATE TABLE IF NOT EXISTS metric_thresholds (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_id    UUID NOT NULL REFERENCES metrics_catalog(id) ON DELETE CASCADE,
    operator     TEXT NOT NULL, -- >, >=, <, <=, ==
    threshold    NUMERIC NOT NULL,
    severity     severity_level NOT NULL
);

CREATE TABLE IF NOT EXISTS alerts (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_id      UUID REFERENCES metrics_catalog(id) ON DELETE SET NULL,
    triggered_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    severity       severity_level NOT NULL,
    message        TEXT NOT NULL,
    acknowledged_by UUID REFERENCES users(id) ON DELETE SET NULL,
    acknowledged_at TIMESTAMPTZ
);

-- =============================
-- Section 11: Workflow (Tasks, Approvals, Notifications)
-- =============================

CREATE TABLE IF NOT EXISTS tasks (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title         TEXT NOT NULL,
    description   TEXT,
    status        status_type NOT NULL DEFAULT 'planned',
    assignee_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    due_date      DATE,
    related_type  TEXT, -- incident, risk, test, finding, etc.
    related_id    UUID
);

CREATE TABLE IF NOT EXISTS approvals (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id       UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    approver_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    decision      TEXT NOT NULL, -- approved/rejected
    decided_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    note          TEXT
);

CREATE TABLE IF NOT EXISTS notifications (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID REFERENCES users(id) ON DELETE CASCADE,
    channel       TEXT NOT NULL, -- email, webhook
    content       TEXT NOT NULL,
    sent_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS subscriptions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID REFERENCES users(id) ON DELETE CASCADE,
    topic         TEXT NOT NULL, -- incidents.major, tpp.changes, metrics.alerts
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, topic)
);

-- =============================
-- Section 12: Helpful indexes
-- =============================

CREATE INDEX IF NOT EXISTS idx_incidents_status ON incidents(status);
CREATE INDEX IF NOT EXISTS idx_incidents_major ON incidents(is_major);
CREATE INDEX IF NOT EXISTS idx_incidents_detected_at ON incidents(detected_at);
CREATE INDEX IF NOT EXISTS idx_findings_severity ON findings(severity);
CREATE INDEX IF NOT EXISTS idx_metric_values_ts ON metric_values(ts);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

-- =============================
-- Section 13: Minimal seed examples (optional)
-- Commented out; adjust as needed.
-- INSERT INTO organizations(name) VALUES ('Example Financial');

-- How to apply:
-- psql -h <host> -U <user> -d <db> -f schema/dora_schema.sql

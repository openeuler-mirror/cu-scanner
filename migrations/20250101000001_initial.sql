-- Initial schema for cu-scanner
-- All tables matching design doc Section 5.3

CREATE TABLE IF NOT EXISTS csaf_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    csaf_id VARCHAR(64) UNIQUE NOT NULL,
    file_name VARCHAR(256) NOT NULL,
    title TEXT,
    category VARCHAR(32),
    severity VARCHAR(16),
    release_date TIMESTAMP,
    csaf_version VARCHAR(8),
    tracking_version VARCHAR(16),
    download_url TEXT,
    downloaded_at TIMESTAMP,
    parsed_at TIMESTAMP,
    oval_numeric_id VARCHAR(16),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS oval_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    csaf_id VARCHAR(64) NOT NULL,
    oval_id VARCHAR(128) NOT NULL,
    class VARCHAR(16) NOT NULL DEFAULT 'patch',
    version INTEGER NOT NULL DEFAULT 1,
    title TEXT,
    description TEXT,
    family VARCHAR(16) DEFAULT 'unix',
    platform VARCHAR(128),
    severity VARCHAR(16),
    issued_date DATE,
    updated_date DATE,
    rights TEXT,
    advisory_from VARCHAR(128),
    generator_timestamp TIMESTAMP,
    content_version BIGINT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(oval_id, version)
);

CREATE TABLE IF NOT EXISTS oval_references (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id INTEGER NOT NULL,
    ref_id VARCHAR(64),
    ref_url TEXT,
    source VARCHAR(16),
    FOREIGN KEY (definition_id) REFERENCES oval_definitions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS oval_cves (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id INTEGER NOT NULL,
    cve_id VARCHAR(32),
    cvss3 VARCHAR(64),
    impact VARCHAR(16),
    href TEXT,
    public_date DATE,
    FOREIGN KEY (definition_id) REFERENCES oval_definitions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS oval_cpes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id INTEGER NOT NULL,
    cpe VARCHAR(256),
    FOREIGN KEY (definition_id) REFERENCES oval_definitions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS oval_tests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    oval_id VARCHAR(128) UNIQUE NOT NULL,
    definition_id INTEGER NOT NULL,
    test_type VARCHAR(32),
    check_expr VARCHAR(32),
    comment TEXT,
    version INTEGER DEFAULT 1,
    object_ref VARCHAR(128),
    state_ref VARCHAR(128),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (definition_id) REFERENCES oval_definitions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS oval_objects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    oval_id VARCHAR(128) UNIQUE NOT NULL,
    object_type VARCHAR(32),
    name VARCHAR(128),
    rpm_version VARCHAR(128),
    filepath TEXT,
    version INTEGER DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS oval_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    oval_id VARCHAR(128) UNIQUE NOT NULL,
    state_type VARCHAR(32),
    evr VARCHAR(128),
    evr_operation VARCHAR(16),
    signature_keyid VARCHAR(64),
    name_pattern VARCHAR(128),
    version_pattern VARCHAR(128),
    version INTEGER DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS oval_criteria (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    definition_id INTEGER NOT NULL,
    parent_id INTEGER,
    operator VARCHAR(8),
    criterion_test_ref VARCHAR(128),
    criterion_comment TEXT,
    sequence INTEGER DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (definition_id) REFERENCES oval_definitions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS download_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_name VARCHAR(256) NOT NULL,
    source_url TEXT,
    status VARCHAR(16) DEFAULT 'pending',
    attempt_count INTEGER DEFAULT 0,
    error_message TEXT,
    sync_batch_id VARCHAR(64),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(64) UNIQUE NOT NULL,
    password_hash VARCHAR(128) NOT NULL,
    role VARCHAR(16) NOT NULL DEFAULT 'user',
    status VARCHAR(16) DEFAULT 'active',
    failed_attempts INTEGER DEFAULT 0,
    locked_until TIMESTAMP,
    last_login_at TIMESTAMP,
    must_change_password BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS rpm_epoch_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pkg_name VARCHAR(128) NOT NULL,
    repo_id VARCHAR(128) NOT NULL,
    epoch VARCHAR(16) NOT NULL,
    source VARCHAR(16),
    resolved_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(pkg_name, repo_id)
);

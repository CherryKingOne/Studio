CREATE TABLE IF NOT EXISTS files (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    bucket VARCHAR(64) NOT NULL,
    object_key VARCHAR(512) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    mime VARCHAR(128) NOT NULL,
    size_bytes BIGINT UNSIGNED NOT NULL,
    sha256 CHAR(64) NOT NULL,
    kind VARCHAR(24) NOT NULL,
    duration_ms INT UNSIGNED NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uk_files_bucket_object (bucket, object_key),
    KEY idx_files_sha256 (sha256),
    KEY idx_files_kind (kind)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS voices (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    provider VARCHAR(32) NOT NULL,
    provider_voice_id VARCHAR(128) NOT NULL,
    display_name VARCHAR(128) NOT NULL,
    locale VARCHAR(16) NOT NULL,
    gender VARCHAR(8) NOT NULL DEFAULT 'unknown',
    styles JSON NULL,
    sample_file_id BIGINT UNSIGNED NULL,
    is_custom TINYINT(1) NOT NULL DEFAULT 0,
    enabled TINYINT(1) NOT NULL DEFAULT 1,
    extra JSON NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uk_voices_provider_voice (provider, provider_voice_id),
    KEY idx_voices_locale_gender (locale, gender),
    CONSTRAINT fk_voices_sample_file FOREIGN KEY (sample_file_id) REFERENCES files (id) ON DELETE SET NULL
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

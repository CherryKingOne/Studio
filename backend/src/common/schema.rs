use anyhow::Context;
use sqlx::MySqlPool;

/// 所有建表 SQL 内嵌在代码中，不使用外部 .sql 文件。
const MIGRATIONS: &[(&str, &str)] = &[("0001_initial", SCHEMA_V1)];

pub async fn apply(pool: &MySqlPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(64) NOT NULL,
            applied_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
            PRIMARY KEY (version)
        ) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci
        "#,
    )
    .execute(pool)
    .await
    .context("create schema_migrations failed")?;

    for (version, sql) in MIGRATIONS {
        let applied = sqlx::query_scalar::<_, u64>(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = ?",
        )
        .bind(version)
        .fetch_one(pool)
        .await
        .with_context(|| format!("check migration {version} failed"))?;

        if applied > 0 {
            continue;
        }

        for statement in sql.split(';') {
            let statement = statement.trim();
            if statement.is_empty() {
                continue;
            }
            sqlx::raw_sql(statement)
                .execute(pool)
                .await
                .with_context(|| format!("apply migration {version} failed"))?;
        }
        sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
            .bind(version)
            .execute(pool)
            .await
            .with_context(|| format!("record migration {version} failed"))?;
    }

    Ok(())
}

const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    username VARCHAR(64) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(64) NULL,
    role VARCHAR(16) NOT NULL DEFAULT 'admin',
    last_login_at DATETIME(3) NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uk_users_username (username)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

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

CREATE TABLE IF NOT EXISTS books (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    title VARCHAR(255) NOT NULL,
    author VARCHAR(128) NULL,
    description TEXT NULL,
    language VARCHAR(16) NOT NULL DEFAULT 'zh-CN',
    cover_file_id BIGINT UNSIGNED NULL,
    source_file_id BIGINT UNSIGNED NULL,
    default_narrator_voice_id BIGINT UNSIGNED NULL,
    settings JSON NOT NULL,
    status VARCHAR(24) NOT NULL DEFAULT 'draft',
    total_chapters INT UNSIGNED NOT NULL DEFAULT 0,
    total_segments INT UNSIGNED NOT NULL DEFAULT 0,
    total_duration_ms BIGINT UNSIGNED NOT NULL DEFAULT 0,
    created_by BIGINT UNSIGNED NULL,
    error_message TEXT NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    deleted_at DATETIME(3) NULL,
    PRIMARY KEY (id),
    KEY idx_books_status (status),
    KEY idx_books_created_at (created_at),
    KEY idx_books_deleted_at (deleted_at),
    CONSTRAINT fk_books_cover_file FOREIGN KEY (cover_file_id) REFERENCES files (id) ON DELETE SET NULL,
    CONSTRAINT fk_books_source_file FOREIGN KEY (source_file_id) REFERENCES files (id) ON DELETE SET NULL,
    CONSTRAINT fk_books_narrator_voice FOREIGN KEY (default_narrator_voice_id) REFERENCES voices (id) ON DELETE SET NULL,
    CONSTRAINT fk_books_created_by FOREIGN KEY (created_by) REFERENCES users (id) ON DELETE SET NULL
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS chapters (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    book_id BIGINT UNSIGNED NOT NULL,
    idx INT UNSIGNED NOT NULL,
    title VARCHAR(255) NOT NULL,
    raw_text MEDIUMTEXT NOT NULL,
    char_count INT UNSIGNED NOT NULL DEFAULT 0,
    status VARCHAR(24) NOT NULL DEFAULT 'pending',
    narrator_voice_id BIGINT UNSIGNED NULL,
    audio_file_id BIGINT UNSIGNED NULL,
    audio_duration_ms INT UNSIGNED NULL,
    audio_rev INT UNSIGNED NOT NULL DEFAULT 1,
    tts_done_segments INT UNSIGNED NOT NULL DEFAULT 0,
    error_message VARCHAR(1000) NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uk_chapters_book_idx (book_id, idx),
    KEY idx_chapters_book_status (book_id, status),
    CONSTRAINT fk_chapters_book FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE,
    CONSTRAINT fk_chapters_narrator_voice FOREIGN KEY (narrator_voice_id) REFERENCES voices (id) ON DELETE SET NULL,
    CONSTRAINT fk_chapters_audio_file FOREIGN KEY (audio_file_id) REFERENCES files (id) ON DELETE SET NULL
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS characters (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    book_id BIGINT UNSIGNED NOT NULL,
    name VARCHAR(64) NOT NULL,
    aliases JSON NOT NULL,
    gender VARCHAR(8) NOT NULL DEFAULT 'unknown',
    age_group VARCHAR(16) NOT NULL DEFAULT 'unknown',
    description TEXT NULL,
    default_voice_id BIGINT UNSIGNED NULL,
    color VARCHAR(16) NULL,
    source VARCHAR(16) NOT NULL DEFAULT 'manual',
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uk_characters_book_name (book_id, name),
    KEY idx_characters_book (book_id),
    CONSTRAINT fk_characters_book FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE,
    CONSTRAINT fk_characters_default_voice FOREIGN KEY (default_voice_id) REFERENCES voices (id) ON DELETE SET NULL
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS segments (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    chapter_id BIGINT UNSIGNED NOT NULL,
    idx INT UNSIGNED NOT NULL,
    text TEXT NOT NULL,
    char_count INT UNSIGNED NOT NULL DEFAULT 0,
    seg_type VARCHAR(16) NOT NULL DEFAULT 'narration',
    speaker_id BIGINT UNSIGNED NULL,
    speaker_confidence FLOAT NULL,
    voice_id BIGINT UNSIGNED NULL,
    emotion VARCHAR(24) NOT NULL DEFAULT 'neutral',
    rate_percent SMALLINT NOT NULL DEFAULT 0,
    pitch_hz SMALLINT NOT NULL DEFAULT 0,
    volume_percent SMALLINT NOT NULL DEFAULT 0,
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    audio_file_id BIGINT UNSIGNED NULL,
    audio_duration_ms INT UNSIGNED NULL,
    cache_key CHAR(64) NULL,
    error_message VARCHAR(500) NULL,
    attempts TINYINT UNSIGNED NOT NULL DEFAULT 0,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uk_segments_chapter_idx (chapter_id, idx),
    KEY idx_segments_chapter_status (chapter_id, status),
    KEY idx_segments_cache_key (cache_key),
    KEY idx_segments_speaker (speaker_id),
    CONSTRAINT fk_segments_chapter FOREIGN KEY (chapter_id) REFERENCES chapters (id) ON DELETE CASCADE,
    CONSTRAINT fk_segments_speaker FOREIGN KEY (speaker_id) REFERENCES characters (id) ON DELETE SET NULL,
    CONSTRAINT fk_segments_voice FOREIGN KEY (voice_id) REFERENCES voices (id) ON DELETE SET NULL,
    CONSTRAINT fk_segments_audio_file FOREIGN KEY (audio_file_id) REFERENCES files (id) ON DELETE SET NULL
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS audio_cache (
    cache_key CHAR(64) NOT NULL,
    provider VARCHAR(32) NOT NULL,
    provider_voice_id VARCHAR(128) NOT NULL,
    text_sha256 CHAR(64) NOT NULL,
    params_json JSON NOT NULL,
    file_id BIGINT UNSIGNED NOT NULL,
    duration_ms INT UNSIGNED NOT NULL,
    hit_count INT UNSIGNED NOT NULL DEFAULT 0,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    last_hit_at DATETIME(3) NULL,
    PRIMARY KEY (cache_key),
    KEY idx_audio_cache_voice (provider, provider_voice_id),
    KEY idx_audio_cache_file (file_id),
    CONSTRAINT fk_audio_cache_file FOREIGN KEY (file_id) REFERENCES files (id) ON DELETE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS jobs (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    job_type VARCHAR(32) NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'queued',
    priority TINYINT UNSIGNED NOT NULL DEFAULT 5,
    book_id BIGINT UNSIGNED NULL,
    chapter_id BIGINT UNSIGNED NULL,
    created_by BIGINT UNSIGNED NULL,
    payload JSON NOT NULL,
    result JSON NULL,
    total_items INT UNSIGNED NOT NULL DEFAULT 0,
    done_items INT UNSIGNED NOT NULL DEFAULT 0,
    failed_items INT UNSIGNED NOT NULL DEFAULT 0,
    progress TINYINT UNSIGNED NOT NULL DEFAULT 0,
    current_step VARCHAR(128) NULL,
    error_message TEXT NULL,
    attempts TINYINT UNSIGNED NOT NULL DEFAULT 0,
    max_attempts TINYINT UNSIGNED NOT NULL DEFAULT 3,
    cancel_requested TINYINT(1) NOT NULL DEFAULT 0,
    worker_id VARCHAR(64) NULL,
    scheduled_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    heartbeat_at DATETIME(3) NULL,
    started_at DATETIME(3) NULL,
    finished_at DATETIME(3) NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    KEY idx_jobs_claim (status, scheduled_at, priority, created_at),
    KEY idx_jobs_book_type_status (book_id, job_type, status),
    KEY idx_jobs_worker_status (worker_id, status),
    CONSTRAINT fk_jobs_book FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE SET NULL,
    CONSTRAINT fk_jobs_chapter FOREIGN KEY (chapter_id) REFERENCES chapters (id) ON DELETE SET NULL,
    CONSTRAINT fk_jobs_created_by FOREIGN KEY (created_by) REFERENCES users (id) ON DELETE SET NULL
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS job_items (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    job_id BIGINT UNSIGNED NOT NULL,
    item_type VARCHAR(24) NOT NULL,
    item_id BIGINT UNSIGNED NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    attempts TINYINT UNSIGNED NOT NULL DEFAULT 0,
    error_message VARCHAR(500) NULL,
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uk_job_items_job_item (job_id, item_type, item_id),
    KEY idx_job_items_job_status (job_id, status),
    CONSTRAINT fk_job_items_job FOREIGN KEY (job_id) REFERENCES jobs (id) ON DELETE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE IF NOT EXISTS exports (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    book_id BIGINT UNSIGNED NOT NULL,
    export_type VARCHAR(24) NOT NULL DEFAULT 'book_zip',
    format VARCHAR(8) NOT NULL DEFAULT 'zip',
    status VARCHAR(16) NOT NULL DEFAULT 'queued',
    options JSON NOT NULL,
    total_chapters INT UNSIGNED NOT NULL DEFAULT 0,
    included_chapters INT UNSIGNED NOT NULL DEFAULT 0,
    file_id BIGINT UNSIGNED NULL,
    error_message TEXT NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    finished_at DATETIME(3) NULL,
    PRIMARY KEY (id),
    KEY idx_exports_book_created (book_id, created_at),
    KEY idx_exports_status (status),
    CONSTRAINT fk_exports_book FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE,
    CONSTRAINT fk_exports_file FOREIGN KEY (file_id) REFERENCES files (id) ON DELETE SET NULL
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;
"#;

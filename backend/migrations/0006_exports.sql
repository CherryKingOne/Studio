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

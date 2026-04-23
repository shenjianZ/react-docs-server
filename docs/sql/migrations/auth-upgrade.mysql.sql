-- React Docs 认证升级迁移（MySQL）
-- 执行顺序：
-- 1. 备份数据库
-- 2. 执行本脚本
-- 3. 再启动新版本服务

ALTER TABLE users
  ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT TRUE,
  ADD COLUMN password_set BOOLEAN NOT NULL DEFAULT TRUE;

CREATE TABLE IF NOT EXISTS oauth_accounts (
  id VARCHAR(255) PRIMARY KEY,
  user_id VARCHAR(32) NOT NULL,
  provider VARCHAR(64) NOT NULL,
  provider_user_id VARCHAR(255) NOT NULL,
  provider_email VARCHAR(255) NULL,
  created_at DATETIME NOT NULL,
  updated_at DATETIME NOT NULL,
  UNIQUE KEY uk_oauth_user_provider (user_id, provider),
  KEY idx_oauth_user_id (user_id),
  CONSTRAINT fk_oauth_accounts_user
    FOREIGN KEY (user_id) REFERENCES users(id)
    ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

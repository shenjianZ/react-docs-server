-- React Docs 认证升级迁移（SQLite）
-- 执行顺序：
-- 1. 备份数据库文件
-- 2. 执行本脚本
-- 3. 再启动新版本服务

ALTER TABLE users ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT 1;
ALTER TABLE users ADD COLUMN password_set BOOLEAN NOT NULL DEFAULT 1;

CREATE TABLE IF NOT EXISTS oauth_accounts (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  provider TEXT NOT NULL,
  provider_user_id TEXT NOT NULL,
  provider_email TEXT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_oauth_user_provider
  ON oauth_accounts (user_id, provider);

CREATE INDEX IF NOT EXISTS idx_oauth_user_id
  ON oauth_accounts (user_id);

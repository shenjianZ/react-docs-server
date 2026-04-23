-- React Docs 认证升级迁移（PostgreSQL）
-- 执行顺序：
-- 1. 备份数据库
-- 2. 执行本脚本
-- 3. 再启动新版本服务

ALTER TABLE users
  ADD COLUMN IF NOT EXISTS email_verified BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE users
  ADD COLUMN IF NOT EXISTS password_set BOOLEAN NOT NULL DEFAULT TRUE;

CREATE TABLE IF NOT EXISTS oauth_accounts (
  id VARCHAR(255) PRIMARY KEY,
  user_id VARCHAR(32) NOT NULL,
  provider VARCHAR(64) NOT NULL,
  provider_user_id VARCHAR(255) NOT NULL,
  provider_email VARCHAR(255) NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_oauth_user_provider
  ON oauth_accounts (user_id, provider);

CREATE INDEX IF NOT EXISTS idx_oauth_user_id
  ON oauth_accounts (user_id);

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM information_schema.table_constraints
    WHERE constraint_name = 'fk_oauth_accounts_user'
      AND table_name = 'oauth_accounts'
  ) THEN
    ALTER TABLE oauth_accounts
      ADD CONSTRAINT fk_oauth_accounts_user
      FOREIGN KEY (user_id) REFERENCES users(id)
      ON DELETE CASCADE;
  END IF;
END $$;

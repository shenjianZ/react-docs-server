-- ============================================
-- React Docs Server 数据库初始化脚本（MySQL）
-- 适用于全新安装；已有库升级请改用 docs/sql/migrations/*.sql
-- ============================================

CREATE DATABASE IF NOT EXISTS `react_docs` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE `react_docs`;

-- ============================================
-- 1. 用户表
-- ============================================
CREATE TABLE IF NOT EXISTS users (
  id VARCHAR(32) PRIMARY KEY COMMENT '用户ID',
  email VARCHAR(255) NOT NULL UNIQUE COMMENT '邮箱',
  email_verified BOOLEAN NOT NULL DEFAULT TRUE COMMENT '邮箱是否已验证',
  password_set BOOLEAN NOT NULL DEFAULT TRUE COMMENT '是否已设置本地密码',
  username VARCHAR(255) NULL UNIQUE COMMENT '用户名',
  nickname VARCHAR(255) NULL COMMENT '昵称',
  avatar_url TEXT NULL COMMENT '头像地址',
  bio TEXT NULL COMMENT '简介',
  role VARCHAR(64) NULL COMMENT '角色',
  status VARCHAR(64) NULL COMMENT '状态',
  last_login_at DATETIME NULL COMMENT '最后登录时间',
  password_hash VARCHAR(255) NOT NULL COMMENT '密码哈希',
  created_at DATETIME NOT NULL COMMENT '创建时间',
  updated_at DATETIME NOT NULL COMMENT '更新时间'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 2. OAuth 账号绑定表
-- ============================================
CREATE TABLE IF NOT EXISTS oauth_accounts (
  id VARCHAR(255) PRIMARY KEY COMMENT 'provider:provider_user_id',
  user_id VARCHAR(32) NOT NULL COMMENT '关联用户ID',
  provider VARCHAR(64) NOT NULL COMMENT '第三方渠道',
  provider_user_id VARCHAR(255) NOT NULL COMMENT '第三方用户ID',
  provider_email VARCHAR(255) NULL COMMENT '第三方返回邮箱',
  created_at DATETIME NOT NULL COMMENT '创建时间',
  updated_at DATETIME NOT NULL COMMENT '更新时间',
  UNIQUE KEY uk_oauth_user_provider (user_id, provider),
  KEY idx_oauth_user_id (user_id),
  CONSTRAINT fk_oauth_accounts_user
    FOREIGN KEY (user_id) REFERENCES users(id)
    ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 初始化完成
-- ============================================
SELECT '✅ React Docs 认证初始化完成' AS status;
SHOW TABLES;

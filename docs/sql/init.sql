-- ============================================
-- Web Template 数据库初始化脚本
-- ============================================

CREATE DATABASE IF NOT EXISTS `web_template` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE `web_template`;

-- ============================================
-- 1. 用户表
-- ============================================
CREATE TABLE IF NOT EXISTS users (
  id VARCHAR(10) PRIMARY KEY COMMENT '10位数字用户ID',
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  created_at DATETIME NOT NULL COMMENT '创建时间',
  updated_at DATETIME NOT NULL COMMENT '更新时间'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ============================================
-- 初始化完成
-- ============================================
SELECT '✅ 数据库初始化完成' AS status;
SHOW TABLES;

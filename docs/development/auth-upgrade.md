# 认证升级执行说明

本文档说明邮箱验证码注册/登录、Google/GitHub OAuth 弹窗登录、微信/QQ 占位能力的上线步骤。

## 1. 先执行数据库迁移

> 不要只依赖服务启动时的 `CREATE TABLE IF NOT EXISTS` 或自动补列逻辑完成升级。

请在启动新版本服务前，按数据库类型执行对应脚本：

- MySQL：`docs/sql/migrations/auth-upgrade.mysql.sql`
- PostgreSQL：`docs/sql/migrations/auth-upgrade.postgresql.sql`
- SQLite：`docs/sql/migrations/auth-upgrade.sqlite.sql`

推荐顺序：

1. 备份数据库
2. 执行对应迁移脚本
3. 检查 `users.email_verified`、`users.password_set` 和 `oauth_accounts` 是否已创建
4. 再启动新版本服务

## 2. 更新后端配置

在 `config/development.toml` 或 `config/production.toml` 中配置：

- `auth.frontend_base_url`
- `auth.email_verification.*`
- `auth.email_verification.smtp.*`
- `auth.providers.google.*`
- `auth.providers.github.*`
- `auth.providers.wechat.*`
- `auth.providers.qq.*`

## 3. 配置 SMTP

必须启用并填写以下字段后，邮箱验证码才能正常发送：

- `auth.email_verification.smtp.enabled = true`
- `host`
- `port`
- `username`
- `password`
- `from_email`
- `from_name`

## 4. 配置 OAuth 回调

确保第三方开放平台回调地址与后端配置完全一致，例如：

- Google：`/api/auth/oauth/google/callback`
- GitHub：`/api/auth/oauth/github/callback`

前端无需单独回调页；后端回调会直接向 popup opener 发送 `postMessage`。

## 5. 占位渠道说明

- `wechat`
- `qq`

当前仅保留统一入口与回调路由，服务端会返回“暂未开放”提示，前端 UI 无需后续重构。

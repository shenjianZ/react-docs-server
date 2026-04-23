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

推荐理解：

- `host`：SMTP 服务器地址
- `port`：SMTP 端口
- `username`：SMTP 登录账号
- `password`：SMTP 密码或应用专用密码
- `from_email`：发件邮箱
- `from_name`：发件人名称
- `starttls = true`：通常配合 `587`
- `starttls = false`：通常配合 `465`

示例（Gmail / Google Workspace 常见写法）：

```toml
[auth.email_verification.smtp]
enabled = true
host = "smtp.gmail.com"
port = 587
username = "your.name@example.com"
password = "your-app-password"
from_email = "your.name@example.com"
from_name = "React Docs"
starttls = true
```

## 4. 配置 OAuth 回调

确保第三方开放平台回调地址与后端配置完全一致，例如：

- Google：`/api/auth/oauth/google/callback`
- GitHub：`/api/auth/oauth/github/callback`

前端无需单独回调页；后端回调会直接向 popup opener 发送 `postMessage`。

### 当前项目的填写方式

当前项目本地开发结构是：

- 前端：`http://localhost:5173`
- 后端：`http://localhost:3000`
- 前端通过 Vite 代理 `/api -> http://localhost:3000`

因此本地开发时应填写：

#### Google

- 已获授权的 JavaScript 来源：`http://localhost:5173`
- 已获授权的重定向 URI：`http://localhost:3000/api/auth/oauth/google/callback`

#### GitHub

- Homepage URL：`http://localhost:5173`
- Authorization callback URL：`http://localhost:3000/api/auth/oauth/github/callback`

### 填写原则

- `auth.frontend_base_url`：填真正打开登录弹窗的前端页面地址
- `auth.providers.*.redirect_uri`：填真正接收第三方回调的后端接口地址

不要混淆：

- JavaScript 来源 / Homepage URL：填前端站点 origin
- Redirect URI / Callback URL：填完整 callback 地址

### 本地开发配置示例

```toml
[auth]
frontend_base_url = "http://localhost:5173"

[auth.providers.google]
enabled = true
client_id = "your-google-client-id"
client_secret = "your-google-client-secret"
redirect_uri = "http://localhost:3000/api/auth/oauth/google/callback"
scopes = ["openid", "email", "profile"]

[auth.providers.github]
enabled = true
client_id = "your-github-client-id"
client_secret = "your-github-client-secret"
redirect_uri = "http://localhost:3000/api/auth/oauth/github/callback"
scopes = ["read:user", "user:email"]
```

### 生产环境对应关系

如果生产环境为：

- 前端：`https://docs.example.com`
- 后端：`https://api.example.com`

则推荐填写：

- `auth.frontend_base_url = "https://docs.example.com"`
- Google Redirect URI：`https://api.example.com/api/auth/oauth/google/callback`
- GitHub Callback URL：`https://api.example.com/api/auth/oauth/github/callback`

## 5. 占位渠道说明

- `wechat`
- `qq`

当前仅保留统一入口与回调路由，服务端会返回“暂未开放”提示，前端 UI 无需后续重构。

# Client API 设计

## 背景

当前 `client/` 的 API 对接代码规模很小，但已经暴露出几个会在后续前端开发中持续放大的问题：

- 前端接口类型完全手写，和 `server/` 的真实返回结构没有单一来源。
- `src/api/` 与 `src/queries/` 按技术层横切拆分，业务边界不清晰。
- 请求封装过薄，只处理了少量 `status` 分支，缺少统一错误模型和扩展点。
- JSON 请求与附件二进制下载共用一层极简封装，后续能力扩展会互相牵扯。
- 还没有契约生成链路，接口变更时前后端很容易静默漂移。

这在“前端基本还没开始写”的阶段反而是优势：可以用一次不兼容重构，把后续开发的基础设施先定型。

## 目标

- 让 `server/` 成为 API 契约的单一来源。
- 消除 `client/` 中重复维护的 DTO 类型定义。
- 用类型安全的请求客户端替代当前手写 `fetch` 包装。
- 按业务域重组前端 API 与查询模块，降低后续页面开发成本。
- 为后续统一认证、日志、错误展示、缓存策略留出明确扩展点。

## 非目标

- 这次不引入完整的前端页面实现。
- 这次不做全量运行时响应校验，避免前后端再维护第二套 schema。
- 这次不为了 API 层重构额外引入重量级状态管理。

## 现状评估

### 当前实现的主要问题

1. `client/src/api/types.ts` 手写复制了服务端模型，未来极易和 Rust 返回结构脱节。
2. `client/src/api/client.ts` 只覆盖了最基础的 URL 拼接与状态码异常，无法承接契约、错误归一化和中间件能力。
3. `client/src/queries/*` 与 `client/src/api/*` 都按技术层组织，未来一个业务域需要跨多个目录维护。
4. 附件下载是 `Blob`，健康检查与消息列表是 JSON，但现在共享同一套薄封装，边界不清晰。
5. `queryClient` 已经做了重试策略；如果后续再在请求层随意加入 retry，很容易形成双重重试和调试混乱。

### 现有实现中保留的部分

- `@tanstack/solid-query` 仍然保留，继续作为前端数据获取与缓存层。
- `client/src/lib/query-client.ts` 的总体职责不变，只做 query 级别的默认策略。
- `server/` 当前 REST 路由结构保持不变，仍然是 `/api/*`。

## 方案选型

本次采用：

- `server`: `utoipa`
- `client` 契约类型生成: `openapi-typescript`
- `client` 传输层: `openapi-fetch`

不采用 `ky` 的原因：

- `ky` 适合替换裸 `fetch`，但它不能解决“前后端契约单一来源”问题。
- 本次重构的核心不是把请求写得更舒服，而是把契约、类型、调用边界一次性打通。
- 在 `openapi-fetch` 已经承担 typed client 角色的前提下，再叠一层 `ky` 只会让错误流与中间件链更复杂。

## 目标架构

请求链路重构为：

1. `server/` 通过 `utoipa` 为现有 REST 路由生成 OpenAPI 文档。
2. 根目录脚本基于 OpenAPI 文档生成前端 `schema.d.ts`。
3. `client` 用 `openapi-fetch` 创建唯一的 typed client 实例。
4. 各业务域在 `features/*/api.ts` 中封装面向页面的调用函数。
5. `features/*/queries.ts` 中承接 TanStack Solid Query 的 query key、缓存策略与 hook。

职责边界如下：

- `server`: API 契约与返回结构真相来源
- `client/src/api/generated/schema.d.ts`: 生成的只读契约类型
- `client/src/api/core/*`: 通用传输配置、错误模型、中间件
- `client/src/features/*/api.ts`: 业务域 API 适配层
- `client/src/features/*/queries.ts`: UI 查询层

## 服务端改造

### 1. 引入 OpenAPI 生成

`server/Cargo.toml` 增加 `utoipa`，并根据集成方式补充 Axum 对应支持。

目标是为当前这些路由生成契约：

- `GET /api/healthz`
- `POST /api/inbound`
- `GET /api/messages`
- `GET /api/messages/{id}`
- `GET /api/attachments/{id}`
- `GET /api/tags`

### 2. 为模型补 schema 派生

优先为这些响应结构增加 OpenAPI schema 派生：

- `InboundMetadata`
- `InboundHeaders`
- `InboundResponse`
- `MessageSummary`
- `MessageDetail`
- `AttachmentMeta`
- `MessageDetailResponse`
- `Paginated<T>` 的具体响应使用方式
- `Tag`
- `HealthResponse`

需要注意的点：

- `chrono::DateTime<Utc>` 的 schema 表达要统一为字符串时间格式。
- `MessageDetailResponse` 当前使用 `#[serde(flatten)]`，文档层要明确最终 JSON 结构，避免生成类型与实际返回不一致。
- `Paginated<T>` 如果直接做泛型 schema 支持成本偏高，可以先为消息列表定义明确的专用响应 schema。

### 3. 为路由补路径注解

在 `routes/health.rs`、`routes/messages.rs`、`routes/tags.rs`、`routes/attachments.rs`、`routes/inbound.rs` 上补 `#[utoipa::path(...)]`，至少明确：

- 方法
- 路径
- path/query 参数
- 成功响应
- 404 / 400 / 500 等主要错误响应

### 4. 暴露 OpenAPI 文档端点

增加一个稳定的文档端点，例如：

- `/api-docs/openapi.json`

这个端点只服务于开发、生成和校验流程，不影响现有 `/api/*` 业务路由结构。

## 前端改造

### 1. 删除当前手写契约层

移除以下文件或目录：

- `client/src/api/types.ts`
- `client/src/api/client.ts`
- `client/src/api/errors.ts`
- `client/src/api/messages.ts`
- `client/src/api/tags.ts`
- `client/src/api/attachments.ts`
- `client/src/api/health.ts`
- `client/src/api/index.ts`
- `client/src/queries/*`

这些文件的问题不是“写法差”，而是边界本身不适合继续扩展。

### 2. 新的目录结构

建议重组为：

```text
client/src/
  api/
    core/
      client.ts
      config.ts
      errors.ts
      response.ts
    generated/
      schema.d.ts
  features/
    attachments/
      api.ts
      queries.ts
    health/
      api.ts
      queries.ts
    messages/
      api.ts
      queries.ts
      models.ts
    tags/
      api.ts
      queries.ts
  lib/
    query-client.ts
```

说明：

- `api/generated/schema.d.ts` 由命令生成，不手改。
- `api/core/client.ts` 只负责创建 `openapi-fetch` client。
- `api/core/errors.ts` 定义前端统一错误模型。
- `api/core/response.ts` 放响应提取与错误转换辅助函数。
- `features/*/api.ts` 对接 typed client。
- `features/*/queries.ts` 对接 Solid Query。
- `features/messages/models.ts` 只存前端特有的筛选参数或 view model，不重复声明服务端 DTO。

### 3. typed client 设计

`openapi-fetch` 作为唯一底层请求客户端，统一处理：

- `baseUrl`
- 默认 headers
- 中间件扩展位
- 错误归一化

不再保留 `apiGet` / `apiPost` 这类无契约的通用请求函数。

### 4. 业务域 API 设计

每个 `features/*/api.ts` 暴露面向业务的函数，而不是面向 HTTP 原语的函数。

例如：

- `messages/api.ts`
  - `listMessages(query)`
  - `getMessageDetail(id)`
- `tags/api.ts`
  - `listTags()`
- `health/api.ts`
  - `getHealth()`
- `attachments/api.ts`
  - `downloadAttachment(id)`

其中：

- JSON 接口走统一响应提取逻辑。
- 附件下载保留 `Blob` 返回，不强行混进 JSON 响应辅助函数。

### 5. Query 层设计

`features/*/queries.ts` 继续使用 `createQuery`，但只承担这些职责：

- query key 定义
- `queryFn`
- `enabled`
- `select`
- 单接口的缓存/重试覆写

这样页面只依赖 feature hook，不直接依赖底层 client。

## 错误模型

前端统一错误类型建议分为三类：

- `NetworkRequestError`
  - 请求未到达服务端，或超时、断网、浏览器层失败
- `HttpResponseError`
  - 服务端返回非 2xx，保留 `status`、`statusText`、`body`
- `ResponseParseError`
  - 返回成功，但响应格式不符合预期或下载内容处理失败

统一错误模型的意义：

- Query 层可以稳定判断是否允许重试。
- 页面层可以基于错误类别统一展示文案。
- 以后接入认证、埋点、错误上报时不需要重改每个业务域。

## 生成与开发流程

### 1. 生成产物提交入库

`client/src/api/generated/schema.d.ts` 直接提交进仓库。

原因：

- 前端开发不依赖本地先跑 server 才能拿到类型。
- CI 中可以直接检查生成结果是否与当前契约一致。
- PR 中能直接看到接口契约变化。

### 2. 根目录生成命令

在根目录增加专用脚本，完成以下流程：

1. 获取最新 OpenAPI JSON
2. 生成 `client/src/api/generated/schema.d.ts`

脚本名称建议保持显式，例如：

- `pnpm gen:client-api`

如果本地流程需要 server 先启动，可以在文档和脚本输出中明确提示；但更理想的方案是 server 在开发命令中直接能导出静态 OpenAPI JSON。

### 3. 质量门

改造后需要保持这些检查路径可用：

- `pnpm check`
- `pnpm --filter @mail-shell/client build`
- OpenAPI 生成命令可重复执行且结果稳定

## 兼容性与迁移

本次允许不兼容重构，但兼容性影响主要发生在代码组织层，不发生在 HTTP 接口路径层：

- 服务端 REST 路径保持不变
- 前端内部模块路径会整体重组
- 现有 API 调用函数导出名可以适当调整，以换取更清晰的业务边界

由于前端页面尚未正式展开，实现成本主要集中在“基础设施重建”，而不是页面迁移。

## 实施顺序

建议按以下顺序落地：

1. `server` 引入 `utoipa` 并为现有模型与路由补文档注解
2. 暴露 `/api-docs/openapi.json`
3. 根目录加入前端契约生成脚本
4. 生成 `client` 侧 OpenAPI TypeScript 类型
5. 重建 `client/src/api/core/*`
6. 按业务域重建 `client/src/features/*/api.ts`
7. 重建 `client/src/features/*/queries.ts`
8. 删除旧的 `client/src/api/*` 与 `client/src/queries/*`
9. 跑通 `pnpm check`

## 风险与控制

### 风险 1：OpenAPI 注解与真实返回不一致

控制方式：

- 优先从当前测试覆盖到的真实返回结构出发补 schema。
- 对关键接口增加最小文档存在性与字段覆盖验证。

### 风险 2：泛型分页响应在 Rust 侧 schema 表达复杂

控制方式：

- 不强求一步到位做“漂亮的泛型文档抽象”。
- 先针对当前消息列表定义明确响应结构，优先保证生成结果稳定。

### 风险 3：请求层与 Query 层重试策略冲突

控制方式：

- 传输层不做复杂 retry。
- 继续让 `query-client.ts` 作为主要 retry 策略入口。

## 结论

这次重构的核心不是“换一个更好用的请求库”，而是把前端 API 对接从“手写散落的 fetch 包装”升级为“契约驱动、按业务域组织、可持续扩展”的基础设施。

最终形态应当是：

- `server` 维护唯一接口契约
- `client` 消费生成类型而不是手写 DTO
- 请求层、业务层、查询层职责清晰
- 后续前端页面开发围绕 feature 模块展开，而不是继续堆积通用 `api/*` 文件

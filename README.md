# 和平精英 .imazingapp 配置生成器

一个基于 Rust + 微信小程序的配置驱动型 `.imazingapp` 生成系统。

项目目标：用户通过小程序选择游戏配置，后端根据 JSON
配置规则修改模板文件，并生成可通过 iMazing 导入 iOS 的配置包。

## 当前状态

已上线：真机 iMazing 导入验证通过，微信审核发布完成，运行于微信云托管。

完成流程：

    微信小程序
        ↓
    schema 驱动动态配置页面
        ↓
    wx.cloud.callContainer（内网直达容器）
        ↓
    Rust Pipeline
        ↓
    Patch Engine
        ↓
    Save/Ini Engine
        ↓
    Template Engine
        ↓
    .imazingapp 文件字节（直接返回，无服务端存储）
        ↓
    小程序本地缓存最新文件 → 分享到微信

## 已实现功能

### 后端 Rust Workspace

包含：

-   domain：基础类型与错误定义
-   config-model：配置数据模型
-   config-compiler：配置加载、校验、编译
-   patch-engine：用户选择转换为 PatchOp
-   save-engine：UE .sav 文件修改
-   ini-engine：ini 配置修改
-   zip-engine：zip 基础处理
-   template-engine：imazingapp 模板流式生成
-   pipeline：生成流程编排
-   server：Axum HTTP 服务

## 配置系统

所有业务规则来自 `config/`：

    config/
    ├── schema.json
    ├── patches.json
    ├── presets.json
    ├── targets.json
    ├── versions.json
    └── translations/

支持配置（当前 10 项字段）：

-   画质
-   FPS
-   分辨率缩放
-   视觉风格
-   抗锯齿
-   阴影
-   渲染优化（开启写入 5 条 CVar，关闭移除）
-   亮度
-   镜头视野
-   冲刺灵敏度

新增配置原则：

优先修改 JSON，不增加 Rust 业务代码。

## API

当前接口：

    GET  /api/v1/health    健康检查
    GET  /api/v1/schema    返回配置 schema（驱动前端 UI）
    GET  /api/v1/presets   返回预设列表（当前为空）
    POST /api/v1/build      生成并直接返回 .imazingapp 文件字节

说明：`/build` 直接以 `application/octet-stream` 返回文件字节，
文件名经 `Content-Disposition` 传递。已移除服务端存储与
`/download/{id}` 接口，服务端因此无状态、可多实例横向扩容。
实例饱和时 `/build` 返回 `503`（服务繁忙），由平台按 CPU 触发扩容。

## 小程序

目录：

    miniprogram/

特点：

-   schema 自动生成配置 UI
-   支持 select/switch/slider
-   支持动态默认值
-   通过 `wx.cloud.callContainer` 调用云托管（无需配置合法域名）
-   本地只缓存最新一个文件，支持反复分享（无需重新生成）
-   生成按钮 5 秒冷却兜底防刷

## 当前待完成

详细计划见 `TODO.md`。核心链路已上线，剩余为运营侧优化
（日志监控、错误追踪、商业化）。

## 并发与稳健性

生成是 CPU 密集操作，采用「应用有界 + 平台弹性」两层策略：

-   `/build` 的 `pipeline::generate` 放入 `spawn_blocking`，不占用
    async 工作线程，保证 `/health`、`/schema` 始终秒回
-   构建并发用信号量闸门限制（`BUILD_CONCURRENCY`，默认按核数夹在
    `[1, 4]`），避免过多 deflate 抢 CPU、内存峰值失控
-   闸门等待超过 20s 返回 `503`，交由云托管按 CPU 使用率自动扩容，
    而非在实例内无限排队
-   模板处理中未改动的 entry 走 `raw_copy_file` 免解压重压缩，
    被改写的 sav/ini 以 `Stored` 写回，大幅降低单请求 CPU

云托管侧：开启自动扩缩容（最大副本 > 1、CPU 阈值 ~60%），
配合无状态服务实现横向扩容。

## 持续集成 / 部署

分工：

-   **GitHub Actions**（`.github/workflows/ci.yml`）= CI 门禁
    -   push / PR 到 `main` 时触发
    -   `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test`
    -   用真实 `Dockerfile` 校验镜像可构建（与云托管产物一致）
-   **微信云托管流水线** = CD
    -   push 到指定分支触发，用 `Dockerfile` 构建镜像并部署到 80 端口

注意：仓库 `rust-toolchain.toml` 固定为 Windows GNU 工具链（本地开发用）。
CI 在 Linux 上用环境变量 `RUSTUP_TOOLCHAIN=stable` 覆盖，避免下载 Windows 工具链失败。

## 开发要求

-   Rust 2024 Edition
-   配置驱动
-   禁止硬编码游戏规则
-   保持 Workspace 分层依赖
-   新功能优先扩展配置

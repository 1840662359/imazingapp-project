# 和平精英 .imazingapp 配置生成器

一个基于 Rust + 微信小程序的配置驱动型 `.imazingapp` 生成系统。

项目目标：用户通过小程序选择游戏配置，后端根据 JSON
配置规则修改模板文件，并生成可通过 iMazing 导入 iOS 的配置包。

## 当前状态

项目已完成核心生成链路，进入部署上线阶段。

完成流程：

    微信小程序
        ↓
    schema 驱动动态配置页面
        ↓
    HTTP API
        ↓
    Rust Pipeline
        ↓
    Patch Engine
        ↓
    Save/Ini Engine
        ↓
    Template Engine
        ↓
    .imazingapp 文件

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

支持配置：

-   FPS
-   画质
-   分辨率缩放
-   亮度
-   阴影
-   抗锯齿
-   视觉风格
-   镜头视野
-   冲刺灵敏度

新增配置原则：

优先修改 JSON，不增加 Rust 业务代码。

## API

当前接口：

    GET  /api/v1/schema

    POST /api/v1/build

    GET  /api/v1/download/{id}

## 小程序

目录：

    miniprogram/

特点：

-   schema 自动生成配置 UI
-   支持 select/switch/slider
-   支持动态默认值
-   支持生成文件分享

## 当前待完成

详细计划见 `TODO.md`。

主要剩余：

1.  微信云托管正式部署
2.  真实存储服务替换 Mock Storage
3.  真机导入验证
4.  微信审核上线

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

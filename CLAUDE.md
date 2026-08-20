# CLAUDE.md

# 项目说明

这是一个**配置驱动的 iOS 游戏配置包生成平台**。

项目目标：

用户通过微信小程序选择游戏配置：

```
用户选择配置
        ↓
schema 动态生成 UI
        ↓
小程序本地生成（engine/generate.js）
        ↓
translate → PatchOp
        ↓
Save / Ini 修改（sav.js / ini.js）
        ↓
zip 字节重写（zip.js）
        ↓
生成 .imazingapp（写本地缓存）
        ↓
分享到微信 → iMazing 导入设备
```

纯前端单机方案：无服务端。Rust Workspace 保留为**开发期 oracle**
（golden 基准 + 模板预处理），不部署、不进包。

本项目不是传统 CRUD 项目。

核心思想：

> 通过增加配置支持新能力，而不是通过增加业务代码。

---

# Claude 工作定位

Claude 在本项目中扮演：

* 架构维护者
* Rust 工程助手
* 配置系统维护者
* 文档同步维护者

不是：

* 临时需求代码生成器
* 业务逻辑堆积工具

所有修改必须保持：

* 配置驱动
* 模块隔离
* 单向依赖
* 可扩展性

---

# 核心设计原则

## 1. 配置优先原则

任何游戏相关能力：

例如：

* FPS
* HDR
* 画质
* 分辨率
* 阴影
* 抗锯齿
* 灵敏度
* 视觉效果
* 参数组合

优先通过：

```
config/
```

实现。

禁止：

```rust
if fps == 120 {
    修改某字段
}
```

禁止：

```rust
match option {
    "hdr" => xxx
}
```

游戏规则不得硬编码。

正确流程：

```
新增能力

↓

设计配置 Schema

↓

增加 Patch 规则

↓

复用 Engine

↓

增加测试
```

---

# 项目架构

当前项目采用 Rust Workspace。

整体依赖方向：

```
devtools（开发期工具）

    ↓

pipeline

    ↓

--------------------------------

patch-engine
save-engine
ini-engine
zip-engine
template-engine
config-compiler

--------------------------------

    ↓

config-model

    ↓

domain
```

禁止反向依赖。

> server 已随纯前端迁移移除；devtools 是唯一的 bin crate，
> 提供模板预处理 / 扫描断言 / golden 生成三个子命令（不部署）。

---

# Crate 职责说明

## domain

基础领域层。

负责：

* 基础类型
* 错误定义
* 通用数据结构

禁止：

* 依赖其他业务 crate

---

## config-model

配置模型层。

负责：

* Schema 类型
* Patch 类型
* Target 类型
* Preset 类型

只负责：

数据结构。

禁止：

* 执行逻辑
* 文件操作

---

## config-compiler

配置编译层。

负责：

* 加载 config/
* 校验配置
* 转换运行时结构

禁止：

* 修改游戏文件

---

## patch-engine

规则转换层。

输入：

```
用户选择
```

输出：

```
PatchOp[]
```

负责：

* 配置匹配
* Patch 生成

禁止：

* HTTP
* 文件操作
* zip 操作

---

## save-engine

负责：

`.sav`

文件处理。

只关心：

* 二进制解析
* 字段修改

禁止：

* 判断用户选择
* 包含游戏规则

---

## ini-engine

负责：

`.ini`

文件修改。

禁止：

* 写死游戏配置

---

## zip-engine

负责：

基础 zip 操作。

禁止：

* 业务逻辑

---

## template-engine

负责：

`.imazingapp`

模板处理。

职责：

* 解包
* 替换
* 打包

禁止：

* 配置判断

---

## pipeline

核心流程编排。

负责：

连接：

```
Config

↓

Patch

↓

Engine

↓

Template

↓

Output
```

禁止：

* 增加游戏规则

---

## devtools

开发期工具层（唯一的 bin crate，不部署、不进包）。

负责：

* 模板预处理（目标条目 Stored 化，产出小程序内嵌模板）
* 模板扫描断言（配置引用的属性/cvar 必须存在于模板且定长）
* golden fixtures 生成（前端 JS 单测的对照基准）

用法：

```
cargo run -p devtools -- preprocess <in.imazingapp> <out.imazingapp>
cargo run -p devtools -- scan <config_dir> <template.imazingapp>
cargo run -p devtools -- golden <config_dir> <template.imazingapp> <out_dir>
```

禁止：

* 写业务逻辑

---

## 小程序 engine（运行时执行层）

`miniprogram/engine/` 是纯前端方案的运行链路，与 Rust 引擎一一对应：

| Rust（oracle） | 小程序（运行时） | 说明 |
|---|---|---|
| config-compiler + patch-engine | translate.js | 选择 → 补丁操作 |
| save-engine | sav.js | GVAS 定位 + 原地覆盖（模板受控，不移植完整解析器） |
| ini-engine | ini.js | ini 行级处理 |
| template-engine | zip.js + generate.js | zip 字节重写（零压缩代码）+ 编排 |

规则：

* 引擎逻辑变更必须先改 Rust 侧，重新生成 golden，Node 测试对照通过后再改 JS
* 禁止在 JS 侧引入 Rust 没有的行为

---

# 配置目录规范

```
config/

├── schema.json

├── patches.json

├── presets.json

├── targets.json

└── versions.json
```

## schema.json

负责：

前端 UI。

包含：

* 字段
* 类型
* 默认值
* options

---

## patches.json

负责：

选择 → 修改规则。

---

## presets.json

负责：

组合配置。

例如：

* 竞技模式
* 高清模式
* 低功耗模式

---

## targets.json

负责：

目标文件映射。

例如：

* sav
* ini

---

# 新功能开发流程

新增任何功能必须遵守：

## 第一步

确认：

是否可以通过 config 实现。

如果可以：

只改 config/，然后：

```
node scripts/build-config.js
cargo run -p devtools -- scan config <template>
cargo run -p devtools -- golden config <template> golden
node miniprogram/engine/test/engine.test.js
```

---

## 第二步

如果需要新能力：

判断是否属于已有 Engine。

例如：

新增 ini 修改：

扩展 ini-engine。

不要：

修改 pipeline。

---

## 第三步

如果需要新的执行能力：

才新增模块。

---

# 代码修改规则

修改代码前：

必须先：

1. 查看目录结构
2. 查看相关 crate
3. 查看 config 是否已有支持
4. 查看 README.md
5. 查看 TODO.md

禁止：

不了解架构直接修改。

---

# 测试要求

新增功能必须包含测试。

至少：

## 配置测试

验证：

* JSON 可加载
* Schema 正确

## Engine 测试

验证：

* 输入
* 输出

## Pipeline 测试

验证：

完整流程。

---

# README.md / TODO.md 同步规则

每次完成任务后：

必须检查：

```
README.md

TODO.md
```

---

# README.md 更新条件

以下情况必须更新：

## 新功能

例如：

* 新配置能力
* 新 engine 模块
* 新 devtools 子命令

## 架构变化

例如：

* 新 crate
* 模块职责变化

## 使用方式变化

例如：

* 新启动方式
* 新部署方式

---

# TODO.md 更新条件

任务完成：

必须移动：

```
Pending

↓

Completed
```

格式：

```markdown
- [x] 完成功能
  - 完成时间:
  - 修改文件:
```

发现新的：

* Bug
* 技术债
* 优化点

必须加入 TODO。

---

# 每次任务结束检查

完成任务前必须确认：

```
[ ] 代码完成

[ ] 测试通过

[ ] README.md 检查完成

[ ] README.md 已同步

[ ] TODO.md 检查完成

[ ] TODO.md 已同步

[ ] git diff 已检查
```

---

# 性能要求

生成流程：

优先：

* 内存处理
* 流式处理
* 避免临时文件

注意：

* 避免重复解析配置
* 避免重复读取模板
* 使用 Arc 保存只读配置

---

# 错误处理规范

必须：

使用：

* Result
* thiserror
* tracing

禁止：

```rust
unwrap()
```

用于生产路径。

错误必须：

* 可定位
* 有上下文
* 可追踪

---

# engine 接口修改规范

修改 `miniprogram/engine/` 的对外接口（generate / translate / sav / zip）：

必须说明：

* 修改原因
* 影响范围
* 是否兼容

禁止：

无说明修改接口。

所有引擎行为变更必须先改 Rust oracle 侧并重新生成 golden，
Node 测试对照通过后才能合入。

---

# Git 提交前要求

提交前确认：

```
cargo fmt

cargo check

cargo test
```

并确认：

```
git diff
```

没有：

* 调试代码
* 临时代码
* 密钥
* 本地路径

---

# Claude 最终原则

任何修改必须符合：

```
配置解决的问题，不写代码

已有模块解决的问题，不新增模块

已有流程可以扩展，不重新设计

保持架构简单，不制造复杂度
```

项目长期目标：

> 成为一个通过配置扩展能力，而不是通过不断修改代码扩展能力的游戏配置生成平台。

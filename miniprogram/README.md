# 微信小程序快速开始

## 前置要求

1. 安装[微信开发者工具](https://developers.weixin.qq.com/miniprogram/dev/devtools/download.html)
2. 注册微信小程序账号（或使用测试号）

---

## 快速启动

### 1. 启动后端服务

```bash
cd d:\imazingapp-project
cargo run --package server
```

服务启动在：`http://localhost:8080`

### 2. 打开小程序项目

1. 打开微信开发者工具
2. 点击"导入项目"
3. 选择目录：`d:\imazingapp-project\miniprogram`
4. 填写 AppID：
   - 有账号：填写实际 AppID
   - 测试：选择"测试号"

### 3. 配置开发环境

在微信开发者工具中：

1. 点击右上角"详情"
2. 勾选"不校验合法域名、web-view（业务域名）、TLS 版本以及 HTTPS 证书"
3. 点击"编译"

### 4. 查看效果

- 开发者工具中查看模拟器效果
- 点击"预览"生成二维码，手机扫码真机调试

---

## 配置 API 地址

### 开发环境（默认）

`miniprogram/app.js`：
```javascript
App({
  globalData: {
    apiBaseUrl: 'http://localhost:8080/api/v1'
  }
});
```

### 生产环境

改为实际域名：
```javascript
App({
  globalData: {
    apiBaseUrl: 'https://your-domain.com/api/v1'
  }
});
```

**注意**：生产环境需要在小程序后台配置合法域名。

---

## 小程序后台配置（生产环境）

### 1. 登录小程序后台

https://mp.weixin.qq.com

### 2. 配置服务器域名

**开发** → **开发管理** → **开发设置** → **服务器域名**

添加以下域名：

- **request 合法域名**：`https://your-domain.com`
- **downloadFile 合法域名**：`https://your-domain.com`

### 3. 配置业务域名（可选）

如果需要用 web-view 打开 H5 页面。

---

## 测试流程

### 1. 首页加载

- 页面加载时自动调用 `GET /schema`
- 显示预设按钮和字段列表

### 2. 选择配置

**方式 A：使用预设**
- 点击"流畅（省电）"等预设按钮
- 自动应用预设配置

**方式 B：自定义**
- 调整 Slider（亮度）
- 切换 Picker（帧率）
- 开关 Switch（如果有）

### 3. 生成文件

- 点击底部"生成配置文件"按钮
- 等待生成（调用 `POST /build`）
- 跳转到下载页

### 4. 下载文件

- 显示文件 ID 和过期时间
- 点击"下载配置文件"
- 保存到本地

---

## 常见问题

### Q: 显示"加载配置失败"

**A:** 检查：
1. 后端服务是否启动（`cargo run --package server`）
2. API 地址是否正确（`app.js` 中的 `apiBaseUrl`）
3. 是否关闭了"域名校验"（开发环境）

### Q: 点击"生成"没有反应

**A:** 打开调试器（Console）查看错误信息：
- 点击"调试器"标签
- 查看 Console 输出
- 查看 Network 请求是否成功

### Q: 下载失败

**A:** 检查：
1. file_id 是否正确
2. 文件是否过期（默认 10 分钟）
3. 下载域名是否配置（生产环境）

### Q: 真机预览白屏

**A:** 检查：
1. 手机是否能访问后端地址（局域网）
2. 是否配置了合法域名（生产环境）
3. 打开微信调试（手机微信 → 我 → 设置 → 帮助与反馈 → 右上角工具 → 打开调试）

---

## 目录结构

```
miniprogram/
├── api/                        # API 封装
│   ├── request.js              # 统一请求
│   └── config.js               # 配置 API
│
├── components/                 # 组件
│   └── dynamic-field/          # 动态字段组件
│
├── pages/                      # 页面
│   ├── index/                  # 首页
│   └── download/               # 下载页
│
├── app.js                      # 应用入口
├── app.json                    # 全局配置
├── app.wxss                    # 全局样式
└── project.config.json         # 项目配置
```

---

## 开发技巧

### 1. 热重载

修改代码后：
- WXML / WXSS / JS：自动重新编译
- JSON：需要手动点击"编译"

### 2. 调试

- **Console**：查看日志和错误
- **Network**：查看网络请求
- **Storage**：查看本地存储
- **AppData**：查看页面数据

### 3. 真机调试

点击"真机调试" → 扫码 → 查看真机 Console 输出

---

## 部署上线

### 1. 上传代码

1. 点击右上角"上传"
2. 填写版本号和备注
3. 点击"上传"

### 2. 提交审核

1. 登录小程序后台
2. **版本管理** → **开发版本**
3. 点击"提交审核"
4. 填写审核信息
5. 等待审核通过（通常 1-7 天）

### 3. 发布

审核通过后：
1. **版本管理** → **审核版本**
2. 点击"发布"
3. 用户可以搜索到小程序

---

## 扩展开发

### 添加新字段类型

编辑 `components/dynamic-field/dynamic-field.wxml`：

```xml
<!-- Radio 单选 -->
<radio-group wx:elif="{{field.type === 'radio'}}" 
             bindchange="onRadioChange">
  <radio wx:for="{{field.options}}" 
         wx:key="id" 
         value="{{item.id}}">
    {{item.label}}
  </radio>
</radio-group>
```

编辑 `components/dynamic-field/dynamic-field.js`：

```javascript
onRadioChange(e) {
  const value = e.detail.value;
  this.triggerEvent('change', {
    fieldId: this.properties.field.id,
    value: value
  });
}
```

**完成**。后端返回 `type: "radio"` 的字段时，自动渲染为单选框。

---

## 性能优化

### 1. 图片优化

- 使用 WebP 格式
- 使用 lazy-load
- 压缩图片大小

### 2. 代码优化

- 避免频繁 setData
- 使用事件委托
- 避免过深的组件嵌套

### 3. 网络优化

- 使用 CDN
- 开启 HTTP/2
- 压缩响应体

---

## 更多资源

- [微信小程序官方文档](https://developers.weixin.qq.com/miniprogram/dev/framework/)
- [微信开发者社区](https://developers.weixin.qq.com/community/develop/mixflow)
- [小程序设计指南](https://developers.weixin.qq.com/miniprogram/design/)

---

## 技术支持

遇到问题？

1. 查看 [MINIPROGRAM_REPORT.md](MINIPROGRAM_REPORT.md) 详细文档
2. 查看开发者工具 Console 错误信息
3. 查看后端服务日志

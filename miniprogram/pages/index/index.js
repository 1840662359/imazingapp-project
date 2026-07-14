// pages/index/index.js

const configApi = require('../../api/config.js');

Page({
  data: {
    loading: true,
    error: null,
    fields: [],
    selections: {},
    generating: false,
    lastGenerateTime: 0  // 添加：上次生成时间
  },

  onLoad() {
    this.loadSchema();
  },

  /**
   * 加载配置 schema
   */
  async loadSchema() {
    this.setData({ loading: true, error: null });

    try {
      const schema = await configApi.getSchema();

      // 初始化 selections（使用默认值）
      const selections = {};
      schema.fields.forEach(field => {
        if (field.default !== null && field.default !== undefined) {
          selections[field.id] = this.parseDefaultValue(field);
        }
      });

      this.setData({
        fields: schema.fields,
        selections: selections,
        loading: false
      });
    } catch (err) {
      console.error('加载 schema 失败:', err);
      this.setData({
        loading: false,
        error: err.message || '加载配置失败'
      });
    }
  },

  /**
   * 解析默认值
   */
  parseDefaultValue(field) {
    const defaultValue = field.default;

    if (field.type === 'select') {
      return defaultValue; // 字符串 ID
    } else if (field.type === 'slider') {
      return parseFloat(defaultValue);
    } else if (field.type === 'switch') {
      // Switch 类型：保持为字符串 ID（"on"/"off"），不转换为 boolean
      return defaultValue;
    } else {
      return defaultValue;
    }
  },

  /**
   * 字段值改变
   */
  onFieldChange(e) {
    const { fieldId, value } = e.detail;

    this.setData({
      [`selections.${fieldId}`]: value
    });
  },

  /**
   * 生成配置文件
   */
  async onGenerate() {
    const now = Date.now();
    const COOLDOWN_TIME = 5000; // 5秒冷却时间

    // 检查冷却时间
    if (this.data.lastGenerateTime > 0) {
      const timeSinceLastGenerate = now - this.data.lastGenerateTime;
      if (timeSinceLastGenerate < COOLDOWN_TIME) {
        const remainingTime = Math.ceil((COOLDOWN_TIME - timeSinceLastGenerate) / 1000);
        wx.showToast({
          title: `您生成太过频繁！请等待 ${remainingTime} 秒后再生成`,
          icon: 'none',
          duration: 2000
        });
        return;
      }
    }

    // 防止重复点击
    if (this.data.generating) {
      return;
    }

    this.setData({
      generating: true,
      lastGenerateTime: now
    });

    // 显示加载提示
    wx.showLoading({
      title: '生成中...',
      mask: true
    });

    try {
      // 构造请求参数（使用自定义选择）
      const params = {
        selections: this.formatSelections(this.data.selections)
      };

      // 调用构建 API（后端直接返回文件字节，已写入本地缓存）
      await configApi.build(params);

      wx.hideLoading();
      this.setData({ generating: false });

      // 跳转到下载页面（文件已缓存，无需传 id）
      wx.navigateTo({
        url: `/pages/download/download`
      });
    } catch (err) {
      console.error('生成失败:', err);
      wx.hideLoading();
      this.setData({ generating: false });

      wx.showModal({
        title: '生成失败',
        content: err.message || '生成配置文件失败，请重试',
        showCancel: false
      });
    }
  },

  /**
   * 格式化 selections（转换为 API 期望的格式）
   */
  formatSelections(selections) {
    const formatted = {};

    Object.keys(selections).forEach(fieldId => {
      const value = selections[fieldId];
      const field = this.data.fields.find(f => f.id === fieldId);

      if (!field) return;

      // 根据字段类型格式化值
      if (field.type === 'slider') {
        formatted[fieldId] = parseFloat(value);
      } else if (field.type === 'switch') {
        // Switch 类型：保持为字符串 ID（"on"/"off"），不转换为 Boolean
        formatted[fieldId] = value;
      } else {
        formatted[fieldId] = value;
      }
    });

    return formatted;
  }
});

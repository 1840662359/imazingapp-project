// pages/index/index.js

const { generate } = require('../../engine/generate.js');
const { getSchemaView } = require('../../engine/schema.js');
const { base64ToBytes } = require('../../engine/util.js');
const config = require('../../engine/config.js');
const { CACHE_FILE_PATH, CACHE_NAME_KEY } = require('../../utils/file-cache.js');

// 内嵌预处理模板的 base64（engine/template.js，随代码打包）。
// 懒解码：onLoad 时解码并包 try/catch，避免模块加载期阻塞首屏、
// 且解码失败时能走页面错误提示而非白屏。
const TEMPLATE_B64 = require('../../engine/template.js');
let TEMPLATE_BYTES = null;

Page({
  data: {
    loading: true,
    error: null,
    fields: [],
    selections: {},
    generating: false,
    lastGenerateTime: 0
  },

  onLoad() {
    this.loadSchema();
  },

  /** 原生模板广告事件。 */
  adLoad() {
    console.log('原生模板广告加载成功');
  },

  adError(err) {
    console.error('原生模板广告加载失败', err);
  },

  adClose() {
    console.log('原生模板广告关闭');
  },

  /**
   * 加载配置 schema（本地内嵌配置，纯前端无网络请求）
   */
  loadSchema() {
    // 懒解码内嵌模板（380KB base64 → 285KB 字节），失败走页面错误提示
    if (!TEMPLATE_BYTES) {
      try {
        TEMPLATE_BYTES = base64ToBytes(TEMPLATE_B64);
      } catch (e) {
        console.error('模板解码失败:', e);
      }
    }
    if (!TEMPLATE_BYTES || TEMPLATE_BYTES.length === 0) {
      this.setData({ loading: false, error: '模板数据缺失，请重新安装最新版本' });
      return;
    }

    const schema = getSchemaView(config);

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

  /** 固定底栏消费空白区域点击，避免事件落到下方的滚动列表。 */
  onBottomBarTap() {},

  /** 固定底栏消费滑动手势，避免列表在栏下方继续滚动。 */
  onBottomBarTouchMove() {},

  /**
   * 生成配置文件
   */
  async onGenerate() {
    if (!TEMPLATE_BYTES || TEMPLATE_BYTES.length === 0) {
      wx.showToast({ title: '模板数据缺失，请重新安装最新版本', icon: 'none' });
      return;
    }

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
      // 本地生成（纯前端，无网络请求）
      const result = generate(config, this.data.selections, TEMPLATE_BYTES);
      if (!result.bytes || result.bytes.length === 0) {
        throw new Error('生成结果为空');
      }

      // 写入本地缓存（覆盖上一个，供下载页反复分享）。
      // 注意：writeFileSync 只保证支持 string | ArrayBuffer，
      // Uint8Array 在部分基础库版本会报 empty data，显式转 ArrayBuffer。
      const fs = wx.getFileSystemManager();
      const { byteOffset, byteLength } = result.bytes;
      const arrayBuffer = result.bytes.buffer.slice(byteOffset, byteOffset + byteLength);
      fs.writeFileSync(CACHE_FILE_PATH, arrayBuffer);
      wx.setStorageSync(CACHE_NAME_KEY, result.filename);

      wx.hideLoading();
      this.setData({ generating: false });

      // 跳转到下载页面（文件已缓存）
      wx.navigateTo({
        url: `/pages/download/download`
      });
    } catch (err) {
      console.error('生成失败:', err);
      wx.hideLoading();
      // 失败不占冷却：用户改配置后可立即重试
      this.setData({ generating: false, lastGenerateTime: 0 });

      wx.showModal({
        title: '生成失败',
        content: err.message || '生成配置文件失败，请重试',
        showCancel: false
      });
    }
  }
});

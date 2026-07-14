// pages/download/download.js

const configApi = require('../../api/config.js');

Page({
  data: {
    error: null,
    filename: 'template.imazingapp',
    ready: false // 缓存文件是否可用，可分享
  },

  onLoad() {
    // 文件已在生成时写入本地缓存，这里直接读取（可反复分享）
    const cached = configApi.getCachedFile();
    if (cached) {
      this._filePath = cached.filePath;
      this.setData({
        ready: true,
        filename: cached.filename
      });
    } else {
      this.setData({ error: '未找到已生成的文件，请返回重新生成' });
    }
  },

  /**
   * 分享文件（同步调用，不能有 await —— 微信要求分享在点击手势同步栈内）
   */
  onShare() {
    if (!this.data.ready || !this._filePath) {
      wx.showToast({ title: '请返回重新生成', icon: 'none' });
      return;
    }

    wx.shareFileMessage({
      filePath: this._filePath,
      fileName: this.data.filename,
      success: () => {
        wx.showToast({ title: '分享成功', icon: 'success' });
      },
      fail: (err) => {
        console.error('分享失败:', err);
        // 用户主动取消不算错误
        if (err && err.errMsg && err.errMsg.indexOf('cancel') !== -1) {
          return;
        }
        wx.showModal({
          title: '分享失败',
          content: (err && err.errMsg) || '请重试',
          showCancel: false
        });
      }
    });
  },

  /**
   * 返回首页
   */
  onBack() {
    wx.navigateBack({
      delta: 1,
      fail: () => {
        wx.redirectTo({ url: '/pages/index/index' });
      }
    });
  }
});

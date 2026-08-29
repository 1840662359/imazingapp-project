// pages/download/download.js

const configApi = require('../../api/config.js');

const AD_UNLOCK_EXPIRES_AT_KEY = 'share_ad_unlock_expires_at';
const AD_UNLOCK_DURATION = 24 * 60 * 60 * 1000;

Page({
  data: {
    error: null,
    filename: 'template.imazingapp',
    ready: false, // 缓存文件是否可用，可分享
    adShowing: false
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

    this.createRewardedVideoAd();
  },

  /**
   * 创建激励视频广告。广告完整播放后才允许分享文件。
   */
  createRewardedVideoAd() {
    if (!wx.createRewardedVideoAd) {
      console.error('当前微信基础库不支持激励视频广告');
      return;
    }

    this.videoAd = wx.createRewardedVideoAd({
      adUnitId: 'adunit-b0196d8b9ef3c679'
    });

    this.videoAd.onLoad(() => {
      console.log('激励视频广告加载成功');
    });

    this.videoAd.onError((err) => {
      console.error('激励视频广告加载失败', err);
      this.grantShareAccessOnAdFailure();
    });

    this.videoAd.onClose((res) => {
      this.setData({ adShowing: false });

      // 仅在用户完整观看广告后才发放权益。
      if (res && res.isEnded) {
        this.unlockShareFor24Hours();
        // shareFileMessage 必须由用户点击同步触发，不能在广告回调中直接调用。
        // 因此解锁后需要用户再次点击“分享到微信”完成分享。
        wx.showToast({ title: '已解锁24小时内无限制使用，请再次点击分享', icon: 'none' });
      } else {
        wx.showToast({ title: '需完整观看广告后才能分享', icon: 'none' });
      }
    });
  },

  /** 点击分享：有效期内直接分享，否则先提示观看激励广告。 */
  onShare() {
    if (!this.data.ready || !this._filePath) {
      wx.showToast({ title: '请返回重新生成', icon: 'none' });
      return;
    }

    this.runWithAdUnlock(() => this.shareFile());
  },

  /** 有效期内执行操作，否则提示观看激励广告。 */
  runWithAdUnlock(action) {
    if (this.data.adShowing) {
      return;
    }

    if (this.isShareUnlocked()) {
      // action 在按钮 TAP 手势的同步调用链中执行，分享 API 可正常使用。
      action();
      return;
    }

    if (!this.videoAd) {
      wx.showToast({ title: '广告暂不可用，请稍后重试', icon: 'none' });
      return;
    }

    this.showRewardPrompt();
  },

  /** 弹出广告说明；用户确认后播放激励视频。 */
  showRewardPrompt() {
    wx.showModal({
      title: '系统提示',
      content: '观看一次完整广告，解锁24小时无限制使用',
      cancelText: '下次再说',
      confirmText: '观看广告',
      success: (res) => {
        if (res.confirm) {
          this.showRewardVideo();
        }
      }
    });
  },

  /** 展示激励视频广告，失败时加载后重试一次。 */
  showRewardVideo() {
    this.setData({ adShowing: true });
    this.videoAd.show().catch(() => {
      // 首次展示可能因尚未加载完成而失败，加载后重试一次。
      return this.videoAd.load().then(() => this.videoAd.show());
    }).catch((err) => {
      console.error('激励视频广告展示失败', err);
      this.grantShareAccessOnAdFailure();
    });
  },

  /**
   * 无可用广告时直接授予本地分享权限，不向用户展示广告失败信息。
   */
  grantShareAccessOnAdFailure() {
    const wasShowing = this.data.adShowing;
    this.setData({ adShowing: false });
    this.unlockShareFor24Hours();

    // 由用户点击触发的广告请求失败后，分享仍需下一次 TAP 手势才能调用。
    if (wasShowing) {
      wx.showToast({ title: '已解锁，请再次点击分享', icon: 'none' });
    }
  },

  /** 当前设备上的广告解锁是否仍在 24 小时有效期内。 */
  isShareUnlocked() {
    return Number(wx.getStorageSync(AD_UNLOCK_EXPIRES_AT_KEY)) > Date.now();
  },

  /** 从完整观看广告的当前时刻起，授予 24 小时分享权限。 */
  unlockShareFor24Hours() {
    wx.setStorageSync(AD_UNLOCK_EXPIRES_AT_KEY, Date.now() + AD_UNLOCK_DURATION);
  },

  /**
   * 已完整观看广告后执行文件分享。
   */
  shareFile() {
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

  /** 原生模板广告事件。 */
  adLoad() {
    console.log('下载页原生模板广告加载成功');
  },

  adError(err) {
    console.error('下载页原生模板广告加载失败', err);
  },

  adClose() {
    console.log('下载页原生模板广告关闭');
  },

  onUnload() {
    if (this.videoAd && this.videoAd.destroy) {
      this.videoAd.destroy();
    }
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

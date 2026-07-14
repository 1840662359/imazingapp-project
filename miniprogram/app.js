// app.js
App({
  globalData: {
    // 云环境 ID：云托管控制台顶部 / 「环境设置」可见，形如 prod-xxxxxx
    cloudEnv: 'prod-d7gztt4pv18829d51',
    // 云托管服务名（部署时填的）
    serviceName: 'imazing-server'
  },

  onLaunch() {
    if (wx.cloud) {
      wx.cloud.init({ env: this.globalData.cloudEnv });
      console.log('小程序启动，cloud 已初始化');
    } else {
      console.error('基础库过低，不支持 wx.cloud，请升级到 2.19.0+');
    }
  }
});

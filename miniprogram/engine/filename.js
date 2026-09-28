// miniprogram/engine/filename.js —— 生成文件名（对齐 Rust routes::generate_filename）。
//
// 格式：<分辨率>K<画质><帧率><画面风格><抗锯齿><阴影><渲染优化><亮度>%亮度<冲刺>%冲刺[超广角]
// 例如：2K高清120帧鲜艳抗锯齿开阴影关渲染优化开200%亮度100%冲刺超广角.imazingapp

'use strict';

/**
 * @param {object} selections { fieldId: string | number }
 * @returns {string} 文件名（含 .imazingapp 扩展名）
 */
function generateFilename(selections) {
  let name = '';

  // 1. 分辨率（resolution_scale: 1.00-2.00，最多显示两位小数）
  const resolution = selections.resolution_scale;
  if (typeof resolution === 'number') {
    // 以两位小数收敛浮点噪声，再移除无意义的末尾 0。
    if (Number.isInteger(resolution)) {
      name += `${resolution}K`;
    } else {
      const formatted = resolution.toFixed(2).replace(/0+$/, '').replace(/\.$/, '');
      name += `${formatted}K`;
    }
  }

  // 2. 画质
  const qualityText = { smooth: '流畅', balanced: '均衡', hd: '高清' };
  const quality = selections.quality;
  if (typeof quality === 'string') {
    name += qualityText[quality] || '未知';
  }

  // 3. 帧率
  const fpsText = { extreme: '60帧', fps_90: '90帧', fps_120: '120帧' };
  const fps = selections.fps;
  if (typeof fps === 'string') {
    name += fpsText[fps] || '未知帧';
  }

  // 4. 画面风格
  const styleText = { classic: '经典', vivid: '鲜艳', realistic: '写实', soft: '柔和', movie: '电影' };
  const style = selections.visual_style;
  if (typeof style === 'string') {
    name += styleText[style] || '未知风格';
  }

  // 4.1 抗锯齿
  if (selections.anti_aliasing === 'on') name += '抗锯齿开';
  else if (selections.anti_aliasing === 'off') name += '抗锯齿关';

  // 4.2 阴影
  if (selections.shadow === 'on') name += '阴影开';
  else if (selections.shadow === 'off') name += '阴影关';

  // 4.3 渲染优化
  if (selections.render_optimization === 'on') name += '渲染优化开';
  else if (selections.render_optimization === 'off') name += '渲染优化关';

  // 5. 屏幕亮度（0.8-5.0，显示 80%-500%）
  if (typeof selections.brightness === 'number') {
    name += `${Math.round(selections.brightness * 100)}%亮度`;
  }

  // 6. 冲刺灵敏度（0.0-10.0，显示 0%-1000%）
  if (typeof selections.sprint_sensitivity === 'number') {
    name += `${Math.round(selections.sprint_sensitivity * 100)}%冲刺`;
  }

  // 7. 超广角（仅开启时标注）
  if (selections.camera_fov === 'on') name += '超广角';

  name += '.imazingapp';
  return name;
}

module.exports = { generateFilename };

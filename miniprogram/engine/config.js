// 由 scripts/build-config.js 从 config/*.json 自动生成，请勿手改。
// 数据源变更后重新运行: node scripts/build-config.js
'use strict';

module.exports = {
  "schema": {
    "fields": [
      {
        "id": "quality",
        "type": "select",
        "group": "画面",
        "label": "画质",
        "options": [
          {
            "id": "smooth",
            "label": "流畅"
          },
          {
            "id": "balanced",
            "label": "均衡"
          },
          {
            "id": "hd",
            "label": "高清"
          }
        ],
        "default": "hd"
      },
      {
        "id": "fps",
        "type": "select",
        "group": "性能",
        "label": "帧率",
        "options": [
          {
            "id": "extreme",
            "label": "极限 (60帧)"
          },
          {
            "id": "fps_90",
            "label": "90帧"
          },
          {
            "id": "fps_120",
            "label": "120帧"
          }
        ],
        "default": "fps_120"
      },
      {
        "id": "resolution_scale",
        "type": "slider",
        "group": "画面",
        "label": "分辨率",
        "min": 1,
        "max": 2,
        "step": 0.1,
        "default": 2
      },
      {
        "id": "visual_style",
        "type": "select",
        "group": "画面",
        "label": "画面风格",
        "options": [
          {
            "id": "classic",
            "label": "经典"
          },
          {
            "id": "vivid",
            "label": "鲜艳"
          },
          {
            "id": "realistic",
            "label": "写实"
          },
          {
            "id": "soft",
            "label": "柔和"
          },
          {
            "id": "movie",
            "label": "电影"
          }
        ],
        "default": "vivid"
      },
      {
        "id": "anti_aliasing",
        "type": "switch",
        "group": "画面",
        "label": "抗锯齿",
        "options": [
          {
            "id": "on",
            "label": "开启"
          },
          {
            "id": "off",
            "label": "关闭"
          }
        ],
        "default": "on"
      },
      {
        "id": "shadow",
        "type": "switch",
        "group": "画面",
        "label": "阴影",
        "options": [
          {
            "id": "on",
            "label": "开启"
          },
          {
            "id": "off",
            "label": "关闭"
          }
        ],
        "default": "on"
      },
      {
        "id": "render_optimization",
        "type": "switch",
        "group": "性能",
        "label": "渲染优化",
        "options": [
          {
            "id": "on",
            "label": "开启"
          },
          {
            "id": "off",
            "label": "关闭"
          }
        ],
        "default": "off"
      },
      {
        "id": "brightness",
        "type": "slider",
        "group": "显示",
        "label": "屏幕亮度",
        "min": 0.8,
        "max": 5,
        "step": 0.1,
        "default": 2,
        "display_unit": "%",
        "display_multiplier": 100
      },
      {
        "id": "camera_fov",
        "type": "switch",
        "group": "视野",
        "label": "超广角",
        "options": [
          {
            "id": "on",
            "label": "开启"
          },
          {
            "id": "off",
            "label": "关闭"
          }
        ],
        "default": "on"
      },
      {
        "id": "sprint_sensitivity",
        "type": "slider",
        "group": "操控",
        "label": "冲刺灵敏度",
        "min": 0,
        "max": 10,
        "step": 0.1,
        "default": 1,
        "display_unit": "%",
        "display_multiplier": 100
      }
    ]
  },
  "patches": {
    "bindings": {
      "fps": [
        {
          "target": "user_ini",
          "cvar": "r.PUBGDeviceFPSDef",
          "value_type": "int",
          "value_map": {
            "extreme": 60,
            "fps_90": 7,
            "fps_120": 8
          }
        },
        {
          "target": "user_ini",
          "cvar": "r.PUBGDeviceFPSLow",
          "value_type": "int",
          "value_map": {
            "extreme": 60,
            "fps_90": 7,
            "fps_120": 8
          }
        },
        {
          "target": "user_ini",
          "cvar": "r.PUBGDeviceFPSMid",
          "value_type": "int",
          "value_map": {
            "extreme": 60,
            "fps_90": 7,
            "fps_120": 8
          }
        },
        {
          "target": "user_ini",
          "cvar": "r.PUBGDeviceFPSHigh",
          "value_type": "int",
          "value_map": {
            "extreme": 60,
            "fps_90": 7,
            "fps_120": 8
          }
        },
        {
          "target": "user_ini",
          "cvar": "r.PUBGDeviceFPSHDR",
          "value_type": "int",
          "value_map": {
            "extreme": 60,
            "fps_90": 7,
            "fps_120": 8
          }
        },
        {
          "target": "user_ini",
          "cvar": "r.PUBGDeviceFPSUltralHigh",
          "value_type": "int",
          "value_map": {
            "extreme": 60,
            "fps_90": 7,
            "fps_120": 8
          }
        },
        {
          "target": "user_ini",
          "cvar": "r.PUBGDeviceFPSUltimateHigh",
          "value_type": "int",
          "value_map": {
            "extreme": 60,
            "fps_90": 7,
            "fps_120": 8
          }
        },
        {
          "target": "active_sav",
          "property": "FPSLevel",
          "value_type": "int",
          "value_map": {
            "extreme": 6,
            "fps_90": 7,
            "fps_120": 8
          }
        },
        {
          "target": "active_sav",
          "property": "FpsLevel_TA",
          "value_type": "int",
          "value_map": {
            "extreme": 6,
            "fps_90": 7,
            "fps_120": 8
          }
        }
      ],
      "quality": [
        {
          "target": "user_ini",
          "cvar": "r.UserQualitySetting",
          "value_type": "int",
          "value_map": {
            "smooth": 0,
            "balanced": 1,
            "hd": 2
          }
        },
        {
          "target": "user_ini",
          "cvar": "r.UserQualitySettingTA",
          "value_type": "int",
          "value_map": {
            "smooth": 0,
            "balanced": 1,
            "hd": 2
          }
        }
      ],
      "resolution_scale": [
        {
          "target": "user_ini",
          "cvar": "r.MobileContentScaleFactor",
          "value_type": "float",
          "value_from_input": true
        }
      ],
      "visual_style": [
        {
          "target": "user_ini",
          "cvar": "r.UserHDRSetting",
          "value_type": "int",
          "value_map": {
            "classic": 1,
            "vivid": 2,
            "realistic": 3,
            "soft": 4,
            "movie": 6
          }
        }
      ],
      "anti_aliasing": [
        {
          "target": "user_ini",
          "cvar": "r.UserMSAASetting",
          "value_type": "int",
          "value_map": {
            "off": 0,
            "on": 1
          }
        }
      ],
      "shadow": [
        {
          "target": "user_ini",
          "cvar": "r.UserShadowSwitch",
          "value_type": "int",
          "value_map": {
            "off": 0,
            "on": 1
          }
        }
      ],
      "brightness": [
        {
          "target": "active_sav",
          "property": "ScreenLightness",
          "value_type": "float",
          "value_from_input": true
        }
      ],
      "camera_fov": [
        {
          "target": "active_sav",
          "property": "TpViewValue",
          "value_type": "int",
          "value_map": {
            "on": 225,
            "off": 80
          }
        }
      ],
      "sprint_sensitivity": [
        {
          "target": "active_sav",
          "property": "JoystickSprintParam",
          "value_type": "float",
          "value_from_input": true
        }
      ],
      "render_optimization": [
        {
          "target": "user_ini",
          "cvar": "r.BloomQuality",
          "value_type": "float",
          "value_map": {
            "on": 0
          },
          "remove_when": "off"
        },
        {
          "target": "user_ini",
          "cvar": "r.LightShaftQuality",
          "value_type": "float",
          "value_map": {
            "on": 0
          },
          "remove_when": "off"
        },
        {
          "target": "user_ini",
          "cvar": "r.Mobile.TonemapperFilm",
          "value_type": "int",
          "value_map": {
            "on": 0
          },
          "remove_when": "off"
        },
        {
          "target": "user_ini",
          "cvar": "r.Mobile.AlwaysResolveDepth",
          "value_type": "int",
          "value_map": {
            "on": 0
          },
          "remove_when": "off"
        },
        {
          "target": "user_ini",
          "cvar": "r.MobileSimpleShader",
          "value_type": "int",
          "value_map": {
            "on": 1
          },
          "remove_when": "off"
        }
      ]
    }
  },
  "targets": {
    "targets": {
      "active_sav": {
        "kind": "save",
        "zip_path": "Container/Documents/ShadowTrackerExtra/Saved/SaveGames/Active.sav"
      },
      "user_ini": {
        "kind": "ini",
        "zip_path": "Container/Documents/ShadowTrackerExtra/Saved/Config/IOS/UserCustom.ini",
        "section": "[UserCustom DeviceProfile]"
      }
    }
  },
  "versions": {
    "config_schema_version": 1,
    "template_version": "1.0.0",
    "comment": "和平精英 imazingapp 配置生成器"
  }
};

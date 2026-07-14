// components/dynamic-field/dynamic-field.js

Component({
  properties: {
    // 字段定义（来自 schema）
    field: {
      type: Object,
      value: {}
    },
    // 当前值
    value: {
      type: null,
      value: null
    }
  },

  data: {
    selectedIndex: 0,
    currentLabel: '',
    displayValue: '',
    switchChecked: false  // 添加 switch 的 checked 状态
  },

  lifetimes: {
    attached() {
      this.initValue();
    }
  },

  observers: {
    'field, value': function(field, value) {
      this.initValue();
    }
  },

  methods: {
    /**
     * 初始化值
     */
    initValue() {
      const { field, value } = this.properties;

      if (field.type === 'select') {
        // 找到当前选中项的索引
        const index = field.options?.findIndex(opt => opt.id === value) || 0;
        const label = field.options?.[index]?.label || '请选择';
        this.setData({
          selectedIndex: index,
          currentLabel: label
        });
      } else if (field.type === 'slider') {
        // 计算显示值
        this.setData({
          displayValue: this.formatDisplayValue(field, value)
        });
      } else if (field.type === 'switch') {
        // Switch 类型：将字符串 ID 转换为 boolean
        // 假设 options[0] 是 "on"，options[1] 是 "off"
        let checked = false;
        if (field.options && field.options.length >= 2) {
          checked = (value === field.options[0].id);
        } else {
          checked = Boolean(value);
        }
        this.setData({
          switchChecked: checked
        });
      }
    },

    /**
     * 格式化显示值（用于 slider）
     */
    formatDisplayValue(field, value) {
      const numValue = parseFloat(value) || 0;

      // 检查是否有自定义显示范围
      if (field.display_min !== undefined && field.display_max !== undefined) {
        // 计算比例
        const actualMin = field.min || 0;
        const actualMax = field.max || 1;
        const actualRange = actualMax - actualMin;
        const displayRange = field.display_max - field.display_min;

        // 转换值
        const ratio = (numValue - actualMin) / actualRange;
        const displayValue = field.display_min + (ratio * displayRange);

        // 格式化显示
        if (field.unit === '%') {
          return Math.round(displayValue) + '%';
        }
        return displayValue.toFixed(1);
      }

      // 特殊处理：分辨率字段添加 K 后缀
      if (field.id === 'resolution_scale') {
        if (Number.isInteger(numValue)) {
          return numValue + 'K';
        }
        return numValue.toFixed(1) + 'K';
      }

      // 默认显示
      if (Number.isInteger(numValue)) {
        return numValue.toString();
      }
      return numValue.toFixed(1);
    },

    /**
     * Picker 改变
     */
    onPickerChange(e) {
      const index = e.detail.value;
      const option = this.properties.field.options[index];

      this.setData({
        selectedIndex: index,
        currentLabel: option.label
      });

      this.triggerEvent('change', {
        fieldId: this.properties.field.id,
        value: option.id
      });
    },

    /**
     * Slider 改变（松开）
     */
    onSliderChange(e) {
      const value = e.detail.value;

      // 更新显示值
      this.setData({
        displayValue: this.formatDisplayValue(this.properties.field, value)
      });

      this.triggerEvent('change', {
        fieldId: this.properties.field.id,
        value: value
      });
    },

    /**
     * Slider 滑动中
     */
    onSliderChanging(e) {
      // 实时更新显示值
      const value = e.detail.value;
      this.setData({
        displayValue: this.formatDisplayValue(this.properties.field, value)
      });
    },

    /**
     * Switch 改变
     */
    onSwitchChange(e) {
      const checked = e.detail.value; // true 或 false
      const field = this.properties.field;

      // 更新 UI 状态
      this.setData({
        switchChecked: checked
      });

      // Switch 类型需要根据 options 返回对应的选项 ID
      let value;
      if (field.options && field.options.length >= 2) {
        // 有 options：返回第一个或第二个选项的 ID
        // 假设 options[0] 是 "on/开启"，options[1] 是 "off/关闭"
        value = checked ? field.options[0].id : field.options[1].id;
      } else {
        // 没有 options：直接返回 boolean（兼容）
        value = checked;
      }

      this.triggerEvent('change', {
        fieldId: field.id,
        value: value
      });
    },

    /**
     * Input 改变
     */
    onInputChange(e) {
      const value = e.detail.value;

      this.triggerEvent('change', {
        fieldId: this.properties.field.id,
        value: value
      });
    }
  }
});

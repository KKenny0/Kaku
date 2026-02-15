# 中文字体测试指南

## 测试步骤

1. 打开 Kaku
2. 运行 `echo '中文测试：你好世界 中文字体对比'`
3. 编辑 `~/.config/kaku/kaku.lua`
4. 依次测试以下配置，观察字体变化

## 测试配置

### 测试1: 系统默认（不指定中文字体）
```lua
config.font = wezterm.font_with_fallback({
  { family = 'JetBrains Mono', weight = 'Regular' },
  'Apple Color Emoji',
})
```

### 测试2: Hiragino Sans GB（冬青黑体）
```lua
config.font = wezterm.font_with_fallback({
  { family = 'JetBrains Mono', weight = 'Regular' },
  { family = 'Hiragino Sans GB', weight = 'Regular' },
  'Apple Color Emoji',
})
```

### 测试3: Songti SC（宋体 - 衬线字体，明显不同）
```lua
config.font = wezterm.font_with_fallback({
  { family = 'JetBrains Mono', weight = 'Regular' },
  { family = 'Songti SC', weight = 'Regular' },
  'Apple Color Emoji',
})
```

### 测试4: Heiti SC（黑体）
```lua
config.font = wezterm.font_with_fallback({
  { family = 'JetBrains Mono', weight = 'Regular' },
  { family = 'Heiti SC', weight = 'Regular' },
  'Apple Color Emoji',
})
```

### 测试5: Hiragino Mincho ProN（明朝体 - 衬线）
```lua
config.font = wezterm.font_with_fallback({
  { family = 'JetBrains Mono', weight = 'Regular' },
  { family = 'Hiragino Mincho ProN', weight = 'Regular' },
  'Apple Color Emoji',
})
```

## 字体特点对比

| 字体 | 类型 | 特点 |
|------|------|------|
| Hiragino Sans GB | 无衬线黑体 | 紧凑，笔画清晰 |
| Songti SC | 衬线宋体 | 笔画有装饰，传统 |
| Heiti SC | 无衬线黑体 | 系统默认，较宽 |
| Hiragino Mincho ProN | 衬线明朝体 | 日本风格，优雅 |

## 如何判断字体生效

1. **宋体**：中文笔画末端有明显的"脚"（衬线）
2. **黑体/冬青黑体**：笔画平直，末端无装饰
3. **明朝体**：横细竖粗，有装饰性笔画

## 推荐配置

如果希望中文更紧凑：
```lua
config.font = wezterm.font_with_fallback({
  { family = 'JetBrains Mono', weight = 'Regular' },
  { family = 'Hiragino Sans GB', weight = 'Regular' },
  'Apple Color Emoji',
})
config.font_size = 17.0
config.cell_width = 0.9  -- 稍微压缩
```

# V0.7.1 Flow 🌊

<div align="center">
  <img src="https://raw.githubusercontent.com/tw93/Kaku/main/assets/logo.png" alt="Kaku Logo" width="120" height="120" />
  <h1 style="margin: 12px 0 6px;">Kaku V0.7.1</h1>
  <p><em>A fast, out-of-the-box terminal built for AI coding.</em></p>
</div>

### Changelog

1. **Auto Theme & Visual Polish**: Auto-switching between dark/light modes with macOS, improved transparency rendering, rounded scrollbars, and Yazi flavor sync.

2. **Safer Close & Interaction**: Tab/pane close confirmations, double-click zoom no longer blocks title bar drag, and refreshed overlay styling.

3. **Settings TUI Redesign**: Clearer grouped sections, pinned footer with contextual key hints, safer config parsing, and reliable config reload when the TUI exits.

4. **AI Config v2**: Antigravity model support, quota tracking, background loading, and more reliable OAuth token refresh.

5. **Pane Input Broadcast**: Synchronized input across multiple panes with safeguards to prevent overlay input from being broadcast.

6. **File & Editor Workflow**: Enhanced file path link opening, remote files shortcut for SSH sessions, and `$EDITOR` environment respect.

7. **Bug Fixes & Stability**: Fixed AeroSpace/yabai window flickering on resize, hardened managed shell/tmux integration, Smart Tab scoped to Kaku sessions only, and Starship RPROMPT fallback fixes.

### 更新日志

1. **自动主题与视觉优化**：跟随 macOS 自动切换深色/浅色模式，优化透明度渲染，新增圆角滚动条，Yazi 风格同步跟随。

2. **更安全的关闭与交互**：标签/窗格关闭确认，双击缩放不再干扰标题栏拖拽，关闭浮层样式重做。

3. **设置 TUI 重构**：`kaku config` 分组结构更清晰，底部操作区固定显示上下文快捷键，配置解析更稳健，TUI 退出后配置重载更可靠。

4. **AI 配置升级**：支持 Antigravity 模型与额度追踪，后台加载，OAuth token 刷新更可靠。

5. **窗格输入广播**：多窗格间同步输入，并防止浮层输入被错误广播。

6. **文件与编辑器工作流**：改进文件路径链接打开，SSH 会话增加远程文件快捷入口，尊重 `$EDITOR` 环境变量。

7. **Bug 修复与稳定性**：修复 AeroSpace/yabai 调整窗口大小时的闪烁问题，加固托管 shell/tmux 集成，Smart Tab 默认仅限 Kaku 会话，修复 Starship RPROMPT 回退。

Special thanks to @frankekn, @crossly, @iwen-conf, and @zxh326 for their contributions to this release.

> https://github.com/tw93/Kaku

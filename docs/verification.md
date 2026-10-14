# 0.1.0 开发预览验证记录

验证日期：2026-10-14。环境：Windows 10 专业版 19045，Node.js 22.19.0，Rust 1.93.0。

## 已通过

- `npm run build`：TypeScript 检查与 Vite 构建成功。
- `npm run format:check`：前端和配置的 Prettier 格式检查通过。
- `cargo test --locked -j 2 --manifest-path src-tauri/Cargo.toml`：7 项测试通过。
- `cargo +1.92.0 fmt --manifest-path src-tauri/Cargo.toml`：使用本机已安装的格式化器完成格式化。
- `cargo +1.92.0 fmt --manifest-path src-tauri/Cargo.toml -- --check`：Rust 格式检查通过。
- `npm run tauri -- build --debug --no-bundle`：生成嵌入前端资源的 Windows 可执行文件。
- 可执行文件 `--scan` 只读扫描：当次发现 99 个注册项、25 个可修改项、0 条读取警告。数量是本机快照，不是产品固定覆盖数量。
- 原生进程启动，窗口标题为“右键管理”，Windows 报告进程正常响应。

## 测试覆盖

1. 拒绝越界菜单目标、子路径注入及不支持的菜单类型。
2. Unicode 注册表字符串的原始字节往返。
3. 修改 `LegacyDisable` 时保留其他值，恢复原始值类型及字节。
4. 在 `HKCU\Software\RightClickManagerTests\<随机标识>` 隔离位置验证真正的修改函数、恢复和外部冲突拒绝。
5. 恢复记录持久化及覆盖更新、记录 ID 路径校验。
6. 部分创建状态与额外外部修改的区分。
7. 使用临时自有菜单执行添加、扫描、停用、启用、按历史逆序恢复和撤销添加；测试菜单及对应临时日志已清理。未修改原有软件的菜单。

## 尚未完成或受环境限制

- 原生窗口截图连续出现 `FrameArrived timed out` / `window capture timed out`，辅助功能接口未返回页面内容。因此没有把原生视觉验收或界面完整交互声明为已通过。
- 管理员 UAC 辅助进程路径已实现并编译，但未实际授权修改机器级菜单；取消授权和实际提权后的 IPC 尚需人工验证。
- 尚未在真实资源管理器中人工确认停用后消失、恢复后重现，以及中文和特殊字符路径的命令执行效果。
- Clippy 使用已安装的 Rust 1.92.0 工具链尝试运行，依赖 `serde_json` 的构建程序被 Windows 以错误 5（拒绝访问）阻止，检查未完成。没有修改系统安全设置。

这些限制不影响已完成的源码构建、只读扫描和隔离注册表测试，但可执行文件应按开发预览使用。

# Windows 右键菜单管理工具

使用 Tauri 2、React、TypeScript 和 Rust 构建的本地桌面工具。查看、停用、启用及恢复传统右键菜单，并添加自定义程序或 PowerShell 脚本。

## 当前状态

0.2.0 开发预览。默认显示“实际菜单”：手动右键一次，读取资源管理器实际弹出的菜单文字、顺序、灰色状态、分隔线和已展开的子菜单。原有扫描与管理入口位于“注册项”。

原生视觉验收、机器级 UAC 路径和真实资源管理器中的显示效果尚待人工验证，详见 [验证记录](docs/verification.md)。

## 本地使用

本机已构建的预览文件：`artifacts/右键管理-0.2.0.exe`。运行时需要 Windows WebView2 Runtime，不依赖 Node.js 服务。

使用流程和第一版支持范围见 [使用说明](docs/usage.md)。构建产物与运行时恢复数据均不提交 Git。

## 开发

目标系统为 Windows 10 x64。当前验证环境使用 Node.js 22.19.0、Rust 1.93.0，另需 MSVC 构建工具、Windows SDK 和 WebView2。

```powershell
npm ci
npm run tauri -- dev
```

生成可执行预览：

```powershell
npm run tauri -- build --debug --no-bundle
```

输出为 `src-tauri/target/debug/right-click-manager.exe`。去掉 `--debug` 可生成 release 构建，当前仓库未启用安装包打包。

```powershell
npm run build
cargo test --locked -j 2 --manifest-path src-tauri/Cargo.toml
```

`npm run dev` 仅启动浏览器界面；真实扫描和系统修改需要 Tauri 桌面环境。

## 项目结构

- `src/`：React 界面与类型明确的 Tauri 接口。
- `src-tauri/src/registry.rs`：扫描、类型判断、注册表快照与受限修改。
- `src-tauri/src/capture.rs`：独立进程读取 Explorer 可见菜单的 MSAA 信息，超时退出，不执行菜单命令。
- `src-tauri/src/service.rs`：变更、恢复和历史顺序校验。
- `src-tauri/src/journal.rs`：本地 JSON 恢复记录。
- `src-tauri/src/elevation.rs`：机器级操作的 UAC 辅助入口与本地管道。
- `prototype/`：独立交互原型及设计截图，使用示例数据。
- `docs/`：架构、使用说明和验证边界。

## 界面原型

运行 `node prototype/serve.mjs`，访问 http://127.0.0.1:4173 。原型不读写系统注册表，正式桌面应用使用真实数据。

## Git

使用本地 `main` 分支管理，保留 npm 与 Cargo 锁文件。尚未连接 GitHub，开源许可证尚未选择。

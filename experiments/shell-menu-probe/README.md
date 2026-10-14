# 只读 Shell 菜单探针

此实验使用 Windows 的 IContextMenu 生成菜单，读取文字、规范动词和子菜单，不调用 InvokeCommand。用途是比较 Shell 自动生成结果与真实桌面菜单的差异；产品使用 MSAA 读取实际弹出的菜单，不使用此探针的结果。

`cargo run --manifest-path experiments/shell-menu-probe/Cargo.toml -- view` 可生成一个文件夹背景菜单。`active` 尝试跨 Explorer 取得桌面活动视图，本机 IContextMenu 跨进程接口返回 0x80040155，实验保留该限制作为诊断证据。

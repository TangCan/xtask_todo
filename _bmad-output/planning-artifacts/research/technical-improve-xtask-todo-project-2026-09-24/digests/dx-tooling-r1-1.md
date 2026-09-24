# D1 开发者体验/工具链 — digest (round 1)

检索日期 2026-09-24。claims 均来自本次联网命中的一手/二手来源。

1. clap v4 是 Rust CLI 参数解析事实标准，derive 是官方默认推荐写法。source: https://lib.rs/crates/clap_complete | publisher: rust-cli/clap-rs | high | 生态信号
2. clap_complete 4.6.x (2026-09) 约 1011 万次/月下载，被 4229 个 crate 使用，原生生成 bash/zsh/fish/elvish/powershell 的静态补全。source: https://lib.rs/crates/clap_complete | high | 版本兼容
3. clap_complete 提供 `unstable-dynamic` 动态补全（CompleteEnv::with_factory + ValueCompleter），Tab 时实时计算候选。source: https://deepwiki.com/clap-rs/clap/6.2-dynamic-completions | medium | 模式
4. 补全分发惯用做法是内置 `completions <shell>` 子命令；clap_complete_command v0.6.x 用 ValueEnum<Shell> 消除样板。source: https://crates.io/crates/clap_complete_command | high | 模式
5. ratatui 2023 年 fork 自 tui-rs，默认后端 crossterm，immediate-mode 渲染。source: https://docs.rs/ratatui | high | 生态信号
6. ratatui-crossterm 2024-09 起拆为独立 crate（MSRV 1.88），集成需版本对齐。source: https://crates.io/crates/ratatui-crossterm | high | 版本兼容
7. 终端交互提示库 dialoguer 下载量居首（约 2759 万，v0.11.0），inquire 次之（约 505 万，v0.7.5）。source: https://crates.io/search?q=prompts | high | 生态信号
8. inquire 提供 Text（内建补全）/Select/MultiSelect/Confirm/Password/Editor 等，支持 UNIX/Windows。source: https://docs.rs/inquire/latest | high | 功能
9. indicatif 是进度条/spinner 事实标准：默认 stderr、每秒≤20 次刷新；配套 tracing-indicatif。source: https://docs.rs/indicatif | high | 模式
10. anstream（rust-cli，14.4 亿下载）跨平台自动降级 ANSI，是 clap 默认输出后端。source: https://crates.io/crates/anstream | high | 生态信号
11. Windows ANSI：rustc ColorChoice::Always 可强制；enable-ansi-support（Win10+）可启用。source: https://doc.rust-lang.org/rustc_errors/ColorChoice | high/medium | 平台
12. just 定位是 command runner 而非 build system；2016 年"不支持 Windows"说法已过时，当前跨平台。source: https://just.systems（安装指南） | medium | 版本兼容
13. cargo-make 用 TOML 定义任务流：DAG 依赖、平台 override、多脚本引擎。source: https://docs.rs/cargo-make | high | 模式
14. watchexec v4+（Tokio）把事件源/过滤/进程监督解耦（notify/supervisor/clearscreen），"watch+run" 可借鉴。source: https://docs.rs/watchexec | high | 模式
15. Nushell 以结构化数据+自定义命令 def 提供 dev 命令的 shell 语言层方案，与 just/cargo-make 不同层。source: https://nushell.sh/book/custom_commands | high | 模式

Leads: ratatui 拆分 API 迁移；clap_complete unstable-dynamic 何时转正；dialoguer/inquire 依赖深度对比。
未找到: WSL/Lima 专属 CLI 一手来源；clap 解析性能实测；Fig 补全生态。
# D4 生态/竞品对标 — digest (round 1)

检索日期 2026-09-24。

1. Taskwarrior：成熟待办 CLI，任务作"列表处理"，含 status/project/tag/priority/urgency 排序/recur/UDA/custom reports 与 `task project:Home +tag` 过滤。source: https://taskwarrior.org/docs/examples/ | high | todo 竞品
2. todo.txt：纯文本一行一任务，支持 (A) 优先级、+project、@context、x 完成、due: 扩展；理念"数据归用户"。source: https://todotxt.org/ | high | todo 竞品
3. todo.txt-cli（todo.sh shell 实现）：add/list/do/archive/deduplicate/depri/pri，归档移 completed 至 done.txt。source: https://sources.debian.org/src/todotxt-cli/2.11.0-2/USAGE.md/ | high | todo 竞品
4. todo.txt 生态长尾但老旧：官方 iOS 最后提交 2014；仍有 Simpletask/Markor/pter 活跃客户端。source: https://prodcs.lwn.net/Articles/824333/ | medium | 生态信号
5. Taskwarrior v3（Taskchampion，Rust 重写）不再支持 Taskserver，同步改 Taskchampion sync server 需手动迁移。source: https://fedoraproject.org/wiki/Changes/Taskwarrior3 | high | 生态信号
6. Taskchampion crate 自述"仿 TaskWarrior 非 drop-in，仍开发中"，服务端保证任务细节不可见。source: https://crates.io/crates/taskchampion/0.4.1 | high | 生态信号
7. just：Rust 命令运行器，justfile recipe 语法，无 .PHONY，跨平台，支持参数/.env/modules。source: https://just.systems/man/en/ | high | dev 任务运行器
8. just 发布密集：183 个 release，约每 2-3 周一版，最新 1.58.0(2026-08)。source: https://releasealert.dev/github/casey/just | medium | 生态信号
9. cargo-make：TOML Makefile.toml 定义 tasks/dependencies/aliases，跨平台 override、predefined flows、install_crate。source: https://docs.rs/crate/cargo-make/0.35.14 | high | dev 任务运行器
10. cargo-xtask 模式：把仓库任务写成 Rust crate，经 .cargo/config.toml alias 调用，matklad 推广，被 rust-analyzer/tokio/egui 采用。source: https://blog.csdn.net/gitblog_00803/article/details/160556856 | medium | dev 任务运行器
11. cli-xtask：可复用 Rust 库，内置 build/clippy/fmt/test/dist/udeps 子命令。source: https://docs.rs/cli-xtask/latest/cli_xtask/struct.Xtask.html | high | dev 任务运行器
12. watchexec：监听路径变化执行命令，Tokio 驱动，maintained。source: https://crates.io/crates/watchexec/4.1.0 | high | dev 任务运行器
13. ninja：极快增量构建，.ninja 应由生成器产出。source: https://blog.csdn.net/gitblog_00189/article/details/163265527 | medium | dev 任务运行器
14. todo-txt crate（4.1.1）提供 todo.txt 解析器，可作 import/export 直接依赖。source: https://docs.rs/crate/todo-txt/latest | high | 差异化机会

Leads: Todoist CLI(td) 面向 AI agent 的 skill/JSON/MCP 输出，提示"本地-first + LLM 导出"差异点。
未找到: vixie 与 mori 官方仓库/文档；目标项目自身指标；ninja 官方 manual 原文。
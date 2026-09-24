# D3 性能与可靠性 — digest (round 1)

检索日期 2026-09-24。

1. Cargo release 默认 opt-level=3, lto=false, codegen-units=16, panic=unwind, strip=none，是"可用未极致"基线。source: https://doc.rust-lang.org/beta/nightly-rustc/cargo/workspace/profiles/struct.Profile.html | high | 配置基线
2. strip 单点性价比最高：-50%~-70% 体积、不损运行时，代价是丢 crash 符号（debug="line-tables-only" 折中）。source: https://blog.csdn.net/gitblog_01067/article/details/160322749 | medium | 性能数字
3. panic="abort" 减 5%~10% 体积但使 catch_unwind 失效。source: https://blog.csdn.net/gitblog_01067/article/details/160322749 | medium | 模式
4. opt-level="z" 专注体积（-15%~-40% 体积、-10%~-20% 速度），CLI 场景 "s"/"3" 常更优。source: https://tauri.app/zh-cn/concept/size/ | medium | 性能数字
5. lto=true 跨 crate 优化（体积 -10%~-20%、编译慢 2~5x）；codegen-units=1 再 -5%~-10%。source: https://blog.csdn.net/gitblog_01067/article/details/160322749 | medium | 性能数字
6. cargo-bloat（RazrFalcon）按函数/按 crate 分析 .text 段，支持 ELF/Mach-O/PE。source: https://libraries.io/cargo/cargo-bloat | high | 工具
7. 错误处理共识：库层 thiserror 强类型、应用层 anyhow 传播+context()；bail!/ensure!。source: https://blog.gitcode.com/1cff719b171275d9afa81ab549f11124.html | high | 模式
8. anyhow main()->Result<()> Err 时打印并退出码 1；panic 默认 101；sysexits/exitcode crate 表达不同失败。source: https://rust-lang-translations.org/rust-cli/in-depth/exit-code.html | high | 模式
9. 退出码实践：参数错误 2、业务失败 3…，勿全部退 1、勿失败仍退 0。source: https://users.rust-lang.org/t/why-have-main-return-result/77175 | medium | 模式
10. tracing 是结构化可观测标准（span+event+subscriber），无 subscriber 关心时近乎零开销。source: https://docs.rs/tracing/latest/tracing/ | high | 模式/性能
11. tracing-subscriber：EnvFilter(RUST_LOG)、JSON、tracing-appender 滚动、tracing-log 桥接。source: https://rustlang.com.br/ecossistema/tracing/ | medium | 模式
12. 数据文件容错：#[serde(default)] 缺字段回退、#[serde(tag="version")] 版本化迁移；figment 分层配置。source: https://users.rust-lang.org/t/is-there-exists-a-config-management-crate/139952 | medium | 模式
13. Rayon par_iter 默认 CPU 核数、工作窃取；小数据量下切分开销可能反超，需 with_min_len 控制粒度。source: https://users.rust-lang.org/t/speeding-up-parallel-iteration-over-large-data/72460 | medium | 性能数字

Leads: Cargo 自身 anyhow+退出码实操；release 从十几 MB 降 4.2MB 示例；nightly 体积开关。
未找到: CLI 冷启动 latency 权威 benchmark；anyhow vs thiserror 官方对比原文；千万级条目量级建议。
# D2 测试与质量 — digest (round 1)

检索日期 2026-09-24。

1. cargo-nextest 采用"每测试一进程"模型，用例级并行，支持泄漏检测与失败重试，快于 cargo test。source: https://nexte.st | publisher: nextest-rs 官方 | high | 测试运行器
2. nextest 相对 cargo test 最高约 3x（tokio 1014 例 27.16s→11.72s 历史基准）。source: https://app.daily.dev/posts/b5uqnxmg2 | medium | 测试运行器
3. nextest 不支持 doctest，官方推荐 `cargo nextest run && cargo test --doc`。source: https://nexte.st | high | 测试运行器
4. cargo-nextest 0.9.143 持续迭代，被 tokio/uv/codex 采用。source: https://newreleases.io/project/github/nextest-rs/nextest | high | 测试运行器
5. 覆盖率选型：cargo-llvm-cov（source-based，最准、跨平台、支持分支）vs cargo-tarpaulin（ptrace、Linux 限定）vs grcov（合并多轮）。source: http://microsoft.github.io/RustTraining/engineering-book/ch04-code-coverage-seeing-what-tests-miss.html | high | 覆盖率
6. cargo-llvm-cov 需 llvm-tools-preview；`--branch` 分支覆盖 unstable（rust-lang/rust#124118）。source: https://www.libhunt.com/compare-cargo-llvm-cov-vs-tarpaulin | medium | 覆盖率
7. cargo-tarpaulin 新增 LLVM 后端但默认仍是 Linux/x86_64 Ptrace；跨平台精确覆盖优先 cargo-llvm-cov。source: https://tools.corrode.dev/llms.txt | high | 覆盖率
8. 覆盖率门禁 `cargo llvm-cov --fail-under-lines 100` 强制 100% 行覆盖（crap-score 项目）。source: https://deepwiki.com/deangrant/crap-score/7.1-ci-workflows-and-quality-gates | medium | CI 门禁
9. clap 官方 Testing 章节推荐 trycmd（批量快照）、snapbox、assert_cmd+assert_fs，指向 rust-cli 官方书。source: https://docs.rs/clap/4.5.59 | high | CLI 测试
10. trycmd 用 TRYCMD=dump/overwrite 生成/更新 .stdout/.stderr 快照，解析 [..]/[EXE] 占位符；"少量精养用 assert_cmd、批量放养用 trycmd"。source: https://docs.rs/trycmd/0.15.11 | high | CLI 测试
11. insta（快照）+ proptest（属性）组合；proptest failure_persistence 写回归文件。source: https://tools.corrode.dev/llms.txt | high | 快照/属性测试
12. Clippy 官方 GitHub Actions 门禁：RUSTFLAGS="-Dwarnings" + cargo clippy --all-targets --all-features。source: https://doc.rust-lang.org/nightly/clippy/continuous_integration/github_actions.html | high | CI 门禁
13. 主流项目 CI 普遍采用 lint+test+coverage+supply-chain 四件套、多平台矩阵、cargo-deny 并行。source: https://deepwiki.com/openai/codex/8.2-cicd-pipeline | medium | CI 门禁
14. 供应链：cargo deny check（安全通告+许可证+禁用/重复 crate）与 cargo audit（RustSec）双跑。source: https://blog.csdn.net/gitblog_00690/article/details/166015800 | medium | 供应链
15. 附加工具：cargo-machete/udeps（未用依赖）、cargo-semver-checks、taplo、cargo-mutants（变异测试）。source: https://tools.corrode.dev/llms.txt | high | 工具链

Leads: rust-cli 书 Testing 章节；cargo-mutants 与 llvm-cov 的"行覆盖→行为覆盖"。
未找到: golden-file 正式规范；覆盖率阈值行业共识；assert_cmd/assert_fs changelog。
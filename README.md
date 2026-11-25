# gpui-el

以 Element Plus 为设计基准的 Rust 原生 GPUI 组件库。组件直接构建在 GPUI 上，不需要 Vue、WebView 或本地 Element Plus 源码。长期设计与验收规则见 [CONSTITUTION.md](CONSTITUTION.md)。

## 使用与展厅

如果 `0.1.0` 尚未发布，可先从本仓库使用：将其克隆到宿主项目同级目录，然后在宿主的 `Cargo.toml` 写入：

```toml
[dependencies]
gpui-el = { path = "../gpui-el" }
gpui = { package = "gpui-pre", version = "=0.3.14" }
```

宿主应使用相同来源和版本的 GPUI，避免 `App`、`Window` 等类型不兼容。展厅使用匹配的 `gpui-pre-platform`，在 Linux 构建中启用了 X11 和 Wayland。展厅需要可用的图形桌面会话和 GPUI 原生依赖；Linux 需要对应的图形、字体（如 FreeType）及窗口系统库（X11 路径包括 `libxcb.so.1`）。下面的命令在本仓库根目录执行：

```bash
cargo run --example gallery
```

展厅入口是 [`examples/gallery.rs`](examples/gallery.rs)，组件页面位于 [`examples/gallery/pages/`](examples/gallery/pages/)；库入口是 [`src/lib.rs`](src/lib.rs)。例如 `el_button().label("确定")` 即可构造默认按钮，`.onclick(...)` 注册点击回调。组件事件统一使用 `on` 加事件名（如 `onselect`、`onclose`），旧的下划线方法名不再保留；GPUI 原生方法名不变。共用尺寸、间距和圆角见 `gpui_el::tokens`。循环中从同一调用位置创建多个 Button、Aside、Main 或 Loading 时，应显式提供稳定且在同级唯一的 `.id(...)`。

## 上游参考快照

参考项目：[element-plus/element-plus](https://github.com/element-plus/element-plus)。当前基线 `element-plus-dev` 与公开 commit 的对应关系未确认，也没有可下载的已校验归档；不能用 `2.14.8` 标签或当前开发分支直接替代。同一工作流若已挂载该快照，可直接使用；换机器请向维护者索取原始源码快照。拿不到快照时可继续做不依赖上游的工作，但不要把视觉和行为对照标为已验证。

拿到快照后，核对[宪法第 2 节](CONSTITUTION.md#2-上游基线与取证规则)的主题文件指纹和下列文件，再按快照自带的锁文件搭建对照环境。`.nvmrc` 指定 Node.js `22.13.0`，`package.json` 指定 pnpm `12.18.0`；这是快照声明的版本，不是浏览器验收记录。

| 上游文件 | `git hash-object` |
| --- | --- |
| `pnpm-lock.yaml` | `948aae0c532c0222cafa91381c8e126f325111d7` |
| `pnpm-workspace.yaml` | `f6421d9f659ddc49c1cb9b8fa6a38778e16f511e` |

后续应补记可获取的固定 commit，或维护者提供的完整源码归档位置和 SHA-256。若只能改用另一固定版本，先按宪法对照组件实现、样式及文档差异，再评审切换，不静默更新默认值。

## 开发交接：首提交 `e9cd627`

| 证据级别 | 已知状态 |
| --- | --- |
| 自动化检查 | 统一令牌与事件命名、拆分展厅后，重新执行了全目标测试（含展厅页面与图标等宽网格）和文档测试。这里不把测试视为桌面实测。 |
| 源码对照 | Button 参考 Theme Chalk 的 `button.scss`、`mixins/_button.scss` 与明暗变量；Menu 参考上游 `packages/components/menu/src/`、`menu.scss` 和 `menu.md`；Tag/CheckTag 参考 `tag.scss`、`check-tag.scss`、`common/var.scss` 的 `$tag-*` 令牌与 `tag.md`。图标目录参考上游 `docs/.vitepress/vitepress/components/globals/icons.vue`：等宽 `7 / 6 / 4 / 3 / 2` 列，普通格高 90 px，窄窗口 80 px；这是展厅布局，不是 `Icon` 的默认尺寸。 |
| 已知差异或缺项 | Button 的指针状态颜色即时切换，未复刻上游 100 ms 过渡，禁用／加载的原生无障碍状态待补；Menu 的折叠与浮层显隐目前无 CSS 对应过渡；Loading 是局部遮罩，不是全局模态服务；Tag/CheckTag 挂载与卸载立即完成，不重放上游 `el-zoom-in-center`（故无 `disable-transitions`），CheckTag 额外支持键盘聚焦、可见焦点环与选中语义（GPUI 0.3.14 无 `aria_pressed`／`aria_disabled`，选中态用 selection 属性传达，禁用态仅靠拦截与光标）；完整的多组件组合演示未补齐。 |
| 尚未验收 | 首提交准备环境缺少 `libxcb.so.1`，未完成干净桌面上的人工验收；没有 `gpui-fast` 兼容结论，也没有在干净环境完成发布包验收。 |

建议按以下顺序接手，可分批提交：

1. **先建立回归基线。** 在具备原生依赖的环境运行下列开发检查。对 Button 核对鼠标／键盘状态和无障碍语义；Menu 已有关闭后清理焦点请求及横向溢出变化的回归用例，从现有用例出发补层间 hover、键盘进入与隐藏计时器竞争、浮层遮挡等边界，不重复实现现有行为。
2. **补桌面对照。** 记录操作系统、后端、缩放、字体和语言；检查标题栏、剪贴板、高 DPI、长中文、减少动态效果，以及图标搜索的输入法、键盘、拖动选区和复制。对照上游时保存相同主题、内容和逻辑视口的结果。`gpui-fast` 是独立兼容目标，须在实际 Linux 宿主确认 GPUI 类型兼容与滚动缓存中的 Loading 动效推进，不以默认展厅的结果代替。
3. **发布前补齐发布验收。** 补组合演示并核对双语表格和可复制源码；在干净的支持环境运行 `cargo package --list` 核对展厅源码、双语资源、文档和许可证，再运行 `cargo publish --dry-run`。不要把发布预演当成正式发布。

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo test --doc
cargo check --examples
```

后续更新此节时，分别记录自动化检查、目标桌面实测、源码对照和仍未验证的项目；不要凭已实现或已编译就将组件标记为完全对齐。

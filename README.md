# gpui-el

以 Element Plus 为设计基准的 Rust 原生 GPUI 组件库。组件直接构建在 GPUI 上，不需要 Vue、WebView 或本地 Element Plus 源码。长期设计与验收规则见 [CONSTITUTION.md](CONSTITUTION.md)。

## 使用与展厅

如果 `0.1.0` 尚未发布，可先从本仓库使用：将其克隆到宿主项目同级目录，然后在宿主的 `Cargo.toml` 写入：

```toml
[dependencies]
gpui-el = { path = "../gpui-el" }
gpui = { package = "gpui-fast", version = "=0.1.19" }
```

宿主应使用相同来源和版本的 GPUI，避免 `App`、`Window` 等类型不兼容。展厅使用匹配的 `gpui-fast-platform`，在 Linux 构建中启用了 X11 和 Wayland。展厅需要可用的图形桌面会话和 GPUI 原生依赖；Linux 需要对应的图形、字体（如 FreeType）及窗口系统库（X11 路径包括 `libxcb.so.1`）。下面的命令在本仓库根目录执行：

```bash
cargo run --example gallery
```

展厅按 dev 构建运行，`Cargo.toml` 中 `[profile.dev.package."*"] opt-level = 2` 只优化依赖（GPUI 的布局、排版和绘制），本仓库自身的增量编译仍然很快。未优化依赖时同一页面的每帧耗时约为优化后的 9 倍，hover 高亮会明显落后于指针；调整渲染性能前先用 `cargo test --release --example gallery perf_draw -- --ignored --nocapture` 记录基线，不要凭未经测量的猜测改结构。

展厅入口是 [`examples/gallery.rs`](examples/gallery.rs)，组件页面位于 [`examples/gallery/pages/`](examples/gallery/pages/)；库入口是 [`src/lib.rs`](src/lib.rs)。例如 `el_button().label("确定")` 即可构造默认按钮，`.onclick(...)` 注册点击回调。组件事件统一使用 `on` 加事件名（如 `onselect`、`onclose`），旧的下划线方法名不再保留；GPUI 原生方法名不变。共用尺寸、间距和圆角见 `gpui_el::tokens`。

循环中从同一调用位置创建多个 Button、Aside、Main、Loading 或 Table 时，应显式提供稳定且在同级唯一的 `.id(...)`。

组件代码不会因控件输入触发 panic：未知图标名绘制为空，非法尺寸回落到默认值，内置语言资源解析失败时使用英语词条。需要调用方决策的初始化失败（例如字体信息被文本系统拒绝）返回 `gpui_el::Result`，错误类型为 `gpui_el::Error`，是否视为致命由宿主决定。库源码在测试之外编译时禁止 `unwrap`、`expect`、`panic!`、`todo!`、`unreachable!` 和可能越界的索引切片（见 `src/lib.rs` 的 lint 门禁）。

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
| 自动化检查 | 统一令牌与事件命名、拆分展厅后，重新执行了全目标测试（含展厅页面与图标等宽网格）和文档测试。Table 的文字选择、拖动不触发行点击、滚轮不带动外层容器均有回归用例；Switch 的指针/键盘单次触发、禁用与 `before_change` 拦截、Checkbox/Radio 的单次触发、半选与分组默认传播、Rate 的整星/半星/方向键步进与只读拦截、Slider 的整行命中、手柄外围抓取不跳值、横向/纵向拖动（含拖出滑道仍跟踪）/键盘步进与范围排序，Input 的键入/Ctrl+A 替换/退格/清空/失焦提交、IME preedit 替换标记区与 end→input 单次提交、多行 Enter/视觉行移动也有回归用例；InputNumber 的数值解析、范围/精度/严格步长、清空值、草稿提交、边界步进、只读拦截、渲染控件点击与上下键步进也有回归用例；Select 的过滤、单选事件与显隐、多选切换/上限/移除、清空保留禁用值、键盘导航与浮层指针选择也有回归用例；AutoComplete 的静态过滤、远程过滤、键盘选择与浮层指针选择也有回归用例；ColorPicker 的颜色解析/格式化、HSV 往返、确认/清空与预设浮层交互也有回归用例；TimePicker 的时间解析/格式化、键盘步进、列选择/确认与浮层指针选择也有回归用例；DatePicker 的日期解析/格式化、六周月历、月份切换与日期选择/关闭也有回归用例；Cascader 的逐级列、父标签过滤、叶节点完整路径选择也有回归用例；TreeSelect 的树展开、节点查找与叶节点选择/关闭也有回归用例；Transfer 的目标排序与禁用项过滤后的批量穿梭/事件也有回归用例；Form 的必填/长度/自定义校验、错误报告与初始值重置也有回归用例；这里不把测试视为桌面实测。 |
| 源码对照 | Button 参考 Theme Chalk 的 `button.scss`、`mixins/_button.scss` 与明暗变量；Menu 参考上游 `packages/components/menu/src/`、`menu.scss` 和 `menu.md`；Tag/CheckTag 参考 `tag.scss`、`check-tag.scss`、`common/var.scss` 的 `$tag-*` 令牌与 `tag.md`；Switch 参考 `switch.scss`、`switch.vue`、`switch.md` 与 `common/var.scss` 的 `$switch-*` 令牌（core 宽 50/40/30 px、高 24/20/16 px，滑块 20/16/12 px，外高 40/32/24 px）；Checkbox/Radio 参考 `checkbox.scss`、`radio.scss`、对应 `.vue`/`.md` 与 `$checkbox-*`/`$radio-*` 令牌（外高 40/32/24 px，输入框 14 px，带边框 12 px，标签左间距 8 px，同组间距 30 px）；Rate 参考 `rate.scss`、`rate.vue`、`rate.md` 与 `$rate-*` 令牌（外高 40/32/24 px，星形 18 px、Small 14 px，间距 6 px，填充色 #f7ba2a）；Slider 参考 `slider.scss`、`slider.vue`、`slider.md`、`composables/use-marks.ts`、`use-slide.ts` 与 `$slider-*` 令牌（外高 40/32/24 px，滑轨 6 px、圆角 3 px，手柄 20 px、包裹 36 px、偏移 15 px）；Input 参考 `input.scss`、`input.vue`、`input.md`、`hooks/use-composition/index.ts` 与 `$input-*` 令牌（外高 40/32/24 px，字号 14/14/12 px，水平内边距 16/12/8 px；圆角按需求扩展为原生 0/4/8/12 px 与胶囊，默认 8 px），并核对 Linux X11/Wayland 的 preedit→commit→unmark 调用顺序。InputNumber 参考 `input-number.scss`、`input-number.vue`、`input-number.ts` 与 `input-number.md`（宽 180/150/120 px，外高 40/32/24 px，默认左右控件、右侧堆叠控件、15 px 无控件内边距）。CapsuleSegments 参考 ChatSpeed GPUI 客户端 `src-tauri/crates/gpui/src/ui/tabs.rs` 的执行频率胶囊（30 px 默认高度、2 px 间隔、换行、边缘相连选项），颜色改用本库 Element Plus 主题。Select 参考 `select.scss`、`option.scss`、`select.vue`、`option.vue`、`useSelect.ts` 与 `select.md`（100% 宽、32/40/24 px 外高、34 px 选项、274 px 下拉列表、6 px 内边距）。AutoComplete 参考 `autocomplete.scss`、`autocomplete.vue`、`autocomplete.ts` 与 `autocomplete.md`（34 px 建议行、280 px 最大高度、10 px 列表内边距、100 px 加载行）。ColorPicker 参考 `color-picker.scss`、`color-picker-panel.scss`、`color-picker.vue`、`color-picker.ts`（32/40/24 px 色块、280×180 SV 面板、280×12 色相/Alpha 条、20 px 预设、清空/确认）。TimePicker 参考 `time-picker.md`、`time-picker.tsx`、`date-picker/time-picker.scss` 与 `time-spinner.scss`（180 px 面板、192 px 滚动列、32 px 时间项、确定/取消）。DatePicker 参考 `date-picker.md`、`date-picker.tsx`、`date-picker/date-picker.scss` 与 `picker-panel.scss`（280 px 月历、七列日期格、月份导航、键盘移动）。Cascader 参考 `cascader.md`、`cascader.vue`、`cascader.ts` 与 `cascader.scss`（逐级列、叶节点路径、多选、过滤、清空）。TreeSelect 参考 `tree-select.md`、`tree-select.vue`、`tree-select/src` 与 `tree-select.scss`（树展开、节点键、多选、严格父节点、过滤、清空）。Transfer 参考 `transfer.md`、`transfer.vue`、`transfer.ts` 与 `transfer.scss`（双栏勾选、批量穿梭、过滤、禁用、计数、目标排序）。Form/FormItem 参考 `form.md`、`form.vue`、`form.ts`、`form-item.vue` 与 `form.scss`/`form-item.scss`（共享模型、规则、标签布局、必填标记、校验、重置、提交）。图标目录参考上游 `docs/.vitepress/vitepress/components/globals/icons.vue`：等宽 `7 / 6 / 4 / 3 / 2` 列，普通格高 90 px，窄窗口 80 px；这是展厅布局，不是 `Icon` 的默认尺寸。 |
| 已知差异或缺项 | Button 的指针状态颜色即时切换，未复刻上游 100 ms 过渡，禁用／加载的原生无障碍状态待补；Menu 的展开收起走 `Transition::Collapse`，默认桌面快速度（0.6 倍 ⇒ 120 ms），浮层淡入淡出与箭头旋转同步缩放，`normal()` 可恢复 Element Plus 的 1 倍时长；Table 多出原生文字选择与滚轮拦截，但选择仅限同一个表格内，行列选中、排序、固定列、树形与汇总行仍未实现；Loading 是局部遮罩，不是全局模态服务；Tag/CheckTag 挂载与卸载立即完成，不重放上游 `el-zoom-in-center`（故无 `disable-transitions`），CheckTag 额外支持键盘聚焦、可见焦点环与选中语义（GPUI 0.3.14 无 `aria_pressed`／`aria_disabled`，选中态用 selection 属性传达，禁用态仅靠拦截与光标）；Splitter 以逻辑像素保存面板尺寸，不支持上流 px/百分比混写，lazy 只移动分隔线，折叠箭头常显（低透明度），分隔条可键盘聚焦；Scrollbar 自绘 6 px 滑块，高度/最大高度直接约束滚动盒，宽内容需显式宽度（GPUI 没有 inline-block 内容包裹层）并以 DOM 式正值报告滚动位置，mask 渐变与 wrap/view 样式钩子未移植，setScrollTop/setScrollLeft/scrollTo 改为传入 ScrollHandle；Text 的 tag 属性无 GPUI 对应物，isTruncated 是 truncated 在定宽容器下的排版结果而非可监听状态，修饰由原生下划线与删除线实现；Slider 已支持 `show-input`（内嵌 InputNumber）与 `step_at_marks`（对应上游 `step: 'mark'`），手柄 1.2× 悬停/拖动放大改为 20→24 px 尺寸变化，拖动中提示气泡保留生成时的文本；整个滑块行/竖直条均可按下，手柄抓取保留指针偏移；Input 已提供单行与 `textarea(true).rows(n)` 多行文本，但未移植 `autosize`/`resize`、`formatter`/`parser` 与原生撤销栈，`change` 仅在失焦与清空时触发（与上游一致，Enter 不提交），插入符号不闪烁，拖动选择仅在字段内跟踪（无越界自动滚动），清空按钮在聚焦或悬停按钮自身时显示；InputNumber 已支持范围、精度、严格步长、清空值、侧边/右侧控件、对齐、格式化与解析，但 `name`/`inputmode`/`tabindex`、命令式 focus/blur 与 Form 的 `validate-event` 尚未映射，反向范围收敛到 min，精度上限为 15 位；CapsuleSegments 作为独立单选组件提供，未并入 RadioGroup 的圆形单选样式，保留原生 Tab 顺序与 Enter/Space 激活，不提供方向键组导航；Select 已支持单选/多选、选项分组、创建项、过滤、清空、禁用选项、加载、键盘与浮层，但多选以逗号摘要显示而非可删除 Tag，Backspace 移除最后一个可移除值；远程搜索由 filter_method 驱动，debounce、自定义选项渲染、折叠标签提示与命令式 focus/blur 尚未映射；AutoComplete 已支持本地/调用方建议、加载、清空、键盘高亮、未匹配选择与浮层，但建议使用 AutoCompleteOption 类型，未移植自定义建议渲染、debounce、placement/popper 配置与命令式 exposes；ColorPicker 已支持 SV、连续色相渐变、Alpha、预设、文本颜色输入、Hex/RGB/RGBA、清空/确认与键盘触发，但未移植自定义 popper 配置与命令式 exposes，拖动仅在指针停留于控件上时跟踪；TimePicker 已支持单选时间、HH:mm/HH:mm:ss、滚动列、禁用时/分/秒、键盘步进、清空与确定/取消，但未移植范围模式、自定义 popper 配置与命令式 exposes；DatePicker 已支持日期/范围/月/年/季度/周与多选、年月下拉、年份切换、联动双面板、禁用日期、键盘移动、清空与日期校验，但未移植 datetime 时间面板、shortcuts、DOM slots、自定义 popper 配置与命令式 exposes；Cascader 已支持静态树、逐级展开、叶节点或严格父节点路径、多选、自定义过滤、完整路径标签、清空与键盘导航，但未移植懒加载、折叠标签提示、自定义渲染、虚拟滚动与命令式 exposes；TreeSelect 已支持静态树、展开/收起、节点键、多选、严格父节点、过滤、清空与键盘导航，但未移植懒加载、cache-data、自定义节点渲染、虚拟滚动与命令式 tree/select exposes；Transfer 已支持静态数据、受控目标键、单项/批量穿梭、过滤、禁用、计数、目标排序、自定义页脚/空态与变更事件，但未移植属性别名、自定义项目渲染、虚拟滚动与命令式 panel/query exposes；Form/FormItem 已支持共享字符串模型、必填/长度/自定义同步规则、change/blur 触发校验、行内错误消息、横向/纵向/行内布局、重置与提交，但未移植嵌套属性路径、异步校验、自动字段注册、滚动定位与命令式 focus/blur；完整的多组件组合演示未补齐。 |
| 尚未验收 | 首提交准备环境缺少 `libxcb.so.1`，未完成干净桌面上的人工验收：自绘标题栏的双击缩放、八向拖拽改尺寸、右键窗口菜单、表格内滚轮归属与文字选择手感，都需在真实 X11/Wayland 会话复核；已切换到 `gpui-fast =0.1.19`（编译与测试通过），但尚未在真实图形会话验收动效推进、滚动缓存与窗口装饰手感，也没有在干净环境完成发布包验收。 |

建议按以下顺序接手，可分批提交：

1. **先建立回归基线。** 在具备原生依赖的环境运行下列开发检查。对 Button 核对鼠标／键盘状态和无障碍语义；Menu 已有关闭后清理焦点请求及横向溢出变化的回归用例，从现有用例出发补层间 hover、键盘进入与隐藏计时器竞争、浮层遮挡等边界，不重复实现现有行为。
2. **补桌面对照。** 记录操作系统、后端、缩放、字体和语言；检查标题栏、剪贴板、高 DPI、长中文、减少动态效果，以及图标搜索的输入法、键盘、拖动选区和复制。对照上游时保存相同主题、内容和逻辑视口的结果。本库现在默认使用 `gpui-fast =0.1.19`，仍须在实际 Linux 宿主确认滚动缓存中的 Loading 动效推进、离场动画和减少动态效果的停止续帧，不以无界面测试的结果代替。
3. **发布前补齐发布验收。** 补组合演示并核对双语表格和可复制源码；在干净的支持环境运行 `cargo package --list` 核对展厅源码、双语资源、文档和许可证，再运行 `cargo publish --dry-run`。不要把发布预演当成正式发布。

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo test --doc
cargo check --examples
```

后续更新此节时，分别记录自动化检查、目标桌面实测、源码对照和仍未验证的项目；不要凭已实现或已编译就将组件标记为完全对齐。

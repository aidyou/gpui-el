# Element Plus 组件排除清单

**上游导航有页面，不等于 `gpui-el` 要实现一个组件。** 仅用于展示设计规范、罗列组件或讲解 Web 接入的页面，不新增独立 `el_*` API，不占展厅组件页，也不计入组件覆盖率。页面里的设计规范仍是已纳入组件的对照依据。

## 不作为组件移植

| Element Plus 条目 | 上游路径（相对于源码根目录） | 在 `gpui-el` 中如何处理 |
| --- | --- | --- |
| Border 边框 | `docs/en-US/component/border.md` | 不做 `el_border`；边框、圆角、阴影作为设计令牌或 GPUI 样式，由实际组件使用。 |
| Color 色彩 | `docs/en-US/component/color.md` | 不做 `el_color`；品牌色、语义色和中性色由主题令牌提供。色板仅是设计参考。 |
| Typography 排版 | `docs/en-US/component/typography.md` | 不做 `el_typography`；字体、字号、字重和行高由主题与组件样式提供。 |
| Overview 组件总览 | `docs/en-US/component/overview.md` | 只是文档索引，不是组件；本库用自己的展厅导航组织组件。 |
| Built-in Transition 内置过渡 | `docs/en-US/guide/transitions.md` | 不照搬 Vue/CSS 的通用过渡包装器；淡入淡出、缩放和折叠等仍按具体组件的行为要求原生实现。 |

主题定制、暗色模式、国际化等上游指南也不按页面立组件；相应能力由本库的主题和应用配置承担。Vue 的 CSS 命名空间、SSR 和 DOM Teleport 不直接移植；这不排除 GPUI 浮层、焦点管理等桌面能力。

## 不要误排

- `ColorPicker`／`ColorPickerPanel` 是颜色输入控件，不是 `Color` 色板；`Divider` 是独立的内容分隔组件，不是 `Border` 样式页；`Text` 有省略等组件行为，不是 `Typography` 规范页。
- `Layout`（Row／Col）、`Container`、`Space`、`Icon` 等已有独立用途；即使无需交互，也不属于这份排除清单。`Config Provider` 是实际的配置入口，不是指南页；原生等价配置是否需要新增 API，应单独评估。

筛选新条目时，先看上游是否定义了独立的布局、内容、交互或配置契约。不要只因外观简单、没有事件，或 GPUI 已有底层原语，就把真实组件列为排除项。

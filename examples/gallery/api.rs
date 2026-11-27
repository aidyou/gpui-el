use super::{Locale, Page};
use gpui::{Div, div, prelude::*, px};
use gpui_el::{Column, el_table};

// Each row has a Rust name, type/default, and matching localized descriptions.
type Row = (&'static str, &'static str, &'static str, &'static str);

const INIT: &[Row] = &[
    (
        "el_init!, init",
        "App, Locale, IconPack, &[&[u8]] → Result",
        "Configure one GPUI application before opening windows.",
        "在创建窗口前配置当前 GPUI 应用。",
    ),
    (
        "locale",
        "Locale · English / SimplifiedChinese",
        "Choose built-in UI language; English if not configured.",
        "内置文案的语言；未设置时为英语。",
    ),
    (
        "icons",
        "IconPack · ElementPlus / Tabler",
        "Choose the default icon pack; Element Plus if not configured.",
        "设置默认图标库；未设置时为 Element Plus。",
    ),
    (
        "fonts",
        "[&'static [u8]] · []",
        "Optional embedded font files. Uses the family name stored in the font.",
        "可选的嵌入字体文件；使用字体文件内部的字族名。",
    ),
];
const BUTTON: &[Row] = &[
    (
        "el_button_group(), direction, size, kind",
        "Direction / ButtonSize / ButtonType · Horizontal / inherited",
        "Create a connected group with optional size and type defaults for its buttons.",
        "创建相连的按钮组，可为组内按钮设置默认尺寸和类型。",
    ),
    (
        "ButtonGroup::button, buttons",
        "Button / IntoIterator<Item = Button> · []",
        "Add individual buttons. Each button retains its own events.",
        "添加按钮；事件仍由各按钮提供，按钮组无独立事件。",
    ),
    (
        "el_button(), Button::new(id), id",
        "ElementId · call site",
        "Create a button. Supply a stable sibling-unique ID when creating repeated buttons at the same call site.",
        "创建按钮。同一调用位置生成多个按钮时，提供稳定且同级唯一的 ID。",
    ),
    (
        "label, content",
        "SharedString / IntoElement · —",
        "Label or custom contents; custom contents take precedence.",
        "文字或自定义内容；自定义内容优先。",
    ),
    (
        "icon, loading_icon",
        "IntoElement · — / Loading",
        "Leading icon, and an optional replacement for the loading spinner.",
        "前置图标；可替换加载时的默认指示器。",
    ),
    (
        "kind",
        "ButtonType · Default",
        "Primary, Success, Warning, Danger and Info also have shortcuts.",
        "Primary、Success、Warning、Danger、Info 均有同名便捷方法。",
    ),
    (
        "variant",
        "ButtonVariant · Solid",
        "plain, dashed, text, text_with_background and link select variants.",
        "用 plain、dashed、text、text_with_background、link 选择变体。",
    ),
    (
        "size, small, large",
        "ButtonSize · Default (32 px)",
        "Small is 24 px; Large is 40 px.",
        "Small 为 24 px，Large 为 40 px。",
    ),
    (
        "shape, round, circle",
        "ButtonShape · Normal",
        "Set rectangular, capsule or circular shape.",
        "设置普通、胶囊或圆形外观。",
    ),
    (
        "font_size",
        "f32 · by size",
        "Set the label size in logical pixels.",
        "设置文字逻辑像素大小。",
    ),
    (
        "color, auto_insert_space",
        "Rgba / bool · theme / false",
        "Derive button states from a custom accent; optionally space labels containing exactly two Han characters.",
        "从自定义色派生按钮状态色；可为恰好两个汉字的标签添加间隔。",
    ),
    (
        "disabled, loading",
        "bool · false",
        "Block activation; loading also shows its indicator.",
        "禁止触发；加载态另外显示指示器。",
    ),
    (
        "tooltip, accessibility_label",
        "SharedString · — / label",
        "Set a hover hint and the accessible name. Label icon-only buttons.",
        "设置悬停提示和无障碍名称；纯图标按钮应提供名称。",
    ),
];
const ICON: &[Row] = &[
    (
        "el_icon(name), Icon::named",
        "SharedString",
        "Resolve a name in the application pack, falling back to Element Plus.",
        "按应用默认图库解析名称，找不到时回退到 Element Plus。",
    ),
    (
        "Icon::from_pack",
        "IconPack, SharedString",
        "Resolve in the supplied pack without fallback.",
        "仅使用指定的图库，不回退。",
    ),
    (
        "Icon::new, from_font, from_svg",
        "char / (family, char) / &'static [u8]",
        "Render an inherited glyph, a font glyph, or embedded SVG bytes.",
        "使用继承字体字形、指定字体字形或嵌入 SVG。",
    ),
    (
        "size, color",
        "f32 / Hsla · inherited",
        "Override the logical size and foreground color.",
        "覆盖继承的图标大小和前景色。",
    ),
    (
        "spin",
        "LoadingMotion · —",
        "Rotate an SVG or font glyph around its center. Respects reduced motion.",
        "围绕中心旋转 SVG 或字体字形。遵从减少动效设置。",
    ),
    (
        "IconPack::set, current, names",
        "Result / IconPack / Iterator",
        "Set, read or enumerate a built-in pack.",
        "设置、读取或枚举内置图库中的名称。",
    ),
    (
        "register_icon_font",
        "&App, &'static [u8] → Result",
        "Register custom font bytes at startup.",
        "启动时注册自定义字体文件。",
    ),
];
const CONTAINER: &[Row] = &[
    (
        "el_container(), Container::new",
        "Container",
        "Create a flex container; nest containers or regions using child/children.",
        "创建弹性容器，用 child/children 嵌套容器或区域。",
    ),
    (
        "Container::direction, vertical",
        "Direction · automatic",
        "Vertical with a direct Header or Footer child; horizontal otherwise.",
        "有直接 Header 或 Footer 子元素时纵向，否则横向。",
    ),
    (
        "el_header(), el_footer(), new, height",
        "f32 · 60 px",
        "Header and Footer use 20 px horizontal padding.",
        "顶栏和底栏默认左右内边距为 20 px。",
    ),
    (
        "el_aside(), Aside::new, id, width",
        "ElementId / f32 · 300 px",
        "Fixed-width scrollable side region.",
        "固定宽度的可滚动侧栏。",
    ),
    (
        "el_main(), Main::new, id, padding",
        "ElementId / f32 · 20 px",
        "Flexible scrollable content region.",
        "分配剩余空间的可滚动主区域。",
    ),
    (
        "background, Aside::end_border",
        "Hsla · —",
        "Paint the region itself or draw a right divider over the whole side region.",
        "为区域本身填色，或为侧栏绘制通高的右分隔线。",
    ),
    (
        "child, children",
        "IntoElement · []",
        "Content for each region.",
        "各布局区域的内容。",
    ),
];
const LAYOUT: &[Row] = &[
    (
        "el_row(), el_col()",
        "Row / Col",
        "Create a row or a column in the 24-column grid.",
        "创建 24 列栅格中的行或列。",
    ),
    (
        "Row::gutter",
        "f32 · 0 px",
        "Horizontal gutter; half is applied at each column edge.",
        "水平间距；每列两边各分担一半。",
    ),
    (
        "Row::justify",
        "RowJustify · Start",
        "Start, End, Center, SpaceBetween, SpaceAround or SpaceEvenly.",
        "支持起点、终点、居中、两端、环绕和均分。",
    ),
    (
        "Row::align",
        "RowAlign · Stretch",
        "Stretch, Top, Middle or Bottom.",
        "支持拉伸、顶部、居中和底部。",
    ),
    (
        "Row::col, cols; Col::child, children",
        "Col / IntoElement · []",
        "Attach typed columns, then place contents in each column.",
        "为行添加类型化的列，再为列添加内容。",
    ),
    (
        "Col::span, offset, push, pull",
        "u8 · 24 / 0 / 0 / 0",
        "Width, blank space before the column, and visual shifts in grid units.",
        "以列为单位设置宽度、前置留白、右移和左移。",
    ),
    (
        "Col::xs, sm, md, lg, xl",
        "u8 / ColSpec · —",
        "Viewport breakpoints: <768, ≥768, ≥992, ≥1200, ≥1920 px. ColSpec can override span, offset, push and pull.",
        "窗口断点：<768、≥768、≥992、≥1200、≥1920 px；ColSpec 可覆盖宽度、偏移、右移和左移。",
    ),
];
const LINK: &[Row] = &[
    (
        "el_link(), Link::new, id",
        "ElementId · call site",
        "Create a link. Supply a stable sibling-unique ID when creating repeated links at the same call site.",
        "创建文字链接。同一调用位置生成多个链接时，提供稳定且同级唯一的 ID。",
    ),
    (
        "label, content, accessibility_label",
        "SharedString / IntoElement / SharedString · —",
        "Label, custom contents, or the accessible name.",
        "文字、自定义内容或无障碍名称。",
    ),
    (
        "kind, primary, success, info, warning, danger",
        "LinkType · Default",
        "Semantic color; default uses the regular text color and hovers to primary.",
        "语义颜色；default 使用常规文字色，悬停为 primary。",
    ),
    (
        "underline",
        "LinkUnderline · Hover",
        "Show the underline always, on hover, or never.",
        "下划线显示时机：总是、悬停或从不。",
    ),
    (
        "href",
        "SharedString · —",
        "Open the URL through the platform handler on activation; ignored while disabled.",
        "激活时由平台处理器打开 URL；禁用时忽略。",
    ),
    (
        "icon, trailing_icon",
        "IntoElement · —",
        "Leading or trailing icon, five pixels from the content.",
        "内容前后的图标，与内容相距 5 px。",
    ),
    (
        "disabled",
        "bool · false",
        "Block activation, the URL, and the hover color.",
        "禁止触发、打开 URL 和悬停变色。",
    ),
];
const SPLITTER: &[Row] = &[
    (
        "el_splitter(), Splitter::new, id",
        "ElementId · call site",
        "Create a splitter. Give each splitter a stable sibling-unique ID when several are made at one call site.",
        "创建分隔面板。同一调用位置创建多个时，提供稳定且同级唯一的 ID。",
    ),
    (
        "panel, panels",
        "SplitterPanel · []",
        "Add panels in order; at least two are needed for a divider.",
        "按顺序添加面板；至少需要两个才会出现分隔条。",
    ),
    (
        "layout, vertical, horizontal",
        "SplitterLayout · Horizontal",
        "Side by side or top to bottom.",
        "横向并排或纵向上下排列。",
    ),
    (
        "lazy",
        "bool · false",
        "Move only the divider line while dragging and apply the sizes at release.",
        "拖动时只移动分隔线，松开后才应用尺寸。",
    ),
    (
        "onresize_start, onresize, onresize_end",
        "usize, SplitterSizes · —",
        "Divider drag lifecycle with the divider index and the sizes in logical pixels.",
        "分隔条拖动的开始、过程与结束事件，带分隔条索引与逻辑像素尺寸。",
    ),
    (
        "oncollapse",
        "usize, SplitterSide, SplitterSizes · —",
        "A panel collapsed or expanded through a divider arrow.",
        "通过分隔条箭头折叠或展开面板时触发。",
    ),
];
const SPLITTER_PANEL: &[Row] = &[
    (
        "el_splitter_panel(), SplitterPanel::new",
        "SplitterPanel",
        "Create a panel that shares the leftover space and can be resized.",
        "创建均分剩余空间且可调整大小的面板。",
    ),
    (
        "size",
        "f32 · shared",
        "Initial size in logical pixels; panels without one share the leftover space.",
        "初始尺寸（逻辑像素）；未设置的面板均分剩余空间。",
    ),
    (
        "min, max",
        "f32 · 0 / unlimited",
        "Drag limits in logical pixels.",
        "拖动的最小与最大尺寸限制（逻辑像素）。",
    ),
    (
        "resizable",
        "bool · true",
        "A divider is draggable when both panels it separates are resizable.",
        "相邻两个面板都可调整时，中间分隔条才能拖动。",
    ),
    (
        "collapsible",
        "bool · false",
        "Show collapse arrows on the divider and let them shrink the panel to zero.",
        "在分隔条上显示折叠箭头，并允许把面板折叠为零。",
    ),
    (
        "child, children",
        "IntoElement · []",
        "Panel contents.",
        "面板内容。",
    ),
];
const SPLITTER_EVENTS: &[Row] = &[
    (
        "onresize_start",
        "usize, SplitterSizes",
        "A divider drag started.",
        "分隔条拖动开始时触发。",
    ),
    (
        "onresize",
        "usize, SplitterSizes",
        "While a divider moves, including keyboard steps.",
        "分隔条移动时触发，键盘步进同样触发。",
    ),
    (
        "onresize_end",
        "usize, SplitterSizes",
        "A divider drag ended; lazy drags apply their sizes here.",
        "分隔条拖动结束时触发；懒更新在此时应用尺寸。",
    ),
    (
        "oncollapse",
        "usize, SplitterSide, SplitterSizes",
        "A divider arrow collapsed a panel or expanded it back.",
        "分隔条箭头折叠或恢复面板时触发。",
    ),
];
const SCROLLBAR: &[Row] = &[
    (
        "el_scrollbar(), Scrollbar::new, id",
        "ElementId · call site",
        "Create a scrollbar. Give each scrollbar a stable sibling-unique ID when several are made at one call site.",
        "创建滚动条。同一调用位置创建多个滚动条时，提供稳定且同级唯一的 ID。",
    ),
    (
        "child, children",
        "IntoElement · []",
        "The scrolled content.",
        "滚动的正文内容。",
    ),
    (
        "height, max_height",
        "f32 · 100% of parent / —",
        "Constrain the scroll box itself. Content wider than the viewport must state a width; GPUI has no inline-block wrapper.",
        "直接约束滚动盒。宽于视口的内容必须声明宽度；GPUI 没有 inline-block 包裹层。",
    ),
    (
        "always, native, min_size",
        "bool / bool / f32 · false / false / 20",
        "Keep the thumbs visible, drop them for a plain overflow container, and set the minimum thumb length.",
        "常显滑块、退化为无滑块容器，以及滑块的最小长度。",
    ),
    (
        "handle",
        "ScrollHandle · internal",
        "Track this viewport: read offset and maximum, or set the offset for manual scrolling.",
        "跟踪该视口：读取偏移与最大值，或设置偏移实现手动滚动。",
    ),
    (
        "onscroll",
        "ScrollbarEvent · —",
        "Reports scroll_left and scroll_top in logical pixels on every scroll.",
        "每次滚动时报告逻辑像素为单位的 scroll_left 与 scroll_top。",
    ),
    (
        "onreached",
        "ScrollbarDirection, f32 · — / 0",
        "Reports a newly reached edge; the distance argument widens it like the upstream attribute.",
        "报告新触到的视口边缘；distance 参数与上游属性一样放宽触边判定。",
    ),
];
const SCROLLBAR_EVENTS: &[Row] = &[
    (
        "onscroll",
        "ScrollbarEvent { scroll_left, scroll_top }",
        "The wheel, a drag, or a manual set_offset moved the viewport.",
        "滚轮、拖拽滑块或手动 set_offset 改变视口位置时触发。",
    ),
    (
        "onreached",
        "ScrollbarDirection",
        "The viewport newly reached the top, bottom, left or right edge.",
        "视口新触到上、下、左或右边缘时触发。",
    ),
];
const TEXT: &[Row] = &[
    (
        "el_text(), Text::new",
        "Text",
        "Create a text in the default type and size.",
        "创建默认类型与尺寸的文本。",
    ),
    (
        "label, content",
        "SharedString / IntoElement · —",
        "Label or custom contents; custom contents take precedence and allow a StyledText with mixed runs.",
        "文字或自定义内容；自定义内容优先，可传入分段的 StyledText。",
    ),
    (
        "kind, primary, success, info, warning, danger",
        "TextType · Default",
        "Semantic color; default uses the regular text color.",
        "语义颜色；default 使用常规文字色。",
    ),
    (
        "size, large, small",
        "TextSize · Default",
        "Large, default and small map to the 15 / 13 / 12 px token sizes.",
        "large、default、small 对应 15 / 13 / 12 px 的字号令牌。",
    ),
    (
        "truncated",
        "bool · false",
        "Render an ellipsis past the container width; needs a definite width to clip against.",
        "超出容器宽度时显示省略号；需要容器提供确定宽度。",
    ),
    (
        "line_clamp",
        "usize · —",
        "Render an ellipsis after the given number of lines; zero is unset.",
        "超过指定行数后显示省略号；0 视为未设置。",
    ),
    (
        "bold, italic, underline, strikethrough",
        "bool · false",
        "Native font weight and text decorations, applied independently.",
        "原生字重与文本修饰，可独立组合。",
    ),
];
const TABLE: &[Row] = &[
    (
        "Table::new(rows)",
        "IntoIterator<Item = T>",
        "Creates a table over typed rows; columns describe how each row renders.",
        "用类型化行数据创建表格；列决定每一行的渲染方式。",
    ),
    (
        "Table::column, Column::new",
        "Column<T> · []",
        "Adds a column. Columns share leftover space equally unless given a width or min_width.",
        "添加列。未设置宽度或最小宽度的列均分剩余空间。",
    ),
    (
        "Column::width, min_width",
        "f32 · —",
        "Fixes a column width, or sets a minimum so leftover space is distributed proportionally.",
        "固定列宽，或设置最小宽度并按其比例分配剩余空间。",
    ),
    (
        "Column::text, render, header",
        "Fn(&T, usize) / Fn(&T, usize) → Element / Fn() → Element",
        "Cell text, custom cell content, and custom header content. Text wraps with the column alignment.",
        "单元格文字、自定义单元格内容和自定义表头内容；文字按列对齐方式换行。",
    ),
    (
        "Column::align, header_align",
        "TableAlign · Left",
        "Aligns cells, and the header separately when given.",
        "设置单元格对齐；表头可单独设置。",
    ),
    (
        "Table::stripe, border, size, show_header",
        "bool / bool / TableSize / bool · false / false / Default / true",
        "Row stripes, vertical borders, cell density, and header visibility.",
        "行条纹、竖向边框、密度与表头显隐。",
    ),
    (
        "Table::height, max_height",
        "f32 · —",
        "Fixes or caps the table height; the header stays while the body scrolls. The body consumes wheel deltas, so the page or window never scrolls with it.",
        "固定或限制表格高度；表头保持可见，主体滚动。主体吞掉滚轮事件，页面与窗口不会随之滚动。",
    ),
    (
        "Table::id, selectable",
        "ElementId / bool · call site / true",
        "Separates per-table hover and selection state; give one when several tables share a call site. selectable allows dragging text and Ctrl+C/Cmd+C to copy it.",
        "隔离各表格的悬停与选中状态；同一调用点创建多个表格时需显式指定。selectable 允许拖动选择文字并用 Ctrl+C／Cmd+C 复制。",
    ),
    (
        "Table::empty_text, empty",
        "SharedString / IntoElement · No Data",
        "Sets the empty state shown when there are no rows.",
        "无数据时显示的空状态。",
    ),
    (
        "Table::row_background",
        "Fn(&T, usize) → Option<Hsla> · —",
        "Paints a status background for chosen rows, replacing upstream row-class-name.",
        "为指定行绘制状态底色，替代上游的 row-class-name。",
    ),
    (
        "Table::copyable",
        "bool · false",
        "A native addition: shows a control in the table's top-right corner while hovered. Its tooltip reads Copy as markdown, and the click puts the whole table on the clipboard as Markdown, confirming next to the icon for three seconds.",
        "原生新增：指针经过表格时在右上角显示复制控件。悬停提示「复制为 Markdown」，点击把整张表格以 Markdown 写入剪贴板，并在图标旁显示三秒的复制成功提示。",
    ),
];
const TABLE_EVENTS: &[Row] = &[(
    "Table::onrowclick",
    "&T, usize, &ClickEvent, &mut Window, &mut App",
    "Activates a row; reports the row data and its index.",
    "点击某行时触发；上报行数据与索引。",
)];
const SPACE: &[Row] = &[
    (
        "el_space(), Space::new, child, children",
        "Space / IntoElement · []",
        "Create a spacing container and add contents.",
        "创建间距容器并添加内容。",
    ),
    (
        "size, small, large",
        "SpaceSize · Small",
        "A new Space uses 8 px; small and large select 8 and 16 px. Use size(SpaceSize::Default) for 12 px.",
        "未指定时为 8 px；small 和 large 分别为 8 和 16 px。12 px 使用 size(SpaceSize::Default)。",
    ),
    (
        "gap",
        "(f32, f32) · by preset",
        "Override the horizontal and vertical gaps in logical pixels.",
        "以逻辑像素覆盖横纵间距。",
    ),
    (
        "direction, vertical",
        "Direction · Horizontal",
        "Arrange items along a row or a column.",
        "横向或纵向排列子元素。",
    ),
    (
        "alignment",
        "SpaceAlignment · Center",
        "Cross-axis alignment: Start, Center, End, Stretch or Baseline.",
        "交叉轴对齐：起点、居中、终点、拉伸或基线。",
    ),
    (
        "wrap, fill, fill_ratio",
        "bool / bool / f32 · false / false / 100",
        "Wrap contents, or expand items using a minimum width percentage.",
        "允许换行，或按最小宽度百分比填充。",
    ),
    (
        "spacer",
        "Fn() → IntoElement · —",
        "Render a separator between neighbors.",
        "在相邻元素之间渲染分隔内容。",
    ),
];
const MENU: &[Row] = &[
    (
        "el_menu(), Menu::new, id, state",
        "ElementId / &Entity<MenuState>",
        "Create a menu; optional external state for programmatic control.",
        "创建菜单；可传入外部状态作程序控制。",
    ),
    (
        "mode, collapse, ellipsis, ellipsis_icon",
        "MenuMode / bool / bool / Icon · Vertical / false / true / More",
        "Control orientation, collapsed sidebar and horizontal overflow.",
        "控制方向、侧栏折叠和横向菜单溢出。",
    ),
    (
        "unique_opened, menu_trigger, close_on_click_outside",
        "bool / MenuTrigger / bool · false / Hover / false",
        "Control submenu exclusivity, horizontal trigger, and outside-click dismissal.",
        "设置子菜单互斥、横向菜单触发方式和外点关闭。",
    ),
    (
        "border, background_color, text_color, active_text_color",
        "bool / Hsla · true / theme",
        "Override the menu divider and theme colors.",
        "覆盖菜单分隔线和主题颜色。",
    ),
    (
        "default_active, active_index, default_openeds",
        "SharedString / iterable · — / — / []",
        "Initial or controlled selected index, and initially open branches.",
        "设置初始或受控选中索引，以及初次展开的子菜单。",
    ),
    (
        "popper_offset, show_timeout, hide_timeout",
        "f32 / Duration · 6 px / 300 ms / 300 ms",
        "Popup gap and hover delays. SubMenu can override each.",
        "浮层间距和悬停延时；SubMenu 可局部覆盖。",
    ),
    (
        "popper_effect, popup_style",
        "MenuPopupTheme / Fn(Div) → Div · Dark / —",
        "Set popup theme or style; popup_style also exists on SubMenu.",
        "设置浮层主题或样式；SubMenu 也可设置 popup_style。",
    ),
    (
        "entry, entries",
        "MenuEntry · []",
        "Add typed items, submenus or groups to Menu/SubMenu.",
        "向 Menu/SubMenu 添加类型化菜单项、子菜单和分组。",
    ),
    (
        "el_menu_item, MenuItem::new, index",
        "index / label · required",
        "Create a leaf with a stable index; new accepts an explicit GPUI ID.",
        "创建带稳定索引的叶子节点；new 接收显式 GPUI ID。",
    ),
    (
        "el_sub_menu, title_slot, disabled, icon",
        "index / title, Fn, bool, Icon · false / —",
        "Create a branch; customize item or branch content, availability or leading icon.",
        "创建子菜单；自定义菜单项或子菜单内容、禁用状态或前置图标。",
    ),
    (
        "expand_icons, collapse_icons",
        "(Icon, Icon) · built-in",
        "SubMenu icons for closed/open states in expanded/collapsed mode.",
        "分别设置展开模式和折叠模式下子菜单的关闭/打开图标。",
    ),
    (
        "el_menu_item_group, item, items, title_slot",
        "title, MenuItem, Fn · []",
        "Group leaves under a title or custom title content.",
        "使用文字或自定义标题对菜单项分组。",
    ),
    (
        "style, item_height, item_radius, item_inset",
        "MenuStyle / f32 · platform preset",
        "Desktop density presets: Element keeps the web 56 px rows; Mac and Kde use compact native rows. The knobs override any preset: row height, hover-fill radius, and the margin between the fill and the menu edge.",
        "桌面密度预设：Element 保持网页的 56 px 行高，Mac 与 Kde 使用原生紧凑行高。这些参数可覆盖任意预设：行高、悬停底色圆角，以及底色与菜单边缘的间距。",
    ),
    (
        "hover_color, hover_text_color, popup_padding, font_size",
        "Hsla / Hsla / f32 / f32 · by preset",
        "Hover fill and text colors, the vertical padding inside popups, and the menu font size.",
        "悬停底色与文字颜色、浮层纵向内边距与菜单字号。",
    ),
    (
        "speed",
        "MotionSpeed · Fast",
        "Scales the collapse, popup fade, and chevron rotation. Menus default to desktop-fast 0.6×; `normal()` restores the Element Plus 1× timings, `slow()` is 1.6×, and `speed(MotionSpeed::Custom(x))` multiplies the normal duration.",
        "缩放展开、浮层淡入淡出与箭头旋转。菜单默认使用桌面快速度 0.6 倍；`normal()` 恢复 Element Plus 的 1 倍时长，`slow()` 为 1.6 倍，`speed(MotionSpeed::Custom(x))` 为自定义倍数。",
    ),
    (
        "MenuState::active_index, is_open, open, close, update_active_index",
        "Option<&str> / bool / index, Window",
        "Inspect or update menu state from your view.",
        "在应用视图中读取或更新菜单状态。",
    ),
];
const LOADING: &[Row] = &[
    (
        "el_loading(), Loading::new, id",
        "ElementId · optional",
        "Create a scoped mask; use a stable ID for multiple named instances.",
        "创建局部遮罩；多个命名实例使用稳定 ID。",
    ),
    (
        "active, fullscreen",
        "bool · false",
        "Show the mask. Mount at the window root for fullscreen coverage.",
        "控制遮罩显示；挂在窗口根部才覆盖全窗。",
    ),
    (
        "text, v_text, h_text",
        "SharedString · —",
        "Set text; v_text is below the indicator, h_text is to its right.",
        "设置提示文字；v_text 在指示器下方，h_text 在指示器右侧。",
    ),
    (
        "color",
        "Hsla · primary",
        "Override the spinner and text foreground color; custom indicators inherit unless they set their own.",
        "覆盖指示器和文字前景色；自定义指示器未单独设色时继承该颜色。",
    ),
    (
        "spinner",
        "LoadingSpinner · Circle",
        "Circle is the default mask arc; Spokes uses the official SVG; Dots uses 14 graduated dots.",
        "Circle 是默认圆弧；Spokes 使用官方 SVG；Dots 为 14 个大小渐变的圆点。",
    ),
    (
        "motion",
        "LoadingMotion · Uniform",
        "Uniform and LinearAcceleration take two seconds per turn; EaseIn takes one. Respects reduced motion.",
        "匀速和线性加速每圈两秒；EaseIn 每圈一秒。遵从减少动效设置。",
    ),
    (
        "mask_opacity, background",
        "f32 / Hsla · 0.9 light / 0.8 dark",
        "Theme mask alpha 0–1; explicit background wins. Alpha 0 still blocks the pointer.",
        "遮罩透明度为 0–1；显式背景优先；0 仍拦截指针。",
    ),
    (
        "loading_icon, indicator",
        "Icon / IntoElement · built-in",
        "Rotate an SVG or font glyph using motion, or supply an element with its own behavior.",
        "按所选动效旋转 SVG 或字体字形，或传入自行控制动效的元素。",
    ),
    (
        "child, children",
        "IntoElement · []",
        "Add the content to cover.",
        "添加被遮罩的内容。",
    ),
];
const RATE: &[Row] = &[
    (
        "el_rate(), Rate::new, id",
        "ElementId · call site",
        "Create a rate. Supply a stable sibling-unique ID when creating repeated rates at the same call site.",
        "创建评分。同一调用位置生成多个评分时，提供稳定且同级唯一的 ID。",
    ),
    (
        "value",
        "f32 · 0",
        "The controlled rating; halves are kept with allow_half.",
        "受控分值；allow_half 时保留半星。",
    ),
    (
        "max",
        "u32 · 5",
        "The highest rating; at least one star.",
        "最高分值；至少为一星。",
    ),
    (
        "size, large, small",
        "RateSize · Default (32 px)",
        "Large is 40 px, Small is 24 px; the star is 18 px, 14 px when small.",
        "Large 为 40 px，Small 为 24 px；星形为 18 px，Small 为 14 px。",
    ),
    (
        "allow_half, clearable",
        "bool · false",
        "Pick half stars, or reset the value to zero by re-picking it.",
        "允许半星，或再次选择当前分值时清零。",
    ),
    (
        "disabled",
        "bool · false",
        "Make the rating read-only; fractional values still display clipped.",
        "只读模式；小数分值仍按比例裁剪显示。",
    ),
    (
        "show_text, show_score",
        "bool · false",
        "Show the matching text or the score; show_score wins, and nothing shows at zero.",
        "显示对应文案或分数；show_score 优先，零分时不显示。",
    ),
    (
        "texts, score_template",
        "Iterator<SharedString> / SharedString · upstream English / {value}",
        "The texts per whole star, and the score template.",
        "每个整星对应的文案，以及分数模板。",
    ),
    (
        "low_threshold, high_threshold",
        "f32 · 2 / 4",
        "The score levels for colors and icons; the low threshold belongs to the low level.",
        "颜色与图标的分数级别；低阈值本身属于低级别。",
    ),
    (
        "colors",
        "[Hsla; 3] · $rate-fill-color",
        "One active color per score level, low to high.",
        "三个级别的激活颜色，由低到高。",
    ),
    (
        "void_color, disabled_void_color, text_color",
        "Hsla · $rate-void-color / $fill-color / $text-color-primary",
        "Colors of the unselected stars, the read-only unselected stars, and the text.",
        "未选中星形、只读未选中星形与文案的颜色。",
    ),
    (
        "icon, icons, void_icon, disabled_void_icon",
        "Icon · StarFilled / Star / StarFilled",
        "The active icons (one for every level, or one per level), the unselected icon, and the read-only unselected icon.",
        "激活图标（全部级别统一或逐级别）、未选中图标与只读未选中图标。",
    ),
    (
        "accessibility_label",
        "SharedString · rating",
        "The accessible name of the slider.",
        "滑块的无障碍名称。",
    ),
];
const INPUT: &[Row] = &[
    (
        "el_input(), Input::new, id",
        "ElementId · call site",
        "Create a field. Supply a stable sibling-unique ID when creating repeated fields at the same call site.",
        "创建输入框。同一调用位置生成多个输入框时，提供稳定且同级唯一的 ID。",
    ),
    (
        "value, placeholder",
        "SharedString · empty",
        "The controlled value, and the placeholder shown while it is empty.",
        "受控值，以及值为空时显示的占位文本。",
    ),
    (
        "size, large, small",
        "InputSize · Default (32 px)",
        "Large is 40 px, Small is 24 px; font size is 14 px, 12 px when small, and horizontal padding 16/12/8 px.",
        "Large 为 40 px，Small 为 24 px；字号为 14 px，Small 为 12 px，水平内边距 16/12/8 px。",
    ),
    (
        "radius, radius_off, radius_small, radius_large, round",
        "InputRadius · Default (8 px)",
        "Native corner profiles: Off 0 px, Small 4 px, Default 8 px, Large 12 px, and Round capsule ends.",
        "原生圆角配置：Off 0 px、Small 4 px、Default 8 px、Large 12 px，Round 为胶囊两端。",
    ),
    (
        "active_border_color, outline, text_color",
        "Rgba / bool · primary / false / theme",
        "Customize the focused border, optional 2 px focus ring, and normal text/caret color. Disabled text keeps its theme token.",
        "自定义激活边框、可选 2 px 焦点环，以及正常文字与插入符号颜色；禁用文字仍使用主题令牌。",
    ),
    (
        "disabled, readonly",
        "bool · false",
        "disabled blocks all editing; readonly still moves the caret and selection and copies.",
        "disabled 禁止一切编辑；readonly 仍可移动插入符号与选择并可复制。",
    ),
    (
        "clearable",
        "bool · false",
        "Adds a clear button at the right edge, shown on focus or by hovering it.",
        "在右缘添加清空按钮，聚焦或悬停该按钮时显示。",
    ),
    (
        "password, show_password",
        "bool · false",
        "password masks the value with bullets; show_password adds a reveal toggle.",
        "password 以圆点掩码显示；show_password 添加显示切换按钮。",
    ),
    (
        "maxlength, show_word_limit",
        "usize / bool · unset / false",
        "Stops typing past the limit and optionally shows a length / limit count that turns red when exceeded.",
        "超过限制后停止输入，可选显示 字数/上限 计数，超限后变红。",
    ),
    (
        "prefix, suffix",
        "IntoElement · none",
        "The leading and trailing slots, sized and colored like the field icons.",
        "前缀与后缀插槽，尺寸与颜色与字段图标一致。",
    ),
    (
        "textarea, rows",
        "bool / usize · false / 2",
        "Switch to a multi-line textarea with a visible row count, soft wrapping, internal scrolling, and caret auto-scroll. Enter inserts a newline; arrows and Home/End move by visual line.",
        "切换为多行文本域，可设置可见行数，支持软换行、内部滚动与插入符号自动滚动。Enter 插入换行，方向键与 Home/End 按视觉行移动。",
    ),
];
const INPUT_EVENTS: &[Row] = &[
    (
        "oninput",
        "SharedString, &mut Window, &mut App",
        "Every committed keystroke reports the value. IME preedit stays silent and reports once when the composition commits, like the web input event.",
        "每次提交的按键报告当前值。输入法组合期间不触发，组合提交后只报告一次，对应 Web 的 input 事件。",
    ),
    (
        "onchange",
        "SharedString, &mut Window, &mut App",
        "A blur or a clear reports the committed value, like the web change event; Enter does not commit.",
        "失焦或清空时报告提交值，对应 Web 的 change 事件；Enter 不提交。",
    ),
    (
        "onfocus, onblur",
        "&mut Window, &mut App",
        "The field gained or lost focus.",
        "字段获得或失去焦点。",
    ),
    (
        "onclear",
        "SharedString, &mut Window, &mut App",
        "The clear button emptied the field.",
        "清空按钮清空了字段。",
    ),
    (
        "oncompositionstart",
        "&mut Window, &mut App",
        "An IME composition started.",
        "输入法组合开始。",
    ),
    (
        "oncompositionupdate",
        "SharedString, &mut Window, &mut App",
        "An IME preedit update reports the composing text; oninput remains silent.",
        "输入法组合更新报告组合文本；oninput 保持静默。",
    ),
    (
        "oncompositionend",
        "SharedString, &mut Window, &mut App",
        "An IME composition ended with its final text; oninput follows with the committed field value.",
        "输入法组合结束并报告最终文本；随后 oninput 报告提交后的字段值。",
    ),
];
const INPUT_NUMBER: &[Row] = &[
    (
        "el_input_number(), InputNumber::new, id",
        "ElementId · call site",
        "Create a numeric field. Supply a stable sibling-unique ID when creating repeated fields at the same call site.",
        "创建数字输入框。同一调用位置生成多个时，提供稳定且同级唯一的 ID。",
    ),
    (
        "value",
        "Option<f64> · None",
        "The controlled value. Non-finite values are treated as empty; the application moves the value in oninput or onchange.",
        "受控值。非有限值按空值处理；应用在 oninput 或 onchange 中移动该值。",
    ),
    (
        "min, max, step, step_strictly",
        "f64 / f64 / f64 / bool · safe integer range / 1 / false",
        "Clamp to the range and step by the increment. Strict mode rounds input to a step multiple. Invalid steps restore 1; an inverted range collapses to min.",
        "钳制到范围并按增量步进。严格模式将输入取整到步长倍数；非法步长恢复为 1，反向范围收敛到 min。",
    ),
    (
        "precision",
        "u32 · unset",
        "Round and display with fixed decimal precision, capped at 15 useful f64 digits.",
        "按固定小数位舍入与显示，最多使用 f64 的 15 位有效小数。",
    ),
    (
        "value_on_clear",
        "InputNumberValueOnClear · None",
        "Choose Min, Max, or a concrete value when the edit is cleared; None commits null.",
        "选择清空时提交 Min、Max 或具体值；None 提交空值。",
    ),
    (
        "size, large, small",
        "InputSize · Default",
        "Widths are 180/150/120 px and outer heights 40/32/24 px, matching Theme Chalk.",
        "宽度为 180/150/120 px，外高为 40/32/24 px，与 Theme Chalk 一致。",
    ),
    (
        "controls, controls_right",
        "bool · true / false",
        "Show side controls, or stack increase/decrease at the right edge. Hidden controls use 15 px horizontal padding.",
        "显示侧边控件，或在右缘上下排列增加/减少按钮；隐藏控件时水平内边距为 15 px。",
    ),
    (
        "align",
        "InputNumberAlign · Center",
        "Align the numeric text left, center, or right.",
        "将数字文字左对齐、居中或右对齐。",
    ),
    (
        "disabled, readonly",
        "bool · false",
        "disabled blocks editing and stepping; readonly keeps caret, selection, and copying but blocks value changes.",
        "disabled 禁止编辑与步进；readonly 保留插入符号、选择与复制，但禁止改值。",
    ),
    (
        "disabled_scientific",
        "bool · false",
        "Remove e/E from typed text so scientific notation cannot be entered.",
        "移除键入文本中的 e/E，阻止科学计数法输入。",
    ),
    (
        "placeholder, accessibility_label",
        "SharedString · empty / unset",
        "Set the empty-state hint and the field's accessible name.",
        "设置空态提示与字段的无障碍名称。",
    ),
    (
        "prefix, suffix",
        "IntoElement · none",
        "Add leading and trailing content inside the field.",
        "在字段内部添加前置与后置内容。",
    ),
    (
        "decrease_icon, increase_icon",
        "Icon · Element Plus arrows or minus/plus",
        "Replace the control icons; they inherit the control color and size.",
        "替换控件图标；图标继承控件颜色与尺寸。",
    ),
    (
        "formatter, parser",
        "Fn(f64) -> String / Fn(&str) -> String · none",
        "Format committed values and extract numeric text from an edit. Draft text remains unformatted.",
        "格式化提交值并从编辑文本提取数字；草稿文本保持未格式化。",
    ),
];
const INPUT_NUMBER_EVENTS: &[Row] = &[
    (
        "oninput",
        "Option<f64>, &mut Window, &mut App",
        "Every parsed keystroke, control press, or keyboard step reports the value; invalid text reports None.",
        "每次解析后的按键、控件按下或键盘步进报告值；非法文本报告 None。",
    ),
    (
        "onchange",
        "Option<f64>, Option<f64>, &mut Window, &mut App",
        "A verified value changed on blur or by stepping; arguments are current and previous values.",
        "失焦或步进后，已验证的值发生变化；参数为当前值与先前值。",
    ),
    (
        "onfocus, onblur",
        "&mut Window, &mut App",
        "The inner text field gained or lost focus.",
        "内部文本字段获得或失去焦点。",
    ),
];
const CAPSULE: &[Row] = &[
    (
        "el_capsule_segments(), CapsuleSegments::new, id",
        "ElementId · call site",
        "Create a connected single-selection capsule with a stable sibling-unique ID.",
        "创建连通单选胶囊；同一调用位置生成多个时提供稳定且同级唯一的 ID。",
    ),
    (
        "options",
        "IntoIterator<CapsuleOption> · empty",
        "Set value/label pairs. Values should be unique because they also key the rendered options.",
        "设置值/标签对。值同时用作渲染选项的键，应保持唯一。",
    ),
    (
        "selected, clear_selection",
        "SharedString / — · none",
        "Control the selected value, or clear it. Unknown values select nothing.",
        "控制选中值，或清空选择。未知值不选中任何选项。",
    ),
    (
        "height, font_size",
        "f32 · 30 / follows height",
        "Set the shared option height and label size. The default label size follows height, clamped to 10–13 px.",
        "设置选项共用高度与标签字号。默认字号随高度变化，并钳制在 10–13 px。",
    ),
    (
        "disabled",
        "bool · false",
        "Block pointer and keyboard activation and remove options from the Tab order.",
        "阻止指针与键盘激活，并将选项移出 Tab 顺序。",
    ),
    (
        "accessibility_label",
        "SharedString · unset",
        "Name the capsule group for assistive technology; each option uses its label as its accessible name.",
        "为胶囊组提供无障碍名称；每个选项使用自身标签作为无障碍名称。",
    ),
];
const CAPSULE_EVENTS: &[Row] = &[(
    "onselect",
    "SharedString, &mut Window, &mut App",
    "Pointer or Enter/Space activation reports the option value, including an already selected option.",
    "指针或 Enter/Space 激活报告选项值；重复激活已选项也会报告。",
)];
const SELECT: &[Row] = &[
    (
        "el_select(), Select::new, id",
        "ElementId · call site",
        "Create a 100%-width select field with a stable sibling-unique ID.",
        "创建宽度为 100% 的选择器；同一调用位置生成多个时提供稳定且同级唯一的 ID。",
    ),
    (
        "value, selected, values",
        "Option<SharedString> / SharedString / Vec<SharedString> · none",
        "Control a single value or the multiple-selection vector. The application moves it in onchange.",
        "控制单选值或多选向量；应用在 onchange 中移动该值。",
    ),
    (
        "options",
        "IntoIterator<SelectOption> · empty",
        "Set value/label pairs; individual options may be disabled.",
        "设置值/标签对；单个选项可禁用。",
    ),
    (
        "multiple, multiple_limit",
        "bool / usize · false / 0",
        "Enable multiple selection and optionally cap it; zero means unlimited.",
        "启用多选并可限制数量；0 表示不限。",
    ),
    (
        "size, large, small",
        "InputSize · Default",
        "Use the Input field's 40/32/24 px heights and 14/14/12 px type sizes.",
        "使用 Input 字段的 40/32/24 px 外高与 14/14/12 px 字号。",
    ),
    (
        "filterable, filter_method, remote",
        "bool / Fn / bool · false / none / false",
        "Enable native text filtering, caller-driven filtering, or remote option updates.",
        "启用原生文本过滤、调用方驱动过滤或远程选项更新。",
    ),
    (
        "clearable, value_on_clear",
        "bool / Option<SharedString> · false / None",
        "Show a clear control on focus or hover and choose the single-select clear value.",
        "聚焦或悬停时显示清空控件，并选择单选清空值。",
    ),
    (
        "reserve_keyword",
        "bool · true",
        "Keep the filter query after a multiple selection, or clear it.",
        "多选后保留过滤关键词，或将其清空。",
    ),
    (
        "disabled, loading",
        "bool · false",
        "Disable all interaction, or show the localized loading row and spinner.",
        "禁用全部交互，或显示本地化加载行与旋转图标。",
    ),
    (
        "placeholder, accessibility_label",
        "SharedString · locale / unset",
        "Set the empty-state hint and accessible field name.",
        "设置空态提示与字段无障碍名称。",
    ),
    (
        "prefix, suffix, header, footer",
        "IntoElement · none",
        "Add field adornments and dropdown header/footer content.",
        "添加字段前后缀与下拉列表页眉/页脚。",
    ),
    (
        "loading_text, no_data_text, no_match_text",
        "SharedString · locale",
        "Override the built-in loading, empty, and no-match messages.",
        "覆盖内置加载、空数据与无匹配文案。",
    ),
];
const SELECT_EVENTS: &[Row] = &[
    (
        "onchange",
        "SelectValue, &mut Window, &mut App",
        "A single option, multiple toggle, removal, or clear changed the controlled value.",
        "单选、多选切换、移除或清空导致受控值变化。",
    ),
    (
        "onvisiblechange",
        "bool, &mut Window, &mut App",
        "The dropdown opened or closed.",
        "下拉列表打开或关闭。",
    ),
    (
        "onremove",
        "SharedString, &mut Window, &mut App",
        "A multiple-selection value was removed.",
        "多选值被移除。",
    ),
    (
        "onclear",
        "&mut Window, &mut App",
        "The clear control was activated.",
        "清空控件被激活。",
    ),
    (
        "onfocus, onblur",
        "&mut Window, &mut App",
        "The inner text field gained or lost focus.",
        "内部文本字段获得或失去焦点。",
    ),
];
const AUTO_COMPLETE: &[Row] = &[
    (
        "el_auto_complete(), AutoComplete::new, id",
        "ElementId · call site",
        "Create an autocomplete input with a stable sibling-unique ID.",
        "创建自动补全输入框；同一调用位置生成多个时提供稳定且同级唯一的 ID。",
    ),
    (
        "value, options",
        "SharedString / IntoIterator<AutoCompleteOption> · empty",
        "Control the input text and suggestion list.",
        "控制输入文本与建议列表。",
    ),
    (
        "fetch_method",
        "Fn(&str, &mut Window, &mut App) · none",
        "Fetch caller-driven suggestions on input or focus; supplied options are shown without local filtering.",
        "输入或聚焦时获取调用方建议；提供的选项不经过本地过滤。",
    ),
    (
        "trigger_on_focus, highlight_first",
        "bool · true / false",
        "Open suggestions on focus and optionally highlight the first item.",
        "聚焦时打开建议，并可默认高亮第一项。",
    ),
    (
        "select_when_unmatched, loop_navigation",
        "bool · false / true",
        "Allow Enter to select raw unmatched text and control keyboard wrapping.",
        "允许 Enter 选择未匹配的原始文本，并控制键盘循环导航。",
    ),
    (
        "clearable, disabled, loading",
        "bool · false",
        "Show the Input clear control, block interaction, or show the loading row.",
        "显示 Input 清空控件、阻止交互，或显示加载行。",
    ),
    (
        "size, large, small",
        "InputSize · Default",
        "Use the Input field's 40/32/24 px heights and 14/14/12 px type sizes.",
        "使用 Input 字段的 40/32/24 px 外高与 14/14/12 px 字号。",
    ),
    (
        "placeholder, accessibility_label",
        "SharedString · empty / unset",
        "Set the empty-state hint and accessible field name.",
        "设置空态提示与字段无障碍名称。",
    ),
    (
        "prefix, suffix, header, footer",
        "IntoElement · none",
        "Add field adornments and suggestion header/footer content.",
        "添加字段前后缀与建议列表页眉/页脚。",
    ),
    (
        "loading_text",
        "SharedString · locale",
        "Override the built-in loading message.",
        "覆盖内置加载文案。",
    ),
];
const AUTO_COMPLETE_EVENTS: &[Row] = &[
    (
        "oninput",
        "SharedString, &mut Window, &mut App",
        "Every edit or accepted suggestion reports the input text.",
        "每次编辑或接受建议报告输入文本。",
    ),
    (
        "onchange",
        "SharedString, &mut Window, &mut App",
        "The Input commits a changed value on blur or clear.",
        "Input 在失焦或清空时报告提交值。",
    ),
    (
        "onselect",
        "AutoCompleteOption, &mut Window, &mut App",
        "A suggestion was accepted by pointer or keyboard.",
        "指针或键盘接受了建议。",
    ),
    (
        "onclear",
        "&mut Window, &mut App",
        "The Input clear control emptied the value.",
        "Input 清空控件清空了值。",
    ),
    (
        "onvisiblechange",
        "bool, &mut Window, &mut App",
        "The suggestion popup opened or closed.",
        "建议列表打开或关闭。",
    ),
    (
        "onfocus, onblur",
        "&mut Window, &mut App",
        "The inner text field gained or lost focus.",
        "内部文本字段获得或失去焦点。",
    ),
];
const COLOR_PICKER: &[Row] = &[
    (
        "el_color_picker(), ColorPicker::new, id",
        "ElementId · call site",
        "Create a square color swatch with a stable sibling-unique ID.",
        "创建方形颜色色块；同一调用位置生成多个时提供稳定且同级唯一的 ID。",
    ),
    (
        "value",
        "Option<SharedString> · None",
        "Control the committed color string; invalid strings fall back to primary.",
        "控制提交的颜色字符串；非法字符串回退到主色。",
    ),
    (
        "format",
        "ColorFormat · Hex",
        "Choose Hex, Rgb, or Rgba output.",
        "选择 Hex、Rgb 或 Rgba 输出。",
    ),
    (
        "show_alpha",
        "bool · false",
        "Show the alpha slider, checkerboard, and alpha in Hex output.",
        "显示 Alpha 滑块、棋盘格，并在 Hex 输出中包含 Alpha。",
    ),
    (
        "size, large, small",
        "InputSize · Default",
        "Use 40/32/24 px square swatches.",
        "使用 40/32/24 px 方形色块。",
    ),
    (
        "disabled, clearable",
        "bool · false / true",
        "Block interaction and optionally show the panel's clear action.",
        "阻止交互，并可显示面板中的清空操作。",
    ),
    (
        "predefine",
        "IntoIterator<SharedString> · empty",
        "Set the preset color swatches; invalid entries are skipped.",
        "设置预设颜色色块；非法条目被跳过。",
    ),
    (
        "accessibility_label",
        "SharedString · locale",
        "Name the trigger for assistive technology.",
        "为触发器提供无障碍名称。",
    ),
];
const COLOR_PICKER_EVENTS: &[Row] = &[
    (
        "onchange",
        "Option<SharedString>, &mut Window, &mut App",
        "Confirm commits the draft color; clear commits None.",
        "确认提交草稿颜色；清空提交 None。",
    ),
    (
        "onactivechange",
        "Option<SharedString>, &mut Window, &mut App",
        "A panel drag or preset changes the draft color.",
        "面板拖动或预设改变草稿颜色。",
    ),
    (
        "onclear",
        "&mut Window, &mut App",
        "The panel's clear action was activated.",
        "面板中的清空操作被激活。",
    ),
    (
        "onfocus, onblur",
        "&mut Window, &mut App",
        "The trigger gained or lost focus.",
        "触发器获得或失去焦点。",
    ),
];
const TIME_PICKER: &[Row] = &[
    (
        "el_time_picker(), TimePicker::new, id",
        "ElementId · call site",
        "Create a single-time picker with a stable sibling-unique ID.",
        "创建单选时间选择器；同一调用位置生成多个时提供稳定且同级唯一的 ID。",
    ),
    (
        "value",
        "Option<SharedString> · None",
        "Control the bound time string; invalid strings display as empty.",
        "控制绑定时间字符串；非法字符串显示为空。",
    ),
    (
        "format",
        "TimeFormat · HourMinute",
        "Choose HH:mm or HH:mm:ss input, display, and output.",
        "选择 HH:mm 或 HH:mm:ss 输入、显示与输出。",
    ),
    (
        "size, large, small",
        "InputSize · Default",
        "Use the Input field's 40/32/24 px heights.",
        "使用 Input 字段的 40/32/24 px 外高。",
    ),
    (
        "disabled, readonly, editable",
        "bool · false / false / true",
        "Disable all interaction, allow read-only selection, or disable text editing while keeping the panel.",
        "禁用全部交互、允许只读选择，或禁用文本编辑但保留面板。",
    ),
    (
        "clearable",
        "bool · true",
        "Show the Input clear control.",
        "显示 Input 清空控件。",
    ),
    (
        "placeholder, accessibility_label",
        "SharedString · locale / unset",
        "Set the empty-state hint and accessible field name.",
        "设置空态提示与字段无障碍名称。",
    ),
    (
        "prefix_icon",
        "Icon · Clock",
        "Replace the leading clock icon.",
        "替换前置时钟图标。",
    ),
];
const TIME_PICKER_EVENTS: &[Row] = &[
    (
        "onchange",
        "Option<SharedString>, &mut Window, &mut App",
        "A column, keyboard step, blur, clear, or confirm changes the time.",
        "列选择、键盘步进、失焦、清空或确认改变时间。",
    ),
    (
        "onvisiblechange",
        "bool, &mut Window, &mut App",
        "The time panel opened or closed.",
        "时间面板打开或关闭。",
    ),
    (
        "onclear",
        "&mut Window, &mut App",
        "The Input clear control emptied the time.",
        "Input 清空控件清空了时间。",
    ),
    (
        "onfocus, onblur",
        "&mut Window, &mut App",
        "The inner text field gained or lost focus.",
        "内部文本字段获得或失去焦点。",
    ),
];
const SLIDER: &[Row] = &[
    (
        "el_slider(), Slider::new, id",
        "ElementId · call site",
        "Create a slider. Supply a stable sibling-unique ID when creating repeated sliders at the same call site.",
        "创建滑块。同一调用位置生成多个滑块时，提供稳定且同级唯一的 ID。",
    ),
    (
        "value, range",
        "f32 · 0 / (f32, f32)",
        "The controlled value of a single slider, or the low and high ends of a range slider. Handles may cross; the reported pair stays ordered.",
        "单滑块的受控值，或范围滑块的两端。手柄可交叉；报告的一端保持有序。",
    ),
    (
        "min, max",
        "f32 · 0 / 100",
        "The value range; a degenerate range pins the value at the minimum.",
        "取值范围；退化的范围将值固定在最小值。",
    ),
    (
        "step",
        "f32 · 1",
        "The snap size. Steps round to the nearest step, the gap before the maximum rounds across it, and the result keeps the endpoints' precision.",
        "步长。按最近步长取整，最大值前的间隙向其取整，结果保留端点精度。",
    ),
    (
        "size, large, small",
        "SliderSize · Default (32 px)",
        "Large is 40 px, Small is 24 px; the runway is 6 px and the handle 20 px in every size.",
        "Large 为 40 px，Small 为 24 px；滑轨均为 6 px，手柄均为 20 px。",
    ),
    (
        "disabled",
        "bool · false",
        "Make the slider read-only; the bar and handles take the placeholder color.",
        "只读模式；进度条与手柄使用占位色。",
    ),
    (
        "vertical, height",
        "bool · false / f32 · 200",
        "Lay the runway out vertically with the given length; horizontal sliders fill their container's width.",
        "按给定长度竖直布局滑轨；水平滑块填满容器宽度。",
    ),
    (
        "show_stops",
        "bool · false",
        "Draw a stop at every step boundary.",
        "在每个步长边界绘制断点。",
    ),
    (
        "show_tooltip, format_tooltip",
        "bool · true / Fn(f32) -> SharedString",
        "Show the value in a tooltip on the handles, and format it; the raw number by default.",
        "在手柄上以提示显示值，并可格式化；默认为原始数字。",
    ),
    (
        "active_color, inactive_color",
        "Hsla · primary / border",
        "Native color overrides for the active bar and handle outline, and the inactive runway. Disabled sliders keep their neutral disabled colors.",
        "原生颜色覆盖：进度条与手柄轮廓使用 active_color，未填充滑轨使用 inactive_color。禁用态保持中性禁用色。",
    ),
    (
        "marks",
        "Iterator<SliderMark>",
        "The labeled breakpoints; points outside [min, max] are dropped and the rest are sorted.",
        "带标签的断点；超出 [min, max] 的点被丢弃，其余排序。",
    ),
];
const SLIDER_EVENTS: &[Row] = &[
    (
        "oninput",
        "SliderValue, &mut Window, &mut App",
        "Dragging, a runway click, or a keyboard step reports the value while it changes, like the web input event.",
        "拖动、点击滑轨或键盘步进时持续报告当前值，对应 Web 的 input 事件。",
    ),
    (
        "onchange",
        "SliderValue, &mut Window, &mut App",
        "A drag end, a runway click, or a keyboard step reports the committed value, like the web change event. Arrows step by step, Page Up/Down by four steps, Home and End jump to the ends.",
        "拖动结束、点击滑轨或键盘步进时报告提交值，对应 Web 的 change 事件。方向键按步长递进，Page Up/Down 为四倍步长，Home 与 End 跳至两端。",
    ),
];
const RATE_EVENTS: &[Row] = &[(
    "onchange",
    "f32, &mut Window, &mut App",
    "Pointer or keyboard activation reports the selected value; not emitted while read-only. Arrow keys step by 1, or 0.5 with allow_half, and stop propagating.",
    "指针或键盘激活时报告所选分值；只读时不触发。方向键按 1 递进，allow_half 时为 0.5，并停止传播。",
)];
const CHECKBOX: &[Row] = &[
    (
        "el_checkbox(), Checkbox::new, id",
        "ElementId · call site",
        "Create a checkbox. Supply a stable sibling-unique ID when creating repeated checkboxes at the same call site.",
        "创建多选框。同一调用位置生成多个多选框时，提供稳定且同级唯一的 ID。",
    ),
    (
        "label, content",
        "SharedString / IntoElement · —",
        "Label or custom contents next to the box; custom contents take precedence.",
        "复选框旁的文字或自定义内容；自定义内容优先。",
    ),
    (
        "checked, indeterminate",
        "bool · false",
        "The controlled state, and the partial state of a check-all parent. Activating an indeterminate checkbox checks it.",
        "受控状态与全选父级的半选状态。半选状态再次激活后变为选中。",
    ),
    (
        "size, large, small",
        "CheckboxSize · Default (32 px)",
        "Large is 40 px, Small is 24 px; the box is 14 px, or 12 px with a border.",
        "Large 为 40 px，Small 为 24 px；方框为 14 px，带边框时为 12 px。",
    ),
    (
        "border",
        "bool · false",
        "Draw the surrounding border with the size's paddings and radius.",
        "按尺寸的内边距与圆角绘制外边框。",
    ),
    (
        "disabled",
        "bool · false",
        "Block activation and show the not-allowed cursor.",
        "禁止触发并显示禁止光标。",
    ),
    (
        "active_color, inactive_color",
        "Hsla · primary / border",
        "Native color overrides for the checked and indeterminate accent, and the unchecked outline. Disabled checkboxes keep their neutral disabled colors.",
        "原生颜色覆盖：选中与半选强调色使用 active_color，未选中轮廓使用 inactive_color。禁用态保持中性禁用色。",
    ),
    (
        "accessibility_label",
        "SharedString · —",
        "The accessible name; label icon-only checkboxes. The checked and indeterminate states use the toggle attribute.",
        "无障碍名称；纯图标多选框应提供。选中与半选状态通过 toggle 属性表达。",
    ),
];
const CHECKBOX_GROUP: &[Row] = &[
    (
        "el_checkbox_group(), CheckboxGroup::id",
        "ElementId · call site",
        "Create a group. The ID carries the accessibility group role.",
        "创建分组；ID 承载无障碍 group 角色。",
    ),
    (
        "checkbox, checkboxes",
        "Checkbox / IntoIterator<Item = Checkbox> · []",
        "Add checkboxes. Each one keeps its own events.",
        "添加多选框；事件仍由各多选框提供。",
    ),
    (
        "direction, horizontal, vertical",
        "Direction · Horizontal",
        "Lay the children out in a row or a column.",
        "按行或列排列子项。",
    ),
    (
        "gap",
        "f32 · 30",
        "The space between siblings, from upstream's margin-right.",
        "子项间距，取自上游的 margin-right。",
    ),
    (
        "size, disabled, border",
        "CheckboxSize / bool · unset",
        "Defaults for children that have not chosen their own.",
        "未显式设置的子项使用这些默认值。",
    ),
    (
        "active_color, inactive_color",
        "Hsla · unset",
        "Native color defaults for children that have not chosen their own.",
        "原生颜色默认值；未显式设置的子项继承。",
    ),
    (
        "accessibility_label",
        "SharedString · —",
        "Names the group for assistive technology.",
        "为辅助技术命名分组。",
    ),
];
const CHECKBOX_EVENTS: &[Row] = &[(
    "Checkbox::onchange",
    "bool, &mut Window, &mut App",
    "Pointer or keyboard activation reports the next checked state; not emitted while disabled.",
    "指针或键盘激活时报告下一个选中状态；禁用时不触发。",
)];
const RADIO: &[Row] = &[
    (
        "el_radio(), Radio::new, id",
        "ElementId · call site",
        "Create a radio. Supply a stable sibling-unique ID when creating repeated radios at the same call site.",
        "创建单选框。同一调用位置生成多个单选框时，提供稳定且同级唯一的 ID。",
    ),
    (
        "label, content",
        "SharedString / IntoElement · —",
        "Label or custom contents next to the input; custom contents take precedence.",
        "单选框旁的文字或自定义内容；自定义内容优先。",
    ),
    (
        "checked",
        "bool · false",
        "Marks the current selection; the application moves it in onchange.",
        "标记当前选中项；由应用在 onchange 中移动选择。",
    ),
    (
        "size, large, small",
        "RadioSize · Default (32 px)",
        "Large is 40 px, Small is 24 px; the input is 14 px, or 12 px with a border.",
        "Large 为 40 px，Small 为 24 px；圆点框为 14 px，带边框时为 12 px。",
    ),
    (
        "border, disabled",
        "bool · false",
        "Draw the surrounding border, and block activation.",
        "绘制外边框与禁止触发。",
    ),
    (
        "active_color, inactive_color",
        "Hsla · primary / border",
        "Native color overrides for the selected accent, and the unchecked outline. Disabled radios keep their neutral disabled colors.",
        "原生颜色覆盖：选中强调色使用 active_color，未选中轮廓使用 inactive_color。禁用态保持中性禁用色。",
    ),
    (
        "accessibility_label",
        "SharedString · —",
        "The accessible name; label icon-only radios. The selected state uses the toggle attribute.",
        "无障碍名称；纯图标单选框应提供。选中状态通过 toggle 属性表达。",
    ),
];
const RADIO_GROUP: &[Row] = &[
    (
        "el_radio_group(), RadioGroup::id",
        "ElementId · call site",
        "Create a group. The ID carries the accessibility radio-group role.",
        "创建分组；ID 承载无障碍 radio group 角色。",
    ),
    (
        "radio, radios",
        "Radio / IntoIterator<Item = Radio> · []",
        "Add radios. Each one keeps its own events.",
        "添加单选框；事件仍由各单选框提供。",
    ),
    (
        "direction, horizontal, vertical, gap",
        "Direction / f32 · Horizontal / 30",
        "Lay the children out and space them.",
        "排列子项并设置间距。",
    ),
    (
        "size, disabled, border",
        "RadioSize / bool · unset",
        "Defaults for children that have not chosen their own.",
        "未显式设置的子项使用这些默认值。",
    ),
    (
        "active_color, inactive_color",
        "Hsla · unset",
        "Native color defaults for children that have not chosen their own.",
        "原生颜色默认值；未显式设置的子项继承。",
    ),
];
const RADIO_EVENTS: &[Row] = &[(
    "Radio::onchange",
    "bool, &mut Window, &mut App",
    "The radio was activated by pointer or keyboard; not emitted while disabled or when the radio is already selected.",
    "指针或键盘激活该单选框时触发；禁用或已选中时不触发。",
)];
const SWITCH: &[Row] = &[
    (
        "el_switch(), Switch::new, id",
        "ElementId · call site",
        "Create a switch. Supply a stable sibling-unique ID when creating repeated switches at the same call site.",
        "创建开关。同一调用位置生成多个开关时，提供稳定且同级唯一的 ID。",
    ),
    (
        "checked",
        "bool · false",
        "The controlled state; the application updates it in onchange.",
        "受控状态；由应用在 onchange 中更新。",
    ),
    (
        "size, large, small",
        "SwitchSize · Default (40 px core)",
        "Large is 50 px, Small is 30 px; the outer height is 40 / 32 / 24 px.",
        "Large 为 50 px，Small 为 30 px；外部高度为 40 / 32 / 24 px。",
    ),
    (
        "width",
        "f32 · by size",
        "Core width in logical pixels, clamped so the knob stays visible.",
        "轨道逻辑像素宽度；过窄时自动夹紧到滑块可见。",
    ),
    (
        "active_text, inactive_text",
        "SharedString · —",
        "Labels beside the core; the current state's label is shown. With inline_prompt they move inside the core.",
        "轨道两侧的文字；只显示当前状态一侧。inline_prompt 时改为显示在轨道内。",
    ),
    (
        "active_icon, inactive_icon",
        "IntoElement · —",
        "Content beside the core, replacing the matching text.",
        "轨道两侧的内容，替换对应的文字。",
    ),
    (
        "active_action_icon, inactive_action_icon",
        "IntoElement · —",
        "Content inside the knob for each state.",
        "滑块内部按状态显示的内容。",
    ),
    (
        "active_color, inactive_color",
        "Hsla · primary / border",
        "Core colors for each state, like the upstream CSS variables.",
        "两个状态的轨道颜色，对应上游的 CSS 变量。",
    ),
    (
        "inline_prompt",
        "bool · false",
        "Draw the text inside the core instead of beside it.",
        "把文字显示在轨道内部而不是两侧。",
    ),
    (
        "disabled, loading",
        "bool · false",
        "Block activation; loading also shows a spinner in the knob. Disabled dims the control to 0.6.",
        "禁止触发；加载态在滑块内显示指示器。禁用态整体透明度 0.6。",
    ),
    (
        "speed, fast, slow, normal",
        "MotionSpeed · theme",
        "Scale the knob slide, which is 0.2 s by default.",
        "缩放滑块滑动时长，默认 0.2 秒。",
    ),
    (
        "before_change",
        "Fn() -> bool · —",
        "Consulted before every activation; returning false keeps the state.",
        "每次触发前调用；返回 false 时状态不变。",
    ),
    (
        "accessibility_label",
        "SharedString · —",
        "The accessible name; label icon-only switches.",
        "无障碍名称；纯图标开关应提供。",
    ),
];
const SWITCH_EVENTS: &[Row] = &[(
    "onchange",
    "bool, &mut Window, &mut App",
    "Pointer or keyboard activation reports the next checked state; not emitted while disabled or loading, or when before_change returns false.",
    "指针或键盘激活时报告下一个选中状态；禁用、加载或 before_change 返回 false 时不触发。",
)];
const TAG: &[Row] = &[
    (
        "el_tag(), Tag::new, id",
        "ElementId · call site",
        "Create a tag. Supply a stable sibling-unique ID when creating repeated tags at the same call site.",
        "创建标签。同一调用位置生成多个标签时，提供稳定且同级唯一的 ID。",
    ),
    (
        "label, content",
        "SharedString / IntoElement · —",
        "Label or custom contents; custom contents take precedence.",
        "文字或自定义内容；自定义内容优先。",
    ),
    (
        "kind, primary, success, info, warning, danger",
        "TagType · Primary",
        "Semantic color. Tag has no neutral variant.",
        "语义颜色；标签没有默认（中性）变体。",
    ),
    (
        "effect, dark, plain",
        "TagEffect · Light",
        "Dark uses the solid type color; plain uses the blank fill with a light border.",
        "Dark 为类型色实底；plain 为留白底加浅色边框。",
    ),
    (
        "size, small, large",
        "TagSize · Default (24 px)",
        "Small is 20 px; Large is 32 px. Font size stays 12 px.",
        "Small 为 20 px，Large 为 32 px；字号均为 12 px。",
    ),
    (
        "round",
        "bool · false",
        "Capsule shape; the radius is derived from the outer height.",
        "胶囊形状；圆角按外框高度计算。",
    ),
    (
        "hit",
        "bool · false",
        "Replace the border with the full-strength type color.",
        "以完整类型色替换边框。",
    ),
    (
        "color",
        "Rgba · —",
        "Override only the background, like the upstream inline style; the alpha channel is preserved.",
        "仅覆盖背景色，与上游内联样式一致；保留透明度通道。",
    ),
    (
        "closable",
        "bool · false",
        "Show the close button. Its accessible name comes from the built-in locale.",
        "显示关闭按钮；其无障碍名称来自内置语言词条。",
    ),
    (
        "accessibility_label",
        "SharedString · label",
        "Name icon-only and custom-content tags for assistive technology.",
        "为纯图标或自定义内容的标签提供无障碍名称。",
    ),
];
const CHECK_TAG: &[Row] = &[
    (
        "el_check_tag(), CheckTag::new, id",
        "ElementId · call site",
        "Create a check tag with a stable sibling-unique ID.",
        "创建可选择标签；同一调用位置生成多个时提供稳定且同级唯一的 ID。",
    ),
    (
        "label, content, accessibility_label",
        "SharedString / IntoElement / SharedString · —",
        "Label, custom contents, or the accessible name.",
        "文字、自定义内容或无障碍名称。",
    ),
    (
        "checked",
        "bool · false",
        "Controlled checked state; the application updates it in onchange.",
        "受控选中状态；由应用在 onchange 中更新。",
    ),
    (
        "disabled",
        "bool · false",
        "Block activation and keep the hover fill unchanged.",
        "禁止切换，悬停底色保持不变。",
    ),
    (
        "kind, primary, success, info, warning, danger",
        "TagType · Primary",
        "Semantic color of the checked state.",
        "选中状态的语义颜色。",
    ),
];
const BUTTON_EVENTS: &[Row] = &[(
    "onclick",
    "&ClickEvent, &mut Window, &mut App",
    "Standard pointer or keyboard activation; not emitted while disabled or loading.",
    "标准指针或键盘激活时触发；禁用或加载时不触发。",
)];
const LINK_EVENTS: &[Row] = &[(
    "Link::onclick",
    "&ClickEvent, &mut Window, &mut App",
    "Standard pointer or keyboard activation; fires before the URL opens, and not while disabled.",
    "标准指针或键盘激活时触发；先于打开 URL，禁用时不触发。",
)];
const TAG_EVENTS: &[Row] = &[
    (
        "Tag::onclick",
        "&ClickEvent, &mut Window, &mut App",
        "Activation of the whole tag; the root becomes a keyboard-focusable button.",
        "点击整个标签时触发；根元素成为可键盘聚焦的按钮。",
    ),
    (
        "Tag::onclose",
        "&ClickEvent, &mut Window, &mut App",
        "Activation of the close button. The application removes the tag from its own state.",
        "点击关闭按钮时触发；由应用从自身状态中移除该标签。",
    ),
    (
        "CheckTag::onchange",
        "bool, &mut Window, &mut App",
        "Reports the next checked state on pointer or keyboard activation; not emitted while disabled.",
        "指针或键盘切换时上报下一个选中状态；禁用时不触发。",
    ),
];
const MENU_EVENTS: &[Row] = &[
    (
        "Menu::onselect",
        "&MenuEvent, &mut Window, &mut App",
        "Leaf selection; event has index and original index_path.",
        "选中菜单项；事件包含 index 和原始 index_path。",
    ),
    (
        "Menu::onopen, onclose",
        "&MenuEvent, &mut Window, &mut App",
        "A submenu opens or closes.",
        "子菜单展开或关闭时触发。",
    ),
    (
        "MenuItem::onclick",
        "&MenuEvent, &mut Window, &mut App",
        "Activate this item.",
        "激活当前菜单项。",
    ),
    (
        "MenuItem::onselect",
        "&ClickEvent, &mut Window, &mut App",
        "Compatibility hook for the native activation event.",
        "原生激活事件的兼容回调。",
    ),
];

pub(super) fn render(page: Page, locale: Locale) -> Div {
    let zh = locale == Locale::SimplifiedChinese;
    let (api, events): (&[Row], &[Row]) = match page {
        Page::Init => (INIT, &[]),
        Page::Button => (BUTTON, BUTTON_EVENTS),
        Page::Icon => (ICON, &[]),
        Page::Container => (CONTAINER, &[]),
        Page::Layout => (LAYOUT, &[]),
        Page::Link => (LINK, LINK_EVENTS),
        Page::Text => (TEXT, &[]),
        Page::Scrollbar => (SCROLLBAR, SCROLLBAR_EVENTS),
        Page::Splitter => (SPLITTER, SPLITTER_EVENTS),
        Page::Space => (SPACE, &[]),
        Page::Table => (TABLE, TABLE_EVENTS),
        Page::Tag => (TAG, TAG_EVENTS),
        Page::Menu => (MENU, MENU_EVENTS),
        Page::Loading => (LOADING, &[]),
        Page::Switch => (SWITCH, SWITCH_EVENTS),
        Page::Checkbox => (CHECKBOX, CHECKBOX_EVENTS),
        Page::Radio => (RADIO, RADIO_EVENTS),
        Page::Rate => (RATE, RATE_EVENTS),
        Page::Slider => (SLIDER, SLIDER_EVENTS),
        Page::Input => (INPUT, INPUT_EVENTS),
        Page::InputNumber => (INPUT_NUMBER, INPUT_NUMBER_EVENTS),
        Page::Capsule => (CAPSULE, CAPSULE_EVENTS),
        Page::Select => (SELECT, SELECT_EVENTS),
        Page::AutoComplete => (AUTO_COMPLETE, AUTO_COMPLETE_EVENTS),
        Page::ColorPicker => (COLOR_PICKER, COLOR_PICKER_EVENTS),
        Page::TimePicker => (TIME_PICKER, TIME_PICKER_EVENTS),
    };
    let table = |key: &'static str, headers: [&'static str; 3], rows: &[Row]| {
        el_table(rows.iter().copied())
            .id(key)
            .border(true)
            .column(
                Column::new(headers[0])
                    .min_width(180.)
                    .render(|row: &Row, _| {
                        div()
                            .font_family("monospace")
                            .child(row.0)
                            .into_any_element()
                    }),
            )
            .column(
                Column::new(headers[1])
                    .min_width(170.)
                    .text(|row: &Row, _| row.1.into()),
            )
            .column(
                Column::new(headers[2])
                    .min_width(200.)
                    .text(
                        move |row: &Row, _| {
                            if zh { row.3.into() } else { row.2.into() }
                        },
                    ),
            )
    };
    let heading = |label| {
        div()
            .text_size(px(22.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(label)
    };
    div().flex().flex_col().gap(px(12.))
        .child(heading("API"))
        .child(table("api", if zh { ["名称", "类型 · 默认值", "说明"] } else { ["Name", "Type · Default", "Description"] }, api))
        .when(page == Page::Tag, |el| {
            el.child(heading("CheckTag API"))
                .child(table("check-tag-api", if zh { ["名称", "类型 · 默认值", "说明"] } else { ["Name", "Type · Default", "Description"] }, CHECK_TAG))
        })
        .when(page == Page::Checkbox, |el| {
            el.child(heading("CheckboxGroup API"))
                .child(table("checkbox-group-api", if zh { ["名称", "类型 · 默认值", "说明"] } else { ["Name", "Type · Default", "Description"] }, CHECKBOX_GROUP))
        })
        .when(page == Page::Radio, |el| {
            el.child(heading("RadioGroup API"))
                .child(table("radio-group-api", if zh { ["名称", "类型 · 默认值", "说明"] } else { ["Name", "Type · Default", "Description"] }, RADIO_GROUP))
        })
        .when(page == Page::Splitter, |el| {
            el.child(heading("SplitterPanel API"))
                .child(table("splitter-panel-api", if zh { ["名称", "类型 · 默认值", "说明"] } else { ["Name", "Type · Default", "Description"] }, SPLITTER_PANEL))
        })
        .child(heading(if zh { "事件" } else { "Events" }))
        .when(events.is_empty(), |el| el.child(if zh { "无事件" } else { "No events." }))
        .when(!events.is_empty(), |el| el.child(table("api-events", if zh { ["事件", "参数", "触发条件"] } else { ["Event", "Parameters", "When it fires"] }, events)))
        .when(page == Page::TimePicker, |el| el.child(if zh { "原生差异：当前仅支持单选时间，未移植范围模式、disabled-hours/minutes/seconds、自定义 popper 配置与命令式 exposes。时间格式固定为 HH:mm 或 HH:mm:ss。" } else { "Native differences: only single-time picking is supported; range mode, disabled-hours/minutes/seconds, custom popper configuration, and imperative exposes are not mapped. Time formats are fixed to HH:mm or HH:mm:ss." }))
        .when(page == Page::ColorPicker, |el| el.child(if zh { "原生差异：面板没有文本颜色输入、自定义 popper 配置或命令式 exposes；拖动仅在指针停留于控件上时跟踪。颜色使用 HSV 转换，支持 #RGB/#RGBA/#RRGGBB/#RRGGBBAA 与 rgb/rgba 字符串。" } else { "Native differences: the panel has no text color input, custom popper configuration, or imperative exposes; dragging is tracked only while the pointer remains over the control. Colors use HSV conversion and accept #RGB/#RGBA/#RRGGBB/#RRGGBBAA plus rgb/rgba strings." }))
        .when(page == Page::AutoComplete, |el| el.child(if zh { "原生差异：建议使用 AutoCompleteOption 类型而非任意记录；未移植自定义建议渲染、debounce、placement/popper 配置与命令式 exposes。远程搜索由 fetch_method 驱动。" } else { "Native differences: suggestions use the AutoCompleteOption type instead of arbitrary records; custom suggestion rendering, debounce, placement/popper configuration, and imperative exposes are not mapped. Remote search is caller-driven through fetch_method." }))
        .when(page == Page::Select, |el| el.child(if zh { "原生差异：多选值以逗号摘要显示，不使用可删除 Tag；Backspace 移除最后一个可移除值并通过 onremove 报告。远程搜索由 filter_method 驱动，未移植 debounce、选项分组、自定义选项渲染、折叠标签提示与命令式 focus/blur。" } else { "Native differences: multiple values are summarized as comma-separated labels instead of removable Tags; Backspace removes the last removable value and reports it through onremove. Remote search is caller-driven through filter_method; debounce, option groups, custom option rendering, collapse-tag tooltips, and imperative focus/blur are not mapped." }))
        .when(page == Page::Capsule, |el| el.child(if zh { "这是独立组件，不是 RadioGroup 的圆形单选样式：它保留 ChatSpeed 执行频率控件的连通胶囊几何，但颜色使用本库 Element Plus 主题。选项使用原生 Tab 顺序与 Enter/Space 激活，不提供方向键组导航。" } else { "This is an independent component rather than RadioGroup's circular radio style: it keeps the connected capsule geometry of ChatSpeed's execution-frequency control while resolving colors through this crate's Element Plus theme. Options use native Tab order and Enter/Space activation; arrow-key group navigation is not provided." }))
        .when(page == Page::InputNumber, |el| el.child(if zh { "原生差异：formatter 接收 f64，parser 返回待解析文本；name、inputmode、tabindex 与命令式 focus/blur 暂未映射；validate-event 属于后续 Form。反向 min/max 收敛到 min，精度最多使用 f64 的 15 位有效小数。" } else { "Native differences: formatter receives an f64 and parser returns text to parse; name, inputmode, tabindex, and imperative focus/blur are not mapped yet; validate-event belongs to the future Form component. An inverted min/max range collapses to min, and precision uses at most f64's 15 useful decimal places." }))
        .when(page == Page::Loading, |el| el.child(if zh { "这是声明式原生遮罩，不提供 Web Loading 服务的 beforeClose / closed 回调或 DOM 挂载选项。" } else { "This declarative mask does not expose the web Loading service's beforeClose / closed callbacks or DOM mounting options." }))
        .when(page == Page::Link, |el| el.child(if zh { "原生差异：href 由平台处理器打开，没有当前视图导航；target 未移植；布尔 underline 已废弃故未支持；上游 outline: none 改为可见聚焦环。" } else { "Native differences: href opens through the platform handler instead of navigating the current view; target is not reproduced; the deprecated boolean underline is not supported; the upstream outline: none is replaced by a visible focus ring." }))
        .when(page == Page::Text, |el| el.child(if zh { "原生差异：tag 属性没有 GPUI 对应物；isTruncated 是 truncated 在确定宽度下的排版结果而非可监听状态；修饰由 GPUI 的下划线与删除线实现，没有 CSS text-decoration 合并写法。" } else { "Native differences: tag has no GPUI equivalent; isTruncated is the layout effect of truncated under a definite width rather than a watched ref; decorations come from GPUI underline and strikethrough instead of a CSS text-decoration shorthand." }))
        .when(page == Page::Tag, |el| el.child(if zh { "原生差异：标签挂载与卸载立即完成，不重放上游 el-zoom-in-center 过渡，因此也没有 disable-transitions 属性；color 仅覆盖背景；CheckTag 支持键盘聚焦与触发，上游 span 不支持。" } else { "Native differences: tags mount and unmount immediately without the upstream el-zoom-in-center transition, so there is no disable-transitions property; color overrides only the background; CheckTag is keyboard focusable and activatable, which the upstream span is not." }))
        .when(page == Page::Table, |el| el.child(if zh { "原生差异：表头使用粗体字面配正文文字色；上游用 600 字重配次要文字色，缺少 semibold 字面的原生字体栈会把它画成普通字重且偏淡。排序、行选择、筛选、固定列、树形数据和合计行未移植。" } else { "Native differences: headers use the bold face with the body text color; upstream uses weight 600 on secondary text, which native font stacks without a semibold face paint as plain and washed out. Sorting, row selection, filtering, fixed columns, tree data, and summary rows are not reproduced." }))
        .when(page == Page::Rate, |el| el.child(if zh { "原生差异：图标为 Icon 值而非 Vue 组件，三个级别用数组传入，不支持任意内容；悬停星形没有 1.15 倍缩放（GPUI 无变换动画），但填充仍会预览选择；半星通过裁剪的激活图标叠加实现，而非上游的 decimal 伪元素；分数模板按 Rust f32 显示，3 而不是 3.0；悬停预览存在窗口键控状态中，应用不需要持有。" } else { "Native differences: the icons are Icon values instead of Vue components, so the three levels take an array and arbitrary content is not supported. The hovered star does not scale 1.15x (GPUI has no transform animation), but the fill still previews the selection. The half star is a clipped overlay of the active icon rather than upstream's decimal pseudo element. The score template renders with Rust's f32 display, 3 instead of 3.0. The hover preview lives in window-keyed state, so the application does not own it." }))
        .when(page == Page::Checkbox, |el| el.child(if zh { "原生差异：没有隐藏 checkbox，选中与半选状态用无障碍 toggle 属性（Mixed）与可见聚焦环表达；勾选标记使用内置 Element Plus Check 轮廓而非 CSS 边框旋转技巧；true-value/false-value 与分组的值集合收集属于应用状态，由各多选框的 onchange 处理；分组不提供上游的方向键导航，用原生 Tab 顺序。" } else { "Native differences: there is no hidden checkbox; the checked and indeterminate states use the accessibility toggle attribute (Mixed) and a visible focus ring. The check mark is the bundled Element Plus Check outline rather than a rotated CSS border trick. true-value/false-value and the group's value collection are the application's concern, handled in each checkbox's onchange. The group does not reproduce upstream's arrow-key navigation; native Tab order applies instead." }))
        .when(page == Page::Radio, |el| el.child(if zh { "原生差异：没有隐藏 radio，选中状态用无障碍 toggle 属性与可见聚焦环表达；选中圆点为 4 px 实心圆而非 CSS 伪元素缩放；分组不收集选中值，由应用在各单选框的 onchange 中移动选择；分组不提供上游的方向键导航，用原生 Tab 顺序；重复点击已选中的单选框不触发事件，与上游一致。" } else { "Native differences: there is no hidden radio; the selected state uses the accessibility toggle attribute and a visible focus ring. The selection dot is a 4 px filled circle rather than a scaled CSS pseudo element. The group does not collect the picked value; the application moves the selection in each radio's onchange. Upstream's arrow-key navigation between group items is not reproduced; native Tab order applies instead. Activating the already selected radio fires nothing, like upstream." }))
        .when(page == Page::Switch, |el| el.child(if zh { "原生差异：没有隐藏 checkbox，选中状态用无障碍 toggle 属性与可见聚焦环表达；滑块滑动为 0.2 秒桌面版上游 300 ms 过渡；inline-prompt 显示完整文字并按宽度省略，而不是仅首个字符；active-value/inactive-value 属于应用状态，由 onchange 映射；before-change 为同步谓词，异步拦截由应用自行调度。" } else { "Native differences: there is no hidden checkbox; the checked state is conveyed with the accessibility toggle attribute and a visible focus ring. The knob slides for 0.2 s, the desktop variant of the upstream 300 ms transition. inline-prompt renders the full text with ellipsis instead of the first character. active-value/inactive-value are the application's state, mapped in onchange. before-change is a synchronous predicate; asynchronous gating is the caller's to schedule." }))
}

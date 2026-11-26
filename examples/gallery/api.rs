use super::{Locale, Page};
use gpui::{Div, div, prelude::*, px};
use gpui_el::{Column, Table};

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
        "Rotate an SVG around its center. Respects reduced motion; font glyphs stay static.",
        "围绕中心旋转 SVG。遵从减少动效设置；字体图标保持静态。",
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
        "Fixes or caps the table height; the header stays while the body scrolls.",
        "固定或限制表格高度；表头保持可见，主体滚动。",
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
        "MotionSpeed · theme",
        "Scales the collapse, popup fade, and chevron rotation. `fast()` is 0.6×, `slow()` is 1.6×, and no call uses the theme speed; `speed(MotionSpeed::Custom(x))` multiplies the normal duration.",
        "缩放展开、浮层淡入淡出与箭头旋转。`fast()` 为 0.6 倍，`slow()` 为 1.6 倍，不调用则使用主题速度；`speed(MotionSpeed::Custom(x))` 为自定义倍数。",
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
        "Rotate an SVG icon using motion, or supply an element with its own behavior. Font glyphs stay static.",
        "按所选动效旋转 SVG 图标，或传入自行控制动效的元素。字体图标保持静态。",
    ),
    (
        "child, children",
        "IntoElement · []",
        "Add the content to cover.",
        "添加被遮罩的内容。",
    ),
];
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
        Page::Space => (SPACE, &[]),
        Page::Table => (TABLE, TABLE_EVENTS),
        Page::Tag => (TAG, TAG_EVENTS),
        Page::Menu => (MENU, MENU_EVENTS),
        Page::Loading => (LOADING, &[]),
    };
    let table = |headers: [&'static str; 3], rows: &[Row]| {
        Table::new(rows.iter().copied())
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
        .child(table(if zh { ["名称", "类型 · 默认值", "说明"] } else { ["Name", "Type · Default", "Description"] }, api))
        .when(page == Page::Tag, |el| {
            el.child(heading("CheckTag API"))
                .child(table(if zh { ["名称", "类型 · 默认值", "说明"] } else { ["Name", "Type · Default", "Description"] }, CHECK_TAG))
        })
        .child(heading(if zh { "事件" } else { "Events" }))
        .when(events.is_empty(), |el| el.child(if zh { "无事件" } else { "No events." }))
        .when(!events.is_empty(), |el| el.child(table(if zh { ["事件", "参数", "触发条件"] } else { ["Event", "Parameters", "When it fires"] }, events)))
        .when(page == Page::Loading, |el| el.child(if zh { "这是声明式原生遮罩，不提供 Web Loading 服务的 beforeClose / closed 回调或 DOM 挂载选项。" } else { "This declarative mask does not expose the web Loading service's beforeClose / closed callbacks or DOM mounting options." }))
        .when(page == Page::Link, |el| el.child(if zh { "原生差异：href 由平台处理器打开，没有当前视图导航；target 未移植；布尔 underline 已废弃故未支持；上游 outline: none 改为可见聚焦环。" } else { "Native differences: href opens through the platform handler instead of navigating the current view; target is not reproduced; the deprecated boolean underline is not supported; the upstream outline: none is replaced by a visible focus ring." }))
        .when(page == Page::Tag, |el| el.child(if zh { "原生差异：标签挂载与卸载立即完成，不重放上游 el-zoom-in-center 过渡，因此也没有 disable-transitions 属性；color 仅覆盖背景；CheckTag 支持键盘聚焦与触发，上游 span 不支持。" } else { "Native differences: tags mount and unmount immediately without the upstream el-zoom-in-center transition, so there is no disable-transitions property; color overrides only the background; CheckTag is keyboard focusable and activatable, which the upstream span is not." }))
}

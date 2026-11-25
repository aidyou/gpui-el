// Element Plus documentation categories (MIT, Element Plus contributors).
// Unlisted names are placed in Other, as in the upstream catalog.
use gpui_el::IconPack;
use std::{collections::HashMap, sync::OnceLock};

const GROUPS: &[(&str, &str, &str)] = &[
    (
        "System",
        "系统",
        "Plus Minus CirclePlus Search Female Male Aim House FullScreen Loading Link Service Pointer Star Notification Connection ChatDotRound Setting Clock Position Discount Odometer ChatSquare ChatRound ChatLineRound ChatLineSquare ChatDotSquare View Hide Unlock Lock RefreshRight RefreshLeft Refresh Bell MuteNotification User Check CircleCheck Warning CircleClose Close PieChart More Compass Filter Switch Select SemiSelect CloseBold EditPen Edit Message MessageBox TurnOff Finished Delete Crop SwitchButton Operation Open Remove ZoomOut ZoomIn InfoFilled CircleCheckFilled SuccessFilled WarningFilled CircleCloseFilled QuestionFilled WarnTriangleFilled UserFilled MoreFilled Tools HomeFilled Menu UploadFilled Avatar HelpFilled Share StarFilled Comment Histogram Grid Promotion DeleteFilled RemoveFilled CirclePlusFilled",
    ),
    (
        "Arrow",
        "箭头",
        "ArrowLeft ArrowUp ArrowRight ArrowDown ArrowLeftBold ArrowUpBold ArrowRightBold ArrowDownBold DArrowRight DArrowLeft Download Upload Top Bottom Back Right TopRight TopLeft BottomRight BottomLeft Sort SortUp SortDown Rank CaretLeft CaretTop CaretRight CaretBottom DCaret Expand Fold",
    ),
    (
        "Document",
        "文档",
        "DocumentAdd Document Notebook Tickets Memo Collection Postcard ScaleToOriginal SetUp DocumentDelete DocumentChecked DataBoard DataAnalysis CopyDocument FolderChecked Files Folder FolderDelete FolderRemove FolderOpened DocumentCopy DocumentRemove FolderAdd FirstAidKit Reading DataLine Management Checked Ticket Failed TrendCharts List",
    ),
    (
        "Media",
        "媒体",
        "Microphone Mute Mic VideoPause VideoCamera VideoPlay Headset Monitor Film Camera Picture PictureRounded Iphone Cellphone VideoCameraFilled PictureFilled Platform CameraFilled BellFilled",
    ),
    (
        "Traffic",
        "交通",
        "Location LocationInformation DeleteLocation Coordinate Bicycle OfficeBuilding School Guide AddLocation MapLocation Place LocationFilled Van",
    ),
    (
        "Food",
        "餐饮",
        "Watermelon Pear NoSmoking Smoking Mug GobletSquareFull GobletFull KnifeFork Sugar Bowl MilkTea Lollipop Coffee Chicken Dish IceTea ColdDrink CoffeeCup DishDot IceDrink IceCream Dessert IceCreamSquare ForkSpoon IceCreamRound Food HotWater Grape Fries Apple Burger Goblet GobletSquare Orange Cherry",
    ),
    (
        "Items",
        "物品",
        "Printer Calendar CreditCard Box Money Refrigerator Cpu Football Brush Suitcase SuitcaseLine Umbrella AlarmClock Medal GoldMedal Present Mouse Watch QuartzWatch Magnet Help Soccer ToiletPaper ReadingLamp Paperclip MagicStick Basketball Baseball Coin Goods Sell SoldOut Key ShoppingCart ShoppingCartFull ShoppingTrolley Phone Scissor Handbag ShoppingBag Trophy TrophyBase Stopwatch Timer CollectionTag Discount TakeawayBox PriceTag Wallet Opportunity PhoneFilled WalletFilled GoodsFilled Flag BrushFilled Briefcase Stamp",
    ),
    (
        "Weather",
        "天气",
        "Sunrise Sunny Ship MostlyCloudy PartlyCloudy Sunset Sunrise Drizzling Pouring Cloudy Moon MoonNight Lightning",
    ),
];

pub(super) struct Group {
    pub en: &'static str,
    pub zh: &'static str,
    pub names: Vec<&'static str>,
}

pub(super) fn groups(pack: IconPack) -> &'static [Group] {
    static ELEMENT: OnceLock<Vec<Group>> = OnceLock::new();
    static TABLER: OnceLock<Vec<Group>> = OnceLock::new();
    if pack == IconPack::ElementPlus {
        ELEMENT.get_or_init(|| {
            let mut remaining: HashMap<_, _> = pack
                .names()
                .map(|name| (name.replace('-', ""), name))
                .collect();
            let mut out: Vec<_> = GROUPS
                .iter()
                .map(|&(en, zh, names)| Group {
                    en,
                    zh,
                    names: names
                        .split_whitespace()
                        .filter_map(|name| remaining.remove(&name.to_ascii_lowercase()))
                        .collect(),
                })
                .collect();
            let mut names: Vec<_> = remaining.into_values().collect();
            names.sort_unstable();
            out.push(Group {
                en: "Other",
                zh: "其他",
                names,
            });
            out
        })
    } else {
        TABLER.get_or_init(|| {
            vec![Group {
                en: "All icons",
                zh: "全部图标",
                names: pack.names().collect(),
            }]
        })
    }
}

pub(super) fn snippet(pack: IconPack, name: &str) -> String {
    format!("gpui_el::Icon::from_pack(gpui_el::IconPack::{pack:?}, {name:?}).size(20.)")
}

pub(super) fn matches(name: &str, query: &str) -> bool {
    name.replace('-', "").contains(query)
}

#[test]
fn catalog_contains_every_name_exactly_once() {
    for pack in [IconPack::ElementPlus, IconPack::Tabler] {
        let mut actual: Vec<_> = groups(pack)
            .iter()
            .flat_map(|g| g.names.iter().copied())
            .collect();
        actual.sort_unstable();
        assert_eq!(actual, pack.names().collect::<Vec<_>>());
    }
    assert!(matches("arrow-down-bold", "arrowdown"));
}

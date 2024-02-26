use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;

// 定义数字字符常量
pub const CHINESE_DIGIS: &str = "零一二三四五六七八九";
pub const CHINESE_DIGIS_ALTS: &str = "〇幺两";
pub const SMALLER_BIG_CHINESE_UNITS_SIMPLIFIED: &str = "十百千万";
pub const LARGER_CHINESE_NUMERING_UNITS_SIMPLIFIED: &str = "亿兆京垓秭穰沟涧正载";
pub const SMALLER_CHINESE_NUMERING_UNITS_SIMPLIFIED: &str = "十百千万";

pub const ZERO: &str = "零";
pub const POSITIVE: &str = "正";
pub const NEGATIVE: &str = "负";
pub const POINT: &str = "点";
pub const PLUS: &str = "加";
pub const GANG: &str = "杠";
pub const FRACTION: &str = "分之";
pub const PERCENT: &str = "百分之";

pub const CURRENCY_NAMES: &str = "(人民币|美元|日元|英镑|欧元|马克|法郎|加拿大元|澳元|港币|先令|芬兰马克|爱尔兰镑|\
                                  里拉|荷兰盾|埃斯库多|比塞塔|印尼盾|林吉特|新西兰元|比索|卢布|新加坡元|韩元|泰铢)";
pub const CURRENCY_UNITS: &str = "((亿|千万|百万|万|千|百)|(亿|千万|百万|万|千|百|)元|\
                                  (亿|千万|百万|万|千|百|)块|角|毛|分)";
pub const COM_QUANTIFIERS: &str = "(匹|张|座|回|场|尾|条|个|首|阙|阵|网|炮|顶|丘|棵|只|支|袭|辆|挑|担|颗|壳|窠|曲|墙|群|\
                            腔|砣|座|客|贯|扎|捆|刀|令|打|手|罗|坡|山|岭|江|溪|钟|队|单|双|对|出|口|头|脚|板|跳|\
                            枝|件|贴|针|线|管|名|位|身|堂|课|本|页|家|户|层|丝|毫|厘|分|钱|两|斤|担|铢|石|钧|锱|忽|\
                            (千|毫|微)?克|毫|厘|分|寸|尺|丈|里|寻|常|铺|程|(千|分|厘|毫|微)?米|撮|勺|合|升|斗|石|盘|碗|\
                            碟|叠|桶|笼|盆|盒|杯|钟|斛|锅|簋|篮|盘|桶|罐|瓶|壶|卮|盏|箩|箱|煲|啖|袋|钵|年|月|日|季|\
                            刻|时|周|天|秒|分|旬|纪|岁|世|更|夜|春|夏|秋|冬|代|伏|辈|丸|泡|粒|颗|幢|堆|条|根|支|道|面|\
                            片|张|颗|块)";

pub const CHINESE_PUNC_STOP: &str = "！？｡。";
pub const CHINESE_PUNC_NON_STOP: &str = "＂＃＄％＆＇（）＊＋，－／：；＜＝＞＠［＼］＾＿｀｛｜｝～｟｠｢｣､、〃《》「」『』【】〔〕〖〗〘〙〚〛〜〝〞〟〰〾〿–—‘’‛“”„‟…‧﹏";
pub const CHINESE_PUNC_OTHER: &str = "·〈〉-";

lazy_static! {
    pub static ref CHINESE_PUNC_LIST: String = format!(
        "{}{}{}",
        CHINESE_PUNC_STOP, CHINESE_PUNC_NON_STOP, CHINESE_PUNC_OTHER
    );
}

lazy_static! {
    pub static ref UNITS: BTreeMap<u8, Vec<&'static str>> = [
        (0, vec!["十", "1"]),
        (1, vec!["百", "2"]),
        (2, vec!["千", "3"]),
        (3, vec!["万", "4"]),
        (4, vec!["亿", "8"]),
        (5, vec!["兆", "8"]),
        (6, vec!["兆", "12"]),
        (7, vec!["京", "16"]),
        (8, vec!["垓", "20"]),
        (9, vec!["秭", "24"]),
        (10, vec!["穰", "28"]),
        (11, vec!["沟", "32"]),
        (12, vec!["涧", "36"]),
        (13, vec!["正", "40"]),
        (14, vec!["载", "44"]),
    ]
    .iter()
    .cloned()
    .collect();
    pub static ref DIGITS: BTreeMap<u8, Vec<&'static str>> = [
        (0, vec!["零", "〇"]),
        (1, vec!["一", "幺"]),
        (2, vec!["二", "两"]),
        (3, vec!["三", ""]),
        (4, vec!["四", ""]),
        (5, vec!["五", ""]),
        (6, vec!["六", ""]),
        (7, vec!["七", ""]),
        (8, vec!["八", ""]),
        (9, vec!["九", ""]),
    ]
    .iter()
    .cloned()
    .collect();
    pub static ref SYMBOLS: BTreeMap<u8, Vec<&'static str>> = [
        (0, vec!["正", "+"]),
        (1, vec!["负", "-"]),
        (2, vec!["点", "."]),
    ]
    .iter()
    .cloned()
    .collect();
}

pub struct Digit {
    pub digit: String,
    pub chntext: Option<String>,
}

impl Digit {
    pub fn new(digit: String) -> Digit {
        Self {
            digit,
            chntext: None,
        }
    }

    pub fn to_chntext(&mut self) -> Option<&str> {
        let mut chntext = self.digit.clone();
        let re = Regex::new(r"((?P<integer>\d+)(((?P<symbol>\.)(?P<zeros>0+)?(?P<decimal>\d+)))?)")
            .unwrap();

        // 进行匹配
        if let Some(captures) = re.captures(&self.digit) {
            // 提取整数和小数部分
            let int: Option<String> = captures
                .name("integer")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let point: Option<String> = captures
                .name("symbol")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let zeros: Option<String> = captures
                .name("zeros")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let dec: Option<String> = captures
                .name("decimal")
                .map(|m| m.as_str().parse().unwrap_or_default());

            if let Some(int_data) = int {
                if !int_data.is_empty() {
                    chntext = chntext.replace(
                        int_data.as_str(),
                        int_data
                            .parse::<u32>()
                            .unwrap()
                            .to_chinese(
                                ChineseVariant::Simple,
                                ChineseCase::Lower,
                                ChineseCountMethod::TenThousand,
                            )
                            .unwrap()
                            .as_str(),
                    );
                }
            }
            if let Some(point_data) = point {
                if let Some(zeros_data) = zeros {
                    if !point_data.is_empty() && !zeros_data.is_empty() {
                        chntext = chntext.replace(
                            format!(".{}", zeros_data.as_str()).as_str(),
                            format!("{}{}", POINT, ZERO.repeat(zeros_data.len())).as_str(),
                        );
                    }
                } else if !point_data.is_empty() {
                    chntext = chntext.replace(point_data.as_str(), POINT);
                }
            }
            if let Some(dec_data) = dec {
                if !dec_data.is_empty() {
                    chntext = chntext.replace(
                        dec_data.as_str(),
                        dec_data
                            .parse::<u32>()
                            .unwrap()
                            .to_chinese_naive(ChineseVariant::Simple, ChineseCase::Lower)
                            .as_str(),
                    );
                }
            }
        }

        self.chntext = Some(chntext);
        self.chntext.as_deref()
    }
}

pub struct Fraction {
    pub fraction: String,
    pub chntext: Option<String>,
}

impl Fraction {
    pub fn new(fraction: String) -> Fraction {
        Self {
            fraction,
            chntext: None,
        }
    }

    pub fn to_chntext(&mut self) -> Option<&str> {
        let mut chntext = self.fraction.clone();
        let re = Regex::new(r"((?P<num>\d+)(((?P<symbol>/)(?P<den>\d+)))?)").unwrap();

        // 进行匹配
        if let Some(captures) = re.captures(&self.fraction) {
            // 提取分子和分母部分
            let num: Option<String> = captures
                .name("num")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let pot: Option<String> = captures
                .name("symbol")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let den: Option<String> = captures
                .name("den")
                .map(|m| m.as_str().parse().unwrap_or_default());

            if let Some(num_data) = num {
                if let Some(den_data) = den {
                    if !num_data.is_empty() && !den_data.is_empty() {
                        chntext = chntext.replace(
                            num_data.as_str(),
                            den_data
                                .parse::<u32>()
                                .unwrap()
                                .to_chinese(
                                    ChineseVariant::Simple,
                                    ChineseCase::Lower,
                                    ChineseCountMethod::TenThousand,
                                )
                                .unwrap()
                                .as_str(),
                        );
                        chntext = chntext.replace(
                            den_data.as_str(),
                            num_data
                                .parse::<u32>()
                                .unwrap()
                                .to_chinese(
                                    ChineseVariant::Simple,
                                    ChineseCase::Lower,
                                    ChineseCountMethod::TenThousand,
                                )
                                .unwrap()
                                .as_str(),
                        );
                    }
                }
            }
            if let Some(point_data) = pot {
                if !point_data.is_empty() {
                    chntext = chntext.replace(point_data.as_str(), FRACTION);
                }
            }
        }

        self.chntext = Some(chntext);
        self.chntext.as_deref()
    }
}
pub struct Percentage {
    pub percent: String,
    pub chntext: Option<String>,
}

impl Percentage {
    pub fn new(percent: String) -> Percentage {
        Self {
            percent,
            chntext: None,
        }
    }

    pub fn to_chntext(&mut self) -> Option<&str> {
        let mut chntext = self.percent.clone();
        let re = Regex::new(r"((?P<num>\d+(\.\d+)?)?(?P<symbol>%))").unwrap();

        // 进行匹配
        if let Some(captures) = re.captures(&self.percent) {
            // 提取数字部分
            let num: Option<String> = captures
                .name("num")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let pre: Option<String> = captures
                .name("symbol")
                .map(|m| m.as_str().parse().unwrap_or_default());

            if let Some(num_data) = num {
                if let Some(pre_data) = pre {
                    if !num_data.is_empty() && !pre_data.is_empty() {
                        chntext = chntext.replace(num_data.as_str(), PERCENT);
                        chntext = chntext.replace(
                            pre_data.as_str(),
                            Digit::new(num_data.as_str().to_string())
                                .to_chntext()
                                .unwrap(),
                        );
                    }
                }
            }
        }

        self.chntext = Some(chntext);
        self.chntext.as_deref()
    }
}

pub struct TelePhone {
    pub telephone: String,
    pub chntext: Option<String>,
}

impl TelePhone {
    pub fn new(telephone: String) -> TelePhone {
        Self {
            telephone,
            chntext: None,
        }
    }

    pub fn to_chntext(&mut self) -> Option<&str> {
        let mut chntext = self.telephone.clone();
        let re =
            Regex::new(r"(((?P<pre>0(10|2[1-3]|[3-9]\d{2}))(?P<symbol>-)?)?(?P<tel>[1-9]\d{6,7}))")
                .unwrap();

        // 进行匹配
        if let Some(captures) = re.captures(&self.telephone) {
            // 提取电话部分
            let symbol: Option<String> = captures
                .name("symbol")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let pre: Option<String> = captures
                .name("pre")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let tel: Option<String> = captures
                .name("tel")
                .map(|m| m.as_str().parse().unwrap_or_default());

            if let Some(symbol_data) = symbol {
                if !symbol_data.is_empty() {
                    chntext = chntext.replace(symbol_data.as_str(), GANG);
                }
            }
            if let Some(pre_data) = pre {
                // prefix 0
                if !pre_data.is_empty() {
                    if pre_data.starts_with('0') {
                        chntext = chntext.replace(
                            pre_data.as_str(),
                            format!(
                                "零{}",
                                pre_data
                                    .parse::<u128>()
                                    .unwrap()
                                    .to_chinese_naive(ChineseVariant::Simple, ChineseCase::Lower)
                                    .as_str()
                            )
                            .as_str(),
                        );
                    } else {
                        chntext = chntext.replace(
                            pre_data.as_str(),
                            pre_data
                                .parse::<u128>()
                                .unwrap()
                                .to_chinese_naive(ChineseVariant::Simple, ChineseCase::Lower)
                                .as_str(),
                        );
                    }
                }
            }
            if let Some(tel_data) = tel {
                if !tel_data.is_empty() {
                    chntext = chntext.replace(
                        tel_data.as_str(),
                        tel_data
                            .parse::<u128>()
                            .unwrap()
                            .to_chinese_naive(ChineseVariant::Simple, ChineseCase::Lower)
                            .as_str(),
                    );
                }
            }
        }

        self.chntext = Some(chntext);
        self.chntext.as_deref()
    }
}

pub struct MobilePhone {
    pub mobilephone: String,
    pub chntext: Option<String>,
}

impl MobilePhone {
    pub fn new(mobilephone: String) -> MobilePhone {
        Self {
            mobilephone,
            chntext: None,
        }
    }

    pub fn to_chntext(&mut self) -> Option<&str> {
        let mut chntext = self.mobilephone.clone();
        let re = Regex::new(
            r"(((?P<symbol>\+)?(?P<pre>86) ?)?(?P<tel>1([38]\d|5[0-35-9]|7[678]|9[89])\d{8}))",
        )
        .unwrap();

        // 进行匹配
        if let Some(captures) = re.captures(&self.mobilephone) {
            // 提取电话部分
            let symbol: Option<String> = captures
                .name("symbol")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let pre: Option<String> = captures
                .name("pre")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let tel: Option<String> = captures
                .name("tel")
                .map(|m| m.as_str().parse().unwrap_or_default());

            if let Some(symbol_data) = symbol {
                if !symbol_data.is_empty() {
                    chntext = chntext.replace(symbol_data.as_str(), PLUS);
                }
            }
            if let Some(pre_data) = pre {
                if !pre_data.is_empty() {
                    chntext = chntext.replace(
                        format!("{} ", pre_data.as_str()).as_str(),
                        pre_data
                            .parse::<u32>()
                            .unwrap()
                            .to_chinese_naive(ChineseVariant::Simple, ChineseCase::Lower)
                            .as_str(),
                    );
                }
            }
            if let Some(tel_data) = tel {
                if !tel_data.is_empty() {
                    chntext = chntext.replace(
                        tel_data.as_str(),
                        tel_data
                            .parse::<u128>()
                            .unwrap()
                            .to_chinese_naive(ChineseVariant::Simple, ChineseCase::Lower)
                            .as_str(),
                    );
                }
            }
        }

        self.chntext = Some(chntext);
        self.chntext.as_deref()
    }
}

pub struct Date {
    pub date: String,
    pub chntext: Option<String>,
}

impl Date {
    pub fn new(date: String) -> Date {
        Self {
            date,
            chntext: None,
        }
    }

    pub fn to_chntext(&mut self) -> Option<&str> {
        let mut chntext = self.date.clone();
        // 定义日期匹配的正则表达式
        let re = Regex::new(r"(((?P<year>([089]\d|(19|20)\d{2}))年)?((?P<month>\d{1,2})月)?((?P<day>\d{1,2})[日号])?)").unwrap();

        // 进行匹配
        if let Some(captures) = re.captures(&self.date) {
            // 提取年、月、日的数字部分
            let year: Option<String> = captures
                .name("year")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let month: Option<String> = captures
                .name("month")
                .map(|m| m.as_str().parse().unwrap_or_default());
            let day: Option<String> = captures
                .name("day")
                .map(|m| m.as_str().parse().unwrap_or_default());

            if let Some(year_data) = year {
                if !year_data.is_empty() {
                    chntext = chntext.replace(
                        year_data.as_str(),
                        year_data
                            .parse::<u32>()
                            .unwrap()
                            .to_chinese_naive(ChineseVariant::Simple, ChineseCase::Lower)
                            .as_str(),
                    );
                }
            }
            if let Some(month_data) = month {
                if !month_data.is_empty() {
                    chntext = chntext.replace(
                        month_data.as_str(),
                        month_data
                            .parse::<u32>()
                            .unwrap()
                            .to_chinese(
                                ChineseVariant::Simple,
                                ChineseCase::Lower,
                                ChineseCountMethod::TenThousand,
                            )
                            .unwrap()
                            .as_str(),
                    );
                }
            }
            if let Some(day_data) = day {
                if !day_data.is_empty() {
                    chntext = chntext.replace(
                        day_data.as_str(),
                        day_data
                            .parse::<u32>()
                            .unwrap()
                            .to_chinese(
                                ChineseVariant::Simple,
                                ChineseCase::Lower,
                                ChineseCountMethod::TenThousand,
                            )
                            .unwrap()
                            .as_str(),
                    );
                }
            }
        }

        self.chntext = Some(chntext);
        self.chntext.as_deref()
    }
}

pub struct Money {
    money: String,
    chntext: Option<String>,
}

impl Money {
    pub fn new(money: String) -> Money {
        Self {
            money,
            chntext: None,
        }
    }

    pub fn to_chntext(&mut self) -> Option<&str> {
        let mut chntext = self.money.clone();
        let re = Regex::new(r"(\d+(\.\d+)?)").unwrap();

        for matcher in re.captures_iter(&self.money) {
            if let Some(matched) = matcher.get(0) {
                if !matched.is_empty() {
                    chntext = chntext.replace(
                        matched.as_str(),
                        Digit::new(matched.as_str().to_string())
                            .to_chntext()
                            .unwrap(),
                    );
                }
            }
        }

        self.chntext = Some(chntext);
        return self.chntext.as_deref();
    }
}

pub struct NSWNormalizer {
    pub raw_text: String,
    pub norm_text: String,
}

impl NSWNormalizer {
    pub fn new(raw_text: &str) -> Self {
        let raw_text = format!("^{}$", raw_text);
        NSWNormalizer {
            raw_text,
            norm_text: String::new(),
        }
    }

    fn _particular(&mut self) {
        let text = &self.raw_text;
        let pattern = Regex::new(r"(([a-zA-Z]+)2([a-zA-Z]+))").unwrap();
        if let Some(matchers) = pattern.captures(text) {
            for matcher in matchers.iter() {
                if let Some(_) = matcher {
                    let replaced = text.replace("2", "图");
                    self.norm_text = replaced;
                }
            }
        }
    }

    pub fn normalize(&mut self) -> &str {
        let mut text = self.raw_text.clone();
        text = text.replace("％", "%");

        // 规范化日期
        let pattern =
            Regex::new(r"((([089]\d|(19|20)\d{2})年)?(\d{1,2}月(\d{1,2}[日号])?))").unwrap();
        for matcher in pattern.captures_iter(&text.clone()) {
            if let Some(match_str) = matcher.get(0) {
                let mut date = Date::new(match_str.as_str().to_string());
                let replaced = text.replace(match_str.as_str(), &date.to_chntext().unwrap());
                text = replaced;
            }
        }

        // 规范化金钱
        let reg = format!(
            r"((\d+(\.\d+)?)[多余几]?{}(\d{}?)?)",
            CURRENCY_UNITS, CURRENCY_UNITS
        );
        let pattern = Regex::new(reg.as_str()).unwrap();
        for matcher in pattern.captures_iter(&text.clone()) {
            if let Some(match_str) = matcher.get(0) {
                let mut money = Money::new(match_str.as_str().to_string());
                let replaced = text.replace(match_str.as_str(), &money.to_chntext().unwrap());
                text = replaced;
            }
        }

        // 规范化固话/手机号码
        // 手机
        // http://www.jihaoba.com/news/show/13680
        // 移动：139、138、137、136、135、134、159、158、157、150、151、152、188、187、182、183、184、178、198
        // 联通：130、131、132、156、155、186、185、176
        // 电信：133、153、189、180、181、177
        let pattern = Regex::new(r"\D((\+?86 ?)?1([38]\d|5[0-35-9]|7[678]|9[89])\d{8})\D").unwrap();
        for matcher in pattern.captures_iter(&text.clone()) {
            if let Some(match_str) = matcher.get(0) {
                let mut mobilephone = MobilePhone::new(match_str.as_str().to_string());
                let replaced = text.replace(match_str.as_str(), &mobilephone.to_chntext().unwrap());
                text = replaced;
            }
        }
        let pattern = Regex::new(r"\D((0(10|2[1-3]|[3-9]\d{2})-?)?[1-9]\d{6,7})\D").unwrap();
        for matcher in pattern.captures_iter(&text.clone()) {
            if let Some(match_str) = matcher.get(0) {
                let mut telphone = TelePhone::new(match_str.as_str().to_string());
                let replaced = text.replace(match_str.as_str(), &telphone.to_chntext().unwrap());
                text = replaced;
            }
        }

        // 规范化分数
        let pattern = Regex::new(r"(\d+/\d+)").unwrap();
        for matcher in pattern.captures_iter(&text.clone()) {
            if let Some(match_str) = matcher.get(0) {
                let mut fraction = Fraction::new(match_str.as_str().to_string());
                let replaced = text.replace(match_str.as_str(), &fraction.to_chntext().unwrap());
                text = replaced;
            }
        }

        // 规范化百分数
        let pattern = Regex::new(r"(\d+(\.\d+)?%)").unwrap();
        for matcher in pattern.captures_iter(&text.clone()) {
            if let Some(match_str) = matcher.get(0) {
                let mut percent = Percentage::new(match_str.as_str().to_string());
                let replaced = text.replace(match_str.as_str(), &percent.to_chntext().unwrap());
                text = replaced;
            }
        }

        // 规范化纯数+量词
        let reg = format!(r"(\d+(\.\d+)?)[多余几]?{}", COM_QUANTIFIERS);
        let pattern = Regex::new(reg.as_str()).unwrap();
        for matcher in pattern.captures_iter(&text.clone()) {
            if let Some(match_str) = matcher.get(0) {
                let replaced = text.replace(
                    match_str.as_str(),
                    Digit::new(match_str.as_str().to_string())
                        .to_chntext()
                        .unwrap(),
                );
                text = replaced;
            }
        }

        // 规范化数字编号
        let pattern = Regex::new(r"(\d{4,32})").unwrap();
        for matcher in pattern.captures_iter(&text.clone()) {
            if let Some(match_str) = matcher.get(0) {
                let replaced = text.replace(
                    match_str.as_str(),
                    match_str
                        .as_str()
                        .parse::<u128>()
                        .unwrap()
                        .to_chinese_naive(ChineseVariant::Simple, ChineseCase::Lower)
                        .as_str(),
                );
                text = replaced;
            }
        }

        // 规范化纯数
        let pattern = Regex::new(r"(\d+(\.\d+)?)").unwrap();
        for matcher in pattern.captures_iter(&text.clone()) {
            if let Some(match_str) = matcher.get(0) {
                let replaced = text.replace(
                    match_str.as_str(),
                    Digit::new(match_str.as_str().to_string())
                        .to_chntext()
                        .unwrap(),
                );
                text = replaced;
            }
        }

        let re_ignore = ['、', '，', '。', '！', '？', '：', '”', '“'];
        // 过滤掉英文标点符号以及空白符
        text = text
            .as_str()
            .trim()
            .chars()
            .filter(|&c| !re_ignore.contains(&c))
            .collect();
        text = text.chars().filter(|&c| !c.is_ascii_punctuation()).collect();

        self.norm_text = text;
        self._particular();

        &self.norm_text.trim_start_matches('^').trim_end_matches('$')
    }
}
// Add implementations for other functions and structs

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_cn_tn_show() {
        print!("chinese_punc_list: {:?} \n", *CHINESE_PUNC_LIST);
        print!("units: {:?} \n", *UNITS);
        print!("units.0: {:?} \n", UNITS.get(&0));
        print!(
            "units.0.0: {:?} \n",
            UNITS
                .get(&0)
                .unwrap()
                .get(1)
                .unwrap()
                .parse::<i32>()
                .unwrap()
        );
        print!("digits: {:?} \n", *DIGITS);
        print!("symbols: {:?} \n", *SYMBOLS);
    }

    #[test]
    fn test_normalize() {
        let texts = vec![
            "固话：0595-23865596或23880880。",
            "手机：+86 19859213959或15659451527或者 +86 18612345678。",
            "分数：32477/76391。",
            "百分数：80.03%。",
            "编号：31520181154418。",
            "纯数：2983.07克或12345.60米。",
            "日期：1999年2月20日或09年3月15号或者3月12日。",
            "金钱：12块5，34.5元，20.1万",
            "特殊：O2O或B2C。",
            "3456万吨",
            "2938个",
            "938",
            "今天吃了115个小笼包231个馒头",
            "有62％的概率",
        ];

        for text in texts {
            let start_time = Local::now();
            let text_norm = NSWNormalizer::new(text).normalize().to_owned();
            let duration = Local::now().signed_duration_since(start_time);
            println!(
                "in: {}\n out: {} cost: {:.2}",
                text,
                text_norm,
                duration.num_milliseconds() as f64 / 1000.0
            );
        }
    }
}

use super::cn_tn::NSWNormalizer;
use lazy_static::lazy_static;
// use pinyin::*;
use pinyin_translator::*;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read};

// Define a regex pattern for Chinese characters
lazy_static! {
    pub static ref ZH_PATTERN: Regex = Regex::new(r"[\u4e00-\u9fa5]").unwrap();
}

// Define a function to check if a string contains Chinese characters
fn is_zh(word: &str) -> bool {
    ZH_PATTERN.is_match(word)
}

// Define the BakerProcessor struct with serde attributes for deserialization
#[derive(Deserialize, Debug)]
pub struct BakerProcessor {
    pinyin_dict: HashMap<String, (String, String)>,
    symbols: Vec<String>,
    speakers_map: HashMap<String, usize>,
    loaded_mapper_path: Option<String>,
    symbol_to_id: HashMap<String, usize>,
    id_to_symbol: HashMap<usize, String>,
    processor_name: Option<String>,
    eos_id: usize,
}

// Define the implementation block for BakerProcessor
impl BakerProcessor {
    // Define the __post_init__ method
    pub fn new() -> Result<Self, Error> {
        // Implement the method
        let mut processor = Self {
            pinyin_dict: HashMap::new(),
            symbols: Vec::new(),
            speakers_map: HashMap::new(),
            loaded_mapper_path: Some("assets/baker_mapper.json".to_string()),
            symbol_to_id: HashMap::new(),
            id_to_symbol: HashMap::new(),
            processor_name: None,
            eos_id: 0,
        };

        processor.load_mapper().unwrap();
        if !processor.setup_eos_token().is_empty() {
            processor.add_symbol(processor.setup_eos_token());
            processor.eos_id = processor.symbol_to_id[&processor.setup_eos_token()];
        }

        Ok(processor)
    }

    // Define the setup_eos_token method
    fn setup_eos_token(&self) -> String {
        // Implement the method
        "eos".to_string() // Replace with actual implementation
    }

    fn get_phoneme_from_char_and_pinyin(&self, chn_char: &str, pinyin: Vec<String>) -> Vec<String> {
        // We do not need #4, use sil to replace it
        let chn_char = chn_char.replace("#4", "");
        let mut result = vec!["sil".to_string()];
        let mut ignored_char = 0;
        for (i, ch) in chn_char.chars().enumerate() {
            if i - ignored_char >= pinyin.len() {
                break;
            }

            if is_zh(ch.to_string().as_str()) {
                if i - ignored_char > 0 && ch.to_string() != "#" {
                    result.push("#0".to_string());
                }
                let mut pinyin_i_clone = pinyin[i - ignored_char].as_str();
                pinyin_i_clone = Box::leak(pinyin_i_clone.replace("ü", "v").into_boxed_str());
                if let Some(tone) = pinyin_i_clone.chars().last() {
                    if !tone.is_ascii_digit() {
                        let new_pinyin = format!("{}5", pinyin_i_clone);
                        pinyin_i_clone = Box::leak(new_pinyin.into_boxed_str());
                    }
                }
                if pinyin_i_clone.trim_end_matches(char::is_numeric) == "n" {
                    let new_pinyin = format!("en{}", pinyin_i_clone.chars().last().unwrap());
                    pinyin_i_clone = Box::leak(new_pinyin.into_boxed_str());
                }
                if let Some(_) = self
                    .pinyin_dict
                    .get(pinyin_i_clone.trim_end_matches(char::is_numeric))
                {
                    if let Some(tone) = pinyin_i_clone.chars().last() {
                        let a = &pinyin_i_clone[..pinyin_i_clone.len() - 1];
                        if let Some((a1, a2)) = self.pinyin_dict.get(a) {
                            result.push(a1.to_string());
                            result.push(format!("{}{}", a2, tone));
                        }
                    }
                } else {
                    if let Some(tone) = pinyin_i_clone.chars().last() {
                        let a = &pinyin_i_clone[..pinyin_i_clone.len() - 2];
                        if let Some((a1, a2)) = self.pinyin_dict.get(a) {
                            result.push(a1.to_string());
                            result.push(format!("{}{}", a2, tone));
                            result.push("er5".to_string());
                        }
                    }
                }
            } else if ch.to_string() == "#" {
                result.push(ch.to_string());
                ignored_char += 1;
            } else {
                ignored_char += 1;
            }
        }

        if result.last() == Some(&"#0".to_string()) {
            // 去掉最后的#0，改为sil
            result.pop();
        }

        if result.last() != Some(&"sil".to_string()) {
            result.push("sil".to_string());
        }

        // assert_eq!(j, pinyin.len());
        result
    }

    fn text_to_phone(&self, text: &str) -> (String, String) {
        let normalized_text = NSWNormalizer::new(text).normalize().to_owned();

        // let pinyin_with_tone = to_pinyin_vec(normalized_text.as_str(), Pinyin::with_tone_num_end);
        let pt = PinyinTranslator::new();
        let pinyin_with_tone = pt.translate_as_slice(normalized_text.clone());
        // println!("\npinyin_with_tone: {:?}", pinyin_with_tone);

        let phonemes = self.get_phoneme_from_char_and_pinyin(&normalized_text, pinyin_with_tone);
        // println!("\nphonemes: {:?}", phonemes);
        let phones = phonemes.join(" ");

        (normalized_text.to_string(), phones)
    }

    pub fn text_to_sequence(&self, text: &str, _inference: bool) -> Vec<i32> {
        let (_, phones) = self.text_to_phone(text);

        let mut sequence: Vec<i32> = phones
            .split_whitespace()
            .map(|symbol| self.symbol_to_id[symbol] as i32)
            .collect();

        // Add eos tokens
        sequence.push(self.eos_id as i32);

        sequence
    }

    fn add_symbol(&mut self, symbol: String) {
        if !self.symbol_to_id.contains_key(&symbol) {
            self.symbols.push(symbol.clone());
            let symbol_id = self.symbols.len();
            self.symbol_to_id.insert(symbol.clone(), symbol_id);
            self.id_to_symbol.insert(symbol_id, symbol);
        }
    }

    // Example method: Load mapper
    fn load_mapper(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(&self.loaded_mapper_path.clone().unwrap())?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        let parsed_data: serde_json::Value = serde_json::from_str(&data)?;

        if let Some(speakers_map) = parsed_data.get("speakers_map") {
            self.speakers_map = serde_json::from_value(speakers_map.clone())?;
        }

        if let Some(symbol_to_id) = parsed_data.get("symbol_to_id") {
            self.symbol_to_id = serde_json::from_value(symbol_to_id.clone())?;
        }

        if let Some(id_to_symbol) = parsed_data.get("id_to_symbol") {
            let id_to_symbol_map: HashMap<String, String> =
                serde_json::from_value(id_to_symbol.clone())?;
            self.id_to_symbol = id_to_symbol_map
                .iter()
                .map(|(k, v)| (k.parse::<usize>().unwrap(), v.clone()))
                .collect();
        }

        if let Some(processor_name) = parsed_data.get("processor_name") {
            self.processor_name = Some(processor_name.as_str().unwrap().to_string());
        }

        if let Some(pinyin_dict) = parsed_data.get("pinyin_dict") {
            self.pinyin_dict = serde_json::from_value(pinyin_dict.clone())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use pinyin::*;

    use super::*;

    #[test]
    fn test_text_cowa() {

        let test_texts: Vec<&str> = vec![
            "接管中",
            "接管",
            "退出接管",
            "停车",
            "解除停车",
            "紧急停止",
            "解除急停",
            "控制器上电成功",
            "传感器上电成功",
            "控制器下电",
            "休眠",
            "调试模式",
            "退出调试模式",
            "电量剩余百分之50",
            "水量不足",
            "控制器上电失败",
            "传感器上电失败",
            "休眠失败",
            "下电失败",
            "倒车，请注意",
            "警告，断连",
            "紧急停车",
            "向左加速",
            "向右加速",
            "车辆加速",
            "向左变道",
            "向右变道",
            "向左行驶",
            "向右行驶",
            "前方有障碍物，停止前进",
            "前方有行人，停止前进",
            "前方被通用障碍删格阻挡，停止前进",
            "红绿灯停车",
            "正在掉头，请注意安全",
            "通过路口",
            "即将到达终点",
            "手动驾驶模式",
            "自动驾驶模式",
            "平行驾驶模式",
            "遥控模式",
            "巡游模式",
            "遍历模式",
            "沿边清扫",
            "垃圾追踪",
            "开启扫盘",
            "关闭扫盘",
            "开始洒水",
            "停止洒水",
            "电量不足",
            "定位模块异常",
            "感知模块异常",
            "占有删格模块异常",
            "底盘模块异常",
            "上装模块异常",
            "控制模块异常",
            "路由模块异常",
            "规划模块异常",
            "车辆受困",
            "电量不足",
            "定位模块异常",
            "感知模块异常",
            "占有删格模块异常",
            "底盘模块异常",
            "上装模块异常",
            "控制模块异常",
            "路由模块异常",
            "规划模块异常",
            "车辆受困",
            "无人参与",
            "与君同行",
        ];

        if let Ok(baker) = BakerProcessor::new() {
            for text in test_texts {
                let (norm, phone) = baker.text_to_phone(text);
                println!("in: {:?}\n out: {:?} {:?}", text, norm, phone);
            }
        } else {
            println!("BakerProcessor::new error");
        }
    }

    #[test]
    fn test_text_to_phone() {
        // let chars = pinyin_translator::vars::CHARS;
        // for char in chars.to_vec().as_slice() {
        //     let char_elements: Vec<&str> = char.split(",").collect();
        //     let key = char_elements.get(0).unwrap();
        //     let value = char_elements.get(1).unwrap();
        //     let fixed = to_pinyin_vec(key, Pinyin::with_tone_num_end);
        //     println!("{:?},", format!("{},{}", key, fixed.join(",")));
        // }

        // let words = pinyin_translator::vars::WORDS;
        // for word in words.to_vec().as_slice() {
        //     let char_elements: Vec<&str> = word.split(",").collect();
        //     let key = char_elements.get(0).unwrap();
        //     let value = char_elements.get(1).unwrap();
        //     let fixed = to_pinyin_vec(key, Pinyin::with_tone);
        //     println!("{:?},", format!("{},{}", key, fixed.join(",")));
        // }

        let test_texts: Vec<&str> = vec![
            "1.我喜欢运动，特别是打篮球。",
            "2.今天晚上我要去看电影。",
            "3.明天是周末，你有什么计划吗？",
            "4.这道菜看起来很美味，你尝过了吗？",
            "5.你在干什么呢？",
            "6.我喜欢旅行，最想去的地方是日本。",
            "7.你看上去很累，需要休息一下吗？",
            "8.这个手机的功能真的很强大。",
            "9.我想去海边散步，感受海风。",
            "10.你最喜欢的动物是什么？我最喜欢的是熊猫。",
            "11.我喜欢听音乐，特别是流行音乐。",
            "12.晚上我要去做瑜伽，你要一起来吗？",
            "13.你在担心什么呢？说出来我可以帮你分担一下。",
            "14.这本书很有趣，我已经看了一半了。",
            "15.明天我要去爬山，你要一起来吗？",
            "16.你喜欢喝咖啡还是茶？我最喜欢的是拿铁咖啡。",
            "17.这个问题很难，我需要一些时间来思考。",
            "18.我们在公园见面吧，一起去散步。",
            "19.你最喜欢的季节是什么？我最喜欢的是春天。",
            "20.我喜欢吃冰淇淋，特别是香草味的。",
            "21.你看上去很高兴，有什么好事吗？",
            "22.这个城市很美丽，有很多值得一看的地方。",
            "23.我想喝一杯橙汁，你去帮我买吧。",
            "24.你最喜欢的运动是什么？我最喜欢的是游泳。",
            "25.晚上我要去做饭，你要一起来吗？",
            "26.你在干什么呢？看上去很忙的样子。",
            "27.这道数学题很难，我需要一些帮助才能解决。",
            "28.明天我要去参加一个派对，你要一起来吗？",
            "29.你喜欢看书吗？我最近对哲学很感兴趣。",
            "30.这个电视的功能真的很强大，可以连接网络。",
            "31.我喜欢吃巧克力，特别是黑巧克力。",
            "32.你看上去很生气，发生了什么事情吗？",
            "33.这个故事很有趣，我已经迫不及待地想继续看下去了。",
            "34.我想去看一场音乐会，你去过吗？",
            "35.你最喜欢的颜色是什么？我最喜欢的是蓝色。",
            "36.我喜欢跳舞，特别是街舞。",
            "37.晚上我要去看电影，你要一起来吗？",
            "38.你在想什么呢？看上去很专注的样子。",
            "39.这道菜很辣，你要尝试一下吗？",
            "40.明天我要去旅行，你想一起去吗？",
            "41.你喜欢逛街吗？我最喜欢去购物中心了。",
            "42.这个相机的功能真的很强大，可以拍出高清照片。",
            "43.我喜欢吃水果，特别是草莓和芒果。",
            "44.你看上去很开心，有什么好事要分享吗？",
            "45.这个游戏很好玩，我已经沉迷其中了。",
            "46.我们在咖啡店见面吧，一起去品尝一下新品咖啡。",
            "47.你喜欢哪种类型的电影？我最喜欢的是科幻片。",
            "48.这件衣服很漂亮，很适合你穿。",
            "49.我喜欢踢足球，特别是和朋友们一起踢。",
            "50.晚上我要去做瑜伽，你要一起来吗？",
            "51.你在干什么呢？看上去很专注的样子。",
            "52.这道物理题很难，我需要一些帮助才能解决。",
            "53.明天我要去参加一个比赛，你要一起来吗？",
            "54.你喜欢听广播剧吗？我最近对一部广播剧很感兴趣。",
            "55.这个电脑的功能真的很强大，可以运行各种软件。",
            "56.我喜欢吃糖果，特别是软糖和巧克力糖。",
            "57.你看上去很惊讶，发生了什么事情吗？",
            "58.这个故事很感人，我已经被深深地打动了。",
            "59.我想去看一场话剧表演，你去过吗？",
            "60.你最喜欢的节日是什么？我最喜欢的是春节和中秋节。",
            "61.我喜欢打网球，特别是和朋友一起双打。",
            "62.晚上我要去做饭，你要一起来吗？我们可以一起做一顿丰盛的晚餐。",
            "63.你在担心什么呢？说出来我可以帮你分担一下压力。",
            "64.这道菜很美味，我已经迫不及待地想品尝一下了。",
            "65.明天我要去爬山看日出你要一起去吗？",
            "66.我喜欢画画，特别是风景画和人物画。",
            "67.今天晚上我要去看一场音乐会，你要一起来吗？",
            "68.明天是晴天，我们可以去野餐或者烧烤。",
            "69.这道历史题目很难，我需要一些时间来查找资料。",
            "70.你在干什么呢？看上去很忙碌的样子。",
            "71.这个手机的屏幕真的很大，看电影和玩游戏都很过瘾。",
            "72.我喜欢喝果汁，特别是橙汁和西瓜汁。",
            "73.你看上去很开心，发生了什么好事吗？",
            "74.这个城市有很多历史遗迹和文化景点，我们可以一起去探索一下。",
            "75.我想喝一杯奶茶，你去帮我买吧。",
            "76.你最喜欢的食物是什么？我最喜欢的是寿司和拉面。",
            "77.晚上我要去做一些家务，你要一起来帮忙吗？",
            "78.你在想什么呢？看上去很专注的样子。",
            "79.这道数学题很有趣，我已经找到解决方法了。",
            "80.明天我要去参加一个展览开幕式，你要一起来吗？",
            "81.在静谧的夜晚，星星闪烁着微弱的光芒，它们仿佛是宇宙的点点明珠，为黑夜增添了神秘的色彩。",
            "82.沐浴在晨光中的大地，微风轻拂着树叶，唤醒了沉睡的自然。这是一个新的开始，充满了希望和活力。",
            "83.时间如梭，岁月匆匆，回首过往，我们曾经历过多少风雨，也收获了无数美好的瞬间。",
            "84.科技的发展让世界变得更加紧密，信息传递的速度之快让人感叹不已。这是一个信息爆炸的时代。",
            "85.在茫茫的大海上，一艘小船孤独地航行着，船上的水手望着远方，心中充满了对未知的向往。",
            "86.夜幕降临，城市的霓虹灯光熠熠闪烁，构成了一幅美丽的夜景画卷，为疲惫的人们带来了一丝慰藉。",
            "87.在雨后的清晨，空气中弥漫着泥土的芬芳，树叶上的露珠闪烁着晶莹的光辉，生命在这一刻变得格外宁静。",
            "88.探索未知的道路，就像漫步在星辰大海中，每一步都可能带来新的奇迹，每一颗星星都是一个未知的秘密。",
            "89.音乐是心灵的良药，它能够穿越时空，触动人们内心最深处的情感，让人沉浸在美妙的旋律中。",
            "90.站在山巅，俯瞰着群山连绵，云雾缭绕。大自然的壮丽让人感叹生命的伟大和宇宙的神秘。",
            "The quick brown fox jumps over the lazy dog.",
            "How much wood would a woodchuck chuck if a woodchuck could chuck wood?",
            "Peter Piper picked a peck of pickled peppers.",
            "She sells seashells by the seashore.",
            "今天天气很好！",
            "2023年9月16日！",
            "一共是945个！",
            "654.245！",
            "分数是654/245！",
            "加载到95%！",
            "倒车，请注意",
            "进入调试模式。",
            "车辆已到达目的地，请下车！",
        ];

        if let Ok(baker) = BakerProcessor::new() {
            for text in test_texts {
                let (norm, phone) = baker.text_to_phone(text);
                println!("in: {:?}\n out: {:?} {:?}", text, norm, phone);
            }
        } else {
            println!("BakerProcessor::new error");
        }
    }
}

use serde::Deserialize;
use std::collections::HashMap;
use std::io::{Error, Read};
use regex::Regex;
use lazy_static::lazy_static;
use std::fs::File;

const ABBREVIATIONS: [(&str, &str); 18] = [
    ("\\b(mrs)\\.", "misess"),
    ("\\b(mr)\\.", "mister"),
    ("\\b(dr)\\.", "doctor"),
    ("\\b(st)\\.", "saint"),
    ("\\b(co)\\.", "company"),
    ("\\b(jr)\\.", "junior"),
    ("\\b(maj)\\.", "major"),
    ("\\b(gen)\\.", "general"),
    ("\\b(drs)\\.", "doctors"),
    ("\\b(rev)\\.", "reverend"),
    ("\\b(lt)\\.", "lieutenant"),
    ("\\b(hon)\\.", "honorable"),
    ("\\b(sgt)\\.", "sergeant"),
    ("\\b(capt)\\.", "captain"),
    ("\\b(esq)\\.", "esquire"),
    ("\\b(ltd)\\.", "limited"),
    ("\\b(col)\\.", "colonel"),
    ("\\b(ft)\\.", "fort"),
];

fn expand_abbreviations(text: &str) -> String {
    let mut expanded_text = String::from(text);

    for (pattern, replacement) in ABBREVIATIONS.iter() {
        let regex = Regex::new(pattern).unwrap();
        expanded_text = regex.replace_all(&expanded_text, *replacement).to_string();
    }

    expanded_text
}

fn number_to_words(n: f64) -> String {
    if n == 0.0 {
        return "zero".to_string();
    }

    let mut result = String::new();

    // 整数部分
    let integer_part = n.trunc() as i64;
    if integer_part != 0 {
        result += &number_to_words_integer(integer_part);
    }

    // 小数部分
    let decimal_part = n.fract();
    if decimal_part != 0.0 {
        result += " point ";
        result += &number_to_words_decimal(decimal_part, 2);
    }

    result
}

fn number_to_words_integer(n: i64) -> String {
    let mut result = String::new();
    let groups = ["", "thousand", "million", "billion", "trillion", "quadrillion", "quintillion"];

    let mut remaining = n.abs();
    let mut group_index = 0;

    while remaining > 0 {
        let chunk = remaining % 1000;
        if chunk > 0 {
            if !result.is_empty() {
                result = format!("{} {}", number_chunk_to_words(chunk), groups[group_index]) + result.as_str();
            } else {
                result = number_chunk_to_words(chunk);
            }
        }
        remaining /= 1000;
        group_index += 1;
    }

    if n < 0 {
        result = format!("negative {}", result);
    }

    result
}

fn number_chunk_to_words(chunk: i64) -> String {
    let ones = ["", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];
    let teens = ["", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen"];
    let tens = ["", "ten", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];

    let mut result = String::new();

    let hundreds_digit = chunk / 100;
    let tens_digit = (chunk / 10) % 10;
    let ones_digit = chunk % 10;

    if hundreds_digit > 0 {
        result += format!("{} hundred", ones[hundreds_digit as usize]).as_str();
        if tens_digit > 0 || ones_digit > 0 {
            result += " and ";
        }
    }

    if tens_digit == 1 && ones_digit > 0 {
        result += teens[ones_digit as usize];
    } else {
        if tens_digit > 0 {
            result += tens[tens_digit as usize];
            if ones_digit > 0 {
                result += "-";
            }
        }
        if ones_digit > 0 {
            result += ones[ones_digit as usize];
        }
    }

    result
}

fn number_to_words_decimal(mut n: f64, precision: usize) -> String {
    let ones = ["", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];

    let mut result = String::new();
    let mut prec = precision;

    // 四舍五入到指定精度
    let multiplier = 10.0_f64.powi(prec as i32);
    n = (n * multiplier).round() / multiplier;

    while n > 0.0 && prec > 0 {
        n *= 10.0;
        let digit = n.floor() as i64;
        result += ones[digit as usize];
        n -= digit as f64;
        prec -= 1;
    }

    result
}

fn expand_numbers(text: &str) -> String {
    let expanded_text = NUMBER_RE.replace_all(text, |captures: &regex::Captures| {
        let number_str = captures.get(0).unwrap().as_str();
        let number = number_str.parse::<f64>().unwrap();
        let words = number_to_words(number);
        words
    });

    expanded_text.to_string()
}

// Define a regex pattern for Chinese characters
lazy_static! {
    pub static ref CURLY_RE: Regex = Regex::new(r"(.*?)\{(.+?)\}(.*)").unwrap();
    pub static ref NUMBER_RE: Regex = Regex::new(r"\b\d+\.\d+\b|\b\d+\b").unwrap();
}

// Define the BakerProcessor struct with serde attributes for deserialization
#[derive(Deserialize, Debug)]
pub struct LJSpeechProcessor {
    cleaner_names: Option<String>,
    symbols: Vec<String>,
    speakers_map: HashMap<String, usize>,
    loaded_mapper_path: Option<String>,
    symbol_to_id: HashMap<String, usize>,
    id_to_symbol: HashMap<usize, String>,
    processor_name: Option<String>,
    eos_id: usize,
}

// Define the implementation block for BakerProcessor
impl LJSpeechProcessor {
    // Define the __post_init__ method
    pub fn new() -> Result<Self, Error>{
        // Implement the method
        let mut processor = Self {
            cleaner_names: Some(String::new()),
            symbols: Vec::new(),
            speakers_map: HashMap::new(),
            loaded_mapper_path: Some("assets/ljspeech_mapper.json".to_string()),
            symbol_to_id: HashMap::new(),
            id_to_symbol: HashMap::new(),
            processor_name: None,
            eos_id: 0
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

    pub fn text_to_sequence(&self, text: &str, _inference: bool) -> Vec<i32> {
        let mut sequence = vec![];

        // Check for curly braces and treat their contents as ARPAbet.
        let mut text = text;
        while !text.is_empty() {
            if let Some(captures) = CURLY_RE.captures(text) {
                sequence.extend_from_slice(&self.symbols_to_sequence(
                    self.clean_text(captures.get(1).map_or("", |m| m.as_str()), &self.cleaner_names),
                ));
                sequence.extend_from_slice(&self.arpabet_to_sequence(captures.get(2).map_or("", |m| m.as_str())));
                text = captures.get(3).map_or("", |m| m.as_str());
            } else {
                sequence.extend_from_slice(&self.symbols_to_sequence(
                    self.clean_text(text, &self.cleaner_names),
                ));
                break;
            }
        }

        // Add EOS tokens
        sequence.push(self.eos_id as i32);
        sequence
    }
    
    fn clean_text(&self, text: &str, _cleaner_names: &Option<String>) -> String {
        let text = expand_abbreviations(&text.to_lowercase());
        let text = expand_numbers(&text);
        text.to_owned()
    }
    
    fn symbols_to_sequence(&self, symbols: String) -> Vec<i32> {
        symbols
            .chars()
            .filter(|&s| self.should_keep_symbol(&s.to_string()))
            .filter_map(|s| self.symbol_to_id.get(&s.to_string()).map(|&id| id as i32))
            .collect()
    }
    
    fn arpabet_to_sequence(&self, text: &str) -> Vec<i32> {
        self.symbols_to_sequence(format!("@{}", text).split_whitespace().collect::<String>())
    }
    
    fn should_keep_symbol(&self, s: &str) -> bool {
        s != "_" && s != "~" && self.symbol_to_id.contains_key(s)
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
            let id_to_symbol_map: HashMap<String, String> = serde_json::from_value(id_to_symbol.clone())?;
            self.id_to_symbol = id_to_symbol_map
                .iter()
                .map(|(k, v)| (k.parse::<usize>().unwrap(), v.clone()))
                .collect();
        }

        if let Some(processor_name) = parsed_data.get("processor_name") {
            self.processor_name = Some(processor_name.as_str().unwrap().to_string());
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        if let Ok(ljspeech) = LJSpeechProcessor::new() {
            let text = "The price is $123.45 and the quantity is 678.9";
            let sequence = ljspeech.clean_text(text, &ljspeech.cleaner_names);
            println!("in: {:?}\nout: {:?}", text, sequence);
        } else {
            println!("LJSpeechProcessor::new error");
        }
    }

    #[test]
    fn test_text_to_sequence() {
        if let Ok(ljspeech) = LJSpeechProcessor::new() {
            let text = "What day is today?";
            let sequence = ljspeech.text_to_sequence(text, true);
            println!("in: {:?}\nout: {:?}", text, sequence);

            let text = "The price is $123.45 and the quantity is 678.9";
            let sequence = ljspeech.text_to_sequence(text, true);
            println!("in: {:?}\nout: {:?}", text, sequence);
        } else {
            println!("LJSpeechProcessor::new error");
        }
    }

}

use {
    crate::constants::Constants, grep_matcher::Matcher, grep_regex::RegexMatcher, grep_searcher::{sinks::UTF8, Searcher}, regex::Regex, std::collections::HashSet, wabt::wasm2wat
};

pub fn extract_constants_from_wasm(wasm: &Vec<u8>) -> Result<Constants, ()> {
    let wat_code = wasm2wat(wasm).unwrap();
    let matcher = RegexMatcher::new(r"i(32|64).const (-?[0-9]+)").unwrap();
    let mut matches: HashSet<(String,String)> = HashSet::new();
    let _ = Searcher::new()
        .search_slice(
            &matcher,
            wat_code.as_bytes(),
            UTF8(|_lnum, line| {
                if let Some(mymatch) = matcher.find(line.as_bytes()).into() {
                    let cap = mymatch.unwrap().unwrap();
                    let re = Regex::new(r"i(32|64)\.const\s+(-?\d+)").unwrap();
                    if let Some(caps) = re.captures(&line[cap]) {
                        let const_type = caps.get(1).map_or("", |m| m.as_str());
                        let const_value = caps.get(2).map_or("", |m| m.as_str());
                        matches.insert((const_type.to_string(), const_value.to_string()));
                    }
                }
                Ok(true)
            }),
        );
    
    let mut constants = Constants::new();
    for (_const_type, const_value) in matches {
        constants.u8_constants.insert(const_value.parse().unwrap_or(0));
        constants.u16_constants.insert(const_value.parse().unwrap_or(0));
        constants.u32_constants.insert(const_value.parse().unwrap_or(0));
        constants.u64_constants.insert(const_value.parse().unwrap_or(0));
        constants.u128_constants.insert(const_value.parse().unwrap_or(0));
        constants.i8_constants.insert(const_value.parse().unwrap_or(0));
        constants.i16_constants.insert(const_value.parse().unwrap_or(0));
        constants.i32_constants.insert(const_value.parse().unwrap_or(0));
        constants.i64_constants.insert(const_value.parse().unwrap_or(0));
        constants.i128_constants.insert(const_value.parse().unwrap_or(0));
        // match const_type.as_str() {
        //     "32" => {
        //         let value = const_value.parse::<i32>().unwrap();
        //         constants.i32_constants.insert(value);
        //     }
        //     "64" => {
        //         let value = const_value.parse::<i64>().unwrap();
        //         constants.i64_constants.insert(value);
        //     }
        //     _ => {}
        // }
    }

    Ok(constants)
}
use crate::utils::Result;

fn hex2u32(hex_str: &str) -> Result<u32>{
    let clean_str = hex_str.trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(clean_str,16).map_err(|e|e.to_string())
}
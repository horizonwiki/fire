#[derive(Clone)]
pub struct Theme {
    pub colors: [String; 4],
}

#[derive(Copy, Clone, PartialEq)]
pub enum ColorMode {
    Theme,
    Rainbow,
}

impl Theme {
    
    pub const CHARS: [char; 10] = [' ', '.', ':', '^', '*', 'x', 's', 'S', '#', '$'];
    
    pub fn std() -> Self {
        Theme { colors: [
            "38;2;255;0;0".into(),
            "38;2;255;85;0".into(),
            "38;2;255;170;0".into(),
            "38;2;255;255;0".into(),
        ]}
    }
    
    pub fn ice() -> Self {
        Theme { colors: [
            "38;2;173;216;230".into(),
            "38;2;135;206;250".into(),
            "38;2;0;191;255".into(),
            "38;2;240;248;255".into(),
        ]}
    }

    pub fn classic() -> Self {
        Theme { colors: [
            "38;2;20;20;20".into(),
            "38;2;220;50;47".into(),
            "38;2;255;200;40".into(),
            "38;2;70;130;180".into(),
        ]}
    }
    
    pub fn pink() -> Self {
        Theme { colors: [
            "38;2;255;105;180".into(),
            "38;2;255;182;193".into(),
            "38;2;255;240;245".into(),
            "38;2;255;255;255".into(),
        ]}
    }

    pub fn blue() -> Self {
        Theme { colors: [
            "38;2;10;15;28".into(),
            "38;2;0;95;135".into(),
            "38;2;0;175;175".into(),
            "38;2;51;225;255".into(),
        ]}
    }

    pub fn forest() -> Self {
        Theme { colors: [
            "38;2;0;40;0".into(),      
            "38;2;34;139;34".into(),  
            "38;2;50;205;50".into(),  
            "38;2;172;255;47".into(), 
        ]}
    }

    pub fn magma() -> Self {
        Theme { colors: [
            "38;2;40;0;0".into(),      
            "38;2;120;0;0".into(),     
            "38;2;255;69;0".into(),    
            "38;2;255;140;0".into(),   
        ]}
    }

    pub fn solar() -> Self {
        Theme { colors: [
            "38;2;139;69;19".into(),  
            "38;2;255;165;0".into(),   
            "38;2;255;215;0".into(),   
            "38;2;255;255;224".into(),
        ]}
    }

    pub fn plasma() -> Self {
        Theme { colors: [
            "38;2;10;0;30".into(),      
            "38;2;75;0;130".into(),    
            "38;2;0;0;255".into(),     
            "38;2;100;149;237".into(),  
        ]}
    }

    pub fn sulfur() -> Self {
        Theme { colors: [
            "38;2;0;20;50".into(),   
            "38;2;65;105;225".into(),
            "38;2;0;191;255".into(),  
            "38;2;175;238;238".into(), 
        ]}
    }

    pub fn emerald() -> Self {
        Theme { colors: [
            "38;2;0;30;10".into(),
            "38;2;0;128;0".into(), 
            "38;2;0;255;127".into(),   
            "38;2;152;251;152".into(),  
        ]}
    }

    pub fn crimson() -> Self {
        Theme { colors: [
            "38;2;60;0;0".into(),  
            "38;2;178;34;34".into(),  
            "38;2;220;20;60".into(),  
            "38;2;255;100;100".into(), 
        ]}
    }

    pub fn ghost() -> Self {
        Theme { colors: [
            "38;2;80;0;80".into(), 
            "38;2;150;50;250".into(),
            "38;2;200;130;255".into(),  
            "38;2;240;220;255".into(),  
        ]}
    }

    pub fn gold() -> Self {
        Theme { colors: [
            "38;2;101;67;33".into(),  
            "38;2;184;134;11".into(), 
            "38;2;218;165;32".into(),  
            "38;2;255;255;0".into(),  
        ]}
    }

    pub fn ash() -> Self {
        Theme { colors: [
            "38;2;30;30;30".into(), 
            "38;2;80;80;80".into(), 
            "38;2;160;160;160".into(), 
            "38;2;240;240;240".into(),
        ]}
    }

    pub fn copper() -> Self {
        Theme { colors: [
            "38;2;0;40;40".into(),  
            "38;2;0;128;128".into(), 
            "38;2;64;224;208".into(), 
            "38;2;224;255;255".into(), 
        ]}
    }

    pub fn nebula() -> Self {
        Theme { colors: [
            "38;2;25;25;112".into(), 
            "38;2;75;0;130".into(),  
            "38;2;219;112;147".into(),
            "38;2;255;182;193".into(), 
        ]}
    }

    pub fn ember() -> Self {
        Theme { colors: [
            "38;2;60;20;0".into(),  
            "38;2;139;0;0".into(),  
            "38;2;204;85;0".into(),  
            "38;2;255;140;0".into(), 
        ]}
    }

}

fn rgb_to_ansi_str(rgb: (u8, u8, u8)) -> String {
    format!("38;2;{};{};{}", rgb.0, rgb.1, rgb.2)
}

pub fn parse_custom_theme(input: &str) -> Option<Theme> {
    // custom:
    let parts: Vec<&str> = input.split('.').collect();
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }

    let mut rgb_list: Vec<(u8, u8, u8)> = Vec::new();
    for part in &parts {
        match parse_hex_color(part.trim()) {
            Some(c) => rgb_list.push(c),
            None => return None,
        }
    }

    while rgb_list.len() < 4 {
        let last = *rgb_list.last().unwrap();
        rgb_list.push(last);
    }

    Some(Theme {
        colors: [
            rgb_to_ansi_str(rgb_list[0]),
            rgb_to_ansi_str(rgb_list[1]),
            rgb_to_ansi_str(rgb_list[2]),
            rgb_to_ansi_str(rgb_list[3]),
        ],
    })
}

fn parse_hex_color(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}

pub fn hue_to_color_bytes(hue: f32, heat: usize) -> [u8; 3] {
    let v = match heat {
        0..=4   => 0.4,
        5..=9   => 0.6,
        10..=15 => 0.85,
        _       => 1.0,
    };

    // groups adjacent pixels of the same color [4]
    let hue = (hue / 4.0).round() * 4.0;

    let h = hue / 60.0;
    let i = h.floor() as u32;
    let f = h - h.floor();
    let p = 0.0f32; 
    let q = v * (1.0 - f);
    let t = v * f;

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
}
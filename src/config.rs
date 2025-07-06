use crate::keys::{find_key_by_name, KeyDef};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RemapConfig {
    pub from: KeyDef,
    pub to_when_alone: KeyDef,
    pub to_with_other: KeyDef,
}

#[derive(Debug)]
pub struct Config {
    pub remaps: Vec<RemapConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            remaps: Vec::new(),
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, String> {
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Cannot open configuration file '{}': {}", path.as_ref().display(), e))?;
    
    parse_config(&content)
}

fn parse_config(content: &str) -> Result<Config, String> {
    let mut config = Config::default();
    let mut current_remap: Option<RemapConfigBuilder> = None;
    
    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;
        let line = line.trim();
        
        // 跳过空行和注释
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // 解析 key=value 格式
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("Config error (line {}): expected key=value", line_num));
        }
        
        let key = parts[0].trim();
        let value = parts[1].trim();
        
        match key {
            "remap_key" => {
                // 如果有未完成的remap，检查是否完整
                if let Some(builder) = current_remap.take() {
                    if !builder.is_complete() {
                        return Err(format!(
                            "Config error (line {}): Incomplete remapping. Each remap needs remap_key, when_alone and with_other before another remap_key.",
                            line_num
                        ));
                    }
                    config.remaps.push(builder.build()?);
                }
                
                let key_def = find_key_by_name(value)
                    .ok_or_else(|| format!("Config error (line {}): invalid key name '{}'", line_num, value))?;
                
                current_remap = Some(RemapConfigBuilder::new(key_def));
            }
            "when_alone" => {
                let key_def = find_key_by_name(value)
                    .ok_or_else(|| format!("Config error (line {}): invalid key name '{}'", line_num, value))?;
                
                if let Some(ref mut builder) = current_remap {
                    builder.when_alone = Some(key_def);
                } else {
                    return Err(format!("Config error (line {}): when_alone must come after remap_key", line_num));
                }
            }
            "with_other" => {
                let key_def = find_key_by_name(value)
                    .ok_or_else(|| format!("Config error (line {}): invalid key name '{}'", line_num, value))?;
                
                if let Some(ref mut builder) = current_remap {
                    builder.with_other = Some(key_def);
                } else {
                    return Err(format!("Config error (line {}): with_other must come after remap_key", line_num));
                }
            }
            _ => {
                // 忽略其他设置（如debug等）
                continue;
            }
        }
        
        // 检查当前remap是否完整，如果是则添加到配置中
        if let Some(ref builder) = current_remap {
            if builder.is_complete() {
                config.remaps.push(current_remap.take().unwrap().build()?);
            }
        }
    }
    
    // 处理最后一个remap
    if let Some(builder) = current_remap {
        if builder.is_complete() {
            config.remaps.push(builder.build()?);
        } else {
            return Err("Config error: Incomplete remapping at end of file".to_string());
        }
    }
    
    Ok(config)
}

#[derive(Debug)]
struct RemapConfigBuilder {
    from: KeyDef,
    when_alone: Option<KeyDef>,
    with_other: Option<KeyDef>,
}

impl RemapConfigBuilder {
    fn new(from: KeyDef) -> Self {
        Self {
            from,
            when_alone: None,
            with_other: None,
        }
    }
    
    fn is_complete(&self) -> bool {
        self.when_alone.is_some() && self.with_other.is_some()
    }
    
    fn build(self) -> Result<RemapConfig, String> {
        Ok(RemapConfig {
            from: self.from,
            to_when_alone: self.when_alone.ok_or("Missing when_alone")?,
            to_with_other: self.with_other.ok_or("Missing with_other")?,
        })
    }
}

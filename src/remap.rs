use crate::config::RemapConfig;
use crate::input::{send_input, Direction};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Idle,
    HeldDownAlone,
    HeldDownWithOther,
}

#[derive(Debug)]
pub struct Remap {
    pub config: RemapConfig,
    pub state: State,
}

impl Remap {
    pub fn new(config: RemapConfig) -> Self {
        Self {
            config,
            state: State::Idle,
        }
    }
}

pub struct RemapManager {
    remaps: HashMap<u32, Remap>,
}

impl RemapManager {
    pub fn new(configs: Vec<RemapConfig>) -> Self {
        let mut remaps = HashMap::new();
        
        for config in configs {
            let virt_code = config.from.virt_code;
            remaps.insert(virt_code, Remap::new(config));
        }
        
        Self { remaps }
    }
    
    pub fn handle_input(&mut self, virt_code: u32, direction: Direction, is_injected: bool) -> bool {
        if is_injected {
            self.event_other_input()
        } else if self.remaps.contains_key(&virt_code) {
            // 处理重映射的键
            match direction {
                Direction::Down => self.handle_remapped_key_down(virt_code),
                Direction::Up => self.handle_remapped_key_up(virt_code),
            }
        } else {
            self.event_other_input()
        }
    }
    
    fn handle_remapped_key_down(&mut self, virt_code: u32) -> bool {
        if let Some(remap) = self.remaps.get_mut(&virt_code) {
            if remap.state == State::Idle {
                remap.state = State::HeldDownAlone;
            }
        }
        true // 阻止原始输入
    }
    
    fn handle_remapped_key_up(&mut self, virt_code: u32) -> bool {
        if let Some(remap) = self.remaps.get_mut(&virt_code) {
            match remap.state {
                State::HeldDownWithOther => {
                    remap.state = State::Idle;
                    let key_def = remap.config.to_with_other;
                    let _ = send_input(&key_def, Direction::Up);
                }
                _ => {
                    remap.state = State::Idle;
                    let key_def = remap.config.to_when_alone;
                    // 发送单独按键的按下和释放
                    let _ = send_input(&key_def, Direction::Down);
                    let _ = send_input(&key_def, Direction::Up);
                }
            }
        }
        true // 阻止原始输入
    }
    
    fn event_other_input(&mut self) -> bool {
        // 收集需要更新的键
        let mut keys_to_update = Vec::new();
        
        for (virt_code, remap) in &self.remaps {
            if remap.state == State::HeldDownAlone {
                keys_to_update.push((*virt_code, remap.config.to_with_other));
            }
        }
        
        // 更新状态并发送输入
        for (virt_code, key_def) in keys_to_update {
            if let Some(remap) = self.remaps.get_mut(&virt_code) {
                remap.state = State::HeldDownWithOther;
                let _ = send_input(&key_def, Direction::Down);
            }
        }
        
        false // 不阻止其他输入
    }
}

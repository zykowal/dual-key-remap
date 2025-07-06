use crate::config::RemapConfig;
use crate::input::{send_input, Direction};
use crate::keys::{friendly_virt_code_name, KeyDef};
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
    debug: bool,
    log_counter: u32,
    log_indent_level: u32,
}

impl RemapManager {
    pub fn new(configs: Vec<RemapConfig>, debug: bool) -> Self {
        let mut remaps = HashMap::new();
        
        for config in configs {
            let virt_code = config.from.virt_code;
            remaps.insert(virt_code, Remap::new(config));
        }
        
        Self {
            remaps,
            debug,
            log_counter: 1,
            log_indent_level: 0,
        }
    }
    
    pub fn is_debug(&self) -> bool {
        self.debug
    }
    
    pub fn handle_input(&mut self, scan_code: u32, virt_code: u32, direction: Direction, is_injected: bool) -> bool {
        self.log_handle_input_start(scan_code, virt_code, direction, is_injected);
        
        let block_input = if is_injected {
            self.event_other_input()
        } else if self.remaps.contains_key(&virt_code) {
            // 处理重映射的键
            match direction {
                Direction::Down => self.handle_remapped_key_down(virt_code),
                Direction::Up => self.handle_remapped_key_up(virt_code),
            }
        } else {
            self.event_other_input()
        };
        
        self.log_handle_input_end(scan_code, virt_code, direction, is_injected, block_input);
        block_input
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
                    self.send_key_def_input("with_other", &key_def, Direction::Up);
                }
                _ => {
                    remap.state = State::Idle;
                    let key_def = remap.config.to_when_alone;
                    // 发送单独按键的按下和释放
                    self.send_key_def_input("when_alone", &key_def, Direction::Down);
                    self.send_key_def_input("when_alone", &key_def, Direction::Up);
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
                self.send_key_def_input("with_other", &key_def, Direction::Down);
            }
        }
        
        false // 不阻止其他输入
    }
    
    fn send_key_def_input(&mut self, input_name: &str, key_def: &KeyDef, direction: Direction) {
        self.log_send_input(input_name, key_def, direction);
        if let Err(e) = send_input(key_def, direction) {
            eprintln!("Failed to send input: {}", e);
        }
    }
    
    // 调试日志方法
    fn print_log_prefix(&mut self) {
        if self.debug {
            print!("\n{:03}. ", self.log_counter);
            self.log_counter += 1;
            for _ in 0..self.log_indent_level {
                print!("\t");
            }
        }
    }
    
    fn log_handle_input_start(&mut self, scan_code: u32, virt_code: u32, direction: Direction, is_injected: bool) {
        if !self.debug {
            return;
        }
        
        self.print_log_prefix();
        println!(
            "[{}] {} {} (scan:0x{:02x} virt:0x{:02x})",
            if is_injected { "output" } else { "input" },
            friendly_virt_code_name(virt_code),
            direction.as_str(),
            scan_code,
            virt_code
        );
        self.log_indent_level += 1;
    }
    
    fn log_handle_input_end(&mut self, _scan_code: u32, virt_code: u32, direction: Direction, _is_injected: bool, block_input: bool) {
        if !self.debug {
            return;
        }
        
        self.log_indent_level -= 1;
        if block_input {
            self.print_log_prefix();
            println!(
                "#blocked-input# {} {}",
                friendly_virt_code_name(virt_code),
                direction.as_str()
            );
        }
    }
    
    fn log_send_input(&mut self, remap_name: &str, key_def: &KeyDef, direction: Direction) {
        if !self.debug {
            return;
        }
        
        self.print_log_prefix();
        println!(
            "(sending:{}) {} {}",
            remap_name,
            key_def.name,
            direction.as_str()
        );
    }
}

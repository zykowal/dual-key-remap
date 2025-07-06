use crate::keys::KeyDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

pub fn send_input(key_def: &KeyDef, direction: Direction) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        
        let mut input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(key_def.virt_code as u16),
                    wScan: key_def.scan_code as u16,
                    dwFlags: {
                        let mut flags = KEYBD_EVENT_FLAGS(0);
                        
                        if direction == Direction::Up {
                            flags |= KEYEVENTF_KEYUP;
                        }
                        
                        if (key_def.scan_code >> 8) == 0xE0 {
                            flags |= KEYEVENTF_EXTENDEDKEY;
                        }
                        
                        flags
                    },
                    time: 0,
                    dwExtraInfo: 0xFFC3CED7, // 标识这是我们注入的按键
                },
            },
        };

        unsafe {
            let result = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            if result == 0 {
                Err("Failed to send input".to_string())
            } else {
                Ok(())
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // 在非Windows系统上，只是打印模拟信息
        let direction_str = match direction {
            Direction::Up => "UP",
            Direction::Down => "DOWN",
        };
        println!("Simulating key input: {} {}", key_def.name, direction_str);
        Ok(())
    }
}

// 鼠标虚拟键码，用于处理鼠标输入
pub const MOUSE_DUMMY_VK: u32 = 0xFF;

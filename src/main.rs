mod config;
mod input;
mod keys;
mod remap;

use config::load_config;
use input::{Direction, MOUSE_DUMMY_VK};
use remap::RemapManager;
use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

// 由于我们在macOS上，这里只是一个演示版本
// 实际的Windows API调用需要在Windows环境中编译和运行

// 全局状态
static REMAP_MANAGER: Mutex<Option<RemapManager>> = Mutex::new(None);

fn get_config_path() -> Result<PathBuf, String> {
    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    let exe_dir = exe_path.parent()
        .ok_or("Failed to get executable directory")?;
    
    Ok(exe_dir.join("config.txt"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Dual Key Remap - Rust Version");
    println!("==============================");
    
    // 加载配置
    let config_path = get_config_path()?;
    let config = match load_config(&config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Error loading config: {}", e);
            println!("Make sure config.txt exists in the same directory as the executable.");
            println!("\nPress Enter to exit...");
            let _ = std::io::stdin().read_line(&mut String::new());
            return Ok(());
        }
    };
    
    println!("Configuration loaded successfully!");
    println!("Debug mode: {}", config.debug);
    println!("Number of remaps: {}", config.remaps.len());
    
    for (i, remap) in config.remaps.iter().enumerate() {
        println!("Remap {}: {} -> {} (alone) / {} (with other)", 
                 i + 1,
                 remap.from.name,
                 remap.to_when_alone.name,
                 remap.to_with_other.name);
    }
    
    // 检查环境变量中的DEBUG设置
    let debug = config.debug || env::var("DEBUG").is_ok();
    
    // 创建重映射管理器
    let manager = RemapManager::new(config.remaps, debug);
    *REMAP_MANAGER.lock().unwrap() = Some(manager);
    
    println!("\nNote: This is a demonstration version running on macOS.");
    println!("The actual key remapping functionality requires Windows APIs");
    println!("and should be compiled and run on Windows.");
    
    #[cfg(target_os = "windows")]
    {
        println!("\nStarting Windows key remapping...");
        // 这里会包含实际的Windows API调用
        windows_main()?;
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        println!("\nSimulating key remapping behavior...");
        println!("Press Enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
    
    Ok(())
}

#[cfg(target_os = "windows")]
fn windows_main() -> Result<(), Box<dyn std::error::Error>> {
    use windows::core::*;
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Console::*;
    use windows::Win32::System::LibraryLoader::*;
    use windows::Win32::System::Threading::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    use windows::Win32::UI::WindowsAndMessaging::*;
    
    static INJECTED_KEY_ID: usize = 0xFFC3CED7;
    static mut KEYBOARD_HOOK: HHOOK = HHOOK(0);
    static mut MOUSE_HOOK: HHOOK = HHOOK(0);
    
    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code == HC_ACTION as i32 {
            let kb_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            let direction = match wparam.0 as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => Direction::Down,
                WM_KEYUP | WM_SYSKEYUP => Direction::Up,
                _ => return CallNextHookExW(KEYBOARD_HOOK, code, wparam, lparam),
            };
            
            let is_injected = kb_struct.dwExtraInfo == INJECTED_KEY_ID;
            
            if let Ok(mut manager_guard) = REMAP_MANAGER.lock() {
                if let Some(ref mut manager) = *manager_guard {
                    let block_input = manager.handle_input(
                        kb_struct.scanCode,
                        kb_struct.vkCode,
                        direction,
                        is_injected,
                    );
                    
                    if block_input {
                        return LRESULT(1);
                    }
                }
            }
        }
        
        CallNextHookExW(KEYBOARD_HOOK, code, wparam, lparam)
    }
    
    unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code == HC_ACTION as i32 {
            match wparam.0 as u32 {
                WM_MOUSEWHEEL | WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN 
                | WM_XBUTTONDOWN | WM_NCXBUTTONDOWN => {
                    if let Ok(mut manager_guard) = REMAP_MANAGER.lock() {
                        if let Some(ref mut manager) = *manager_guard {
                            let block_input = manager.handle_input(0, MOUSE_DUMMY_VK, Direction::Down, false);
                            
                            if block_input {
                                return LRESULT(1);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        CallNextHookExW(MOUSE_HOOK, code, wparam, lparam)
    }
    
    // 检查单实例
    unsafe {
        let mutex_name = w!("dual-key-remap.single-instance");
        let mutex = CreateMutexW(None, true, mutex_name)?;
        
        if GetLastError() == ERROR_ALREADY_EXISTS {
            println!("dual-key-remap.exe is already running!");
            println!("\nPress Enter to exit...");
            let _ = std::io::stdin().read_line(&mut String::new());
            return Ok(());
        }
        
        std::mem::forget(mutex);
    }
    
    // 提升进程和线程优先级
    unsafe {
        SetPriorityClass(GetCurrentProcess(), HIGH_PRIORITY_CLASS)?;
        SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST)?;
    }
    
    // 设置钩子
    unsafe {
        KEYBOARD_HOOK = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_proc),
            GetModuleHandleW(None)?,
            0,
        )?;
        
        MOUSE_HOOK = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(mouse_proc),
            GetModuleHandleW(None)?,
            0,
        )?;
    }
    
    if KEYBOARD_HOOK.is_invalid() || MOUSE_HOOK.is_invalid() {
        println!("Failed to set keyboard or mouse hooks, aborting.");
        println!("\nPress Enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
        return Ok(());
    }
    
    // 检查是否为调试模式
    let debug = env::var("DEBUG").is_ok() || {
        if let Ok(manager_guard) = REMAP_MANAGER.lock() {
            manager_guard.as_ref().map(|m| m.is_debug()).unwrap_or(false)
        } else {
            false
        }
    };
    
    if debug {
        println!("-- DEBUG MODE --");
    } else {
        // 隐藏控制台窗口
        FreeConsole()?;
    }
    
    // 消息循环
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    
    Ok(())
}

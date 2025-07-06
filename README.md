# Dual Key Remap (Rust版本)

这是一个用Rust重写的双键映射程序，可以将一个键映射为两种不同的行为：
- 单独按下时发送一个键（如Escape）
- 与其他键组合按下时作为修饰键（如Ctrl）

## 功能特性

- **双键映射**：一个键可以有两种不同的行为
- **低延迟**：使用高优先级进程和线程减少输入延迟
- **单实例运行**：防止多个实例同时运行
- **调试模式**：可以查看详细的按键处理日志
- **配置文件**：通过简单的文本文件配置映射规则
- **跨平台编译**：支持Windows和其他平台的编译（功能仅在Windows上可用）

## 系统要求

- **Windows 10/11**（实际功能需要）
- **Rust 1.70+**（编译需要）
- **管理员权限**（运行时需要，用于设置全局钩子）

## 编译和运行

### 方法1：使用构建脚本（推荐）

**Windows:**
```cmd
build.bat
```

**Linux/macOS:**
```bash
./build.sh
```

### 方法2：手动编译

```bash
# 编译发布版本
cargo build --release

# 复制配置文件
cp config.txt target/release/

# 运行程序
./target/release/dual-key-remap      # Linux/macOS
target\release\dual-key-remap.exe    # Windows
```

### 方法3：直接运行（开发模式）

```bash
# 复制配置文件到调试目录
cp config.txt target/debug/

# 运行
cargo run
```

## 配置

程序会在可执行文件同目录下查找 `config.txt` 文件。

### 配置文件格式

```ini
# 注释以 # 开头
debug=false

# 每个映射需要三行配置
remap_key=CAPSLOCK    # 要重映射的键
when_alone=ESCAPE     # 单独按下时发送的键
with_other=CTRL       # 与其他键组合时的行为
```

### 支持的键名

程序支持大多数常用键，包括：

**修饰键：**
- `CTRL`, `LCTRL`, `RCTRL`
- `SHIFT`, `LSHIFT`, `RSHIFT`
- `ALT`, `LALT`, `RALT`

**特殊键：**
- `CAPSLOCK`, `ESCAPE`, `SPACE`, `ENTER`, `TAB`
- `BACKSPACE`, `DELETE`, `HOME`, `END`
- `PAGEUP`, `PAGEDOWN`

**方向键：**
- `UP`, `DOWN`, `LEFT`, `RIGHT`

**字母键：**
- `A`-`Z`

**数字键：**
- `0`-`9`

**功能键：**
- `F1`-`F12`

### 配置示例

#### 将Caps Lock映射为Escape/Ctrl（经典配置）
```ini
remap_key=CAPSLOCK
when_alone=ESCAPE
with_other=CTRL
```

#### 将右Alt映射为Escape/Alt
```ini
remap_key=RALT
when_alone=ESCAPE
with_other=ALT
```

#### 多个映射
```ini
debug=false

# 第一个映射
remap_key=CAPSLOCK
when_alone=ESCAPE
with_other=CTRL

# 第二个映射
remap_key=RALT
when_alone=BACKSPACE
with_other=ALT
```

## 调试

要启用调试模式，可以：

1. **配置文件方式：** 在 `config.txt` 中设置 `debug=true`
2. **环境变量方式：** 设置环境变量 `DEBUG=1`

调试模式会在控制台显示详细的按键处理日志，包括：
- 输入事件的详细信息
- 键状态转换
- 输出事件的生成

## 工作原理

1. **钩子监听**：程序使用Windows低级键盘和鼠标钩子监听所有输入
2. **状态管理**：为每个重映射的键维护状态（空闲、单独按下、与其他键组合）
3. **输入注入**：根据键的状态和行为注入相应的键盘输入
4. **循环检测**：通过标记注入的输入避免无限循环

### 状态转换图

```
空闲 (Idle)
    ↓ 按下重映射键
单独按下 (HeldDownAlone)
    ↓ 按下其他键        ↓ 释放重映射键
与其他键组合 (HeldDownWithOther)    发送单独按键
    ↓ 释放重映射键        ↓
发送修饰键释放 → 空闲 ← 空闲
```

## 与C版本的对比

### 优势

- **内存安全**：Rust的所有权系统避免了内存泄漏和悬空指针
- **错误处理**：更好的错误处理和报告机制
- **模块化**：代码组织更清晰，模块化程度更高
- **类型安全**：编译时类型检查避免了许多运行时错误
- **现代语法**：使用Rust的现代语法特性，代码更简洁易读
- **测试支持**：内置单元测试支持
- **包管理**：使用Cargo进行依赖管理和构建

### 性能

- **编译优化**：Rust编译器的优化能力强，生成高效的机器码
- **零成本抽象**：高级特性不会带来运行时开销
- **内存效率**：无垃圾回收器，内存使用更可预测

## 注意事项

- **管理员权限**：程序需要管理员权限才能设置全局钩子
- **防病毒软件**：某些防病毒软件可能会误报，需要添加白名单
- **单实例运行**：同时只能运行一个实例
- **Windows专用**：核心功能仅在Windows上可用
- **输入阻断**：程序会阻止原始按键的输入，只发送重映射后的按键

## 故障排除

### 程序无法启动
- 确保以管理员权限运行
- 检查 `config.txt` 文件是否存在且格式正确
- 查看错误信息并修正配置

### 按键映射不工作
- 确认配置文件中的键名正确
- 启用调试模式查看按键处理日志
- 检查是否有其他键盘软件冲突
- 确保程序以管理员权限运行

### 程序崩溃
- 检查Windows事件日志
- 尝试在调试模式下运行
- 确保使用的键名在支持列表中
- 检查配置文件语法是否正确

### 性能问题
- 关闭调试模式以提高性能
- 检查系统资源使用情况
- 确保没有与其他输入软件冲突

## 开发

### 项目结构

```
src/
├── main.rs          # 主程序入口
├── config.rs        # 配置文件解析
├── keys.rs          # 键定义和查找
├── input.rs         # 输入处理和注入
└── remap.rs         # 重映射逻辑和状态管理
```

### 构建配置

- **目标平台**：Windows x86_64
- **Rust版本**：2021 edition
- **主要依赖**：windows crate (Windows API绑定)

### 测试

```bash
# 运行单元测试
cargo test

# 运行特定测试
cargo test config::tests

# 运行测试并显示输出
cargo test -- --nocapture
```

## 许可证

本项目基于原C版本重写，保持相同的开源精神。

## 贡献

欢迎提交Issue和Pull Request来改进这个项目。

## 致谢

感谢原C版本的开发者提供了优秀的设计思路和实现参考。

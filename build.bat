@echo off
REM Dual Key Remap - Build Script for Windows

echo Building Dual Key Remap (Rust Version)...

REM 清理之前的构建
cargo clean

REM 构建发布版本
echo Building release version...
cargo build --release

REM 检查构建是否成功
if %ERRORLEVEL% EQU 0 (
    echo Build successful!
    
    REM 复制配置文件到发布目录
    copy config.txt target\release\
    
    echo Executable created at: target\release\dual-key-remap.exe
    echo Configuration file copied to: target\release\config.txt
    echo.
    echo To run the program:
    echo   target\release\dual-key-remap.exe
    echo.
    echo Note: The program requires administrator privileges to work properly.
) else (
    echo Build failed!
    pause
    exit /b 1
)

pause

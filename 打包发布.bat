@echo off
setlocal
title Retake 打包发布
cd /d "%~dp0"

rem ---- 本机工具链 PATH（Rust、ffmpeg；node/npm 已在系统 PATH）----
set "PATH=%USERPROFILE%\.cargo\bin;D:\ffmpeg\bin;%PATH%"

if not exist node_modules (
  echo [首次运行] 正在安装前端依赖，请稍候...
  call npm install --no-audit --no-fund
  if errorlevel 1 (
    echo [错误] 依赖安装失败
    pause
    exit /b 1
  )
)

echo 正在打包：前端 + release 编译 + NSIS 安装包，约需数分钟...
call npm run tauri build

echo.
echo 产物位置：
echo   程序：  src-tauri\target\release\retake.exe
echo   安装包：src-tauri\target\release\bundle\nsis\
pause

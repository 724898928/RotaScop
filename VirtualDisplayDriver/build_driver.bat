@echo off
REM ============================================
REM Build script for Virtual Display Driver
REM Requires: Visual Studio + WDK installed
REM ============================================

echo RotaScope Virtual Display Driver Builder
echo ========================================

REM Check for Visual Studio
if not defined VSINSTALLDIR (
    if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat"
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat"
    ) else (
        echo Error: Visual Studio 2022 not found.
        echo Please run this script from a Developer Command Prompt.
        exit /b 1
    )
)

REM Check for WDK
if not defined WDK_PATH (
    if exist "C:\Program Files (x86)\Windows Kits\10\Include" (
        set "WDK_PATH=C:\Program Files (x86)\Windows Kits\10"
    ) else (
        echo Error: Windows WDK not found.
        echo Please install Windows Driver Kit (WDK).
        exit /b 1
    )
)

echo Visual Studio: Found
echo WDK: %WDK_PATH%
echo.

REM Create build directory
if not exist build mkdir build
pushd build

REM Configure with CMake
echo Configuring...
cmake .. -G "Visual Studio 17 2022" -A x64 -DWDK_PATH="%WDK_PATH%"

if %ERRORLEVEL% neq 0 (
    echo CMake configuration failed.
    popd
    exit /b 1
)

REM Build
echo Building...
cmake --build . --config Release

if %ERRORLEVEL% neq 0 (
    echo Build failed.
    popd
    exit /b 1
)

echo.
echo Build successful!
echo Driver output: build\Release\VirtualDisplayDriver.sys
echo.
echo To install:
echo   1. Enable testsigning: bcdedit /set testsigning on
echo   2. Install driver:      devcon install VirtualDisplay.inf Root\VirtualDisplay
echo   3. Reboot

popd

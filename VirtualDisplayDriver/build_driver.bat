@echo off
REM ============================================
REM Build script for RotaScope Virtual Display Driver
REM Requires: Visual Studio 2022 + WDK 10 installed
REM ============================================

setlocal enabledelayedexpansion

echo RotaScope Virtual Display Driver Builder
echo ========================================

REM Detect Visual Studio
if not defined VSINSTALLDIR (
    if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat"
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat"
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat"
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2019\Professional\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2019\Professional\VC\Auxiliary\Build\vcvars64.bat"
    ) else if exist "C:\Program Files\Microsoft Visual Studio\2019\Enterprise\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\2019\Enterprise\VC\Auxiliary\Build\vcvars64.bat"
    ) else (
        echo Error: Visual Studio 2019/2022 not found.
        echo Please run this script from a Developer Command Prompt.
        exit /b 1
    )
)

echo Visual Studio: Found

REM Detect WDK
if not defined WDK_PATH (
    if exist "C:\Program Files (x86)\Windows Kits\10\Include" (
        set "WDK_PATH=C:\Program Files (x86)\Windows Kits\10"
    ) else (
        echo Warning: Standard WDK path not found.
        echo Attempting fallback detection...
        
        for /d %%i in ("C:\Program Files (x86)\Windows Kits\10\Include\10.*") do (
            set "WDK_PATH=C:\Program Files (x86)\Windows Kits\10"
        )
        
        if not defined WDK_PATH (
            echo Error: Windows WDK not found.
            echo Please install Windows Driver Kit (WDK 10).
            exit /b 1
        )
    )
)

echo WDK: %WDK_PATH%

REM Detect WDK version
set WDK_VERSION=
for /d %%i in ("%WDK_PATH%\Include\10.*") do (
    set "WDK_VERSION=%%~nxi"
)

if defined WDK_VERSION (
    echo WDK Version: %WDK_VERSION%
) else (
    echo Warning: Could not determine WDK version. Using default.
    set "WDK_VERSION=10.0.22621.0"
)

echo.

REM Create build directory
if not exist build mkdir build
pushd build

REM Configure with CMake
echo Configuring...
echo WDK_PATH=%WDK_PATH%  WDK_VERSION=%WDK_VERSION%

cmake .. ^
    -G "Visual Studio 17 2022" ^
    -A x64 ^
    -DWDK_PATH="%WDK_PATH%" ^
    -DWDK_VERSION="%WDK_VERSION%" ^
    -DBUILD_COMPANION=ON

if %ERRORLEVEL% neq 0 (
    echo.
    echo CMake configuration failed.
    echo.
    echo Possible issues:
    echo   - WDK not installed at %WDK_PATH%
    echo   - WDK version %WDK_VERSION% not found
    echo   - Visual Studio 2022 not properly set up
    popd
    exit /b 1
)

REM Build driver
echo.
echo Building driver (Release)...
cmake --build . --config Release --target VirtualDisplayDriver

if %ERRORLEVEL% neq 0 (
    echo.
    echo Driver build failed.
    popd
    exit /b 1
)

REM Build companion service
echo.
echo Building companion service (Release)...
cmake --build . --config Release --target RotaScopeCompanion

if %ERRORLEVEL% neq 0 (
    echo.
    echo Warning: Companion service build failed (non-critical)
)

echo.
echo ========================================
echo Build successful!
echo ========================================
echo.
echo Output files:
echo   Driver:          build\Release\VirtualDisplayDriver.sys
echo   Companion:       build\Release\RotaScopeCompanion.exe
echo.
echo To install the driver:
echo   1. Enable testsigning:
echo        bcdedit /set testsigning on
echo        (reboot)
echo.
echo   2. Install driver:
echo        devcon install VirtualDisplay.inf Root\RotaScope_VirtualDisplay
echo        (or use pnputil)
echo.
echo   3. Start companion service:
echo        RotaScopeCompanion.exe
echo.
echo To uninstall:
echo        devcon remove Root\RotaScope_VirtualDisplay
echo.

popd
endlocal

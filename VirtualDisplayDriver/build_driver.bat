@echo off
setlocal enabledelayedexpansion

echo RotaScope Virtual Display Driver Builder
echo ========================================

REM --- Step 1: Find Visual Studio ---
if defined VSINSTALLDIR goto FOUND_VS

set "VS0=D:\Program Files"
set "VS1=C:\Program Files\Microsoft Visual Studio\2022\Community"
set "VS2=C:\Program Files\Microsoft Visual Studio\2022\Professional"
set "VS3=C:\Program Files\Microsoft Visual Studio\2022\Enterprise"

set "VS_FOUND="
if exist "!VS0!\VC\Auxiliary\Build\vcvars64.bat" ( call "!VS0!\VC\Auxiliary\Build\vcvars64.bat" & set "VS_FOUND=1" )
if not defined VS_FOUND if exist "!VS1!\VC\Auxiliary\Build\vcvars64.bat" ( call "!VS1!\VC\Auxiliary\Build\vcvars64.bat" & set "VS_FOUND=1" )
if not defined VS_FOUND if exist "!VS2!\VC\Auxiliary\Build\vcvars64.bat" ( call "!VS2!\VC\Auxiliary\Build\vcvars64.bat" & set "VS_FOUND=1" )
if not defined VS_FOUND if exist "!VS3!\VC\Auxiliary\Build\vcvars64.bat" ( call "!VS3!\VC\Auxiliary\Build\vcvars64.bat" & set "VS_FOUND=1" )

if not defined VS_FOUND (
    echo Error: Visual Studio 2022 not found.
    exit /b 1
)

:FOUND_VS
echo Visual Studio: Found

REM --- Step 2: Find WDK (use short 8.3 path) ---
set "WDK_PATH=C:\PROGRA~2\WI3CF2~1\10"
if not exist "!WDK_PATH!\Include" (
    echo Error: WDK not found at expected short path.
    echo Please ensure WDK is installed at the standard location.
    exit /b 1
)
echo WDK Path: !WDK_PATH!

REM --- Step 3: Detect WDK version ---
set "WDK_VERSION="
for /d %%d in (!WDK_PATH!\Include\10.*) do (
    set "WDK_VERSION=%%~nxd"
)
if not defined WDK_VERSION set "WDK_VERSION=10.0.26100.0"
echo WDK Version: !WDK_VERSION!

REM --- Step 4: Detect KMDF version ---
set "KMDF_VERSION="
for /d %%v in (!WDK_PATH!\Include\wdf\kmdf\*.*) do (
    set "KMDF_VERSION=%%~nxv"
)
if not defined KMDF_VERSION set "KMDF_VERSION=1.35"
echo KMDF Version: !KMDF_VERSION!

REM --- Step 5: Verify required files ---
set "ALL_OK=1"

if exist "!WDK_PATH!\Include\!WDK_VERSION!\km\ntddk.h" (echo   ntddk.h: OK) else (echo   ntddk.h: MISSING & set "ALL_OK=0")
if exist "!WDK_PATH!\Include\!WDK_VERSION!\km\dispmprt.h" (echo   dispmprt.h: OK) else (echo   dispmprt.h: MISSING & set "ALL_OK=0")
if exist "!WDK_PATH!\Include\!WDK_VERSION!\um\iddcx" (echo   IddCx headers: OK) else (echo   IddCx headers: MISSING & set "ALL_OK=0")
if exist "!WDK_PATH!\Include\wdf\kmdf\!KMDF_VERSION!\wdf.h" (echo   wdf.h: OK) else (echo   wdf.h: MISSING & set "ALL_OK=0")
if exist "!WDK_PATH!\Lib\wdf\kmdf\x64\!KMDF_VERSION!\wdfdriverentry.lib" (echo   wdfdriverentry.lib: OK) else (echo   wdfdriverentry.lib: MISSING & set "ALL_OK=0")
if exist "!WDK_PATH!\Lib\!WDK_VERSION!\km\x64\ntoskrnl.lib" (echo   ntoskrnl.lib: OK) else (echo   ntoskrnl.lib: MISSING & set "ALL_OK=0")

if "!ALL_OK!" neq "1" (
    echo.
    echo ERROR: Required WDK files missing. Install WDK 10.0.26100 or later.
    exit /b 1
)
echo.

REM --- Step 6: Build ---
if not exist build mkdir build
pushd build

echo Configuring CMake...
cmake .. -G "Visual Studio 17 2022" -A x64 -DWDK_PATH="!WDK_PATH!" -DWDK_VERSION="!WDK_VERSION!" -DKMDF_VERSION="!KMDF_VERSION!" -DBUILD_COMPANION=ON
echo ^<Project^>^<PropertyGroup^>^<WindowsTargetPlatformVersion^>10.0.26100.0^</WindowsTargetPlatformVersion^>^</PropertyGroup^>^</Project^> > Directory.Build.props
if !ERRORLEVEL! neq 0 (
    echo CMake configure failed.
    popd
    exit /b 1
)

echo.
echo Building driver...
cmake --build . --config Release --target VirtualDisplayDriver
if !ERRORLEVEL! neq 0 (
    echo Driver build failed.
    popd
    exit /b 1
)

echo.
echo Building companion...
cmake --build . --config Release --target RotaScopeCompanion 2>nul

popd
echo.
echo ========================================
echo Build successful!
echo   Driver:    build\Release\VirtualDisplayDriver.sys
echo   Companion: build\Release\RotaScopeCompanion.exe
echo ========================================
endlocal

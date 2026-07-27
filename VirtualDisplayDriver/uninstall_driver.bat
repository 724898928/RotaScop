@echo off
setlocal

echo ============================================
echo  RotaScope Virtual Display Driver Uninstaller
echo ============================================
echo.

:: Check administrator privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] This script requires administrator privileges.
    pause
    exit /b 1
)

echo Uninstalling driver...
pnputil /delete-driver oem*.inf /uninstall /force 2>nul

if %errorLevel% equ 0 (
    echo Driver uninstalled successfully.
) else (
    echo [INFO] No driver found to uninstall, or already removed.
)

echo.
echo To disable test signing (requires reboot):
echo   bcdedit /set testsigning off
echo.
pause

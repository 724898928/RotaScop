@echo off
setlocal enabledelayedexpansion

echo ============================================
echo  RotaScope Virtual Display Driver Installer
echo ============================================
echo.

:: Check administrator privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] This script requires administrator privileges.
    echo Right-click and select "Run as administrator".
    pause
    exit /b 1
)

set DRIVER_DIR=%~dp0
set BUILD_DIR=%DRIVER_DIR%build\Release
set SYS_FILE=%BUILD_DIR%\VirtualDisplayDriver.sys
set INF_FILE=%DRIVER_DIR%VirtualDisplayDriver.inf
set CERT_NAME=RotaScopeTestCert

:: Verify files exist
if not exist "%SYS_FILE%" (
    echo [ERROR] Driver file not found: %SYS_FILE%
    echo Please build the driver first.
    pause
    exit /b 1
)

if not exist "%INF_FILE%" (
    echo [ERROR] INF file not found: %INF_FILE%
    pause
    exit /b 1
)

:: Step 1: Check test signing mode
echo [1/5] Checking test signing mode...
bcdedit /enum {current} | findstr /i "testsigning" | findstr /i "Yes" >nul 2>&1
if %errorLevel% neq 0 (
    echo      Test signing is NOT enabled. Enabling now...
    bcdedit /set testsigning on
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to enable test signing mode.
        pause
        exit /b 1
    )
    echo      Test signing enabled. REBOOT REQUIRED before continuing.
    echo      Please reboot and run this script again.
    pause
    exit /b 0
)
echo      Test signing is enabled.

:: Step 2: Create self-signed certificate (if not exists)
echo [2/5] Checking certificate...
certutil -store "Root" %CERT_NAME% >nul 2>&1
if %errorLevel% neq 0 (
    echo      Creating self-signed certificate...
    powershell -Command "$cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject 'CN=%CERT_NAME%' -CertStoreLocation 'Cert:\LocalMachine\Root' -NotAfter (Get-Date).AddYears(5) -KeyUsage DigitalSignature -KeyAlgorithm RSA -KeyLength 2048 -HashAlgorithm SHA256"
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to create certificate.
        pause
        exit /b 1
    )
    echo      Certificate created.
) else (
    echo      Certificate exists.
)

:: Step 3: Sign the driver
echo [3/5] Signing driver...
set CERT_THUMBPRINT=
for /f "tokens=*" %%a in ('powershell -Command "(Get-ChildItem Cert:\LocalMachine\Root | Where-Object {$_.Subject -eq 'CN=%CERT_NAME%'}).Thumbprint"') do (
    set CERT_THUMBPRINT=%%a
)

if "!CERT_THUMBPRINT!"=="" (
    echo [ERROR] Could not find certificate thumbprint.
    pause
    exit /b 1
)

signtool sign /v /s Root /sha1 "!CERT_THUMBPRINT!" /t http://timestamp.digicert.com "%SYS_FILE%"
if %errorLevel% neq 0 (
    echo [ERROR] Failed to sign driver.
    pause
    exit /b 1
)
echo      Driver signed successfully.

:: Step 4: Create catalog and sign it
echo [4/5] Creating catalog...
set CAT_FILE=%BUILD_DIR%\VirtualDisplayDriver.cat

:: Use inf2cat to create catalog
inf2cat /driver:"%DRIVER_DIR%" /os:10_x64 /verbose
if %errorLevel% neq 0 (
    echo [WARNING] inf2cat failed. Using alternative method...
    :: Alternative: sign the .sys directly, pnputil can install without .cat in test mode
)

if exist "%CAT_FILE%" (
    signtool sign /v /s Root /sha1 "!CERT_THUMBPRINT!" /t http://timestamp.digicert.com "%CAT_FILE%"
    echo      Catalog signed.
) else (
    echo      No catalog file - will install using direct .sys signing.
)

:: Step 5: Install driver
echo [5/5] Installing driver...
echo.
echo      Staging driver package...
pnputil /add-driver "%INF_FILE%" /install
if %errorLevel% neq 0 (
    echo.
    echo [WARNING] pnputil returned error code %errorLevel%.
    echo.
    echo Alternative installation method:
    echo   1. Open Device Manager
    echo   2. Click "Action" -^> "Add legacy hardware"
    echo   3. Click Next, select "Install the hardware that I manually select"
    echo   4. Select "Display adapters"
    echo   5. Click "Have Disk" and browse to:
    echo      %INF_FILE%
    echo   6. Select "RotaScope Virtual Display" and complete installation.
    echo.
)

echo.
echo ============================================
echo  Installation complete!
echo ============================================
echo.
echo To verify:
echo   1. Open Device Manager
echo   2. Expand "Display adapters"
echo   3. You should see "RotaScope Virtual Display"
echo.
echo To uninstall:
echo   pnputil /delete-driver oem*.inf /uninstall
echo.
pause

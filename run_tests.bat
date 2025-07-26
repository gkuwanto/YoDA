@echo off
REM YoDA API Test Runner for Windows
REM This script runs the Postman test collection using Newman CLI

echo.
echo ========================================
echo    YoDA API Test Suite Runner
echo ========================================
echo.

REM Check if Node.js is installed
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Node.js is not installed or not in PATH
    echo Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)

REM Check if Newman is installed
newman --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Installing Newman CLI...
    npm install -g newman
    if %errorlevel% neq 0 (
        echo ERROR: Failed to install Newman
        pause
        exit /b 1
    )
)

echo Checking if YoDA server is running...
curl -s http://localhost:3000/health >nul 2>&1
if %errorlevel% neq 0 (
    echo WARNING: YoDA server doesn't seem to be running on localhost:3000
    echo Please start your YoDA server first with: cargo run
    echo.
    set /p continue="Continue anyway? (y/N): "
    if /i not "%continue%"=="y" (
        echo Test run cancelled.
        pause
        exit /b 1
    )
)

echo.
echo Starting API tests...
echo.

REM Run the test collection
newman run YoDA_API_Tests.postman_collection.json ^
    --env-var "base_url=http://localhost:3000" ^
    --reporters cli,json ^
    --reporter-json-export test-results.json ^
    --iteration-count 1 ^
    --delay-request 1000

if %errorlevel% equ 0 (
    echo.
    echo ========================================
    echo    All tests completed successfully!
    echo ========================================
    echo.
    echo Test results saved to: test-results.json
) else (
    echo.
    echo ========================================
    echo    Some tests failed!
    echo ========================================
    echo.
    echo Check test-results.json for details
)

echo.
pause 
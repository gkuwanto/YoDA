# YoDA API Test Runner for Windows PowerShell
# This script runs the Postman test collection using Newman CLI

param(
    [string]$BaseUrl = "http://localhost:3000",
    [int]$Iterations = 1,
    [int]$Delay = 1000,
    [switch]$LoadTest,
    [switch]$Verbose,
    [switch]$Help
)

if ($Help) {
    Write-Host @"
YoDA API Test Suite Runner

Usage:
    .\run_tests.ps1 [options]

Options:
    -BaseUrl <url>        Base URL for API (default: http://localhost:3000)
    -Iterations <number>   Number of test iterations (default: 1)
    -Delay <ms>           Delay between requests in ms (default: 1000)
    -LoadTest             Run load test with 100 iterations
    -Verbose              Show detailed output
    -Help                 Show this help message

Examples:
    .\run_tests.ps1                           # Run basic tests
    .\run_tests.ps1 -LoadTest                 # Run load test
    .\run_tests.ps1 -BaseUrl "https://api.example.com"  # Custom URL
    .\run_tests.ps1 -Iterations 10 -Delay 500 # Custom settings
"@
    exit 0
}

# Set execution policy if needed
if ($Verbose) {
    Write-Host "Setting execution policy..." -ForegroundColor Yellow
}

# Check if Node.js is installed
try {
    $nodeVersion = node --version 2>$null
    if ($Verbose) {
        Write-Host "Node.js version: $nodeVersion" -ForegroundColor Green
    }
} catch {
    Write-Host "ERROR: Node.js is not installed or not in PATH" -ForegroundColor Red
    Write-Host "Please install Node.js from https://nodejs.org/" -ForegroundColor Yellow
    exit 1
}

# Check if Newman is installed
try {
    $newmanVersion = newman --version 2>$null
    if ($Verbose) {
        Write-Host "Newman version: $newmanVersion" -ForegroundColor Green
    }
} catch {
    Write-Host "Installing Newman CLI..." -ForegroundColor Yellow
    npm install -g newman
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Failed to install Newman" -ForegroundColor Red
        exit 1
    }
}

# Check if YoDA server is running
Write-Host "Checking if YoDA server is running..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "$BaseUrl/health" -Method GET -TimeoutSec 5
    if ($Verbose) {
        Write-Host "Server is running: $($response.StatusCode)" -ForegroundColor Green
    }
} catch {
    Write-Host "WARNING: YoDA server doesn't seem to be running on $BaseUrl" -ForegroundColor Yellow
    Write-Host "Please start your YoDA server first with: cargo run" -ForegroundColor Yellow
    $continue = Read-Host "Continue anyway? (y/N)"
    if ($continue -ne "y" -and $continue -ne "Y") {
        Write-Host "Test run cancelled." -ForegroundColor Red
        exit 1
    }
}

# Set load test parameters if requested
if ($LoadTest) {
    $Iterations = 100
    $Delay = 100
    Write-Host "Running load test with $Iterations iterations..." -ForegroundColor Cyan
}

Write-Host @"

========================================
    YoDA API Test Suite Runner
========================================

Base URL: $BaseUrl
Iterations: $Iterations
Delay: ${Delay}ms
Collection: YoDA_API_Tests.postman_collection.json

"@ -ForegroundColor Cyan

# Create results directory
$resultsDir = "test-results"
if (!(Test-Path $resultsDir)) {
    New-Item -ItemType Directory -Path $resultsDir | Out-Null
}

$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$resultsFile = "$resultsDir\test-results-$timestamp.json"

# Build Newman command
$newmanArgs = @(
    "run",
    "YoDA_API_Tests.postman_collection.json",
    "--env-var", "base_url=$BaseUrl",
    "--reporters", "cli,json",
    "--reporter-json-export", $resultsFile,
    "--iteration-count", $Iterations,
    "--delay-request", $Delay
)

if ($Verbose) {
    $newmanArgs += "--verbose"
}

Write-Host "Starting API tests..." -ForegroundColor Green
Write-Host "Command: newman $($newmanArgs -join ' ')" -ForegroundColor Gray

# Run the tests
try {
    & newman @newmanArgs
    $exitCode = $LASTEXITCODE
} catch {
    Write-Host "ERROR: Failed to run Newman" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

# Display results
if ($exitCode -eq 0) {
    Write-Host @"

========================================
    All tests completed successfully!
========================================

Test results saved to: $resultsFile

"@ -ForegroundColor Green
} else {
    Write-Host @"

========================================
    Some tests failed!
========================================

Check $resultsFile for details

"@ -ForegroundColor Red
}

# Show summary if results file exists
if (Test-Path $resultsFile) {
    try {
        $results = Get-Content $resultsFile | ConvertFrom-Json
        $totalTests = $results.run.stats.assertions.total
        $failedTests = $results.run.stats.assertions.failed
        $passedTests = $totalTests - $failedTests
        
        Write-Host "Test Summary:" -ForegroundColor Cyan
        Write-Host "  Total Assertions: $totalTests" -ForegroundColor White
        Write-Host "  Passed: $passedTests" -ForegroundColor Green
        Write-Host "  Failed: $failedTests" -ForegroundColor Red
        
        if ($failedTests -gt 0) {
            Write-Host "`nFailed Tests:" -ForegroundColor Red
            foreach ($execution in $results.run.executions) {
                if ($execution.testFailReason) {
                    Write-Host "  - $($execution.item.name): $($execution.testFailReason)" -ForegroundColor Red
                }
            }
        }
    } catch {
        Write-Host "Could not parse test results" -ForegroundColor Yellow
    }
}

Write-Host "`nPress any key to continue..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") 
@echo off
REM Quick Start Script for Comunism Project
REM This script checks if make is installed and guides you through setup

echo =======================================
echo  Comunism Project - Quick Start
echo =======================================
echo.

REM Check if make is installed
where make >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Make is not installed!
    echo.
    echo To install Make on Windows, you have a few options:
    echo.
    echo 1. Install Chocolatey and run: choco install make
    echo 2. Install via Scoop: scoop install make
    echo 3. Install Git for Windows which includes make
    echo 4. Use WSL (Windows Subsystem for Linux)
    echo.
    echo After installing, run this script again.
    echo.
    pause
    exit /b 1
)

echo [OK] Make is installed!
echo.

REM Check if .env files exist
if not exist "backend\.env.development" (
    echo Setting up development environment...
    make setup-dev
    echo.
) else (
    echo [OK] Development environment already configured!
    echo.
)

echo What would you like to do?
echo.
echo 1. Start Development Environment
echo 2. Start Production Environment
echo 3. View Logs
echo 4. Stop All Services
echo 5. Show All Commands
echo 6. Exit
echo.

set /p choice="Enter your choice (1-6): "

if "%choice%"=="1" (
    echo.
    echo Starting development environment...
    cd ..
    make dev-up
    echo.
    echo Services started!
    echo Frontend: http://localhost:3000
    echo Backend:  http://localhost:5122
    echo.
) else if "%choice%"=="2" (
    echo.
    echo Starting production environment...
    cd ..
    make prod-up
) else if "%choice%"=="3" (
    echo.
    echo Showing logs... (Press Ctrl+C to exit)
    cd ..
    make logs
) else if "%choice%"=="4" (
    echo.
    echo Stopping all services...
    cd ..
    make dev-down
    make prod-down
) else if "%choice%"=="5" (
    echo.
    cd ..
    make help
) else if "%choice%"=="6" (
    echo.
    echo Goodbye!
    exit /b 0
) else (
    echo.
    echo Invalid choice!
)

echo.
pause

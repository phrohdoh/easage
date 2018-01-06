@echo off
rem Contributor: Teteros
rem Non-alphanumeric filenames were untested and might cause problems

title Drag'n'Drop a single BIG file onto this script file to list packed filenames
rem Usage: Drag'n'Drop a single BIG file onto this script file to list packed filenames
if not exist easage.exe goto FileMissing

rem Set ignore_ext to some value e.g. '1'
rem to disable .big extension checking
set ignore_ext=
rem Set verbose to some value e.g. '1'
rem to be more verbose when listing
rem i.e list more information about the BIG file
set verbose=

if not defined ignore_ext (
    if "%~nx1" NEQ "%~n1.big" goto NotBIG
)

echo Listing: "%~nx1"
echo Press space to scroll a page.
echo.

if defined verbose (easage list --verbose "%~nx1" | more) else (easage list "%~nx1" | more)

echo.
echo Press any key to exit...
pause >nul & exit /b
:FileMissing
echo Current Directory is %CD%
echo You need the easage.exe tool from easage
echo library in your current directory to use this script.
echo.
echo See https://github.com/Phrohdoh/easage
echo Aborting.
pause & exit /b 1
:NotBIG
echo.
echo File %~nx1 does not have a .big extension
echo or you have not dropped a file on this script.
echo.
echo Aborting.
pause & exit /b 1

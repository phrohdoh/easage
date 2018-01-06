@echo off
rem Contributor: Teteros
rem Non-alphanumeric filenames were untested and might cause problems

title Drag'n'Drop BIG files onto this script file to unpack them
rem Usage: Drag'n'Drop BIG files onto this script file to unpack them
rem Each file in your selection is looped until they are unpacked
rem Unpack folder is determined by the BIG file(s) name or the 'out' var
if not exist easage.exe goto FileMissing

rem Set ignore_ext to some value e.g. '1'
rem to disable .big extension checking
set ignore_ext=
rem Set out to unpack to a specific folder.
set out=
set exit_timeout=5

:NotUnpacked
if (%1) EQU () goto Unpacked
if not defined ignore_ext (
    if "%~nx1" NEQ "%~n1.big" goto NotBIG
)
if defined out (
    echo Unpacking: "%~nx1" to "%out%"
    easage unpack --all --source "%~nx1" --output "%out%"
) else (
    echo Unpacking: "%~nx1" to "%~n1"
    easage unpack --all --source "%~nx1" --output "%~n1"
)
shift
goto NotUnpacked

:Unpacked
echo.
echo All BIGs unpacked or you have not selected anything.
echo.
echo Exiting in %exit_timeout% seconds...
timeout %exit_timeout% & exit /b
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
echo File %~nx1 does not have a .big extension!
echo.
echo If you are sure this is a BIG archive
echo Please rename the extension of %~nx1 to .big or
echo disable extension checking by setting ignore_ext in the script
echo.
echo Aborting.
pause & exit /b 1

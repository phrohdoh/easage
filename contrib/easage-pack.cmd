@echo off
rem Contributor: Teteros
rem Non-alphanumeric filenames were untested and might cause problems

title Drag'n'Drop a selection onto this script to pack into BIGs
rem Usage: Drag'n'Drop files and folders onto easage-pack.cmd
rem to pack into BIG format files
rem easage.exe 'pack' archives recursively
rem i.e sub-directories are also packed
if not exist easage.exe goto FileMissing

rem Use wmic command to get current date information in YYYYMMDD format
for /f %%# in ('wMIC Path Win32_LocalTime Get /Format:value') do @for /f %%@ in ("%%#") do @set %%@
set "curdate=%year%%month%%day%"

rem Set BIG target to pack for (BIG4 or BIGF)
set kind=BIGF

rem A prefix can be set for the output big files
rem e.g. You may remove the date but keep the bangs
rem
rem Common sage modding convention:
rem one '!' for mod release, two '!!' for hotfix/patch

set modname=MOD
set modver=001
set "prefix=!%curdate%_%modname%%modver%_"
set exit_timeout=5

rem Uncomment line below (remove 'rem') to show your prefix and exit
rem echo %prefix% & pause & exit

:NotPacked
if (%1) EQU () goto Packed
echo Packing: "%~nx1" to "%prefix%%~n1.big"
easage pack --kind %kind% --source "%~nx1" --output "%prefix%%~n1.big"
shift
goto NotPacked

:Packed
echo.
echo Selected files/folders packed into BIGs or you have not selected anything.
echo.
echo Exiting in %exit_timeout% seconds...
timeout %exit_timeout% & exit /b 0

:FileMissing
echo.
echo Current Directory is %CD%
echo You need the easage.exe tool from easage
echo library in your current directory to use this script.
echo.
echo See https://github.com/Phrohdoh/easage
echo Aborting.
pause & exit /b 1
$SRC_DIR = $PWD.Path
$STAGE = [System.Guid]::NewGuid().ToString()

Set-Location $ENV:Temp
New-Item -Type Directory -Name $STAGE
Set-Location $STAGE

$ZIP = "$SRC_DIR\$($ENV:CRATE_NAME)-$($ENV:APPVEYOR_REPO_TAG_NAME)-$($ENV:TARGET).zip"

Copy-Item "$SRC_DIR\target\$($ENV:TARGET)\release\$($ENV:CRATE_NAME).exe" '.\'

7z a "$ZIP" *

Push-AppveyorArtifact "$ZIP"

Remove-Item *.* -Force
Set-Location ..
Remove-Item $STAGE
Set-Location $SRC_DIR

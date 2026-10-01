param([string]$Iteration='baseline',[string]$Executable='.\target\release\journey-to-a-black-hole.exe',[string[]]$ExtraArgs=@())
$ErrorActionPreference='Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
$exe=(Resolve-Path -LiteralPath $Executable).Path
New-Item -ItemType Directory -Force logs,captures | Out-Null
$cases=@(
    @{Name='distant';Args=@('--preset','1')},
    @{Name='edge';Args=@('--preset','2')},
    @{Name='above';Args=@('--preset','3')},
    @{Name='close';Args=@('--preset','4')},
    @{Name='ring';Args=@('--preset','5','--no-disk','--no-craft')},
    @{Name='probe';Args=@('--preset','6')},
    @{Name='limit';Args=@('--preset','7')},
    @{Name='physical';Args=@('--preset','2','--no-quantization')}
)
foreach($case in $cases){
    $label="$Iteration-$($case.Name)"
    $arguments=$case.Args+$ExtraArgs+@('--fullscreen','--freeze-input','--hide-hud','--capture','--capture-at','4','--capture-label',$label,'--seconds','6')
    & $exe @arguments *> "logs/$label.log"
    if($LASTEXITCODE -ne 0 -or (Select-String -Path "logs/$label.log" -Pattern ' ERROR |error:')){throw "Review render failed: $label"}
    Write-Output $label
}

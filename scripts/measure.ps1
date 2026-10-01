param(
    [string]$Executable = '.\target\release\journey-to-a-black-hole.exe',
    [int]$Seconds = 18
)
$ErrorActionPreference = 'Stop'
$env:RUST_LOG = 'info'
Set-Location (Split-Path $PSScriptRoot -Parent)
$exe = (Resolve-Path -LiteralPath $Executable).Path
New-Item -ItemType Directory -Force logs | Out-Null
$cases = @(
    @{Name='relativity-schwarzschild'; Args=@('--quality','1','--preset','6','--freeze-input')},
    @{Name='relativity-kerr-balanced'; Args=@('--kerr','--quality','1','--preset','6','--freeze-input')},
    @{Name='relativity-kerr-cheap'; Args=@('--kerr','--quality','0','--preset','6','--freeze-input')},
    @{Name='relativity-kerr-high'; Args=@('--kerr','--quality','2','--preset','6','--freeze-input')},
    @{Name='relativity-kerr-unquantized'; Args=@('--kerr','--quality','1','--preset','6','--no-quantization','--freeze-input')},
    @{Name='relativity-kerr-taa-auto'; Args=@('--kerr','--quality','1','--preset','6','--taa','--auto-exposure','--freeze-input')},
    @{Name='relativity-kerr-camera-sweep'; Args=@('--kerr','--quality','1','--benchmark')},
    @{Name='relativity-kerr-full-path'; Args=@('--kerr','--quality','1','--journey','--journey-seconds','60')},
    @{Name='relativity-kerr-inspector'; Args=@('--kerr','--quality','1','--preset','4','--inspect-rays','--sky-both','--freeze-input')},
    @{Name='relativity-fractional-grid'; Args=@('--kerr','--internal','1333x751','--grid','167x94','--preset','6','--freeze-input')}
)
$results = @()
foreach ($case in $cases) {
    $duration = if ($case.Name -eq 'relativity-kerr-camera-sweep') {58} elseif ($case.Name -eq 'relativity-kerr-full-path') {64} else {$Seconds}
    $arguments = $case.Args + @('--fullscreen','--seconds',"$duration")
    $logPath = Join-Path 'logs' ($case.Name + '.log')
    & $exe @arguments 2>&1 | Tee-Object -FilePath $logPath
    if ($LASTEXITCODE -ne 0) {throw "Demo failed for $($case.Name). See $logPath"}
    $rows = @()
    foreach ($line in Get-Content -LiteralPath $logPath) {
        if ($line -match 'LAB_METRICS t=(?<t>[\d.]+) fps=(?<fps>[\d.]+) frame_ms=(?<frame>[\d.]+) main_cpu_ms=(?<cpu>[\d.]+) bh_gpu_ms=(?<bh>[\d.]+) ms quant_gpu_ms=(?<quant>[\d.]+) ms render_gpu_ms=(?<gpu>[\d.]+) ms') {
            $culture = [Globalization.CultureInfo]::InvariantCulture
            if ([double]::Parse($Matches.t,$culture) -lt 6) {continue}
            $rows += [pscustomobject]@{
                FPS=[double]::Parse($Matches.fps,$culture)
                FrameMs=[double]::Parse($Matches.frame,$culture)
                MainCpuMs=[double]::Parse($Matches.cpu,$culture)
                BlackHoleGpuMs=[double]::Parse($Matches.bh,$culture)
                QuantizationGpuMs=[double]::Parse($Matches.quant,$culture)
                RenderGpuMs=[double]::Parse($Matches.gpu,$culture)
            }
        }
    }
    if ($rows.Count -eq 0) {Write-Warning "No GPU timing rows for $($case.Name); inspect unavailable timestamps."; continue}
    $results += [pscustomobject]@{
        Case=$case.Name; Samples=$rows.Count
        MeanFPS=($rows.FPS | Measure-Object -Average).Average
        MeanFrameMs=($rows.FrameMs | Measure-Object -Average).Average
        MeanMainCpuMs=($rows.MainCpuMs | Measure-Object -Average).Average
        MeanBlackHoleGpuMs=($rows.BlackHoleGpuMs | Measure-Object -Average).Average
        MeanQuantizationGpuMs=($rows.QuantizationGpuMs | Measure-Object -Average).Average
        MeanRenderGpuMs=($rows.RenderGpuMs | Measure-Object -Average).Average
    }
}
$results | ConvertTo-Json | Set-Content -LiteralPath logs/relativity-summary.json
$results | Format-Table -AutoSize

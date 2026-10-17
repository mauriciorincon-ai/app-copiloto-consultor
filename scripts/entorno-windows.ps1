# entorno-windows.ps1 — sprint 009 («La Surface»): ¿este PC con Windows puede construir Angel Ghost?
#
# SOLO LEE. No instala nada, no cambia el PATH ni el registro y no abre la red: imprime «tengo · necesito · falta» y
# sale con 1 si falta algo. Lo que falta se instala con su fila de la regla 24, una por una y con tu «sí» (README,
# «Desarrollar en Windows ARM»). La arquitectura se lee del registro del sistema y de `rustc -vV`, nunca de `uname`:
# en un PC ARM64 Git Bash es x64 emulado y diría x86_64 (prep. de la Surface, § 4.5).
#
# Va con BOM: Windows PowerShell 5.1 lee un .ps1 sin BOM como ANSI y rompería los acentos.
# Uso: powershell -NoProfile -File scripts/entorno-windows.ps1

# 'Continue' y no 'Stop': en PowerShell 5.1 una línea de stderr de un programa nativo, con 'Stop', se vuelve una
# excepción y la pieza saldría «no está» teniéndola.
$ErrorActionPreference = 'Continue'
$raiz = Split-Path -Parent $PSScriptRoot
$filas = New-Object System.Collections.Generic.List[object]

function Fila([string]$pieza, [string]$tengo, [string]$necesito, [string]$estado) {
  # estado: ok · falta · aviso (aviso no hace fallar el script: es información, como Git Bash emulado).
  $filas.Add([pscustomobject]@{ Pieza = $pieza; Tengo = $tengo; Necesito = $necesito; Estado = $estado })
}

function Version-De([string]$programa, [string[]]$argumentos) {
  $c = Get-Command $programa -ErrorAction SilentlyContinue
  if (-not $c) { return $null }
  try { return ((& $c.Source @argumentos 2>$null) | Select-Object -First 1).ToString().Trim() } catch { return $null }
}

# La arquitectura de un ejecutable o una DLL, leída de su cabecera PE (campo Machine).
function Arquitectura-PE([string]$ruta) {
  if (-not $ruta -or -not (Test-Path $ruta)) { return $null }
  $fs = [System.IO.File]::OpenRead($ruta)
  try {
    $b = New-Object byte[] 4096
    [void]$fs.Read($b, 0, $b.Length)
  } finally { $fs.Close() }
  $pe = [BitConverter]::ToInt32($b, 0x3C)
  if ($pe -lt 0 -or $pe + 6 -gt $b.Length) { return 'desconocida' }
  switch ([BitConverter]::ToUInt16($b, $pe + 4)) {
    0xAA64 { 'arm64' } 0x8664 { 'x64' } 0x014C { 'x86' } default { 'desconocida' }
  }
}

# --- El sistema -----------------------------------------------------------------------------------------------------
$entorno = Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment'
$arqSistema = $entorno.PROCESSOR_ARCHITECTURE   # ARM64 o AMD64: el del sistema, no el de este proceso
$esArm = $arqSistema -eq 'ARM64'
$destino = if ($esArm) { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }

$build = [Environment]::OSVersion.Version.Build
Fila 'Windows' "compilacion $build, $arqSistema" '10 version 2004 (19041) o posterior (ADR 022)' $(if ($build -ge 19041) { 'ok' } else { 'falta' })
Fila 'Idioma de la interfaz' (Get-UICulture).Name 'el que sea: los rotulos de toda mirada van en este idioma' 'ok'
$largas = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem' -ErrorAction SilentlyContinue).LongPathsEnabled
Fila 'Rutas largas (LongPathsEnabled)' "$largas" '1' $(if ($largas -eq 1) { 'ok' } else { 'falta' })

# --- git, Git Bash, gh ----------------------------------------------------------------------------------------------
$git = Get-Command git -ErrorAction SilentlyContinue
if ($git) {
  Fila 'git' "$(Version-De git @('--version')) ($(Arquitectura-PE $git.Source))" 'cualquiera' 'ok'
  # El bash de Git, nunca el de WSL (C:\Windows\System32\bash.exe): el que corren los hooks del kit.
  $exec = (& git --exec-path).Trim()
  $bash = Join-Path (Split-Path (Split-Path (Split-Path $exec))) 'usr\bin\bash.exe'
  $arqBash = Arquitectura-PE $bash
  if (-not $arqBash) { Fila 'Git Bash' 'no esta' 'el de Git for Windows (los hooks corren en el)' 'falta' }
  elseif ($esArm -and $arqBash -ne 'arm64') { Fila 'Git Bash' "$arqBash, emulado" 'cualquiera: corre emulado sin problema' 'aviso' }
  else { Fila 'Git Bash' $arqBash 'cualquiera' 'ok' }
} else {
  Fila 'git' 'no esta' 'git for Windows' 'falta'
}
$gh = Version-De gh @('--version')
Fila 'gh' $(if ($gh) { $gh } else { 'no esta' }) 'cualquiera (para los PR)' $(if ($gh) { 'ok' } else { 'falta' })

# --- node y pnpm ----------------------------------------------------------------------------------------------------
$node = Version-De node @('-p', "process.version + ' ' + process.arch")
$nodeMayor = if ($node -match '^v(\d+)') { [int]$Matches[1] } else { 0 }
Fila 'node' $(if ($node) { $node } else { 'no esta' }) '22 o posterior' $(if ($nodeMayor -ge 22) { 'ok' } else { 'falta' })
$pide = ((Get-Content (Join-Path $raiz 'package.json') -Raw | ConvertFrom-Json).packageManager) -replace '^pnpm@', ''
$pnpm = Version-De pnpm @('--version')
Fila 'pnpm' $(if ($pnpm) { $pnpm } else { 'no esta' }) "$pide (packageManager)" $(if ($pnpm -eq $pide) { 'ok' } else { 'falta' })

# --- Rust -----------------------------------------------------------------------------------------------------------
$rustc = Get-Command rustc -ErrorAction SilentlyContinue
if ($rustc) {
  $vv = & rustc -vV
  $host_ = (($vv | Where-Object { $_ -like 'host:*' }) -replace '^host:\s*', '').Trim()
  $version = ($vv | Select-Object -First 1).Trim()
  Fila 'rustc' "$version, host $host_" "host $destino" $(if ($host_ -eq $destino) { 'ok' } else { 'falta' })
  $targets = & rustup target list --installed 2>$null
  Fila 'Target de Rust' (($targets | Where-Object { $_ }) -join ', ') $destino $(if ($targets -contains $destino) { 'ok' } else { 'falta' })
} else {
  Fila 'rustc' 'no esta' "rustup con host $destino" 'falta'
}

# --- C y C++: MSVC, el SDK, CMake, clang y libclang -----------------------------------------------------------------
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$componente = if ($esArm) { 'Microsoft.VisualStudio.Component.VC.Tools.ARM64' } else { 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64' }
$vs = if (Test-Path $vswhere) { & $vswhere -products * -requires $componente -property installationPath | Select-Object -First 1 } else { $null }
$vsVersion = if ($vs) { & $vswhere -products * -requires $componente -property catalog_productDisplayVersion | Select-Object -First 1 } else { $null }
Fila 'MSVC (Build Tools)' $(if ($vs) { "$vsVersion en $vs" } else { 'no esta' }) $componente $(if ($vs) { 'ok' } else { 'falta' })
$carpetaSdk = if ($esArm) { 'arm64' } else { 'x64' }
$sdk = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\Lib\*\um\$carpetaSdk\kernel32.lib" -ErrorAction SilentlyContinue |
  ForEach-Object { $_.Directory.Parent.Parent.Name } | Sort-Object | Select-Object -Last 1
Fila 'Windows SDK' $(if ($sdk) { $sdk } else { 'no esta' }) "las librerias $carpetaSdk" $(if ($sdk) { 'ok' } else { 'falta' })
$cmake = Version-De cmake @('--version')
Fila 'CMake' $(if ($cmake) { $cmake } else { 'no esta' }) 'cualquiera (whisper-rs-sys)' $(if ($cmake) { 'ok' } else { 'falta' })
if ($esArm) {
  # `ring` (el HTTPS de la descarga del modelo) compila su C con `clang` en Windows ARM64 (su build.rs).
  $clang = Get-Command clang -ErrorAction SilentlyContinue
  Fila 'clang en el PATH' $(if ($clang) { $clang.Source } else { 'no esta' }) 'clang.exe (lo pide ring en ARM64)' $(if ($clang) { 'ok' } else { 'falta' })
  # whisper.cpp se compila con clang-cl y Ninja (src-tauri/cmake/windows-arm64-clang-cl.cmake): el toolchain los busca en
  # estos mismos sitios y falla con REQUIRED si no están. Sin estas dos filas el script salía verde y el build, no.
  $llvm = @($env:AG_LLVM_BIN, (Join-Path $env:ProgramFiles 'LLVM\bin'), $(if ($vs) { Join-Path $vs 'VC\Tools\Llvm\ARM64\bin' })) | Where-Object { $_ }
  $clangCl = (Get-Command clang-cl -ErrorAction SilentlyContinue).Source
  if (-not $clangCl) { $clangCl = $llvm | ForEach-Object { Join-Path $_ 'clang-cl.exe' } | Where-Object { Test-Path $_ } | Select-Object -First 1 }
  Fila 'clang-cl (whisper.cpp)' $(if ($clangCl) { $clangCl } else { 'no esta' }) 'clang-cl.exe de LLVM (ggml no compila con MSVC en ARM)' $(if ($clangCl) { 'ok' } else { 'falta' })
  $ninja = (Get-Command ninja -ErrorAction SilentlyContinue).Source
  if (-not $ninja -and $vs) { $ninja = Join-Path $vs 'Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe'; if (-not (Test-Path $ninja)) { $ninja = $null } }
  Fila 'ninja (whisper.cpp)' $(if ($ninja) { $ninja } else { 'no esta' }) 'ninja.exe (el de las Build Tools sirve)' $(if ($ninja) { 'ok' } else { 'falta' })
}
$libclang = if ($env:LIBCLANG_PATH) { Join-Path $env:LIBCLANG_PATH 'libclang.dll' } else { $null }
$arqLibclang = Arquitectura-PE $libclang
$quiere = if ($esArm) { 'arm64' } else { 'x64' }
Fila 'libclang (LIBCLANG_PATH)' $(if ($arqLibclang) { "$libclang ($arqLibclang)" } else { 'no esta' }) "libclang.dll $quiere (bindgen la carga dentro de cargo)" $(if ($arqLibclang -eq $quiere) { 'ok' } else { 'falta' })

# --- WebView2, gitleaks, jq -----------------------------------------------------------------------------------------
$clave = 'Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
$wv = @("HKLM:\SOFTWARE\WOW6432Node\$clave", "HKLM:\SOFTWARE\$clave", "HKCU:\Software\$clave") |
  ForEach-Object { (Get-ItemProperty $_ -ErrorAction SilentlyContinue).pv } | Where-Object { $_ -and $_ -ne '0.0.0.0' } | Select-Object -First 1
Fila 'WebView2 Runtime' $(if ($wv) { $wv } else { 'no esta' }) 'cualquiera' $(if ($wv) { 'ok' } else { 'falta' })
$gitleaks = Version-De gitleaks @('version')
Fila 'gitleaks' $(if ($gitleaks) { $gitleaks } else { 'no esta' }) '8.30.1 (la de la CI); lo exigen el pre-commit y el hook' $(if ($gitleaks) { 'ok' } else { 'falta' })
$jq = Version-De jq @('--version')
Fila 'jq' $(if ($jq) { $jq } else { 'no esta' }) 'cualquiera; sin el, el hook bloquea toda escritura de Claude Code' $(if ($jq) { 'ok' } else { 'falta' })

# --- La tabla -------------------------------------------------------------------------------------------------------
# Simbolo + texto + color (regla dura 8): el color solo no dice nada.
$etiqueta = @{ ok = '[ok]   '; falta = '[FALTA]'; aviso = '[aviso]' }
$color = @{ ok = 'Green'; falta = 'Red'; aviso = 'Yellow' }
Write-Host "Angel Ghost - entorno de este PC (destino $destino)"
foreach ($f in $filas) {
  Write-Host ("{0} {1,-32} tengo: {2}" -f $etiqueta[$f.Estado], $f.Pieza, $f.Tengo) -ForegroundColor $color[$f.Estado]
  if ($f.Estado -ne 'ok') { Write-Host ("{0,-40} necesito: {1}" -f '', $f.Necesito) }
}
$faltan = @($filas | Where-Object { $_.Estado -eq 'falta' }).Count
if ($faltan -gt 0) {
  Write-Host "$faltan pieza(s) faltan. Cada una se instala con su fila de la regla 24 (README, 'Desarrollar en Windows ARM')." -ForegroundColor Red
  exit 1
}
Write-Host 'Nada falta.' -ForegroundColor Green
exit 0

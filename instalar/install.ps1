# Instalador de una linea de tc2 (TaskCodeTwo) para Windows
# (TASK-026, F2-11; ADR F1-14).
#
#   irm https://github.com/charliebk/TaskCodeTwo/releases/latest/download/install.ps1 | iex
#
# Baja de la release el binario de este sistema y SHA256SUMS, comprueba el
# sha256 y, si hay `gh` con sesion, la attestation de procedencia. Despues deja
# `tc2.exe` en %LOCALAPPDATA%\taskcodetwo\bin y anade ese directorio al PATH
# del usuario, sin permisos de administrador. Nunca deja en el destino un
# binario sin comprobar. Funciona en Windows PowerShell 5.1 (el de un Windows
# limpio) y en PowerShell 7. No toca `taskctl` ni nada de TaskCode 0.16.
#
# Variables (todas opcionales):
#   TC2_VERSION         version que se instala (2.0.0, 2.0.0-rc.1...); por
#                       defecto, la ultima release estable.
#   TC2_INSTALL_DIR     directorio del binario.
#   TC2_NO_MODIFY_PATH  con 1, no se toca el PATH del usuario.
#   TC2_BASE_URL        solo para ensayos: URL con los ficheros de la release
#                       (https, o http a 127.0.0.1 o localhost). Exige
#                       TC2_VERSION y no comprueba la attestation.
#
# Nunca usa `exit`: con `irm | iex` cerraria la ventana de la persona. Los
# errores se lanzan con throw (codigo 1 con -File o -Command). Todo va dentro
# de Install-Tc2, que se llama en la ultima linea: una descarga cortada no
# ejecuta medio script.

function Install-Tc2 {
    Set-StrictMode -Version 2.0
    $ErrorActionPreference = 'Stop'
    # Sin barra de progreso, Invoke-WebRequest de la 5.1 es mucho mas rapido.
    $ProgressPreference = 'SilentlyContinue'
    $repo = 'charliebk/TaskCodeTwo'

    function Decir([string]$texto) {
        Write-Host "tc2-instalar: $texto"
    }

    function Fallo([string]$problema, [string]$arreglo) {
        throw "tc2-instalar: ERROR: $problema`ntc2-instalar: arreglo: $arreglo"
    }

    # Un ejecutable externo, sin que su stderr se vuelva un error terminante
    # (en la 5.1 lo hace con ErrorActionPreference = Stop).
    function Invoke-Externo([string]$exe, [string[]]$argumentos) {
        $ErrorActionPreference = 'Continue'
        $salida = (& $exe @argumentos 2>&1 | ForEach-Object { "$_" }) -join "`n"
        return @{ Codigo = $LASTEXITCODE; Salida = $salida.Trim() }
    }

    function Get-Descarga([string]$url, [string]$fichero) {
        for ($intento = 1; $intento -le 3; $intento++) {
            try {
                Invoke-WebRequest -Uri $url -OutFile $fichero -UseBasicParsing
                return
            } catch {
                if ($intento -eq 3) {
                    Fallo "no se pudo bajar $url ($($_.Exception.Message))" "comprueba la version en https://github.com/$repo/releases, la conexion y el proxy"
                }
                Start-Sleep -Seconds 2
            }
        }
    }

    # La ultima release estable, sin la API (tiene limite anonimo): la
    # redireccion de /releases/latest a /releases/tag/v<version>. Sin ninguna
    # release estable, GitHub redirige a /releases. Si no hay respuesta, el
    # fallo es de red (conexion, proxy, certificado), no de version.
    function Get-UltimaVersion {
        $url = "https://github.com/$repo/releases/latest"
        $destino = $null
        try {
            $peticion = [Net.HttpWebRequest]::Create($url)
            $peticion.Method = 'HEAD'
            $peticion.AllowAutoRedirect = $false
            $respuesta = $peticion.GetResponse()
            $destino = $respuesta.Headers['Location']
            $respuesta.Close()
        } catch [Net.WebException] {
            if ($null -eq $_.Exception.Response) {
                Fallo "no se pudo consultar $url ($($_.Exception.Message))" "comprueba la conexion, el proxy y los certificados, o fija la version con `$env:TC2_VERSION = '<version>'"
            }
            $destino = $_.Exception.Response.Headers['Location']
        }
        if ("$destino" -match '/releases/tag/v([0-9A-Za-z.\-]+)$') {
            return $Matches[1]
        }
        Fallo 'no hay ninguna release estable de tc2 todavia' "elige una version de https://github.com/$repo/releases: `$env:TC2_VERSION = '<version>'; irm <url de install.ps1> | iex"
    }

    function Get-AliasP12 {
        # El valor nativo del registro: una PowerShell x64 emulada en un
        # Windows ARM64 baja igualmente el binario arm64.
        $clave = 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment'
        $nativa = (Get-ItemProperty -Path $clave -Name PROCESSOR_ARCHITECTURE).PROCESSOR_ARCHITECTURE
        switch ($nativa) {
            'AMD64' { return 'win-x64' }
            'ARM64' { return 'win-arm64' }
            default { Fallo "arquitectura no soportada: $nativa" 'tc2 se publica para Windows en x64 y arm64' }
        }
    }

    function Add-PathUsuario([string]$dir) {
        $normal = $dir.TrimEnd('\')
        if ($env:TC2_NO_MODIFY_PATH -eq '1') {
            Decir "TC2_NO_MODIFY_PATH=1: no se toca el PATH; anade tu $dir"
            return
        }
        # Se lee y se escribe en crudo para no expandir las %VARIABLES% que ya
        # tenga el PATH de la persona.
        $clave = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
        try {
            $actual = [string]$clave.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
            $tipo = [Microsoft.Win32.RegistryValueKind]::ExpandString
            if ($clave.GetValueNames() -contains 'Path') {
                $tipo = $clave.GetValueKind('Path')
            }
            $entradas = @($actual -split ';' | Where-Object { $_ -ne '' })
            $ya = @($entradas | Where-Object { [Environment]::ExpandEnvironmentVariables($_).TrimEnd('\') -ieq $normal })
            if ($ya.Count -gt 0) {
                Decir "$dir ya esta en el PATH del usuario"
            } else {
                $clave.SetValue('Path', (($entradas + $dir) -join ';'), $tipo)
                # Avisa a las ventanas abiertas (WM_SETTINGCHANGE): .NET lo
                # difunde al cambiar una variable de usuario.
                [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', '1', 'User')
                [Environment]::SetEnvironmentVariable('TC2_INSTALAR_AVISO', $null, 'User')
                Decir "anadido $dir al PATH del usuario"
            }
        } finally {
            $clave.Close()
        }
        # Y a esta sesion, para usar tc2 sin abrir otra ventana.
        if (-not (@($env:Path -split ';') | Where-Object { $_.TrimEnd('\') -ieq $normal })) {
            $env:Path = "$env:Path;$dir"
        }
    }

    if ($PSVersionTable.PSEdition -eq 'Core' -and -not $IsWindows) {
        Fallo 'install.ps1 es para Windows' "en macOS y Linux: curl -fsSL https://github.com/$repo/releases/latest/download/install.sh | sh"
    }
    # La 5.1 no ofrece TLS 1.2 por defecto en todos los sistemas.
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

    $comprobarAttestation = $true
    if ($env:TC2_BASE_URL) {
        if ($env:TC2_BASE_URL -notmatch '^(https://|http://(127\.0\.0\.1|localhost)(:|/))') {
            Fallo "TC2_BASE_URL tiene que ser https, o http a 127.0.0.1 o localhost: $env:TC2_BASE_URL" 'corrige TC2_BASE_URL o quitala'
        }
        if (-not $env:TC2_VERSION) {
            Fallo 'TC2_BASE_URL exige TC2_VERSION' "define `$env:TC2_VERSION = '<version>'"
        }
        $comprobarAttestation = $false
    }

    $version = $env:TC2_VERSION
    if (-not $version) {
        $version = Get-UltimaVersion
    }
    $version = $version -replace '^v', ''
    if ($version -notmatch '^[0-9A-Za-z.\-]+$') {
        Fallo "version no valida: '$version'" 'usa una version como 2.0.0 o 2.0.0-rc.1'
    }
    # Un tag de prerelease es v<version>-<sufijo>; el binario lleva <version>.
    $versionBin = ($version -split '-', 2)[0]
    $base = $env:TC2_BASE_URL
    if (-not $base) {
        $base = "https://github.com/$repo/releases/download/v$version"
    }
    $base = $base.TrimEnd('/')

    $aliasP12 = Get-AliasP12
    $fichero = "tc2-$versionBin-$aliasP12.exe"
    $destino = $env:TC2_INSTALL_DIR
    if (-not $destino) {
        $destino = Join-Path $env:LOCALAPPDATA 'taskcodetwo\bin'
    }
    # Absoluto (respecto al directorio actual de PowerShell): el directorio
    # acaba en el PATH del usuario.
    $destino = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($destino)

    $tmp = Join-Path ([IO.Path]::GetTempPath()) ('tc2-instalar-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmp | Out-Null
    try {
        Decir "instalando tc2 $version ($aliasP12) desde $base"
        $sumas = Join-Path $tmp 'SHA256SUMS'
        Get-Descarga "$base/SHA256SUMS" $sumas
        $esperados = @()
        foreach ($linea in [IO.File]::ReadAllLines($sumas)) {
            $partes = $linea -split '  ', 2
            if ($partes.Count -eq 2 -and $partes[1] -ceq $fichero) {
                $esperados += $partes[0]
            }
        }
        if ($esperados.Count -ne 1) {
            Fallo "SHA256SUMS no trae una sola linea para $fichero ($($esperados.Count))" "esa release no tiene binario para ${aliasP12}: prueba otra TC2_VERSION"
        }
        $descargado = Join-Path $tmp $fichero
        Get-Descarga "$base/$fichero" $descargado
        $real = (Get-FileHash -LiteralPath $descargado -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($real -cne $esperados[0]) {
            Fallo "el sha256 de $fichero no casa con SHA256SUMS (esperado $($esperados[0]), obtenido $real); no se ha instalado nada" "repite la descarga; si sigue, avisa en https://github.com/$repo/issues"
        }
        Decir "sha256 correcto: $real"

        if (-not $comprobarAttestation) {
            Decir 'TC2_BASE_URL: no se comprueba la attestation (solo ensayos)'
        } elseif ((Get-Command gh -ErrorAction SilentlyContinue) -and (Invoke-Externo 'gh' @('auth', 'status')).Codigo -eq 0) {
            $r = Invoke-Externo 'gh' @('attestation', 'verify', $descargado, '--repo', $repo, '--signer-workflow', "$repo/.github/workflows/release.yml")
            if ($r.Codigo -ne 0) {
                Fallo "la attestation de procedencia de $fichero no es valida; no se ha instalado nada ($($r.Salida))" "avisa en https://github.com/$repo/issues"
            }
            Decir "attestation correcta: lo construyo $repo con release.yml"
        } else {
            Decir 'aviso: sin gh con sesion no se comprueba la attestation (el sha256 si); ver docs/distribucion/instalar.md'
        }

        # Se prueba en el temporal: un .exe recien ejecutado puede quedar
        # bloqueado un momento (el antivirus lo analiza) y no se podria mover.
        $r = Invoke-Externo $descargado @('--version')
        if ($r.Codigo -ne 0 -or $r.Salida -cne "tc2 $versionBin") {
            Fallo "tc2 --version escribio '$($r.Salida)' (codigo $($r.Codigo)), se esperaba 'tc2 $versionBin'; no se ha instalado nada" "avisa en https://github.com/$repo/issues"
        }
        $nuevo = Join-Path $destino ".tc2.nuevo.$PID.exe"
        try {
            New-Item -ItemType Directory -Force -Path $destino | Out-Null
            Copy-Item -LiteralPath $descargado -Destination $nuevo -Force
        } catch {
            Fallo "no se pudo escribir en $destino ($($_.Exception.Message))" 'usa $env:TC2_INSTALL_DIR con un directorio con permiso de escritura'
        }
        $final = Join-Path $destino 'tc2.exe'
        for ($intento = 1; $true; $intento++) {
            try {
                Move-Item -LiteralPath $nuevo -Destination $final -Force
                break
            } catch {
                if ($intento -ge 10) {
                    Remove-Item -LiteralPath $nuevo -Force -ErrorAction SilentlyContinue
                    Fallo "no se pudo dejar tc2.exe en $destino ($($_.Exception.Message))" 'cierra lo que este usando tc2.exe y repite'
                }
                Start-Sleep -Milliseconds 500
            }
        }

        Add-PathUsuario $destino
        Decir "instalado: $final ($($r.Salida))"
    } finally {
        Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Install-Tc2

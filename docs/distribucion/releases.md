# Releases de `tc2`: cómo se publican y cómo se verifican

Lo construyó TASK-025 (F2-10). Aplica lo decidido en
[ADR F1-14](../adr/F1-14-distribucion-e-instalacion.md) (D4a) y en
[ADR F1-3](../adr/F1-3-flujo-de-ramas-worktrees-release.md) (D3a).

## Qué trae cada release

- Un binario por cada uno de los 6 targets de P12, con el nombre
  `tc2-<version>-<alias>[.exe]`:

  | alias | triple de Rust | runner |
  |---|---|---|
  | win-x64 | x86_64-pc-windows-msvc | windows-latest |
  | win-arm64 | aarch64-pc-windows-msvc | windows-11-arm |
  | darwin-x64 | x86_64-apple-darwin | macos-15-intel |
  | darwin-arm64 | aarch64-apple-darwin | macos-latest |
  | linux-x64 | x86_64-unknown-linux-musl | ubuntu-latest |
  | linux-arm64 | aarch64-unknown-linux-musl | ubuntu-24.04-arm |

  En Linux el binario es musl estático: no depende de la glibc del sistema.
- `SHA256SUMS`, con el formato de `sha256sum`.
- Una attestation de procedencia de GitHub (Sigstore) por binario.
- En macOS, firma ad hoc (`codesign --sign -`).

"Firmado" significa eso (D4a). Authenticode y Developer ID con notarización
llegan con winget y Homebrew, así que en esta versión Windows puede mostrar
SmartScreen y macOS pone en cuarentena lo que se descarga con el navegador.

## Cómo se publica

El workflow `.github/workflows/release.yml` se lanza de dos formas:

- **Tag `v<version>`** (o `v<version>-<sufijo>`, que se publica como
  prerelease), anotado sobre el merge en `main`. `<version>` tiene que ser la
  de `Cargo.toml`, y un tag estable tiene que estar sobre un commit de `main`:
  si no se cumple, el job `preparar` falla y no se publica nada.
- **`workflow_dispatch`, o un push a una rama que toque lo que entra en la
  release** (el workflow, el humo, `xtask`, Cargo): ensayo. Compila y prueba
  los 6 binarios, pero no publica.

Jobs, en orden:

1. `build`: compila cada target en un runner nativo con
   `cargo xtask empaquetar`.
2. `humo`: comprueba que cada binario arranca sin Node, con
   `.github/scripts/humo.sh`.
3. `publicar`: genera `SHA256SUMS`, crea la attestation y la release, primero
   como borrador y después la publica.
4. `verificar-release`: vuelve a descargar los binarios desde la URL pública,
   sin credenciales, y comprueba todo otra vez.

Si un solo target falla, no se publica la release.

Si `publicar` se corta a medias, queda como mucho un borrador, que no ven los
demás. Para reintentar, borra el borrador (`gh release delete <tag>`, sin
`--cleanup-tag`, para conservar el tag) y relanza el run. Si la release ya
está publicada, relanzar `publicar` falla con «already exists» y no cambia
nada.

Los mismos pasos se pueden ejecutar en local:

```bash
cargo xtask empaquetar --target x86_64-pc-windows-msvc --salida dist
cargo xtask sumas dist
cargo xtask comprobar-tag v0.0.0
```

## El hash de referencia es el de la CI

Un build local no da el mismo sha256 que el del runner (TASK-016: rutas del
checkout distintas). El hash que vale es **siempre** el del `SHA256SUMS` de la
release, y `cli.lock` (F2-12) se rellena desde ese fichero, nunca desde un
build local.

## Cómo se verifica un binario descargado

El hash solo prueba que el fichero llegó entero. Que lo construyó este
repositorio con `release.yml` lo prueba la attestation.

Hash, en Linux:

```bash
sha256sum -c --ignore-missing SHA256SUMS
```

En macOS:

```bash
grep " tc2-<version>-<alias>$" SHA256SUMS | shasum -a 256 -c
```

En Windows (PowerShell), compara el resultado con la línea de `SHA256SUMS`:

```powershell
(Get-FileHash tc2-<version>-<alias>.exe -Algorithm SHA256).Hash.ToLower()
```

Procedencia, en cualquier sistema:

```bash
gh attestation verify tc2-<version>-<alias> --repo charliebk/TaskCodeTwo --signer-workflow charliebk/TaskCodeTwo/.github/workflows/release.yml
```

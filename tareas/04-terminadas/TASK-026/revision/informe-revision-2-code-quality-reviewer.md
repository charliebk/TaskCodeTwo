# Informe de revision — TASK-026 (ronda 2)

- Commit revisado: dedddf7567e00cff3b26cb454c4e8b9ac67e7822
- Revisor: code-quality-reviewer (general-purpose, haiku)
- Veredicto: cambios-solicitados

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | instalar/install.sh:226-231 | implementacion |
| IMP-2 | IMPORTANTE | corregido | instalar/install.sh:126-151 | implementacion |
| R2-1 | CRITICO | corregido | instalar/install.sh:135-136 | implementacion |
| R2-MEN-1 | MENOR | corregido | instalar/install.sh:124-136 | implementacion |
| R2-MEN-2 | MENOR | corregido | instalar/install.sh:146-150 | implementacion |

(Nota de quien orquesta: R2-1 lo introdujo la correccion de IMP-2. R2-MEN-1 y R2-MEN-2 venian en las «Notas» del revisor; se pasan a la tabla para que no se pierdan.)

## Entorno y comandos

- Clon de pruebas en `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-026-r2-calidad\repo`, situado en dedddf7. El repo original no se ha modificado. La carpeta se ha borrado al terminar.
- Contenedor `python:3.12-alpine` (Linux x86_64), con `python3 -m http.server 8765 --bind 127.0.0.1` y una release falsa: `tc2-0.0.0-linux-x64` (imprime "tc2 0.0.0") y su SHA256SUMS. Comando: `docker run --rm -v <work>:/work -v <repo>:/repo:ro python:3.12-alpine sh /work/run.sh`. `curl` no estaba disponible en la imagen (`apk add curl` falló), así que el instalador usó `wget` de busybox. Las instalaciones de prueba usan `HOME` y `TC2_INSTALL_DIR` propios; el PATH real no se ha tocado.
- Suite: `cargo xtask verificar --perfil rapida` en el clon, una sola vez. Resultado: EXIT=0, "xtask: verificacion Rapida en verde", todos los bloques de tests en ok, 0 fallos. Entre ellos, `instaladores::` con 13 tests ok, incluidos `cada_sistema_detectado_da_su_alias`, `tiene_palabra_solo_cuenta_palabras_enteras` e `instaladores_ascii_y_con_la_llamada_en_la_ultima_linea`.
- `probar-instalador.sh local http://127.0.0.1:8765 0.0.0 0.0.0`: EXIT=0 (VERDE en medida, hash alterado y path_idempotente). Incluye la reinstalación en otro directorio, que pasa.

## Verificacion de IMP-1 (caracteres en TC2_INSTALL_DIR): corregido

Con `TC2_INSTALL_DIR` en cada caso, sh install.sh:
- Rechazado con rc=1, sin crear `.profile` ni `tc2`: `a"b`, `a$b`, `` a`b ``, `a\b`, `a:b`, `a$(touch /tmp/pwned)b` (el comando inyectado no se ejecuta; `/tmp/pwned` no existe), un salto de linea, y `<dir>/tc2` si es un directorio.
- Aceptados, y una terminal nueva (`sh -l`) encuentra `<dir>/tc2`: `a'b`, `a b`, `a*b`, `a[b`, `a!b`, `a%b`, `a;b`.

## Verificacion de IMP-2 (reinstalar en otro directorio): corregido en el caso normal

Sobre un mismo `HOME`:
- Dos instalaciones en el mismo dir: un solo bloque.
- dirA, luego dirB: un bloque, sin rastro de dirA, y la terminal nueva encuentra `dirB/tc2`.
- Volver a dirA: un bloque, y la terminal encuentra `dirA/tc2`.
- Las lineas del usuario antes y despues del bloque se conservan.
- No quedan `.profile.tc2-tmp`.
- Un `.profile` que es enlace simbolico sigue siendo enlace, y el bloque se reescribe en su destino.
- Con `SHELL=/bin/bash`, `.profile`, `.bashrc` y `.bash_profile` tienen un bloque cada uno, con dirB.
- Con `TC2_NO_MODIFY_PATH=1`, el perfil no cambia (cksum igual).

## R2-1 (CRITICO, introducido por la correccion de IMP-2): abierto

Reproduccion. El perfil tiene la marca de inicio pero no la de fin (por ejemplo porque alguien la borro a mano):

```
# cabecera
# >>> tc2 (TaskCodeTwo) >>>
export PATH="<dirA>:$PATH"
# linea del usuario tras el bloque
alias ll="ls -l"
```

Se reinstala con `TC2_INSTALL_DIR=<dirB>`. La salida dice "quitado de ... el bloque de tc2 con otro directorio", y `alias ll` desaparece del perfil. Con el mismo directorio (control) no hay cambios y la linea sobrevive.

Causa: en `escribir_bloque` (install.sh:135), `awk '$0 == i { s = 1; next } $0 == f { s = 0; next } !s'` no comprueba que exista la marca de fin. Si no aparece, `s` queda a 1 hasta el final del fichero y todo lo posterior se descarta. Antes de la correccion ese caso salia con "ya tiene el bloque" sin tocar el perfil; la correccion es lo que lo abre. Hace falta un perfil con las marcas alteradas, por eso lo clasifico como CRITICO (perdida de datos) segun la plantilla; si el coordinador lo considera demasiado condicionado, puede bajarse a IMPORTANTE.

Arreglo sugerido: antes de reescribir, comprobar que hay exactamente una marca de inicio y una de fin (`grep -cxF`). Si falta alguna, fallar sin tocar el perfil, con un arreglo del tipo "quita a mano el bloque entre las marcas de tc2".

## Notas

- R2-MEN-1. Perfil con finales CRLF: las marcas no casan con `$0 == i` (comparacion exacta). El bloque viejo queda y se anade el nuevo: 2 bloques (dirA 1, dirB 1). Es un residuo de IMP-2 en ese caso. MENOR.
- R2-MEN-2. Cada reinstalacion en otro directorio deja una linea en blanco de mas antes del bloque, que se acumula. MENOR, cosmetico.
- install.ps1 (correcciones de la ronda 2): probadas las tres validaciones nuevas sin red y sin tocar el PATH. `TC2_BASE_URL=http://127.0.0.1:80@evil.example` da Fallo con arreglo; `TC2_INSTALL_DIR` con `;` da Fallo; `<dir>\tc2.exe` que es un directorio da Fallo. Las tres fallan antes de descargar. No hay hallazgos aqui. No se ha ejecutado una instalacion completa en Windows.
- Profundidad: verificacion acotada a IMP-1, IMP-2 y las correcciones de este diff. No se ha revisado la rama entera.
- `git status` del repo original muestra una entrada que no he creado; no la he tocado. La rama esta en ff07e6a, que solo anade la peticion; la revision es sobre dedddf7, como indica la peticion.

## Lecturas fuera del contexto permitido

- `.taskcode/config.yml` (lineas 1-30), para localizar `comando_verificacion`.
- Listado del directorio raiz del clon, para orientarme.

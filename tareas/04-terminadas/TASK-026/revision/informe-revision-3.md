# Informe de revision — TASK-026 (ronda 3)

- Commit revisado: 0274adfdab383318f050b9939c0d1f1503c62525 (el HEAD de la rama, 19833e3, solo añade la petición y el tarea.md: `instalar/` y `.github/` son idénticos a 0274adf)
- Revisor: code-quality-reviewer (general-purpose, sonnet)
- Veredicto: aprobada

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|

sin hallazgos nuevos. R2-1 queda verificado como **corregido** (IMP-1 e IMP-2 de la petición no se pedían en este encargo; no los he revisado de nuevo).

## Reproducción

Entorno: Git Bash + `python:3.12-alpine` en Docker. Los ficheros se pasaron por `tar` a stdin del contenedor, porque el volumen montado salía vacío con la ruta de `%LOCALAPPDATA%`. Dentro del contenedor, `python3 -m http.server 8765 --bind 127.0.0.1`, `TC2_BASE_URL=http://127.0.0.1:8765`, `TC2_VERSION=0.0.0`, `SHELL=/bin/sh` y un binario falso `tc2-0.0.0-linux-x64` con su línea en `SHA256SUMS`. Cada caso instala con `TC2_INSTALL_DIR` distinto del que lleva el perfil y compara `cksum` del `.profile` antes y después.

### R2-1: marcas alteradas, perfil intacto y error con instrucción

| Variante del perfil | Marcas detectadas | Resultado |
|---|---|---|
| inicio sin fin, con 3 líneas de la persona detrás | `1 0 desorden` | rc=1, perfil INTACTO |
| dos inicios y un fin | `2 1 orden` | rc=1, INTACTO |
| un inicio y dos fines | `1 2 orden` | rc=1, INTACTO |
| fin antes que inicio | `1 1 desorden` | rc=1, INTACTO |
| inicio con un espacio delante | `0 1 desorden` | rc=1, INTACTO |
| fin con un espacio detrás | `1 0 desorden` | rc=1, INTACTO |
| dos bloques completos | `2 2 orden` | rc=1, INTACTO |
| CRLF con inicio y sin fin | `1 0 desorden` | rc=1, INTACTO |
| inicio citado dentro de un comentario de la persona | `0 0 desorden` | rc=1, INTACTO |

Todos terminan con este mensaje:

```
tc2-instalar: ERROR: tc2 esta instalado en <dir>, pero el bloque de tc2 de <perfil> esta incompleto o repetido (marcas: 1 0 desorden); no se ha tocado
tc2-instalar: arreglo: quita a mano de <perfil> las lineas entre '# >>> tc2 (TaskCodeTwo) >>>' y '# <<< tc2 (TaskCodeTwo) <<<' (ambas incluidas) y repite, o repite con TC2_NO_MODIFY_PATH=1
```

El mensaje dice qué hacer. El binario ya está instalado y el texto lo dice ("tc2 esta instalado en ...").

**Fin sin inicio**: no hay grep del inicio, así que no entra en la reescritura. El perfil no pierde nada; se añade un bloque nuevo al final. Es seguro, pero ver la nota 1.

**Mutación**: con `if [ "$marcas" != "1 1 orden" ]` cambiado para que nunca dispare, `probar-instalador.sh local` sale `exit=1` con `ROJO: con la marca de fin borrada, el instalador no lo dijo`. La red de regresión discrimina.

### MENOR: perfil CRLF

Entrada: `# mio\r\n\r\n<inicio>\r\n...\r\n<fin>\r\n# mio tambien\r\n`.

Salida: un solo bloque, sin `/viejo`, con `# mio` y `# mio tambien` conservadas (con su `^M`) y el bloque nuevo al final. Corregido.

### MENOR: no se acumulan líneas en blanco

Cada perfil se reinstaló en 3 o 4 directorios distintos y el perfil queda idéntico entre reinstalaciones:

| Perfil inicial | Resultado tras 3-4 reinstalaciones |
|---|---|
| `a1`, 2 blancos, `b2` | idéntico: a1, 2 blancos, b2, 1 blanco y el bloque |
| vacío | 1 blanco y el bloque |
| `a1` sin salto de línea final | `a1`, 1 blanco y el bloque (se añade el `\n` final, inocuo) |
| `a1` y 2 blancos de la persona | 3 blancos antes del bloque, estable |
| bloque al principio del fichero | no queda blanco inicial de más |
| bloque con blancos de la persona antes y después | solo se quita el que está pegado al inicio |
| mismo directorio dos veces | cksum idéntico (`ya tiene el bloque de tc2`) |

### Búsqueda del fallo que abre la corrección

El awk que quita la línea en blanco previa solo descarta **un** blanco: el pegado a la marca de inicio. Los demás blancos de la persona se conservan, y los blancos dentro del bloque se descartan con el bloque. El perfil vacío o sin salto de línea final no rompe nada.

Único efecto colateral: el blanco inmediatamente anterior al bloque se quita siempre, aunque lo hubiera puesto la persona. En ese caso se pierde un blanco a cambio de que el instalador escriba otro delante. No hay pérdida de contenido (ver nota 2).

### Suite

- `cargo test -p xtask --locked` en clon limpio: 13 passed, 0 failed.
- `sh probar-instalador.sh local http://127.0.0.1:8765 0.0.0 0.0.0` en el contenedor, sirviendo `install.sh`, `install.ps1`, `SHA256SUMS`, el binario y `malo/` con el `SHA256SUMS` alterado: salió `exit=0`. Incluye `VERDE: el PATH queda una sola vez...` y `VERDE: instalador de unix (local)`, además de los dos casos nuevos de la ronda (sin fin y CRLF).
- No ejecuté `cargo xtask verificar --perfil rapida` entera.

## Notas (fuera de la tabla, sin abrir ronda)

1. Con un perfil que solo tiene la marca de fin (sin inicio), el instalador añade un bloque nuevo y deja la marca de fin huérfana. La siguiente reinstalación en otro directorio ve `1 2 orden` y se detiene con el error. No hay pérdida de datos; la persona tiene que arreglarlo a mano.
2. Si la persona escribió un blanco justo antes del bloque, el instalador lo trata como el suyo y lo quita al reescribir. Efecto estético.
3. Cuando las marcas no aparecen como línea exacta (con espacios o en un comentario), el mensaje dice "quita las líneas entre las marcas", que es poco útil si la marca está tocada. Es seguro, y el texto podría mencionar que las marcas deben ir solas en su línea.
4. El perfil CRLF recibe el bloque nuevo con finales LF (mezcla), que no daña a `sh`.

## Lecturas fuera del contexto permitido

Ninguna fuera de lo listado (`tarea.md`, `planificacion/` no hizo falta, `probar-instalador.sh`, `install.sh`). Sí leí `instalar/install.sh` entero (para el flujo de `main`) y ejecuté `xtask`, dentro del propósito de reproducir. La carpeta de prueba `%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-026-r3-calidad` (clon y servidores) se borró al terminar y no queda ningún contenedor en marcha.

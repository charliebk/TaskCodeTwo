# Plan — TASK-147: Auditoria 4D: A10 scripts de CI e instaladores (4 IMPORTANTE)

(Es la respuesta del unico rol de brainstorm, arquitectura,
volcada aqui por quien orquesta.
Con un solo rol no hay unificador ni desacuerdos que resolver; en su lugar,
el plan senala lo que ese rol no cubrio.)

## Enfoque propuesto

Duracion estimada: 120 min (de la rama abierta al cierre con merge; senales: 3 ficheros de codigo (dos scripts de CI y `release.yml`), 2 capas (scripts de CI y workflow), 5 criterios, arneses de shell)

Regla de una hora: pasa de 60 min, pero por la consigna de Carlos del 2026-11-23 (modo automatico, opcion recomendada) la tarea se hace sin trocear.

Arreglos pequenos, cada uno en el fichero del fallo; sin modulo nuevo. Un commit por ID:

1. **A10-01 y A11-01** (mismo patron, juntos):
   - `.github/scripts/probar-actualizar.sh:60`: `! cmp -s "$tmp/srv/y/SHA256SUMS" "$tmp/srv/malo/SHA256SUMS" || rojo "el SHA256SUMS alterado es igual al bueno"` (usa el `rojo()` de `:34-37`).
   - `.github/workflows/release.yml:284`: `! cmp -s Cargo.toml Cargo.toml.orig || { echo "::error::el sed no cambio la version de Cargo.toml ($VERSION -> $vy)"; exit 1; }`.
   - `release.yml:214` no se toca: es la ultima orden del paso y ahi si falla (contraste del arnes A11).
2. **A10-02** (`.github/scripts/probar-instalador.sh`):
   - `linea_unix` (`:45-51`): la tuberia `curl|sh` se queda (es la linea real que se mide); la descarga se envuelve en `{ curl ... || : >"$tmp/descarga-fallo"; } | ... sh`, se llama a `rojo` si aparece la marca y se devuelve el codigo de `sh` recogido con `if` (para que `set -e` no corte). Igual en la rama `wget`.
   - `sin_version` (`:268-287`): con `codigo` 0 exige primero el binario (`$tmp/home-ultima/.local/bin/tc2` o `$tmp/dest-ultima/tc2.exe`, como en `:165` y `:210`); despues que `--version | tr -d '\r'` sea `tc2 <tag de /releases/latest>`, sacando el tag de la redireccion de `${url%/releases/download/*}/releases/latest` con `curl -fsSLI -o /dev/null -w '%{url_effective}'` o, sin curl, `wget -S --spider` (como `install.sh:67-76`).
   - Cabecera `:12-14` actualizada si cambia lo que se promete.
3. **A10-05**: NO se reproduce en esta tarea (decision de Carlos: sin Docker, y sin BusyBox wget en la maquina no hay reproduccion fiel sin red externa). Queda anotado como pendiente en el `## Resultado` y en el backlog, sin cambiar `instalar/install.sh`.
4. **Criterio 5 sin push** (decision de Carlos): en vez de ver `release.yml` en el CI de la rama, se ejecutan en local los scripts del job: `probar-actualizar.sh` (si su entrada local se puede montar) y el modo `local` de `probar-instalador.sh` en Git Bash, mas `cargo test -p xtask` (lint de workflows e instaladores). El `## Resultado` dice que el CI de la rama no se ejecuto.

## Reutilizacion (Regla 4D)

- `rojo()` de `probar-actualizar.sh:34-37` y de `probar-instalador.sh:33-36`.
- Patron binario + version de `probar-actualizar.sh:83-85` y `probar-instalador.sh:165`, `:210` (dos lineas, cada script tiene el suyo; no se crea helper compartido entre scripts).
- Consulta de la ultima version como en `install.sh:63-89`: no se puede cargar desde el test (`main "$@"` en la ultima linea, comprobado por `xtask/src/instaladores.rs:129`), asi que el test hace su propia consulta de una linea.
- Arneses de TASK-138 (`tareas/04-terminadas/TASK-138/seguimiento/arneses/a10/d4-cmp.sh`, `a10/d4-sin-version.sh`, `a11/d4-release-284.sh` con `release-281-285.txt`): se copian a `TASK-147/seguimiento/arneses/` con la forma corregida; la carpeta terminada no se toca.

Nuevo: nada salvo las copias corregidas de los arneses.

## Lo que el rol no cubrio

- Pruebas, riesgos y dominio no se lanzaron como roles: modos de fallo de GNU wget frente a BusyBox, y si `/releases/latest` puede apuntar a un tag con sufijo.
- `sin_version` solo corre en modo `publico` (`release.yml:438-440`, con tag): su evidencia es el arnes.
- `linea_windows` (`irm | iex`) tiene el mismo riesgo y no se toca (fuera del criterio).

## Riesgos aceptados y que los contiene

- Lo nuevo de `sin_version` no corre de verdad hasta la proxima release: lo contiene el arnes, que da rojo sin red.
- Sin CI en la rama (sin push): lo contienen la ejecucion local de los scripts y `cargo test -p xtask`; queda declarado en el Resultado.
- A10-05 sin reproducir: queda abierto en el backlog; nada se instala sin pasar el sha256 (`install.sh:277-280`), lo que acota el impacto.
- `release.yml:214` sigue con `! cmp -s` sin `||`: vale mientras sea la ultima orden del paso.

## Plan de pruebas

Tests nuevos por criterio (presupuesto de 1 a 3 por criterio; si se pasa, el motivo):

- A10-01: copia de `d4-cmp.sh` con la linea corregida, sobre dos ficheros iguales: codigo distinto de 0 y sin `SIGUE`. Mutante: quitar `|| rojo ...` → `SIGUE` con 0.
- A11-01: copia de `d4-release-284.sh` con `release-281-285.txt` sacado de la `release.yml` corregida: sin `SIGUE` y codigo distinto de 0. Mutante: quitar `|| { ...; exit 1; }` → `SIGUE`.
- A10-02: (a) copia de `d4-sin-version.sh` sobre el `sin_version` nuevo: rojo «no se instalo»; mutante: volver al `if` sin el `else` → verde. (b) `linea_unix` contra `http://127.0.0.1:1`: rojo «no se pudo bajar»; mutante: quitar la marca → sale 0. (c) binario con otra version: rojo; mutante: quitar la comparacion.
- A10-05: sin test (pendiente).
- Criterio 5: scripts del job en local y `cargo test -p xtask`, con target propio. Ningun test pesado.

## Lo que necesita decision de una persona

1. Arreglo de A10-05 si se reproduce: recomendada comprobar las cabeceras `-S` en la rama wget.
2. Origen de la version esperada en `sin_version`: recomendada la consulta propia a `/releases/latest`.
3. Donde reproducir A10-05: recomendado Docker local.
4. Regresion de A10-05 en CI: recomendado no; basta el arnes.
5. Criterio 5 exige push para ver el CI de la rama.

Respuestas de la persona (Carlos, 2026-11-25, preguntado en esta fase; el resto por la consigna del 2026-11-23: modo automatico con la opcion recomendada): 5) sin push, prueba local de los scripts y lo anoto en el Resultado; 3) sin Docker, A10-05 queda sin reproducir y se anota como pendiente (con ello 1 y 4 no aplican); 2) consulta propia a `/releases/latest`.

# Informe de revision — TASK-026 (ronda 1)

- Commit revisado: 6609149e64b240c697b4eba4036d5e8006865279
- Revisor: code-quality-reviewer (general-purpose, sonnet)
- Veredicto: cambios-solicitados

(Nota de quien orquesta: el revisor propuso «aprobada-con-correcciones», pero
con IMP-1 e IMP-2 abiertos la regla del flujo exige «cambios-solicitados», y
asi se escribe con `taskctl veredicto`.)

Alcance: pase genérico, sin revisor de dominio. Profundidad: a fondo en install.sh y install.ps1; arnés y workflow solo leídos, sin ejecutar en CI.

Reproducido en clon propio de la rama (HEAD a5ab364; contra 6609149 solo cambian ficheros de tareas, no los instaladores).
- install.sh: en alpine (wget de busybox) con servidor local en 127.0.0.1, y con curl desde Git Bash con un shim de `uname`.
- install.ps1: en powershell.exe 5.1, con un `tc2.exe` falso compilado con csc, `TC2_NO_MODIFY_PATH=1` y `TC2_INSTALL_DIR` dentro de la carpeta de pruebas. No se tocó el PATH real. Por eso Add-PathUsuario (el registro) solo se revisó leyendo, sin ejecutarlo.
- No ejecuté la suite Rust: la tarea no toca Rust y la petición no trae verificación de la ronda.

Funciona bien y reproducido: hash alterado (sale con 1, no queda nada instalado, mensaje con el arreglo), SHA256SUMS sin línea o con dos (error, no instala), binario que sale con 1 o que es basura (no se instala, no queda `.tc2.nuevo`), versiones `1.0/../x`, `1.0;id` y `v` (rechazadas), TC2_BASE_URL `http://evil.com` o `ftp://` (rechazada), TC2_BASE_URL sin TC2_VERSION (error), reinstalar con el mismo directorio (un solo bloque en `.profile`), ruta relativa (pasa a absoluta), directorio de Windows con corchetes, y `tc2.exe` bloqueado por un proceso en marcha (a los 6 s da error con arreglo y no deja restos).

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | instalar/install.sh:140-147 | implementacion |
| IMP-2 | IMPORTANTE | corregido | instalar/install.sh:121-124 y 148-165 | implementacion |
| MEN-1 | MENOR | corregido | instalar/install.sh:176-181 e instalar/install.ps1:789 | implementacion |
| MEN-2 | MENOR | corregido | instalar/install.sh:226-231 y 134-146 | implementacion |
| MEN-3 | MENOR | corregido | .github/scripts/probar-instalador.sh:197-209 | implementacion |

### IMP-1 — TC2_INSTALL_DIR se escribe sin escapar en el perfil del shell

`escribir_bloque` mete el directorio dentro de `case ... *":%s:"*` y de `export PATH="%s:$PATH"`. Una `"`, un `$` o un backtick en el directorio rompe el perfil o ejecuta código en cada shell de login.

```
TC2_INSTALL_DIR='/h/g/x$(touch /h/g/PWNED2)y'  (HOME=/h/g, instalación rc=0)
$ env -i HOME=/h/g PATH=/usr/bin:/bin sh -l -c 'echo ok'
ok
$ ls /h/g
PWNED2
x$(touch

TC2_INSTALL_DIR='/h/f/a"b$(touch /h/f/PWNED)'
.profile:  case ":$PATH:" in *":/h/f/a"b$(touch /h/f/PWNED):"*) ;; *) export PATH="/h/f/a"b$(touch /h/f/PWNED):$PATH" ;; esac
$ env -i HOME=/h/f PATH=/usr/bin:/bin sh -l -c 'echo ok'
sh: syntax error: unexpected ";;" (expecting ")")
```

El instalador sale con 0 y deja el perfil roto o con código ejecutable. La fuente es una variable de entorno de la propia persona, por eso lo clasifico IMPORTANTE y no CRITICO. Aun así es entrada externa escrita como código en un fichero que se carga siempre, y un `$` o una comilla en una ruta real basta para romper todos los shells.

Arreglo posible: rechazar `"`, `$`, backtick y `\` en el directorio antes de instalar, o escribir el valor escapado con comillas simples.

### IMP-2 — Reinstalar con otro directorio dice "instalado" pero no queda en el PATH

`escribir_bloque` se salta el perfil si ya tiene la marca, sin mirar si el bloque contiene el directorio nuevo.

```
TC2_INSTALL_DIR=/h/e/A  -> anadido /h/e/A al PATH en /h/e/.profile
TC2_INSTALL_DIR=/h/e/B  -> /h/e/.profile ya tiene el bloque de tc2
                           abre otra terminal (o ejecuta: export PATH="/h/e/B:$PATH") ...
                           instalado: /h/e/B/tc2 (tc2 0.0.0)
$ env -i HOME=/h/e PATH=/usr/bin:/bin sh -l -c 'command -v tc2'
/h/e/A/tc2        <- el binario viejo; B no está en el PATH
```

La persona queda con el binario antiguo, y el mensaje "abre otra terminal" es falso. Ocurre al pasar de un directorio personalizado al de por defecto, o al revés. install.ps1 sí lo gestiona, porque compara por directorio.

Arreglo posible: comprobar que el bloque contiene el directorio, o añadir un bloque por directorio.

### MEN-1 — La validación de TC2_BASE_URL se salta con `usuario@host`

Los patrones `http://127.0.0.1:*` y `http://localhost:*` (sh), y `(:|/)` en la regex de PowerShell, aceptan un `@` que cambia el host real.

```
TC2_BASE_URL=http://127.0.0.1:80@evil.invalid
  sh/wget:  wget: bad address 'evil.invalid'  (intentó conectar a evil.invalid)
TC2_BASE_URL=http://localhost:@evil.invalid
  powershell 5.1: "No se puede resolver el nombre remoto: 'evil.invalid'"
```

Debería rechazarse con el mensaje de "https, o http a 127.0.0.1 o localhost". La variable es solo para ensayos, así que no hay riesgo para quien instala; el fallo es que el guard no protege lo que dice. Arreglo posible: exigir que tras el puerto venga `/` o el final de la cadena, y rechazar `@` en el host.

### MEN-2 — Casos borde de install.sh que dan resultado engañoso

- Con HOME sin definir y TC2_INSTALL_DIR puesto, el binario se instala y después `escribir_bloque "$HOME/.profile"` aborta por `set -u`:
  ```
  /tmp/i.sh: line 148: HOME: parameter not set
  rc=2
  ```
  Sin línea `ERROR`/`arreglo`, contra lo que dice instalar.md:540. El binario ya está en su sitio.
- Si `$destino/tc2` es un directorio, `mv -f` mueve el fichero dentro. El instalador imprime `instalado: .../tc2 (tc2 0.0.0)` con rc=0, y deja `tc2/.tc2.nuevo.<pid>`. `tc2` no queda instalado.

Basta comprobar `[ -d "$destino/tc2" ]` y un `${HOME:-}` explícito con su error.

### MEN-3 — La prueba de hash alterado acepta cualquier fallo

`hash_alterado` solo exige que la línea salga con código distinto de 0 y que no quede binario. Un fallo cualquiera (404 por falta de `malo/`, red, sintaxis) lo pasa en verde. No mira que el error sea el del sha256 ("no casa con SHA256SUMS").

Se queda sin discriminar la causa, aunque la instalación normal sí falla si se quita la comparación. Arreglo: capturar la salida y exigir `*"no casa con SHA256SUMS"*`.

## Lecturas fuera del contexto permitido

Ninguna. Solo leí los ficheros listados (con los instaladores desde mi clon de la rama) y el resto de la petición. Con la petición no tuve el diff de `.github/scripts/humo.sh`: leí solo la línea `imagen=` que usa el workflow.

Carpeta de pruebas borrada (`%LOCALAPPDATA%\pruebas-claude\TaskCodeTwo\task-026-r1-calidad`), tras comprobar que no había commits ni cambios que conservar. Paré mis servidores en los puertos 8771-8773, no los de otros procesos.

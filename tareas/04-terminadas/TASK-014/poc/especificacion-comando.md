# Especificacion unica del comando de la PoC de F1-14

Las dos implementaciones (Rust en `poc-rust/`, TypeScript en `poc-ts/`) cumplen esta misma especificacion y su salida por stdout tiene que ser identica byte a byte.

## Invocacion

```
tablero <raiz-del-repo> [--sin-red]
```

## Comportamiento

1. Recorre `<raiz>/tareas/**/tarea.md` (cualquier profundidad). Orden: por ruta relativa con `/`, comparacion de bytes UTF-8.
2. De cada `tarea.md` lee el frontmatter YAML entre la primera linea `---` y la siguiente `---`, con una libreria YAML estandar (no un parser propio). Si el YAML tiene una **clave duplicada**, la tarea cuenta como error (`clave duplicada`); si no parsea, error (`yaml invalido`); si no hay frontmatter, error (`sin frontmatter`).
3. De cada tarea valida toma `id` (texto), `titulo` (texto) y `estado` (texto; si falta, `desconocido`).
4. Ejecuta `git -C <raiz> log -1 --format=%H` como proceso hijo (no libgit2) y toma la linea.
5. Si no se pasa `--sin-red`, hace un GET anonimo HTTPS a `https://api.github.com/zen` con cabecera `User-Agent: taskcodetwo-poc`; solo cuenta si respondio 2xx (`si`/`no`). Con `--sin-red` no hace la peticion y escribe `omitida`.
6. Salida (LF, UTF-8, sin espacios finales):

```
tareas: <N validas>
errores: <M>
<estado>: <n>          (una linea por estado, orden alfabetico de bytes)
...
git: <hash completo>
red: si|no|omitida
error <ruta relativa>: <motivo>   (una linea por error, en el orden del paso 1)
```

7. Sale con 0 si no hay errores de E/S propios del programa (los errores de tareas no cambian el codigo de salida).

## Reglas

- Solo dependencias mantenidas; anota cada una con version y estado (ultima publicacion) en `DEPENDENCIAS.md` de tu carpeta.
- Nada de runtime externo en el binario final: Rust compilado en release; TypeScript empaquetado como ejecutable unico con Node SEA (Node 22 instalado), de modo que se ejecute con `node` fuera del PATH.
- Anota en `NOTAS.md`: minutos, iteraciones de compilar y corregir, lineas de codigo propias y problemas encontrados.

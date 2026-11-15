# MENORES pendientes — TASK-016

MENORES abiertos de la ronda 1 que no se corrigieron en la verificacion ligera (`taskctl verificar-menores`). Son backlog de la tarea: no abren otra ronda ni impiden `finish`.

| ID | Fichero | Motivo |
|---|---|---|
| MEN-1 | .cargo/config.toml:7 | remap-path-prefix: no hace falta (mismo sha256 en dos rutas distintas sin el, medido en local y CI) y no se puede fijar estatico en .cargo/config.toml por depender de la ruta del checkout; se anota en el Resultado para F2-10. rapida con tests: decision deliberada para que finish y el freno de la 0.16 corran los tests (3 s); se anota en el Resultado. |
| MEN-2 | .taskcode/config.yml:5 | remap-path-prefix: no hace falta (mismo sha256 en dos rutas distintas sin el, medido en local y CI) y no se puede fijar estatico en .cargo/config.toml por depender de la ruta del checkout; se anota en el Resultado para F2-10. rapida con tests: decision deliberada para que finish y el freno de la 0.16 corran los tests (3 s); se anota en el Resultado. |
| MEN-4 | .cargo/config.toml:7 | remap-path-prefix: no hace falta (mismo sha256 en dos rutas distintas sin el, medido en local y CI) y no se puede fijar estatico en .cargo/config.toml por depender de la ruta del checkout; se anota en el Resultado para F2-10. rapida con tests: decision deliberada para que finish y el freno de la 0.16 corran los tests (3 s); se anota en el Resultado. |

# Informe de revision — TASK-015 (ronda 2)

- Commit revisado: dd39c4108a460d33676e2d283ea2012b370bd381
- Revisor: code-quality-reviewer
- Veredicto: aprobada-con-correcciones

Alcance: pase generico, verificacion acotada al diff de las correcciones
(ca2d75f frente a 91a8078): `docs/adr/F1-13-adaptador-de-plataforma.md`,
`docs/adr/README.md`, `poc/fuentes.md`, `poc/poc-f1-13.py` y su log. No se
han vuelto a revisar los MENOR de la ronda 1 mas alla de comprobar que sus
correcciones estan en el ADR.

## Hallazgos

| ID | Severidad | Estado | Fichero | Origen |
|---|---|---|---|---|
| IMP-1 | IMPORTANTE | corregido | docs/adr/F1-13-adaptador-de-plataforma.md:302 | implementacion |
| IMP-2 | IMPORTANTE | corregido | docs/adr/F1-13-adaptador-de-plataforma.md:294 | implementacion |
| MEN-6 | MENOR | abierto | tareas/03-en-revision/TASK-015/poc/poc-f1-13.py:209 | implementacion |
| MEN-7 | MENOR | abierto | docs/adr/F1-13-adaptador-de-plataforma.md:294 | implementacion |
| MEN-8 | MENOR | abierto | docs/adr/README.md:23 | implementacion |

Sin hallazgos nuevos CRITICOS ni IMPORTANTES: las correcciones no abren
ningun fallo de los que bloquean.

### IMP-1 — G09 en GitLab (corregido)

Fichero `docs/adr/F1-13-adaptador-de-plataforma.md:302` (fila G09). La fila
pasa a tier `ninguno`, sustituto `autor-del-estado` en todos los tiers (en
Ultimate, el check externo con «Status checks must succeed» es una defensa
mas, no el origen) y cita `api/status_checks` (L29, nueva en `fuentes.md`).
Contraste en solo lectura hoy (`curl` de la pagina, Tier Ultimate): el token
tiene que pertenecer a un usuario con rol Developer, Maintainer u Owner en
el proyecto destino, y la llamada se ejecuta como cualquier usuario con
derecho a aprobar el MR. Coincide con la celda.

Cifras recalculadas, coherentes en todos los sitios: GitLab 11 Free (G02,
G07, G11, G13 a G20), 6 Premium (G03 a G06, G10, G12), 3 `ninguno` (G01,
G08, G09), 0 Ultimate; capacidades `gitlab free` `11 / 9 / 0`, `gitlab
premium` y `gitlab ultimate` ambas `17 / 3 / 0`; el resumen (l.77), la prosa
(l.321) y la fila de `docs/adr/README.md` dicen 11 / 6 / 3. `grep` de
`gitlab-ultimate`, `18 / 2` y «1 en Ultimate»: sin restos en docs. Mutacion
propia sobre copia: G12 de Premium a Free hace fallar tier, capacidades,
resumen y prosa; `g09-tier-ultimate` del log sigue fallando.

### IMP-2 — Evidencia empirica que no prueba nada (corregido)

Fichero `docs/adr/F1-13-adaptador-de-plataforma.md:294` (columna de
evidencia; reglas bajo la tabla). G07, G11, G19 y G20 pasan a `documental`
y se quitan los ids que eran un 401 o 403 anonimo o un 200 sin relacion con
la fila (GL-05, GL-16, GL-18, GL-20, GL-21, GL-22, GL-23, GL-04, GL-01,
GH-14). Recuento de filas empiricas, a mano: G01, G02, G03, G06, G08, G09,
G10, G12, G14, G15, G16, G17, G18 = **13**; coincide con la prosa, con
`PRUEBA` y con el calculo de la PoC. `PRUEBA` (poc-f1-13.py, bajo `GH_PRIV`
y antes de `PLANES_GH`) liga cada id a las filas para las que vale y a un
predicado sobre lo observado. Mutaciones propias (copia del repo con
`--solo-adr`, todas fallan por su motivo):

- G17: `GH-05, GL-13` por `GH-05, GL-14`: falla («GL-14 no prueba nada de
  esta garantia»).
- G12 y G15: `GL-09` por `GL-08`: falla en las dos filas.
- `observado.json` con GH-12 `aviso_de_plan: false`: falla G01 y G03 («no
  tiene en lo observado el hecho que lo hace prueba»); el predicado
  discrimina, no solo la lista de ids.
- Prosa «3 garantias… en ningun tier» por «2»: falla (frase y cifra).

## Reproduccion

- **PoC completa** (`python -I -B`, carpeta
  `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-015-r2\`): sale 0, «todo
  como se espera», 38 GET reales, 11 rechazos del envoltorio sin red (0
  peticiones, 0 procesos `gh`, 0 en el mock). Comparada con `poc-f1-13.log`
  (sin `\r`): igual salvo la seccion V (limites de peticiones y el hash de
  GL-11). Las 25 mutaciones del log (las 23 de la ronda 1 mas
  `ids-de-g15-cambiados` y `empirica-con-401`, y `g09-tier-ultimate`)
  salen «falla como se espera». La carpeta de trabajo se borro al terminar.
- **Cifras de tabla, prosa, resumen e indice**: GitHub 10 por plan de los
  rulesets (G01 a G09 y G11), 9 sin plan (G12 a G20), cola (G10) con
  organizacion; GitLab 11 / 6 / 0 / 3; 13 empiricas; 44 URL en `fuentes.md`
  (D01 a D15 y L01 a L29, contadas); 11 rechazos. Coherente.
- **Cita de cuentas de servicio** (`docs.gitlab.com/user/profile/service_accounts/`,
  hoy): «They do not use a seat», no son usuarios facturables, y en
  GitLab.com Free hasta 100 por grupo de nivel superior (subgrupos y
  proyectos incluidos); GA en 18.11. Coincide con la celda de G12, con la
  decision 6 y con L12.
- **Prueba negativa propia de la lista blanca** (importando el guion y
  llamando a `Lector._guardia`, sin red: 0 peticiones, 0 procesos `gh`):
  rechaza `https://api.github.com/user`, `http://127.0.0.1:5555@evil.example`,
  otro puerto local, `localhost`, `gitlab.com.evil.example`, `users/1` de
  GitLab, `file://`, los metodos `Get` y `HEAD`, y con `gh` `/user`, `-X`,
  `graphql`, `repos/CharlieBK/x` y una ruta con salto de linea y
  `--method=POST`. Ver MENOR-A: dos casos con `..` pasan.
- `git status` limpio al terminar; solo se escribio este informe y la carpeta
  temporal, ya borrada.

## Notas (no abren ronda)

- **MENOR-A — La lista blanca deja pasar `..`** (`poc/poc-f1-13.py:209` y
  `:212`). `RUTA_GH` admite `.` y `/` en los segmentos y `destino_permitido`
  solo mira el prefijo: `repos/charliebk/../../user` y
  `https://gitlab.com/api/v4/projects/278964/../1` pasan la guardia. Sigue
  siendo GET, la PoC no los emite (destinos constantes) y no hay escritura
  posible, pero la prosa del ADR («solo rutas `repos/charliebk/…`», «solo
  la API v4 de GitLab.com para el proyecto 278964») deja de ser cierta con
  `..`. Correccion barata: rechazar segmentos `..` y `%2e` y anadir un caso
  NEG, o suavizar la frase.
- **MENOR-B — GH-03 como evidencia de G01 y G02** (`:294-295`). Su
  predicado es solo `200` con `[]`: prueba que el endpoint lee, no que
  `deletion`, `non_fast_forward` o `pull_request` existan en un publico
  gratuito. Es el defecto de IMP-2 en pequeno, atenuado porque GH-12 (403
  con aviso de plan) prueba la otra mitad. Dejarlo o decirlo en la celda.
- **MENOR-C** — la fila de F1-13 en `docs/adr/README.md` es una sola linea de
  mas de 900 caracteres; legible solo con ajuste de linea.

## Lecturas fuera del contexto permitido

- `poc/poc-f1-13.py`, `poc/poc-f1-13.log`, `poc/fuentes.md`, `poc/fixtures/`
  y la peticion y el informe de la ronda 1: los pidio el encargo para
  reproducir y contrastar.
- `docs/plan/REQUISITOS-FUNCIONALES.md` y el resto de `docs/` (copiados a la
  carpeta temporal): los lee la PoC en la seccion G.
- `https://docs.gitlab.com/api/status_checks/` y
  `https://docs.gitlab.com/user/profile/service_accounts/`: solo lectura.

> Nota del orquestador: MEN-6 (la lista blanca deja pasar `..`/`%2e`), MEN-7 (GH-03, un `200 []`, no prueba G01 ni G02 por si solo) y MEN-8 (fila del indice de mas de 900 caracteres) son las notas MENOR-A, B y C del revisor, pasadas a la tabla para corregirlas.

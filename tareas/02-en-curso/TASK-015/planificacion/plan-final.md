# Plan — TASK-015: F1-13 ADR adaptador de plataforma GitHub y GitLab

(Respuesta del unico rol de brainstorm, arquitectura, volcada tal cual en lo sustancial. Con un solo rol no hay unificador: el plan dice que lados no reviso ese rol.)

## Enfoque propuesto

Duracion estimada: 180 min

Mas ancha que la mediana de un ADR de la Fase 1 (45,5 a 50 min, F1-18) y cerca de F1-14 (210 estimados): unas 20 garantias × 2 plataformas × 3 tiers, cada celda verificada contra la documentacion oficial con URL y fecha; GET reales en las dos plataformas; PoC del adaptador contra un servidor local. **No se trocea.**

### Decisiones ya tomadas por la persona (Carlos, 2026-11-14)

1. **Nombres aprobados**: CLI `tc2`, plugin `taskcodetwo`, rama `taskcodetwo/tareas`. F1-5 y F1-6 dicen `taskcode/tareas` y `taskcode-*`: divergencia anotada, sin editar esos ADR.
2. **Comandos deterministas antes que hooks.** Seguridad y trabajo en grupo imprescindibles: identidades separadas (agente, revisor, persona; F1-6), permisos minimos por identidad en cada plataforma, interferencia de companeros (dos personas o agentes en el mismo MR/PR, aprobaciones de terceros).
3. **Sin trocear y fiel a los datos**: cada equivalencia verificada contra la documentacion oficial de GitHub y GitLab, en solo lectura, con URL y fecha; empiricamente donde se pueda.
4. **Restricciones de la PoC**: GitHub y GitLab solo lectura (sin crear repos, proyectos, rulesets, Apps ni tokens; sin POST/PUT/PATCH/DELETE). Permitido: GET anonimos o con el token que ya tiene `gh` sobre repos propios de Carlos (p. ej. `charliebk/TaskCodeTwo`), GET anonimos a GitLab.com sobre proyectos publicos, `glab` solo lectura, mock local. Temporales en `%LOCALAPPDATA%\pruebas-claude\taskcodetwo\task-015\`, borrados al terminar. Sin descargar toolchains ni imagenes.
5. **«Que depende del plan de pago»** se mide con las paginas oficiales de precios y tiers (GitHub Free/Pro/Team/Enterprise; GitLab Free/Premium/Ultimate), con fecha; lo no confirmado, «no verificado».

### Herramientas en esta maquina (por ruta, sin ejecutar)

- `gh`: `C:\Program Files\GitHub CLI\gh.exe`, con sesion (`hosts.yml`). **No es un token de solo lectura**: la lectura la garantiza el envoltorio de D6.
- `glab`: `%LOCALAPPDATA%\Programs\glab\glab.exe` (PATH y sesion sin comprobar).
- Python 3.12 y Rust (F1-14).
- La implementacion empieza con `where`, `--version`, `gh auth status` y `glab auth status` en el log.

### Donde encaja: cuatro piezas, dos se extienden y dos se crean

1. **Puerto `Plataforma` de F1-4** (se extiende): lista cerrada de F1-4 mas las operaciones de F1-5 y F1-6 (`abrir-pr`, `estado-puertas`, `merge-pr`, `aplicar-puertas`, `leer-puertas`, `leer-permisos-actions`). Para cada operacion, su equivalente en GitLab (MR, approvals, pipelines, ramas protegidas por API REST v4) con el mismo contrato de salida y codigos 0–7. Sin equivalente → `rechazada motivo=plataforma` o `motivo=plan`, nunca exito vacio. HTTP con el cliente de F1-14 (`ureq`, TLS del sistema), sin `glab` ni `gh` (P12/RR-16).
2. **Modelo neutral de puertas de F1-5** (se extiende): traduccion a GitLab destino a destino (`develop`, `main`, ramas de tarea, `taskcodetwo/tareas`, tags `v*`). Punto delicado: los checks con origen (`integration_id` en GitHub) pueden degradarse en GitLab a «el pipeline entero debe pasar»; el ADR lo verifica y declara el sustituto, con su efecto en F1-8 (quien controla `.gitlab-ci.yml` en un MR).
3. **Tabla de equivalencias como dato** (se crea), entre marcadores `<!-- tabla-equivalencias:inicio/fin -->`, columnas `garantia | regla neutral | GitHub (API) | tier GitHub | GitLab (API) | tier GitLab | sustituto si falta | evidencia (doc URL+fecha; GET id del log | no verificado)`. Filas minimas a verificar: (1) sin borrado ni force-push; (2) solo por PR/MR; (3) N aprobaciones de persona; (4) descartar aprobaciones en cada push; (5) aprobacion del ultimo push / impedir aprobacion del autor y de quien anade commits; (6) propietarios de codigo; (7) solo merge commit; (8) rama al dia; (9) checks obligatorios con origen / pipeline obligatoria / checks de estado externos; (10) cola / tren de merge; (11) tags protegidos; (12) identidad del agente (App / token de proyecto / cuenta de servicio); (13) identidad del revisor y como publica; (14) firma verificada del commit de `approve`; (15) reviews de persona y no de bot; (16) token de la CI sin poder aprobar; (17) lectura de checks/pipelines por sha (cache de F1-7); (18) diff con renombrados (F1-10); (19) pipelines de forks y secretos protegidos (F1-8); (20) runner propio por etiqueta y runners protegidos (F1-7, F1-8).
4. **`capacidades` para `doctor`** (se crea el contrato; lo implementan F3-8, F2-6, F2-9):
   ```
   capacidades(plataforma: GitHub|GitLab, plan_observado: Plan, puertas: PuertasLeidas) -> Vec<Garantia>
   Garantia { id, estado: impuesta|sustituida(por)|no_disponible, motivo: plan|plataforma|config, arreglo }
   ```
   `doctor` imprime las garantias no impuestas (5 lineas max., F1-4) en cada ejecucion (RR-15, RF-64). El plan se **observa** (403/404 de plan de cada API), no se declara. Plataforma por el host de `origin`, con clave de config para self-managed (D8).

**Se crea**: `docs/adr/F1-13-adaptador-de-plataforma.md`, su fila en el indice y la PoC (`poc-f1-13.py`, fixtures JSON, `fuentes.md`, log de GET). **No se toca**: ADR F1-1..F1-18, el plan, pilotos y remotos; las correcciones van a «Divergencias» con el texto propuesto.

### Coste que fija la forma

`estado-puertas` en GitLab puede necesitar mas lecturas que en GitHub; el ADR cuenta llamadas por operacion en cada plataforma (status line RR-19 y `finish`). La cache de F1-7 en GitLab seria un GET de pipelines por sha y nivel: se cuenta, no se optimiza.

### Orden de construccion

1. Inventario y guardia: envoltorio unico de solo lectura (D6) que rechaza todo lo que no sea GET y registra metodo, URL, estado, hash del cuerpo y fecha; prueba negativa: un POST se rechaza antes de salir a la red.
2. Documentacion: por fila, pagina oficial de GitHub, de GitLab y de precios/tiers, con URL, fecha y cita corta en `fuentes.md`.
3. GET reales: GitHub con `gh api --method GET` sobre `charliebk/TaskCodeTwo` (rulesets, `rules/branches/develop`, proteccion, check-runs, reviews de un PR existente, permisos de Actions); GitLab.com anonimo sobre un proyecto publico (proyecto, ramas protegidas, MR, approvals, pipelines por sha, diffs, firma). Un 401/403 anonimo tambien es evidencia.
4. Mock local en `127.0.0.1` con las respuestas grabadas y fixtures para lo no observable (tier sin aprobaciones, sin tren, token de bot); adaptador minimo por plataforma con el mismo `EstadoPuertas` neutral y la misma `capacidades`: misma bateria para las dos (P11).
5. Escritura: tabla, decision, «Como se ve en GitHub y en GitLab», divergencias, indice y borrado de temporales.

### Motivo para no trocear

Consigna de Carlos; equivalencias, tiers y `capacidades` salen de la misma tabla.

## Lo que el rol no cubrio

- Criterio 1: sin escribir en ninguna plataforma, el comportamiento de las puertas (bloquear un merge, descartar aprobaciones, tren) no se verifica empiricamente: esas filas quedan «documental» y pasan como criterio a F2-6/F2-9; ningun tier de pago se verifica empiricamente.
- Criterio 2: forma cubierta (misma bateria contra mock, E2E en F2-6/F2-9); sin criterio de cobertura revisado por testing.
- Criterio 3: RR-15, P11, RF-54, RF-64, RN-01 y las cesiones de F1-4..F1-8 y F1-10; no se disenan la plantilla de GitLab CI (F5-1) ni la creacion real de tokens y bots (F6-1, F3-1).
- Riesgos sin enumerar: documentacion que cambie tras la fecha citada; GitLab.com frente a self-managed; `.gitlab-ci.yml` del MR controlando sus propios checks; token de proyecto que caduca; limites de peticiones en la status line; interferencia en un MR compartido.
- Dominio: si un sustituto cuenta como la misma garantia de RR-15 o como garantia degradada declarada.
- Rendimiento: solo se cuentan llamadas.

## Riesgos aceptados y que los contiene

- Token de `gh` con escritura: envoltorio solo GET con prueba negativa y log revisable; `gh` no se usa en otro sitio.
- Tiers solo por documentacion: URL y fecha por celda; `doctor` observa el plan real.
- Proyecto publico ajeno de GitLab: solo GET anonimos; en el repo solo hashes, el cuerpo en la carpeta temporal.
- Sesgo hacia GitHub: cada fila primero como regla neutral; «sustituida» admitido en las dos columnas.

## Plan de pruebas

`poc-f1-13.py` (`python -I -B`, log determinista; precedente F1-11, F1-12, F1-14, F1-18) lee la tabla del ADR y sale con 1 si una fila no tiene URL y fecha por plataforma, si una fila «empirica» no tiene id de llamada del log, si una URL no esta en `fuentes.md` o si falta o sobra alguna de las 20 garantias. Copias mutadas del ADR fallan cada una por su motivo; tiempo de reloj a stderr.

- **Criterio 1 → 3 pruebas**: tabla completa con evidencia por celda; el log solo contiene GET y el envoltorio rechaza un POST sin salir a la red; las celdas de tier coinciden con la cita de la pagina de precios.
- **Criterio 2 → 2 pruebas**: la misma bateria contra el mock da el mismo `EstadoPuertas` neutral en las dos (verde, `esperando` 7, sin aprobacion, aprobacion de bot que no cuenta, `rechazada motivo=plan`, `sin-cambios`); `capacidades` da una linea por garantia no impuesta con su sustituto.
- **Criterio 3 → 1 prueba**: tabla requisito → decision → evidencia (RR-15, P11, RF-54, RF-64, RN-01, F1-4..F1-8, F1-10) con IDs comprobados y fila del indice con enlace que resuelve; al cerrar, `taskctl doctor` sin errores y temporales borrados.

## Lo que necesita decision de una persona

Respuestas (Carlos, consigna «opcion recomendada»): **D1a, D2a, D3a, D4a, D5a, D6a, D7a, D8a**.

- **D1. Forma del adaptador**: **(a, recomendada)** puerto `Plataforma` de F1-4 con dos implementaciones y `capacidades` como dato que lee `doctor`; (b) superficies distintas por plataforma; (c) minimo comun denominador de los planes Free.
- **D2. Cuando el plan no da una garantia**: **(a, recomendada)** sustituto declarado en la tabla, `init` sigue y `doctor` lo dice siempre; (b) `init` rechaza; (c) solo avisar.
- **D3. Alcance de GitLab**: **(a, recomendada)** GitLab.com y self-managed con la misma tabla por tier, verificado solo en GitLab.com (self-managed «no verificado»); (b) solo GitLab.com.
- **D4. Identidad del agente en GitLab**: **(a, recomendada)** token de proyecto (bot) con rol Developer si el tier lo da; si no, cuenta de servicio o usuario dedicado como sustituto; (b) token personal (descartada: rompe RN-01).
- **D5. Lenguaje de la PoC**: **(a, recomendada)** Python `-I -B` con fixtures JSON reutilizables por F2-6/F2-9 en Rust; (b) Rust ya.
- **D6. Guardia de solo lectura**: **(a, recomendada)** envoltorio unico solo GET con log y prueba negativa; (b) disciplina.
- **D7. Divergencias**: **(a, recomendada)** el ADR las anota con el texto propuesto; (b) editarlas aqui.
- **D8. Deteccion de plataforma**: **(a, recomendada)** por el host de `origin` con clave explicita para self-managed, fallo cerrado si es ambigua; (b) siempre clave explicita.

## Suposiciones no verificadas

- `glab` en el PATH y su sesion.
- Que GitLab.com deje leer anonimamente ramas protegidas, approvals y firmas de un proyecto publico (si da 401, evidencia solo documental).
- Tiers de GitLab.com para tokens de proyecto, reglas de aprobacion obligatorias, reinicio de aprobaciones, CODEOWNERS obligatorio, trenes y checks de estado externos.
- Si en GitLab un estado de commit publicado por un token bloquea el merge sin depender del pipeline (pregunta abierta de F1-6).
- Si la cola de merge de GitHub exige organizacion y en que planes (pregunta de F1-5).

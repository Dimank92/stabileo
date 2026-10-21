# Fallos de detallado y estrategia de búsqueda del diseño

Auditoría del 21 de octubre de 2026 sobre PR536, revisión `8509b272d`, con el ejemplo
`rc-design-frame` y su declaración lateral por defecto. El problema principal es la distancia
entre las alternativas que explora el diseño y las geometrías que finalmente construye.
La prioridad es cerrar esa distancia y medir todos los bloqueos físicos antes de ampliar
presupuestos de búsqueda.

## Corrección de la medición

La revisión auditada mostraba 84 miembros `ok` o `warn`, pero 12 todavía participaban en
conflictos no marginales: 6, 10, 12, 14, 44, 72, 73, 87, 88, 92, 93 y 108.
`repairConflicts` omitía de `unresolved` los pares para los que no intentaba movimientos:
estribos/trabas y barras fijadas. La verificación del miembro leía las condiciones abiertas
producidas desde `unresolved`, por lo que esos conflictos detectados no siempre lo bloqueaban.

La corrección conserva los conflictos bloqueantes aunque no haya movimiento disponible.
No cambia ninguna barra ni tolerancia. El benchmark también cruza los aprobados contra
`assembly.conflicts`, de forma independiente de las condiciones abiertas.

| Medición del mismo modelo | Antes de mejorar las semillas | Con semillas mejoradas |
| --- | ---: | ---: |
| Aprobados que mostraba la aplicación | 80 | 84 |
| Aprobados sin conflictos físicos bloqueantes | 70 | 72 |
| Conflictos físicos bloqueantes | 2987 | 2981 |

La comparación de 80 a 84 sobreestimaba el progreso. Se conservan los 70 aprobados sin
conflictos anteriores y se agregan las columnas 32 y 34. Las columnas 12 y 14 mejoraron sus
empalmes, pero todavía tienen interferencias entre piezas transversales. La regresión comprueba
las identidades de los 72 y exige que ningún aprobado participe en un conflicto bloqueante.

Con el reporte corregido: 72 aprobados, 288 fallidos, 35 no disponibles por búsqueda agotada y
13 provisionales. Las condiciones `clash.unresolved` pasan de 1818 a 3323 porque ahora exponen
también los conflictos antes omitidos; no son interferencias nuevas. Este total incluye
conflictos marginales que el algoritmo ya había intentado reparar; los bloqueantes son 2981.

## Causas medidas

Los 276 fallidos originales tenían bloqueos de detallado: 197 vigas y 79 columnas. De ellos,
272 no figuraban entre los fallos de `run.reverification`. El ciclo de reparación selecciona
exclusivamente esa segunda lista (`design-feedback-loop.ts`, `failingRecords`), por lo que no
explora nuevos armados para la mayoría de los problemas geométricos. Tras corregir el reporte,
hay 284 de los 288 fallidos fuera de esa selección.

Los conflictos físicos bloqueantes se distribuyen en categorías excluyentes:

| Par que entra en conflicto | Cantidad |
| --- | ---: |
| Dos barras declaradas como pareja de empalme | 541 |
| Otras dos barras longitudinales | 935 |
| Barra longitudinal y pieza transversal | 1411 |
| Dos piezas transversales | 94 |
| Total | 2981 |

Las condiciones de empalme se cuentan por separado y pueden afectar a los mismos miembros:
97 empalmes superpuestos, 42 condiciones de integridad, 9 de separación, 4 de distancia entre
parejas y 2 de falta de pareja. No deben sumarse como miembros distintos.

Casos que permiten reproducir las causas:

- **Vigas 178 y 179:** barras superiores de un empalme separadas 3 mm entre ejes frente a los
  13 mm que requieren sus radios. El planificador acepta una transición de contacto;
  `lap-materialize.ts` extiende sus segmentos, pero no construye el desplazamiento transversal
  necesario. Declarar una pareja no elimina la interpenetración que detecta `classifyPair`.
- **Viga 207:** una barra de la segunda capa inferior intersecta el cierre de estribos;
  un par medido tiene 4,75 mm de interpenetración. El defecto se repite a lo largo de la viga.
  Regenerar la jaula con otro cierre o asiento es una operación distinta de desplazar una capa.
- **Columnas 12 y 14:** el cambio de armado resuelve su congestión longitudinal, pero permanece
  el choque entre la traba y el estribo. Es el ejemplo de por qué hace falta evaluar toda la jaula.
- **Integridad en el nudo 26:** el comprobador registra cuatro empalmes superiores en el apoyo
  y ninguna barra continua. Se necesita decidir continuidad y zona de empalme a escala de línea.
- **Columna 27:** barras que comienzan en el siguiente piso sin pareja inferior. La elección
  independiente por tramo no asegura una transición realizable entre cantidades y posiciones.

La búsqueda inicial agotada en 35 miembros tiene estas causas: congestión sola (21), anclaje
solo (8), ambos (3), anclaje/corte/congestión (2) y flexión (1). Agotar las alternativas actuales
no demuestra que no exista un armado adecuado. Los 13 provisionales tienen una comprobación
biaxial no implementada; ampliar la búsqueda no sustituye ese verificador.

## Qué hace la búsqueda actual y dónde se detiene

`coordination-search.ts` ya tiene propagación de restricciones, programación dinámica para
cadenas y búsqueda acotada para el resto. En la ejecución auditada produjo 1735 candidatos,
eliminó 981 por propagación, evaluó 3070 transiciones de programación dinámica y visitó un nodo
de búsqueda. No agotó el presupuesto. El resultado fue `PARTIAL_ENVELOPE_EXHAUSTED`, con
`beamLayoutsOnly`: 84 dominios se vaciaron y recibieron una alternativa de menor interferencia.
Hay 213 asignaciones, pero no constituyen una solución físicamente verificada.

El materializador produjo 964 empalmes y 647 fusiones, sin transiciones marcadas como
`unmaterialised`. Los 541 conflictos entre parejas de empalme muestran que haber producido
geometría no demuestra que esa geometría sea aceptable.

El ciclo posterior corrigió 33 fallos de reverificación y terminó en dos iteraciones. Quedaron
cuatro miembros aceptados con fallos resistentes al considerar la geometría final, más los 13
provisionales. Estos últimos aparecen como `FEEDBACK_LOOP_TRUNCATED` pese a que
`stats.truncated` es falso: se omiten al no tener `accepted` y reciben después una clasificación
por defecto. Esa clasificación debe distinguir falta de cobertura del verificador y presupuesto.

Otros límites del algoritmo, identificados en el código, no cuantificados como causa en este caso:

- Los hashes rechazados sobreviven entre geometrías; una alternativa rechazada en un entorno
  puede funcionar cuando cambian los vecinos. El rechazo debe depender del contexto que lo prueba.
- La monotonía del costo por miembro puede impedir una reparación más barata que además mejora
  el detalle. Para evitar ciclos bastan estados visitados y presupuestos; el costo no es una prueba.
- Una propuesta que empeora el detalle detiene el lote completo. Conviene probar alternativas
  por conjunto de miembros relacionados, conservando siempre una solución previa coherente.

## Costo de cálculo

Medición instrumentada de una ejecución local; tiempos inclusivos, sin valor de SLA ni conclusión
estadística sobre otras máquinas. Los tiempos de planta y búsqueda están contenidos en el detallado.

| Fase | Llamadas | Tiempo acumulado |
| --- | ---: | ---: |
| Diseño inicial | 1 | 3,99 s |
| Detallado completo | 2 | 6,70 s |
| Coordinación física por planta | 16 | 4,17 s |
| Búsqueda combinatoria de posiciones | 2 | 0,064 s |

La mejora de rendimiento más prometedora es evitar regeneraciones y verificaciones físicas
completas para candidatos incompatibles de antemano. Cambiar el solver combinatorio no ataca el
costo dominante observado. La reparación física sólo despejó 16 conflictos netos en la última
pasada completa; las propuestas restantes se rechazaron principalmente por invadir otras barras
(3863 intentos), salir de la sección (1602) o reducir la altura útil (357).

## Algoritmo propuesto

1. **Unificar el diagnóstico.** Producir restricciones incumplidas con miembros, barras, nudo,
   magnitud, regla, geometría y operación aplicable. La cola debe incluir resistencia, interferencias,
   continuidad, empalmes y jaulas. Separar verificaciones no implementadas y datos faltantes.
2. **Generar alternativas construibles.** Ofrecer pocas jaulas completas, diversas y verificadas
   localmente: posiciones, diámetros, capas, cierres, radios y trabas. Para columnas, considerar
   la transición con el piso vecino. Para vigas, mantener un armado continuo de referencia a lo
   largo de la línea y agregar refuerzos donde los requieren las demandas.
3. **Planificar transiciones con geometría testigo.** La compatibilidad debe devolver un plan que
   el materializador pueda ejecutar: parejas, desplazamientos, longitudes, zona de empalme y
   continuidad. Comprobar separación respecto de terceros y cierres; no sólo longitud y distancia
   máxima entre parejas. Rechazar el plan si su geometría no satisface los mismos verificadores finales.
4. **Conservar la propagación y la programación dinámica.** Resolver cadenas de tramos y líneas
   con tablas de compatibilidad entre candidatos completos. Mantener alternativas suficientes para
   los cruces; una decisión óptima de una línea no debe fijarse si tiene restricciones pendientes con otra.
5. **Reparar conjuntos de miembros relacionados.** Elegir un nudo conflictivo, sus vigas y los
   tramos de columna relevantes; reconsiderar conjuntamente sus elecciones. Expandir el conjunto si
   no mejora. Esto es una aplicación propuesta de búsqueda de vecindarios grandes o LNS: liberar una
   parte de la solución y reconstruirla, conservando el resto. Usar orden estable y límites por operaciones.
6. **Aceptar progreso sobre el resultado completo.** Conservar el mejor estado y sus certificados.
   Priorizar cobertura de verificaciones y número de miembros sin fallos; después, restricciones
   incumplidas y magnitudes, y por último acero y simplicidad. Ninguna reducción de acero compensa
   un fallo. Las exploraciones internas pueden pasar por alternativas peores sin publicarlas; una
   actualización visible debe mantener los miembros ya aprobados y superar la validación completa.
7. **Recalcular por dependencias.** Invalidar barras, empalmes, jaulas y vecinos alcanzados por el
   cambio, no sólo el miembro editado. Incluir propagación entre pisos y a lo largo de líneas. Memoizar
   por materiales, demandas, edición, candidato y geometría de sus vecinos. Una pasada completa
   final debe comprobar que el cálculo incremental coincide con el resultado independiente.

La LNS está descrita por [Pisinger y Røpke](https://backend.orbit.dtu.dk/ws/portalfiles/portal/5293785/Pisinger.pdf).
Aplicarla a estas jaulas es una propuesta de arquitectura, no un resultado demostrado para este modelo.
Un solver de restricciones como [CP-SAT](https://developers.google.com/optimization/cp/cp_solver)
podría elegir candidatos discretos a partir de tablas de compatibilidad. Requiere representación entera;
los verificadores no lineales y geométricos seguirían siendo oráculos externos. No hay evidencia en esta
medición que justifique agregar esa dependencia antes de corregir candidatos y transiciones.

## Secuencia y criterios de aceptación

La corrección del reporte y la regresión independiente son el primer paso implementado. El siguiente
incremento debería atacar los empalmes entre diámetros distintos y la continuidad superior, por su alcance
de 135 miembros fallidos en los conflictos de parejas. En paralelo conceptual, los cierres de estribos
requieren un generador de jaulas que compruebe todas sus capas. Estos incrementos deben medirse por separado
para atribuir sus efectos; no se estima una cantidad de aprobados futura sin ejecutar los verificadores.

Antes de ampliar al modelo completo, reproducir cada causa en un nudo o dos tramos. Probar cambios de
diámetro, cantidad, capa, sección y orientación, y restricciones de usuario. Verificar que trasladar/rotar
una geometría o renombrar IDs no altera el veredicto físico. Comparar cálculos incrementales y completos,
invalidación de caché y determinismo con presupuestos iguales. En el conjunto de fixtures, conservar los
miembros aprobados, medir nuevas aprobaciones y bloqueos pendientes, llamadas a verificadores, regeneraciones,
memoria y tiempos p50/p95 tras separar inicialización WASM. No promover un caso aislado a garantía general.

## Reproducción

Desde `web/`, con el WASM de esta rama compilado:

```sh
STABILEO_DESIGN_AUDIT=/tmp/design-audit.json npx vitest run --project unit src/lib/store/__tests__/design-quality-audit.test.ts
npx vitest run --project unit src/lib/store/__tests__/design-quality-progress.test.ts
```

El primer comando guarda geometrías, conflictos, estados, decisiones de búsqueda y tiempos; es opt-in
y no impone umbrales de tiempo en CI. El segundo exige progreso físico real y que no haya aprobados con
conflictos bloqueantes.

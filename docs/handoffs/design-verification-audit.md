# Fallos de detallado y estrategia de búsqueda del diseño

Auditoría del 21 de octubre de 2026 sobre PR536, revisión `8509b272d`, con el ejemplo
`rc-design-frame` y su declaración lateral por defecto. El problema principal es la distancia
entre las alternativas que explora el diseño y las geometrías que finalmente construye.
La prioridad es cerrar esa distancia y medir todos los bloqueos físicos antes de ampliar
presupuestos de búsqueda.

## Ampliación: continuidad superior e inferior por líneas completas

La búsqueda por parejas dejaba tramos fuera del reemplazo: una unión ya fusionada no tiene
registro de empalme. Los nuevos operadores `harmonizeTopLine` y `harmonizeBottomLine` usan
las líneas del coordinador y las transiciones materializadas, comprueban conectividad y
colinealidad y verifican todos sus miembros como un paquete. Incluyen uniones ya continuas;
no incorporan vigas cruzadas, solapadas ni ramas ambiguas. Los catálogos se intersectan y se
preservan las terminaciones de cada miembro, incluidos los ganchos inferiores explícitos.

También se corrigió la medida de avance: una condición agrupada puede representar ocho
parejas mal colocadas. Pasar de ocho a dos es una mejora real aunque siga abierto un aviso.
Se cuenta esa reducción sin alterar la verificación física ni las protecciones contra regresión.
Los ensayos ahora registran qué miembro perdió resistencia o qué categoría de bloqueo aumentó.

### Resultado medido en los 408 miembros

| Etapa | Aprobados finales | Conflictos bloqueantes |
| --- | ---: | ---: |
| Estado anterior comunicado (`e1993c32d`) | 76 | 1986 |
| Parejas con avance contado por barras pendientes | 77 | 1982 |
| Continuidad superior por líneas completas | 81 | 1641 |
| Continuidad inferior por líneas completas | 84 | 1542 |

La mejora acumulada desde 1986 es de **444 conflictos bloqueantes (22,4%) y ocho aprobaciones**.
Se conservan las 76 identidades y se agregan las columnas 72, 105, 110, 120, 126, 130, 131 y 135.
Son 81 columnas y tres vigas aprobadas: todavía no se sumaron vigas completamente aprobadas en
este edificio. Se mantienen 276 fallidos, 35 búsquedas iniciales agotadas y 13 provisionales.
Ningún aprobado participa en un conflicto bloqueante. Las condiciones agrupadas de empalme
superpuesto bajan de 95 a 77, y las de integridad de 41 a 31; separación, distancia entre parejas
y falta de pareja permanecen en 9, 4 y 2. Hay dos condiciones de generación pendientes.

La etapa superior realizó 44 detallados y 188 verificaciones nominales; la inferior, 63 y 223.
Ambas completaron el barrido de los patrones existentes y ejecutaron cero análisis estructurales
nuevos. No es una prueba de agotamiento del espacio de diseño: faltan familias de barras nuevas,
transiciones con desplazamiento construido, cambios de jaulas de columnas y cambios de sección.
Los tiempos de una ejecución local fueron 121,72 s y 174,92 s para esas etapas, respectivamente.
La integración completa consumió 395,71 s dentro del test; estos tiempos no son un SLA.
La repetición final con geometría guardada reprodujo exactamente 84 aprobados y 1542 conflictos:
365,09 s dentro del test, 368,10 s totales y 153,59 s en la etapa inferior.

La geometría retenida permite particionar los 1542 bloqueos sin duplicar pares:

| Par que entra en conflicto | Cantidad |
| --- | ---: |
| Dos barras registradas como pareja de empalme | 349 |
| Otras dos barras longitudinales | 701 |
| Barra longitudinal y pieza transversal | 398 |
| Dos piezas transversales | 94 |

Los 276 miembros aún fallidos son 197 vigas y 79 columnas; todos tienen condiciones de detallado
abiertas. Esta partición pertenece al estado final de 1542, no al diagnóstico anterior de 1992.


### Verificación y límites

- Siete casos pequeños con solver real: dos tramos con rotación/traslación, extremos invertidos
  y renumeración, y tres tramos con continuidad superior, extremos invertidos y continuidad
  inferior. Todos cierran sin condiciones abiertas ni conflictos bloqueantes. En tres luces
  iguales de 6 m se contrasta el momento con wL²/10 = 122,49 kN·m. Se comprueban barras compartidas
  en cada unión, estribos presentes y separados como máximo 125 mm, y regeneración estable.
- Regresiones de reemplazo atómico, presupuesto para toda la línea, comprobación de todos los
  certificados, continuidad ya fusionada, rechazo de cruces/solapes y avance dentro de un aviso
  agrupado. Las siete suites dirigidas suman 165 pruebas aprobadas.
- La integración del edificio conserva los aprobados por identidad y regenera exactamente los
  mismos veredictos y conflictos sin ejecutar la búsqueda. Fija además los 84 aprobados y los
  1542 conflictos como regresión de la secuencia completa.
- Typecheck sin errores nuevos: 442 existentes frente a una base de 443.

Reproducción desde `web/`, con WASM release:

```sh
npx vitest run --project unit src/lib/engine/detailing/__tests__/beam-continuity-integration.test.ts
STABILEO_CONTINUITY_EXPERIMENT=80 STABILEO_LINE_EXPERIMENT=1 npx vitest run --project unit src/lib/store/__tests__/design-continuity-experiment.test.ts
STABILEO_CONTINUITY_EXPERIMENT=64 STABILEO_BOTTOM_EXPERIMENT=1 npx vitest run --project unit src/lib/store/__tests__/design-continuity-experiment.test.ts
```

`STABILEO_CONTINUITY_AUDIT=/tmp/design-continuity-audit.json` guarda además miembros, geometría,
empalmes, conflictos y reverificación final. Los operadores continúan **opt-in**; su integración
al diseño habitual y la regeneración incremental siguen pendientes. El diseño completo aún no
verifica: 1542 bloqueos no son una solución final, y los patrones comunes no reemplazan el
trabajo necesario sobre transiciones y jaulas de los nudos.

## Actualización: diagnóstico después de las reparaciones de diámetros y filas

Ejecución del 22 de octubre sobre `93ed6fa70`, con instrumentación de diagnóstico local:
8 ensayos físicos de familias de diámetros y luego 8 de redistribución inferior. Se guardaron
los estados completos antes y después del segundo operador. Los números de las secciones
históricas siguientes corresponden al diseño anterior; los resultados para esta secuencia son
los de la tabla. El primer incremento de continuidad, documentado más abajo, bajó los bloqueantes
de 1992 a 1986 y conserva estos mismos estados de miembros:

| Estado final | Miembros |
| --- | ---: |
| Aprobados, incluidos 9 con advertencias | 76 |
| Fallidos con condiciones de detallado abiertas | 284 |
| Búsqueda inicial agotada, sin armado aceptado | 35 |
| Provisionales por verificación biaxial no cubierta | 13 |
| Total | 408 |

Los 76 aprobados son 73 columnas y 3 vigas. Los 284 fallidos son 87 columnas y 197 vigas:
280 pasan la reverificación resistente y 4 también fallan en ella. Por tanto, aumentar acero
de forma general no responde al bloqueo dominante. Los 35 agotados se reparten en congestión
(21), anclaje (8), ambos (3), anclaje/corte/congestión (2) y flexión (1). Esto describe el espacio
explorado, no demuestra imposibilidad de diseñar esas secciones.

### Geometría pendiente

Los **1992 conflictos bloqueantes** se particionan sin duplicar pares:

| Tipo de par | Conflictos |
| --- | ---: |
| Parejas registradas de empalme | 534 |
| Otras barras longitudinales | 765 |
| Longitudinal y pieza transversal | 599 |
| Dos piezas transversales | 94 |

La partición cruza `lapping.laps` por IDs de ambas barras y, para los demás pares, `bar.role`.
Se excluye `severity === 'marginal'`. No es un conteo de miembros: un miembro participa en muchos
conflictos y varias categorías. Persisten además 97 condiciones agrupadas de empalmes superpuestos,
42 de integridad, 9 de separación, 4 de distancia entre parejas y 2 de falta de pareja; estas
condiciones afectan respectivamente a 153, 68, 9, 4 y 2 miembros, con solapamientos.
Las 97 condiciones pueden representar varias parejas cada una; no equivalen a 97 pares físicos.

Casos confirmados en el estado final:

- **Vigas 178–179:** el empalme superior conserva 3 mm entre ejes, con 13 mm necesarios para
  que no se interpenetren. Prolongar las barras no construye el desplazamiento transversal.
- **Nudo 26, vigas 177–178:** cuatro empalmes superiores y cero barras continuas. El comprobador
  deja abierta la condición de integridad; no se resuelve cambiando sólo las filas inferiores.
  Es una condición conservadora no acreditada, no una prueba general de que todo empalme allí sea ilegal.
- **Columna 21:** separación libre de 14,4 mm frente a los 40 mm exigidos por el comprobador.
- **Columna 26:** pareja de empalme separada 184,2 mm frente al límite calculado de 150 mm.
- **Columna 27:** barras del tramo superior sin pareja inferior.
- **Vigas 177 y 189:** el generador sigue dejando abierta la condición de barra longitudinal
  en los dobleces del estribo (`generation`, referencia 25.7.1.2).

### Por qué la viga 227 mejora pero no verifica

La modificación conserva 11 barras inferiores Ø12 y pasa de **8+3 a 7+4**. Su reverificación
resistente sigue en `ok`, pero los conflictos físicos evolucionan así:

| Tipo | Antes | Después |
| --- | ---: | ---: |
| Longitudinal–transversal | 80 | 0 |
| Otras longitudinales | 16 | 20 |
| Parejas de empalme | 6 | 8 |
| Total bloqueante de la viga | 102 | 28 |

Quedan empalmes superiores con la viga 228: una pareja tiene **4,4 mm entre ejes frente a
18 mm necesarios**, y también separaciones insuficientes con barras inferiores de los tramos
vecinos. Aparecen seis choques de barras inferiores con barras de las columnas **46 y 47**,
con interpenetración de 16 mm. Ambas columnas ya estaban bloqueadas antes del cambio.

Esto expone una limitación del objetivo actual: conserva los miembros aprobados y reduce
conflictos globales, pero puede introducir nuevos pares conflictivos entre miembros que ya
fallaban. El descenso neto de 74 es real; no significa que cada restricción local mejore.
Tampoco los 80 choques eliminados representan 80 causas independientes: un mismo problema de
posición se repite a lo largo de los estribos. Conviene agrupar por causa geométrica y medir
qué falta para completar cada miembro/nudo, además del número bruto de pares.

### Límites de la búsqueda y siguiente intervención

El operador nuevo sólo redistribuye barras inferiores de una viga, con diámetro y número total
constantes. No cambia empalmes superiores, continuidad de línea ni transiciones de columnas.
La cola se ordena por número de conflictos y no incluye miembros que sólo tienen condiciones
de continuidad/empalme sin choques. Se detiene por presupuesto tras ocho ensayos; esto no es
una prueba de agotamiento de alternativas.

La propuesta para la viga 225 bajaba el total de 2066 a 1948, pero bloqueaba la columna 43,
previamente aprobada, y se rechazó correctamente. Reparar una viga por vez limita la búsqueda
cuando hace falta modificar conjuntamente el nudo y los vecinos. La búsqueda de posiciones
tampoco encontró una solución dentro de su dominio: reporta 62 nudos incompatibles y 80
eventos de vaciado de dominio sobre 43 miembros distintos, sin agotar su presupuesto de
cómputo. Su dominio es `beamLayoutsOnly`, con columnas y arreglos de nudo incompletos.

Los cuatro fallos resistentes restantes son las vigas 192 y 195 (utilización final 1,029),
y 262 y 265 (1,005). Sus registros incluyen una tolerancia desfavorable de altura útil de
15 mm. La reparación encuentra límites de anclaje en el primer par, y de acero máximo/anclaje
en el segundo; emite consejo de sección para este último. Los límites de reparación no deben
confundirse con la categoría exacta del chequeo que produjo cada utilización.

La siguiente intervención prioritaria es construir y comprobar conjuntamente las transiciones
de empalme y la continuidad superior de una línea, empezando por dos tramos reproducibles.
Después, reparar el conjunto viga–columnas de un nudo y sus cierres. Cada aceptación debe
conservar los aprobados y demostrar que completa restricciones, no sólo elimina muchas
repeticiones de un choque. Los 13 provisionales necesitan cobertura biaxial del verificador;
seguir probando jaulas no sustituye esa implementación.

### Reproducción y validación de esta actualización

Desde `web/`:

```sh
STABILEO_LAYER_SEARCH=8 STABILEO_LAYER_AUDIT=/tmp/design-layer-audit npx vitest run --project unit src/lib/store/__tests__/design-layer-search-experiment.test.ts
```

Guarda `.before.json` y `.after.json` con miembros, resultados, geometría, empalmes y conflictos.
La ejecución pasó **1/1 test**, incluido regenerar el estado aceptado sin volver a ejecutar la
búsqueda: 71,18 s de test y 74,19 s totales. Reprodujo los mismos IDs aprobados, ensayos y 1992
conflictos de las ejecuciones anteriores. La etapa de filas llevó 25,40 s. Es una ejecución
local satisfactoria posterior a los dos timeouts documentados, no una garantía de rendimiento.

## Primer incremento del plan: continuidad superior por parejas

Implementado el 22 de octubre, después de la auditoría anterior. El operador `harmonizeTop`
lee transiciones reales con condiciones de empalme/integridad pendientes y propone patrones
superiores comunes para ambos tramos y sus dos extremos. Reutiliza hasta cuatro patrones ya
presentes, dentro del catálogo y el espacio de ambas secciones. No modifica el armado inferior,
los estribos ni las terminaciones; excluye grupos superiores explícitos, continuidad editada,
miembros fijados, provisionales y vigas compuestas por varios elementos de análisis.

Cada propuesta se evalúa como un paquete: se actualizan los asientos de ambos vecinos, se
verifican los dos armados y se reconstruye el detalle completo antes de aceptar. También se
exige reverificación resistente favorable para ambos tramos y que disminuyan las condiciones
del nudo objetivo. Los ganchos pedidos pero todavía no provistos se rechazan: una advertencia
que pide construir un gancho no es una aprobación de la geometría actual. Se conservan las
protecciones globales y las aprobaciones por identidad de las búsquedas anteriores.

### Caso completamente resuelto

Fixture de dos luces de 6 m, sección 35×65 cm, H25, cargas de 12 kN/m permanentes y 8 kN/m
variables, más peso propio. Los apoyos declaran sus seis restricciones; el momento del apoyo
intermedio calculado por el solver coincide con el valor independiente wL²/8 = 153,1125 kN·m.
El armado inicial, 7Ø12 superiores en un tramo y 4Ø16 en el otro, pasa el chequeo nominal pero
deja cuatro interferencias y condiciones de empalme e integridad en el apoyo.

La búsqueda genera un patrón común y deja **ambos miembros con resistencia verificada, cero
conflictos bloqueantes y cero condiciones abiertas**. Se comprueba que existe el número correcto
de barras superiores compartidas, que los estribos siguen presentes con separación ≤125 mm,
y que regenerar los armados aceptados conserva el cierre. Pasa también al rotar/trasladar el
modelo, invertir sus extremos y renumerar los elementos; se compara el veredicto, no se exige
que dos alternativas igualmente válidas produzcan la misma selección.

El fixture expuso otro defecto: en apoyos sin ancho de columna, las dos jaulas automáticas
colocaban sus estribos extremos en el mismo plano. Ahora cada una reserva hacia su tramo el
espacio de su cierre: un diámetro transversal, que cubre medio diámetro de desfase del cierre
y el radio de la barra. Es una elección geométrica, no una nueva exigencia normativa. Se
mantienen las estaciones prescritas de líneas explícitas y el comportamiento en apoyos con
ancho de columna; las interferencias continúan midiéndose sobre las piezas reales.

### Resultado en el edificio de 408 miembros

Después de las búsquedas de diámetros y filas, con ocho ensayos completos adicionales:

| Medida | Antes | Después |
| --- | ---: | ---: |
| Aprobados finales | 76 | 76, mismas identidades |
| Conflictos bloqueantes | 1992 | 1986 |
| Condiciones agrupadas de empalme superpuesto | 97 | 95 |
| Condiciones de integridad | 42 | 41 |

Se aceptaron las propuestas de los pares 211–212 y 178–179; cambiaron los armados de 212 y 179.
Hubo diez propuestas, veinte llamadas al verificador, ocho detallados completos y cero nuevos
análisis estructurales. Se mantienen 35 búsquedas agotadas, 13 provisionales y el resto de condiciones;
no hay nuevas aprobaciones completas en este edificio. El operador no resuelve todavía las
transiciones que requieren cambios de posición, de columnas o de toda una línea.

Validación: **154 tests en siete archivos**, más la integración de 408 miembros, todos pasaron.
La integración incluye persistencia y regeneración sin volver a ejecutar la búsqueda, con
igualdad de estados, aprobados y conflictos. Su última ejecución llevó 101,80 s totales;
la etapa nueva, 30,26 s. El typecheck no agrega errores: 442 existentes frente a una base de 443.
Estos son tiempos de una ejecución local, no un compromiso de rendimiento.

Reproducción desde `web/`:

```sh
npx vitest run --project unit src/lib/engine/detailing/__tests__/beam-continuity-integration.test.ts
STABILEO_CONTINUITY_EXPERIMENT=8 npx vitest run --project unit src/lib/store/__tests__/design-continuity-experiment.test.ts
```

El operador está disponible en `physicalSearch.operators: ['harmonizeTop']`; sigue siendo opt-in.
Se puede combinar con los operadores previos, compartiendo presupuesto. La integración al
flujo automático general sigue siendo el paso 6 del plan; este incremento prueba una primera
familia de soluciones construibles y no declara terminado ese plan.

## Plan de implementación para cerrar diseños

Objetivo: encontrar armados íntegramente verificables para casos construibles dentro del
catálogo y las restricciones del proyecto, y distinguir los casos que requieren ampliar ese
dominio, cambiar sección o completar un verificador. No asumir que los 408 miembros actuales
admiten solución con sus secciones fijas. El objetivo final es resolver el modelo completo;
los 76 aprobados actuales constituyen un piso de regresión para las reparaciones de armado.

### 1. Casos con solución conocida y criterios de aceptación

Antes de cambiar el algoritmo, construir fixtures pequeños a partir de las causas medidas:
dos vigas con diámetros distintos (178–179), empalme superior 227–228, nudo 26 con continuidad,
transición de columnas con cambio de cantidad/posición, cierre con dos y tres filas, y un nudo
que necesite cambiar más de un miembro. Cada fixture tendrá una disposición válida de referencia,
además de la actual que falla. Si no se encuentra esa referencia con la sección fija, registrar
esa incertidumbre y no usar el fixture como prueba de factibilidad.

Verificar las referencias mediante cálculos y geometría independientes del generador: capacidad,
anclaje, continuidad, separaciones, radios y tolerancias aplicables. Incluir configuraciones
inválidas próximas al límite para detectar tanto falsos positivos como falsos negativos. Una
imagen plausible o una aprobación del mismo generador no basta como referencia.

Instrumentar una lista de restricciones con miembros, barras, nudo, magnitud y operaciones
aplicables. Agrupar las repeticiones de un mismo defecto sin perder las comprobaciones individuales.
Registrar el motivo exacto de cada rechazo; `regression` solo no alcanza. Separar explícitamente
`budget`, agotamiento del dominio explorado, verificador no disponible y solicitud de sección.

**Aceptación:** los fixtures válidos pasan el camino completo y los inválidos fallan por la causa
esperada. El resumen distingue las cuatro categorías anteriores y reproduce el estado de partida.

### 2. Alternativas con geometría construible

Extender `splice.ts`, `lap-materialize.ts`, `generate-beam.ts` y los generadores transversales.
La compatibilidad de un empalme debe devolver una construcción concreta: parejas de barras,
trayectorias, zona de solape y terminaciones. Materializar ese mismo plan sin volver a elegir
parejas o posiciones incompatibles. Una transición puede usar continuidad, empalme con posiciones
compatibles o desplazamiento admisible; cada opción necesita comprobaciones propias, incluidos
terceros y estribos. La bandera `contactLap` no autoriza una interpenetración.

Generar jaulas completas con alternativas de diámetro, cantidad, filas, asientos, cierres y trabas.
Comprobar temprano la sección, pero conservar varias familias geométricamente distintas: menor
área o menos filas no implican mejor comportamiento en el nudo. Reutilizar los verificadores finales
para evitar que el filtro inicial y la geometría construida apliquen reglas diferentes.

**Aceptación:** el generador encuentra soluciones para los fixtures de empalme y cierre, sin
conflictos ni condiciones abiertas; persistir y regenerar conserva la solución. Rotar, trasladar
o invertir los extremos del modelo conserva el veredicto físico. No alterar tolerancias para pasar.

### 3. Diseño de líneas de vigas y pilas de columnas

Usar `beam-line.ts` y `column-stacks.ts` para decidir el armado que debe ser compatible entre
tramos. En vigas, proponer continuidad de barras base y refuerzos locales, con zonas admisibles
de terminación/empalme según las demandas y reglas del proyecto. Incluir ambas caras e inversión
de momentos. En columnas, planificar correspondencias entre pisos, cambios de diámetro/cantidad
y espacio ocupado por las transiciones y el armado transversal.

Extender la programación dinámica y las tablas de compatibilidad existentes para elegir entre
los planes geométricos del paso 2. Mantener alternativas de línea para resolver después los cruces
con otras líneas y columnas. Una viga compuesta por varios elementos de análisis debe sustituirse
como unidad, conservando todas sus demandas y propietarios de barras.

**Aceptación:** el generador cierra los casos de continuidad superior, falta de pareja y transición
de columnas. Los conflictos entre parejas disminuyen junto con nuevas aprobaciones completas en
los casos objetivo. Comprobar propiedad de barras fusionadas, cantidades, dibujos y persistencia.

### 4. Reparación conjunta guiada por la causa

Extender `physical-design-search.ts`: la cola debe incluir todas las restricciones pendientes,
también continuidad o empalmes sin choque. Cada causa propone operaciones adecuadas y el conjunto
mínimo de decisiones conectadas: dos extremos, un nudo viga–columnas o un tramo de línea/pila.
Ampliar ese conjunto si las alternativas pequeñas no funcionan, con límites por operaciones.

Mantener varias alternativas de trabajo y el último diseño aceptado por separado. Los estados
intermedios pueden necesitar más de un cambio para ser viables; evaluar y aceptar el paquete
completo de forma atómica. Usar estados visitados y claves que incluyan a los vecinos para evitar
ciclos y rechazos cacheados fuera de contexto.

La aceptación conservará cobertura, comprobaciones aplicables y miembros aprobados por identidad.
Ordenar propuestas por miembros completamente aprobados y restricciones del conjunto resueltas;
después, defectos agrupados y sus magnitudes; por último acero, formas y costo. Mantener también
el control de regresión por clase de fallo. Registrar los nuevos conflictos entre miembros ya
fallidos, como los de 227 con 46/47, sin esconderlos detrás del descenso del contador global.
Un movimiento parcial podrá servir como estado de búsqueda; no se presentará como cierre del nudo.

**Aceptación:** resolver un fixture con solución conocida que requiere dos cambios conjuntos,
y reproducir la conservación de aprobados. En el caso 227, exigir eliminar todas sus condiciones
y no trasladar bloqueos a sus vecinos antes de declararlo resuelto. Las pruebas de búsqueda sobre
dominios pequeños se compararán con enumeración exhaustiva del mismo dominio.

### 5. Resolver los límites ajenos al detallado

Dos entregas separadas, que no deben bloquear las correcciones geométricas:

- **Cobertura biaxial:** implementar y contrastar las comprobaciones que faltan con referencias
  independientes y casos límite, sin convertir los 13 provisionales en aprobados por omisión.
  Integrar su resultado al dimensionado y a la reverificación final.
- **Alternativas de sección:** cuando el catálogo de armado no alcance, proponer dimensiones
  dentro de los límites del proyecto. Evaluar cada propuesta en un estado separado, recalcular
  análisis, peso, rigidez y demandas, y volver a diseñar/verificar el modelo afectado. El resultado
  publicable debe tener análisis y certificados de la misma geometría. Aplicar los cambios según
  las restricciones de edición del proyecto; conservar la versión anterior hasta completar el ciclo.

**Aceptación:** casos biaxiales independientes; un caso que cierre cambiando sección y otro que
alcance el presupuesto sin solución, con estado explícito y sin certificados anteriores reutilizados.
Un cambio de sección puede invalidar aprobaciones previas: sólo se acepta tras verificarlas de nuevo.

### 6. Eficiencia e integración al flujo habitual

Primero validar las reparaciones con reconstrucción completa. Después introducir filtros locales,
compatibilidades reutilizables y caché por geometría/materiales/demandas/vecinos/versiones. Invalidar
por dependencias reales, incluidos empalmes, barras compartidas y pisos vecinos. Comprobar equivalencia
entre evaluación incremental y completa y conservar una validación completa antes de publicar.

Medir en una colección de modelos pequeños, medianos y el de 408 miembros: aprobaciones por identidad,
restricciones pendientes, evaluaciones, regeneraciones, memoria y tiempos p50/p95 con condiciones
documentadas. Comparar base, búsqueda actual y nueva con igual presupuesto; el tiempo de máquina no
debe cambiar el significado de «dominio agotado». Mantener pruebas pequeñas en CI y el edificio completo
en una integración acotada, sin multiplicar ejecuciones pesadas innecesariamente.

Con los criterios satisfechos, integrar las nuevas semillas y reparaciones al diseño habitual:
no dejarlas exclusivamente en experimentos opt-in. Mostrar progreso, causa pendiente, presupuesto
consumido y próximo tipo de acción; permitir continuar desde el último estado consistente.

**Aceptación:** mismo resultado entre cálculo completo e incremental, presupuesto respetado,
regeneración estable y operación desde el flujo real. Un diseño sólo se declara completo si todos
sus miembros y comprobaciones aplicables están verificados y no quedan bloqueos físicos abiertos.

### Secuencia de entregas

1 → 2 → 3 → 4 es el camino principal. El paso 5 se divide en verificador biaxial y ciclo de secciones;
pueden desarrollarse como entregas independientes una vez fijados los contratos de diagnóstico.
El paso 6 optimiza e integra operadores ya validados. Cada paso puede dividirse en PRs pequeños;
ningún PR necesita esperar a que los 408 elementos estén resueltos, pero debe cerrar sus fixtures
objetivo y conservar las aprobaciones existentes. El primer objetivo demostrable es generar desde
cero dos tramos compatibles; el siguiente, cerrar su línea y luego el nudo completo.

La [investigación de procedimientos de dimensionado y heurísticas](design-sizing-heuristics.md)
añade referencias de ingeniería y cuatro experimentos de catálogo del 22 de octubre. Ninguna
restricción global de diámetros evaluada se adopta: aumentan las pérdidas de cobertura aunque
algunos contadores de aprobaciones y conflictos mejoren.

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

# Dimensionado y heurísticas para cerrar el detallado

Investigación y experimento del 22 de octubre de 2026, sobre `55fd1e973` de PR536.
La recomendación es generar alternativas de armado compatibles entre miembros, conservar
las alternativas del catálogo y elegir la reparación según el fallo. Restringir globalmente
los diámetros no produjo una mejora aceptable en el ejemplo evaluado.

## Procedimientos de referencia

Las referencias siguientes aportan procedimientos; sus límites numéricos no sustituyen la
edición CIRSOC, los materiales ni la declaración lateral del proyecto.

- **Predimensionar, analizar y ajustar.** La guía NIST para pórticos especiales parte de la
  distribución estructural y secciones preliminares, obtiene esfuerzos y combina diseño por
  resistencia y capacidad. Recomienda evitar varias capas cuando sea posible; si la congestión
  lo exige, reconsiderar la sección. También describe barras superiores continuas con refuerzos
  locales, y advierte que los empalmes de columnas duplican barras en una zona. La ubicación
  de empalmes y las reglas de capacidad de esta guía corresponden a pórticos especiales, no a
  cualquier sistema lateral. [NIST GCR 16-917-40, §§5 y 7](https://nvlpubs.nist.gov/nistpubs/gcr/2016/NIST.GCR.16-917-40.pdf).
- **Iterar diámetro y cantidad con el espacio real.** spBeam describe calcular una cantidad
  inicial desde el área requerida, comprobar separaciones y ensayar otros diámetros. Incluye
  el espacio consumido por los dobleces de estribos y permite una separación mayor por tolerancias.
  Recomienda continuidad cuando las longitudes calculadas se superponen.
  [Manual de spBeam, §2.5](https://www.structurepoint.org/spbeam-online-manual/spBeam-v10-Manual-Book/2.5.Detailing-Provisions/2.5.Detailing-Provisions.htm).
- **El detalle puede cambiar la solución resistente elegida.** En el ejemplo 4.1, The Concrete
  Centre advierte que 3H25 puede generar problemas de separación al detallar jaula y empalmes,
  y plantea 2H32 como alternativa. El cambio no es una regla universal de preferir barras grandes:
  responde a esa disposición concreta. El ejemplo incluye revisiones de flecha, anclaje,
  cortes y mínimos además de flexión y corte.
  [Worked Examples to Eurocode 2, §4.1, pp. 95–103](https://www.concretecentre.com/TCC/media/TCCMediaLibrary/Events/Online%20course/CCIP_Worked_Examples_EC2.pdf).

## Traducción propuesta al algoritmo

Esta secuencia es una propuesta para este repositorio, derivada de las referencias y de la
[auditoría del código y las geometrías](design-verification-audit.md). No está implementada
por el experimento de paletas.

1. **Elegir secciones preliminares por familias.** Considerar luces, alturas disponibles,
   servicio, estabilidad, capacidad y espacio de nudos. Una relación luz/altura puede orientar
   una semilla bajo sus hipótesis; no debe convertirse en certificado. Los límites y supuestos
   de cada comprobación deben venir del adaptador normativo.
2. **Resolver demandas antes de elegir armaduras.** Conservar las combinaciones que gobiernan
   cada cara y región, incluida inversión de momentos. Para columnas, usar interacción axial
   y biaxial y efectos de esbeltez aplicables, no solamente área obtenida del axil.
3. **Convertir áreas requeridas en pocas jaulas alternativas.** El área aproximada
   `As ≈ Mu / (φ fy z)` sirve como semilla para vigas. Cada alternativa necesita cantidad,
   diámetro, filas, posiciones, altura útil real y estribo completo. Mantener alternativas
   con distinto compromiso entre anclaje, congestión y acero, aunque una use algo más de área.
4. **Coordinar continuidad y transiciones.** Elegir barras de referencia por línea de vigas,
   agregar refuerzos locales y decidir dónde pueden terminar o empalmarse. En columnas,
   compatibilizar posiciones entre pisos. La pareja de empalme debe incluir geometría ejecutable,
   recubrimiento, separación frente a terceros, longitud disponible y confinamiento.
5. **Verificar y reparar por causa.** Construir el detalle y evaluar resistencia, servicio y
   constructibilidad dentro de la cobertura implementada. Una verificación no implementada
   permanece pendiente. Reabrir las decisiones del nudo o tramo afectado cuando falle.
6. **Reanalizar al cambiar secciones.** Actualizar peso, rigidez, demandas y verificaciones
   dependientes. Una sección propuesta con los esfuerzos anteriores sigue siendo preliminar.

### Heurísticas condicionadas por el fallo

| Fallo observado | Alternativas que conviene ofrecer | Contrapartida que hay que verificar |
| --- | --- | --- |
| Falta de separación en una fila o empalme | Menos barras de mayor diámetro; otra posición o ancho | Anclaje, radios, recubrimiento y cuantía máxima |
| Anclaje insuficiente | Más barras de menor diámetro; otra terminación admisible; mayor apoyo | Separación, congestión y continuidad |
| Capas interiores contra cierre de estribo | Otra orientación o geometría de cierre; cambiar filas y asientos conjuntamente | Apoyo lateral, confinamiento y altura útil |
| Demasiados empalmes en un nudo | Continuidad de barras base; mover o escalonar empalmes a zonas admisibles | Longitud disponible, demanda y restricciones del sistema lateral |
| Barras sin pareja entre pisos | Patrón compatible entre jaulas; transición explícita de cantidades y posiciones | Capacidad de ambos tramos y geometría de la transición |
| Flecha, deriva o capacidad de sección insuficiente | Revisar canto, ancho o sistema según la causa | Reanálisis y nuevas demandas en los vecinos |

Estas son operaciones candidatas, no instrucciones para omitir verificaciones. «Más acero»,
«menos barras» y «mayor diámetro» no son direcciones de mejora universales.

## Experimento: reducir familias de diámetros en vigas

Se ejecutó el camino de producción completo en `rc-design-frame`: 408 miembros, misma
declaración lateral, secciones y cargas. Cada perfil corrió en un proceso nuevo con el mismo
WASM release. Sólo se interceptó el generador de vigas para filtrar su catálogo; se conservaron
verificadores, umbrales y generador de columnas. Se filtró también la regeneración por feedback.
Es una ablación de alternativas, no una implementación de continuidad por línea ni una
búsqueda exhaustiva. El resto del recorrido de búsqueda puede cambiar al cambiar la paleta.

| Perfil | Diámetros de vigas (mm) | Estados finales `ok/warn` | Aprobados anteriores perdidos | Vigas no disponibles | Conflictos bloqueantes | Tiempo local (s) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Base | 10, 12, 16, 20, 25, 32 | 72 | 0 | 35 | 2981 | 14,73 |
| Desde Ø16 | 16, 20, 25, 32 | 97 | 19 | 71 | 364 | 15,61 |
| Familia acotada | 16, 20, 25 | 102 | 18 | 75 | 327 | 15,53 |
| Desde Ø20 | 20, 25, 32 | 134 | 3 | 248 | 132 | 38,79 |

La tabla usa una ejecución por perfil; los tiempos incluyen la carga del ejemplo y no son un
benchmark estadístico. Una segunda ejecución de Ø16–25 reprodujo las mismas identidades,
estados y conflictos (14,05 s). «No disponibles» es el estado final, no sólo `SEARCH_EXHAUSTED`: con Ø20 hay
235 búsquedas agotadas y 13 resultados `UNSUPPORTED`. Los primeros tres perfiles conservan
13 propuestas biaxiales, que no cuentan como aprobaciones.

La familia Ø16–25 agrega 48 aprobaciones y pierde 18, todas estas últimas de columnas,
aunque su generador no cambió. Las vigas 231–234 pasan de fallidas a aprobadas, pero las
columnas 42, 48 y 70 hacen el recorrido inverso. Además, aparecen 24 condiciones de
recubrimiento. Los miembros con búsqueda agotada que incluyen anclaje entre sus causas pasan
de 13 a 56; las categorías pueden superponerse.

La variante Ø20 deja **todas las vigas sin diseño disponible**. Sus 134 estados aprobados
son de columnas; la reducción de interferencias ocurre en un modelo con menor cobertura
de armado. No demuestra cierre del diseño global ni mejora de su constructibilidad. Tampoco
se puede atribuir toda la reducción de conflictos de los otros perfiles a mejores jaulas:
parte puede deberse a la pérdida de cobertura.

**Ninguna variante se incorpora como política de producción.** El resultado útil es identificar
alternativas y contrapartidas que la búsqueda debe conservar, y una métrica engañosa que debe
evitar: maximizar aprobaciones o minimizar choques perdiendo diseños de miembros vecinos.

## Cambios prioritarios y aceptación

1. **Unificar fallos resistentes y físicos en la cola.** El feedback actual deja fuera 284 de
   los 288 miembros fallidos de la base porque selecciona fallos de reverificación resistente.
   Cada restricción debe identificar las piezas y decisiones que pueden repararla.
2. **Generar candidatos completos y diversos.** La prioridad actual por filas y área en
   `buildRegionOptions` no modela la transición con otra viga. Su catálogo reducido a 40
   opciones y el recorrido por regiones tampoco garantizan explorar todas las combinaciones.
   Conservar familias por cantidad/diámetro/filas y compatibilidad, sin imponer una paleta global.
   Compartir las reglas geométricas con el materializador y el verificador.
3. **Probar el conjunto afectado.** Empezar por un empalme o jaula, incluir vecinos del nudo
   y continuidad entre pisos, ampliar el conjunto si hace falta. Usar tablas de compatibilidad
   y la programación dinámica existente para cadenas. Un rechazo se memoiza con la geometría
   de sus vecinos; no basta el hash del armado aislado.
4. **Mantener una solución de referencia completa.** Explorar alternativas con presupuesto
   determinista y conservar el mejor estado aceptable. No exigir que cada movimiento interno
   mejore el acero ni publicar estados intermedios incompletos. Al aceptar un lote, conservar
   por identidad los miembros cubiertos y aprobados, no introducir nuevas verificaciones
   pendientes y volver a validar el conjunto. Dentro de esos límites, priorizar cierre y
   restricciones antes que peso de acero.

El primer incremento de generación debería resolver un empalme con diámetros distintos
(caso 178/179 de la auditoría), con posiciones y recorrido comprobados. El siguiente debe
coordinar continuidad superior y cierres de estribos con todas las capas. Cada incremento
necesita casos pequeños con variaciones de sección, diámetro, giro y restricciones de usuario,
y luego el modelo completo. Este único edificio no prueba robustez general.

## Reproducción y estado de implementación

Se agrega únicamente un experimento opt-in; no cambia el generador de producción. Desde `web/`:

```sh
STABILEO_BEAM_PALETTE=baseline STABILEO_HEURISTICS_OUTPUT=/tmp/beam-baseline.json npx vitest run --project unit --maxWorkers=2 src/lib/store/__tests__/design-heuristics-experiment.test.ts
```

Repetir en procesos separados con `min16`, `family` y `min20`, cambiando el archivo de salida.
El JSON guarda identidades, estados, resultados de búsqueda y cobertura por tipo. Las aserciones
comprueban ejecución completa y ausencia de aprobados con conflictos físicos detectados;
que el experimento pase **no significa que la heurística sea aceptable**. Comparar cobertura,
identidades, condiciones abiertas y costo antes de proponer su adopción.

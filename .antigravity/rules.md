# Reglas de Arquitectura - MetasysControl

## 1. Persistencia y Estado
- **NUNCA** mantener datos críticos de infraestructura (Equipos, Puntos de Control, Credenciales) exclusivamente en memoria volátil (RAM).
- Toda mutación de estado en Tauri debe reflejarse inmediatamente en un almacenamiento persistente (`.json` o SQLite).
- El estado global debe estar protegido por `Mutex` o `RwLock` para prevenir condiciones de carrera.

## 2. Concurrencia y Workers
- **PROHIBIDO** el uso de bucles de polling (`loop { sleep(X) }`) para leer la cola de tareas.
- Usar primitivas de sincronización asíncrona como `tokio::sync::mpsc` o `tokio::sync::Notify` para despertar a los hilos de trabajo únicamente cuando hay eventos reales. Esto garantiza 0% de uso de CPU en reposo.

## 3. Integración con APIs de Terceros (Metasys)
- **Manejo de Errores Estricto**: Distinguir explícitamente entre fallos de red (Timeouts, 503) y fallos de Payload (400 Bad Request, 500 InvalidInput).
- **Reintentos (Retries)**: Aplicar *Exponential Backoff* SÓLO a errores transitorios de red. Un error de validación de datos NUNCA debe ser reintentado.
- **Seguridad**: Nunca hardcodear IPs ni contraseñas. Siempre usar una capa de configuración cifrada o archivos de configuración inyectados.

## 4. Estándares Frontend
- **Tipado Fuerte**: La sanitización de tipos debe ocurrir antes de que los datos salgan del Frontend.
- Los valores flotantes deben enviarse con punto (`.`), nunca con coma (`,`), ya que el estándar de APIs REST y JSON serializa numéricos exclusivamente con punto decimal.

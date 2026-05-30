# Skills Requeridos - MetasysControl

Para interactuar y extender este proyecto, los desarrolladores (o agentes de IA) deben dominar los siguientes *skills* técnicos:

## Backend (Rust & Tauri)
1. **Manejo Asíncrono Avanzado (Tokio)**:
   - Uso de `Arc<Mutex<T>>` para compartir memoria entre comandos de Tauri y Workers.
   - Orquestación de hilos mediante `tokio::spawn`.
   - Uso de `tokio::sync::Notify` para notificaciones inter-hilos sin consumo de CPU.

2. **Reqwest & Redes Industriales**:
   - Comprensión de saltos de seguridad en redes locales (bypass de SSL mediante `danger_accept_invalid_certs`).
   - Parseo y deserialización estricta mediante `serde_json`.

3. **Arquitectura Tauri**:
   - Inyección de dependencias mediante `tauri::State`.
   - Comunicación segura IPC (Inter-Process Communication) sin exponer el backend directamente a HTTP.

## Frontend (React & TypeScript)
1. **Manejo de Estado Remoto**:
   - Sincronización optimizada entre la UI y el Backend usando Hooks (`useMetasys.ts`).
   - Renderizado condicional basado en estados de cola (`PENDING`, `PROCESSING`, `FAILED`).

2. **Estilizado Moderno (CSS Modules)**:
   - Capacidad de mantener y extender un sistema de diseño "Glassmorphism Premium" sin sobrecargar con librerías CSS monolíticas.
   - Uso de variables CSS para consistencia de tema oscuro.

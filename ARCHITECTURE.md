# Arquitectura y Buenas Prácticas (Metasys Control)

Este documento contiene las reglas de diseño y decisiones arquitectónicas del proyecto para mantener el contexto entre sesiones y agentes. **LEER ESTO ANTES DE MODIFICAR EL CÓDIGO**.

## 1. Stack Tecnológico
- **Frontend**: React 19 + TypeScript + Vite.
- **Backend**: Rust + Tauri v2.
- **Comunicación**: IPC (Inter-Process Communication) nativo de Tauri (Vía `invoke`). No se hacen peticiones HTTP desde React, todo el tráfico de red lo maneja Rust (`reqwest`).
- **Paquetería**: `pnpm`.

## 2. Reglas de Negocio (Backend - Rust)
### Principios SOLID Aplicados
- **SRP (Single Responsibility)**: 
  - `metasys_client.rs`: **SÓLO** se encarga de las peticiones HTTP seguras hacia Metasys (10.6.18.102). Implementa un caché in-memory del Token JWT y valida la expiración (`.expires`).
  - `queue_manager.rs`: **SÓLO** se encarga de vaciar la cola de forma asíncrona. Maneja el estado de las peticiones (PENDING, PROCESSING, FAILED) y aplica **Backoff Exponencial** en caso de fallo (5s, 10s, 20s...). No bloquea la UI.
  - `state.rs`: Define las estructuras de datos protegidas por `Mutex` y `RwLock` para concurrencia segura.

### Jerarquía de Equipos
- No tratamos las válvulas o los interruptores como entidades sueltas. El sistema es jerárquico: Un **Equipo (Uma)** contiene múltiples **Puntos de Control (UmaPoint)**.
- Un `UmaPoint` puede ser de tipo `toggle` (On/Off) o `analog` (0-100%).

### Optimización del Worker (Queue)
- El comando encolado (`QueuedCommand`) lleva consigo el `reference` final y la `action`. Esto permite que `queue_manager.rs` se dedique a disparar las peticiones a ciegas sin tener que buscar en la estructura de equipos en cada iteración, logrando un rendimiento O(1).

## 3. Reglas de Negocio (Frontend - React)
### Estructura
- **No se usan frameworks CSS pesados (como Tailwind)** a menos que sea estrictamente necesario. Se usa **Vanilla CSS Modules** (`.module.css`).
- Estética obligatoria: **Glassmorphism Premium**, Dark Mode (Slate 900 base), bordes sutiles translúcidos y tipografía **Inter**.
- Todo el manejo de estado global con Tauri se abstrae en el hook `useMetasys.ts`.

### Polling IPC
- La UI (el Dashboard) se actualiza usando Polling cada 2 segundos a través del IPC interno de Tauri (`get_queue_status`, `get_umas`). Como es memoria local RAM a RAM (sin red), este polling es extremadamente barato y seguro, y mantiene la interfaz reactiva sin websockets.

## 4. Convenciones de Código
- Evitar "if dentro de if" (Código Espagueti). Usar retornos tempranos (*Early returns*) o *Pattern Matching* (`match` en Rust).
- Nombres de variables explícitas y en inglés para el código, pero textos de interfaz en Español.
- No dejar archivos muertos ni código comentado en producción.

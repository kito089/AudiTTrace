# AudiTrace - Sistema de Auditoría en Solana

## Descripción
**AudiTrace** es un sistema desarrollado en **Solana** usando **Anchor**, diseñado para auditar los cambios realizados en bases de datos.  
Permite registrar operaciones, creando eventos que documentan **creación**, **actualización** y **eliminación** de datos.

---

## Funcionalidades principales

  1. **Crear Registry**: Registro principal de auditoría (Base de Datos a auditar).
  2. **Crear Record**: Subregistro con datos auditados (Creacion de tablas, insercion de datos (hash)).
  3. **Actualizar Record**: Modificar datos (hash) de un subregistro.
  4. **Eliminar Record**: Desactivar un subregistro.
  5. **Ver Records**: Listar todos los subregistros (cambios) de un registro principal (BD).
  6. **Ver Eventos**: Listar los eventos generados por acciones en un subregistro.

---

## Estructura de datos

  - `AuditRegistry` → Cuenta principal del sistema.
  - `AuditRecord` → Subregistro auditado con hash y estado (`active`).
  - `AuditEvent` → Evento que ocurre sobre un subregistro (CREATE, UPDATE, DELETE).

---

## Pruebas

El archivo **client.ts** contiene funciones para probar todas las funcionalidades.  
Para realizar pruebas en **Solana Playground**:

  1. Abre el proyecto usando [Solana Playground](https://beta.solpg.io).
  2. Compila el programa usando **Build**.
  3. Abre la pestaña `client.ts`.
  4. Descomenta el bloque de ejemplo al final del archivo (`(async () => { ... })`) y cambia los valores de los métodos y variables para ejecutar pruebas secuenciales:

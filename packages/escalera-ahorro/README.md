# Bindings de escalera-ahorro v2

Bindings TypeScript generados desde el WASM local de la versión 2. El paquete exporta `Client`, `EstadoMeta` y `Errors` desde `src/index.ts`. No incluye un `contractId` de testnet: la instancia v1 existente no es compatible; configura el ID de una instancia v2 nueva cuando se despliegue.

Para regenerar los bindings desde la raíz del workspace:

```powershell
stellar contract bindings typescript --wasm target\wasm32v1-none\release\escalera_ahorro.wasm --output-dir target\escalera-ahorro-bindings-v2
Copy-Item target\escalera-ahorro-bindings-v2\src\index.ts packages\escalera-ahorro\src\index.ts -Force
```

Para comprobar el paquete:

```powershell
Set-Location packages\escalera-ahorro
npm run build
```

El ABI expone `initialize`, `abrir_meta`, `configurar_reparto`, `depositar`, `acceso_desbloqueado`, `retirar_desbloqueado`, `retirar_colchon`, `retirar_primer_paso`, `retirar_todo_emergencia` y `ver_estado`.

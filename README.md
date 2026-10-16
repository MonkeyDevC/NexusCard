# NexusCard

Personaliza el diseño de tus tarjetas de Apple Wallet desde Windows: arrastra tu imagen, encuádrala con zoom, mira cómo se verá en la pantalla de Apple Pay y aplícala a tu iPhone. Sin jailbreak y en español.

> [!WARNING]
> **Úsalo bajo tu responsabilidad.** NexusCard usa un método de sincronización de iOS para cambiar **solo la imagen** de tus tarjetas en Wallet; no toca datos de pago. Aun así, haz un respaldo del iPhone y prueba primero con una tarjeta secundaria. No está afiliado a Apple.

**Sitio, precio y descarga:** https://scalvache.shop/productos/NexusCard/

## Compatibilidad

| iOS | Estado |
| :-- | :-- |
| 18.0 a 27.0.1 | ✅ Compatible |
| 27.2 beta 1 y 2 | ✅ Compatible |
| 27.2 beta 3 en adelante | ❌ Apple corrigió el método; no funciona |

No actualices a 27.2 beta 3 o posterior si quieres seguir usándolo.

## Instalación

1. Descarga `NexusCard-Setup-<versión>.exe` desde [Releases](../../releases).
2. Verifica el hash con `SHA256SUMS.txt` (`Get-FileHash .\NexusCard-Setup-*.exe`).
3. Ejecútalo. Si falta **Apple Devices**, el instalador ofrece instalarlo.
4. Conecta el iPhone por USB, desbloquéalo y toca **Confiar**.

El instalador aún **no está firmado**: Windows SmartScreen puede mostrar un aviso (*Más información → Ejecutar de todos modos*). Todo se compila en GitHub Actions desde este código.

## Uso

1. Pulsa **Escanear** y toca tu tarjeta en Wallet.
2. Suelta tu imagen en la zona punteada y encuádrala (arrastrar y zoom). La pestaña **Vista previa** la muestra sobre una pantalla de Apple Pay.
3. Pulsa **Aplicar diseño**, cierra Wallet por completo y vuelve a abrirla.
4. **Restaurar** devuelve el diseño original (siempre gratis).

## Licencia y versión de prueba

- La **prueba** incluye 3 cambios de diseño por equipo.
- La **licencia completa** (pago único) quita el límite. Precio: Colombia $10.000 COP, Estados Unidos US$5, resto del mundo US$3.
- Se compra desde la propia app; se activa sola al confirmarse el pago.
- Al reinstalar en el mismo PC se reconoce automáticamente. En otro PC, usa tu clave `NXC-XXXX-XXXX-XXXX`.

El proyecto es de código abierto (MIT). La comprobación de licencia está en [`src/license.rs`](src/license.rs); quien compile el código por su cuenta puede quitarla. Lo que se paga es el instalador listo, las mejoras, el soporte y las actualizaciones.

### Privacidad

La app se comunica con `scalvache.shop` solo para comprobar y registrar la licencia. Envía: un **hash anónimo del equipo** (SHA-256 del identificador de Windows, no reversible), la versión de la app y, según tu IP, el país aproximado. Si compras, la pasarela de pago (Bold o Wompi) procesa tu pago y nos informa del correo y el monto; nunca vemos datos de tu tarjeta. No se leen tus tarjetas de Wallet, ni tus fotos, ni se envían imágenes.

## Arquitectura y tecnologías

NexusCard tiene dos piezas: la **app de escritorio** (este repositorio) y un **servidor de licencias y pagos** (repositorio privado).

```
 App de Windows (Rust + egui)  ──USB / WiFi──►  iPhone (usbmuxd · AFC · AirTraffic)
          │
          └──HTTPS──►  scalvache.shop  (Node.js + Fastify + SQLite)  ──►  Bold / Wompi
```

### App de escritorio

| Capa | Tecnología |
| :-- | :-- |
| Lenguaje | **Rust** (edición 2024) |
| Interfaz | **egui / eframe 0.33**, selector de archivos con `rfd` |
| Conexión con el iPhone | Bibliotecas de Apple (`CoreFoundation`, `MobileDevice`, `AirTrafficHost`) cargadas en tiempo de ejecución con `libloading`, desde **Dispositivos Apple** (Microsoft Store) o iTunes |
| Transporte | **usbmuxd** (`127.0.0.1:27015`) por **USB o WiFi**, servicios **AFC** (archivos), **syslog_relay** (detección de tarjetas) y **AirTraffic** (sincronización) |
| Imagen de la tarjeta | `image` (PNG, JPEG, WebP), encuadre con zoom y conversión propia a PDF |
| Formatos de iOS | `plist`, `zip`, `miniz_oxide` |
| Licencia | Cliente HTTP `ureq`, tokens firmados con **Ed25519** (`ed25519-dalek`) y hash anónimo del equipo con **SHA-256** |
| Windows | `winreg`, `winresource` (icono y versión en el `.exe`), instalador con **Inno Setup** |
| CI/CD | **GitHub Actions**: tests, compilación, instalador, `SHA256SUMS.txt` y publicación en Releases |

Módulos principales (`src/`):

| Módulo | Qué hace |
| :-- | :-- |
| `apple.rs` | Localiza y carga las bibliotecas de Apple (iTunes o Dispositivos Apple) |
| `device.rs` | Lista iPhones por usbmuxd (USB y WiFi), abre sesiones y arranca Dispositivos Apple si su servicio no responde |
| `scanner.rs` | Escucha el registro del iPhone para capturar el identificador de cada tarjeta de Wallet |
| `card_info.rs` | Lee el diseño actual de una tarjeta, sus últimos dígitos y la red |
| `image_skin.rs` | Encuadra la imagen al tamaño de la tarjeta (1536 × 969) y genera PNG y PDF |
| `afc.rs`, `airlift.rs`, `airtraffic.rs`, `flasher.rs` | Escriben el diseño en el iPhone mediante el método de sincronización |
| `wallet_backup.rs` | Guarda el diseño original para poder restaurarlo |
| `license.rs` | Prueba gratis y licencia completa contra el servidor |
| `app.rs`, `ui_kit.rs`, `i18n*.rs` | Interfaz, componentes visuales e idiomas (español e inglés) |

### Servidor de licencias y pagos

**Node.js 22 + Fastify 5**, base de datos **SQLite** (`better-sqlite3`), pagos con **Bold** y **Wompi** (webhooks verificados), licencias firmadas con **Ed25519**, precio por país con `geoip-lite`, panel de administración con **Chart.js** y landing en `scalvache.shop`. Corre en **Docker Compose** detrás de **Caddy** en un VPS.

### Diagnóstico

La app guarda su registro en `%LOCALAPPDATA%\NexusCard\nexuscard.log`. Si algo falla, ese archivo dice en qué paso.

## Compilar desde el código

Requisitos: Rust (estable), Visual Studio Build Tools (C++) y Dispositivos Apple (Microsoft Store) o iTunes de 64 bits.

```sh
cargo test
cargo build --release
# binario: target\release\nexuscard.exe
```

Para probar la licencia contra un servidor local: `NEXUSCARD_API=http://localhost:3300/productos/NexusCard/api/v1` y `NEXUSCARD_PUBLIC_KEY=<clave pública del servidor>`.

El instalador se genera con [Inno Setup](https://jrsoftware.org/isinfo.php): `installer\nexuscard.iss`.

## Créditos

NexusCard se basa en [AirCard-Windows](https://github.com/Lumid-Off/AirCard-Windows) de Lumid-Off, que a su vez es un port de [AirCard](https://github.com/Mak5er/AirCard) de Mak5er (ambos MIT). El método de sincronización (`airlift`) y el cliente original son obra de sus autores. Ver [NOTICE.md](NOTICE.md).

## Licencia

MIT. Consulta [LICENSE](LICENSE).

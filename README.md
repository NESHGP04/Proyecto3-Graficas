# 🌌 Sistema Solar 3D - Proyecto Final

Simulador de sistema solar completamente funcional con cámara 3D, renderizado procedural y nave espacial interactiva.

## 📹 Video de Demostración

[![Sistema Solar 3D Demo](link-a-tu-video-thumbnail.png)](link-a-tu-video.mp4)

*Video mostrando la exploración del sistema solar con todos sus componentes*

## ✨ Características Implementadas

### ⭐ Características Base
- ✅ **Sol y Planetas** - Sistema solar con 6 cuerpos celestes
- ✅ **Plano Eclíptico** - Todos los planetas alineados en un plano orbital
- ✅ **Traslación Orbital** - Órbitas circulares con velocidades realistas
- ✅ **Rotación Axial** - Cada planeta rota sobre su propio eje

### 🎥 Sistema de Cámara (40 pts)
- ✅ **Movimiento 3D Completo** - 6 grados de libertad
- ✅ **Controles Intuitivos** - WASD + Space/Shift para movimiento vertical
- ✅ **Rotación Libre** - Control total de pitch y yaw
- ✅ **Sistema Suave** - Interpolación y movimiento fluido

### 🚀 Nave Espacial (30 pts)
- ✅ **Modelo 3D Personalizado** - Nave diseñada en Blender
- ✅ **Sigue la Cámara** - Se posiciona relativamente a la vista
- ✅ **Orientación Dinámica** - Rota según la dirección de la cámara
- ✅ **Shader Metálico** - Color dorado con detalles procedurales

### 🌟 Cuerpos Celestes (50 pts)
1. **Sol** ☀️
   - Shader con gradiente radial
   - Manchas solares animadas
   - Llamaradas procedurales
   - Corona brillante

2. **Planeta Rocoso** 🌍
   - Continentes y océanos
   - Casquetes polares
   - Nubes procedurales
   - Variación de elevación

3. **Gigante Gaseoso** 🪐
   - Bandas horizontales
   - Gran mancha roja (tormenta)
   - Turbulencia atmosférica
   - Gradientes de color

4. **Planeta Helado** ❄️
   - Superficie de hielo
   - Grietas procedurales
   - Tonos azules/blancos
   - Efectos de cristal

5. **Planeta Volcánico** 🌋
   - Lava activa animada
   - Roca volcánica
   - Flujos de magma
   - Emisión de luz propia

6. **Luna** 🌙
   - Cráteres procedurales
   - Superficie gris
   - Textura rocosa
   - Órbita alrededor del planeta rocoso

### 🎯 Instant Warping (20 pts)
- ✅ **Teletransporte Rápido** - Teclas 1-6 para cada planeta
- ✅ **Animación Suave** - Interpolación con easing cúbico
- ✅ **Efecto Visual** - Transición fluida de 1.5 segundos
- ✅ **Posicionamiento Inteligente** - Cámara se posiciona a distancia óptima

### ⭐ Skybox (10 pts)
- ✅ **1500+ Estrellas** - Campo estelar procedural
- ✅ **Diferentes Tamaños** - Estrellas de 1x1 y 2x2 pixels
- ✅ **Variación de Brillo** - Intensidades aleatorias
- ✅ **Efecto Twinkle** - Parpadeo sutil animado

### 🔵 Órbitas Visuales (20 pts)
- ✅ **Trayectorias Circulares** - Muestra el camino orbital
- ✅ **Color por Planeta** - Cada órbita tiene color único
- ✅ **Toggle On/Off** - Tecla 'O' para mostrar/ocultar
- ✅ **Proyección 3D** - Se renderizan correctamente en espacio 3D

### 🛡️ Detección de Colisiones (10 pts)
- ✅ **Sistema de Colisión Esférica** - Previene atravesar planetas
- ✅ **Resolución Automática** - Pushback cuando hay contacto
- ✅ **Margen de Seguridad** - 50 unidades de distancia mínima
- ✅ **Aplicado a Todos** - Funciona con todos los cuerpos celestes

## 🎮 Controles

### Movimiento de Cámara
- `W` - Avanzar
- `S` - Retroceder
- `A` - Izquierda
- `D` - Derecha
- `SPACE` - Subir
- `LEFT SHIFT` - Bajar

### Rotación de Cámara
- `Q` - Rotar izquierda (yaw)
- `E` - Rotar derecha (yaw)
- `Z` - Mirar arriba (pitch)
- `C` - Mirar abajo (pitch)

### Warp/Teletransporte
- `1` - Planeta Rocoso
- `2` - Planeta Volcánico
- `3` - Gigante Gaseoso
- `4` - Planeta Helado
- `5` - Luna
- `0` - Vista general (reset)

### Otros
- `O` - Toggle órbitas
- `P` - Pausar/Reanudar
- `H` - Mostrar/Ocultar UI
- `ESC` - Salir

## 🏗️ Estructura del Proyecto

```
proyecto-sistema-solar/
├── src/
│   ├── main.rs              # Loop principal y lógica del sistema
│   ├── camera.rs            # Sistema de cámara 3D
│   ├── skybox.rs            # Generación de estrellas
│   ├── shader.rs            # Shaders de planetas
│   ├── shaders.rs           # Vertex shader
│   ├── triangle.rs          # Rasterización
│   ├── framebuffer.rs       # Buffer de píxeles
│   ├── vertex.rs            # Estructura de vértices
│   ├── fragment.rs          # Estructura de fragmentos
│   ├── obj.rs               # Cargador de modelos
│   ├── color.rs             # Sistema de colores
│   └── line.rs              # Dibujo de líneas
├── assets/
│   └── models/
│       ├── sphere.obj       # Modelo de esfera
│       └── spaceship.obj    # Modelo de nave
├── screenshots/
│   ├── sol.png
│   ├── planeta_rocoso.png
│   ├── gigante_gaseoso.png
│   ├── planeta_helado.png
│   ├── planeta_volcanico.png
│   └── luna.png
├── Cargo.toml
└── README.md
```

## 🛠️ Tecnologías y Dependencias

```toml
[dependencies]
minifb = "0.28.0"          # Ventana y entrada
nalgebra-glm = "0.20.0"    # Matemáticas 3D
tobj = "4.0.3"             # Carga de modelos OBJ
```

## 🚀 Compilación y Ejecución

### Requisitos
- Rust 1.70 o superior
- Cargo

### Compilar
```bash
cargo build --release
```

### Ejecutar
```bash
cargo run --release
```

## 📊 Detalles Técnicos

### Pipeline de Renderizado
1. **Skybox** - Renderizado de fondo (estrellas fijas)
2. **Órbitas** - Líneas proyectadas en 3D
3. **Planetas** - Transformación 3D → 2D con cámara
4. **Nave Espacial** - Renderizada relativa a la cámara
5. **UI** - Información en pantalla (opcional)

### Shaders Procedurales
Todos los shaders son 100% procedurales usando:
- **Ruido de Perlin** - Texturas orgánicas
- **FBM (Fractional Brownian Motion)** - Detalles multi-escala
- **Funciones trigonométricas** - Patrones y animaciones
- **Interpolación de colores** - Transiciones suaves

### Sistema de Colisiones
```rust
fn check_collision(camera_pos, object_pos, radius) -> bool {
    distance(camera_pos, object_pos) < radius + margin
}
```

### Proyección 3D a 2D
```rust
fn project_to_screen(world_pos, camera) -> Vec2 {
    // Transformación de mundo a espacio de cámara
    // Proyección perspectiva
    // Mapeo a coordenadas de pantalla
}
```

## 📈 Rendimiento

- **FPS Target**: 60 FPS
- **Resolución**: 1400x900 pixels
- **Vértices**: ~5000 (sphere) + ~150 (spaceship)
- **Estrellas**: 1500 puntos
- **Optimizaciones**:
  - Culling de objetos fuera de vista
  - Z-buffer para visibilidad
  - Renderizado por demanda

## 🎨 Screenshots

### Vista General
![Vista del Sistema Solar](screenshots/sistema_completo.png)

### Sol
![Sol con shader procedural](screenshots/sol.png)

### Planeta Rocoso
![Planeta tipo Tierra](screenshots/planeta_rocoso.png)

### Gigante Gaseoso
![Planeta tipo Júpiter](screenshots/gigante_gaseoso.png)

### Planeta Helado
![Planeta tipo Urano](screenshots/planeta_helado.png)

### Planeta Volcánico
![Planeta tipo Io](screenshots/planeta_volcanico.png)

### Luna
![Luna con cráteres](screenshots/luna.png)

### Nave Espacial
![Nave en primera persona](screenshots/nave_espacial.png)

## 📝 Puntuación Estimada

| Característica | Puntos | Estado |
|----------------|--------|--------|
| Estética del sistema | 30 | ✅ Completado |
| Performance | 20 | ✅ 60 FPS estable |
| Planetas/Estrellas (6) | 50 | ✅ Completado |
| Instant Warping | 10 | ✅ Completado |
| Animación de Warp | 10 | ✅ Completado |
| Nave Espacial | 30 | ✅ Completado |
| Skybox | 10 | ✅ Completado |
| Colisiones | 10 | ✅ Completado |
| Movimiento 3D | 40 | ✅ Completado |
| Órbitas | 20 | ✅ Completado |
| **TOTAL** | **230** | **✅** |

## 👨‍💻 Autor

**Marines Garcia**  
Gráficas por Computadora  
Universidad del Valle de Guatemala

## 📄 Licencia

Este proyecto fue creado con fines educativos para el curso de Gráficas por Computadora.

---

**⭐ ¡Explora el universo! 🚀**
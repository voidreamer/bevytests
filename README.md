# Advanced Bevy 3D Demo

This project is a demonstration of advanced Bevy 3D rendering techniques and game mechanics. It showcases a simple third-person game with player movement, camera controls, and dynamic lighting.

## Features

- **Third-person character control**: WASD movement with camera-relative controls
- **Physics**: Basic gravity, jumping, and ground collision
- **Camera system**: Smooth follow camera with mouse rotation and zoom
- **Advanced rendering**: 
  - PBR (Physically Based Rendering) materials
  - Bloom for emissive objects
  - Directional light with cascaded shadow mapping
  - Dynamic point lights with shadows
  - Depth prepass for post-processing

## Controls

- **WASD**: Move the player character
- **Space**: Jump
- **Mouse**: Control camera orientation
- **Mouse Wheel**: Zoom in/out
- **ESC**: Exit game

## Code Structure

The project is organized into several key components:

### Components

- **Player**: Handles character state and physics properties
- **ThirdPersonCamera**: Controls camera behavior and smoothing
- **HighQualityObject**: Marks meshes for higher quality rendering
- **AnimatedLight**: Identifies lights that should move/animate

### Systems

- **player_controller**: Processes input and updates player position/rotation
- **third_person_camera**: Handles camera movement and rotation
- **animate_lights**: Creates dynamic lighting effects
- **spawn_scene**: Sets up the game world with materials and objects

### Resources

- **AdvancedRenderingSettings**: Configuration for visual effects

## How It Works

### Player Movement

The player movement system works by:
1. Reading keyboard input (WASD)
2. Converting input to camera-relative movement direction
3. Applying physics (gravity, velocity)
4. Handling jumping and ground collision
5. Smoothly rotating the player in the direction of movement

### Camera System

The third-person camera:
1. Orbits around the player based on mouse input
2. Maintains a configurable distance, height, and smoothing
3. Handles zooming with mouse wheel
4. Uses smooth interpolation for natural following

### Rendering Pipeline

The rendering pipeline includes:
1. PBR materials with properties like metallic, roughness, reflectance
2. Bloom effect for emissive materials
3. High-resolution shadow mapping
4. Depth prepass for improved rendering quality

## Extending the Project

Ways to extend this project:
1. Add more complex physics and collision
2. Implement additional post-processing effects
3. Add game objectives or interactive elements
4. Create a more detailed environment
5. Implement a UI or HUD

## Learning Resources

To learn more about Bevy:
- [Bevy Documentation](https://bevyengine.org/docs/)
- [Bevy Examples](https://github.com/bevyengine/bevy/tree/main/examples)
- [Bevy Cheatbook](https://bevy-cheatbook.github.io/)
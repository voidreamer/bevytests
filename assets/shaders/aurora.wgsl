#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

// OKLab color space conversions for perceptually accurate color blending
fn oklab_to_linear_srgb(c: vec3<f32>) -> vec3<f32> {
    let L = c.x;
    let a = c.y;
    let b = c.z;

    let l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = L - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    return vec3<f32>(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    );
}

// Hash function for random values
fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// 2D Noise function based on Perlin noise principles
fn noise21(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    // Cubic Hermite interpolation for smoother blending
    let u = f * f * (3.0 - 2.0 * f);
    
    // Four corners hash values
    let a = hash(i + vec2<f32>(0.0, 0.0));
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    
    // Bilinear interpolation
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Fractional Brownian Motion for layered noise
fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 2.0;
    
    for (var i = 0; i < octaves; i = i + 1) {
        value += amplitude * noise21(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value;
}

// Nebula effect for subtle space dust
fn nebula(coord: vec2<f32>, time: f32) -> vec3<f32> {
    // Very subtle shifting noise field
    let nebula_noise = fbm(coord * 4.0 + vec2<f32>(time * 0.01, 0.0), 3) * 0.15;
    
    // Vary nebula color based on position
    let hue = fbm(coord * 2.0 - vec2<f32>(time * 0.02, 0.0), 2);
    
    // Very subtle bluish/purplish dust
    let nebula_color = mix(
        vec3<f32>(0.02, 0.035, 0.05),  // Deep blue
        vec3<f32>(0.04, 0.02, 0.06),   // Purple tint
        hue
    );
    
    return nebula_color * nebula_noise * smoothstep(0.4, 0.6, noise21(coord * 3.0 + vec2<f32>(time * 0.03, 0.0)));
}


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Aurora-specific parameters
    let time = globals.time * 0.2;
    let coord = in.uv;
    
    // The variable we'll use for the final color
    var result_color = vec3<f32>(0.0);
    
    // Aurora tends to appear in bands across the sky
    let y_stretch = 3.0; // Stretch the effect vertically
    let waviness = 0.8; // How wavy the aurora bands are
    
    // Create multiple layers of noise with different frequencies
    let noise_coord = vec2<f32>(coord.x * 2.0, coord.y * y_stretch) + vec2<f32>(time * 0.1, time * 0.05);
    let large_noise = fbm(noise_coord, 3) * waviness;
    
    // Create wave-like vertical displacement
    let wave_effect = sin(coord.y * 15.0 + time + large_noise * 5.0) * 0.05;
    let displaced_x = coord.x + wave_effect;
    
    // Create flow and movement
    let flow = sin(displaced_x * 10.0 + large_noise * 3.0 + time * 0.7) * 0.5 + 0.5;
    let height_mask = smoothstep(0.0, 0.7, 1.0 - abs(coord.y - 0.5) * 2.0); // Stronger in the middle
    
    // Aurora intensity varies with height and flow
    let intensity = flow * height_mask * smoothstep(0.0, 0.4, large_noise + 0.1);
    
    // Distance from center affects color mixing
    let dist_center = distance(coord, vec2<f32>(0.5, 0.5));
    let dist_factor = smoothstep(0.0, 1.2, dist_center);
    
    // Layer of smaller, faster moving details
    let small_noise = fbm(noise_coord * 4.0 + vec2<f32>(time * 0.2, 0.0), 2) * 0.4;
    let detail_intensity = small_noise * intensity * 0.8;
    
    // Additional fine detail layer for more complexity
    let fine_noise = fbm(noise_coord * 8.0 + vec2<f32>(time * 0.3, time * 0.15), 3) * 0.3;
    let fine_detail = fine_noise * intensity * smoothstep(0.2, 0.8, large_noise) * 0.6;
    
    // Dynamic intensity layer - random surges in brightness on a different cadence
    // Create a different pulsing pattern with irregular timing for variety
    let surge_frequency = 0.03; // Different frequency from curtains for variety
    let surge_phase = hash(floor(vec2<f32>(time * surge_frequency, 1.0))); // Different seed
    let surge_timing = smoothstep(0.7, 0.9, sin(time * surge_frequency * 6.28 + surge_phase * 10.0));
    
    // Make surges more localized and subtle
    let surge_x = coord.x * 3.0 + large_noise * 2.0;
    let surge_y = coord.y * 4.0 + large_noise * 3.0;
    
    // Spatial patterns for surges
    let surge_pattern1 = smoothstep(0.45, 0.65, sin(time * 0.3 + surge_x + surge_y));
    let surge_pattern2 = smoothstep(0.55, 0.75, sin(time * 0.2 - surge_x + surge_y * 2.0));
    let surge_pattern3 = smoothstep(0.65, 0.85, cos(time * 0.15 + surge_x * 2.0 - surge_y));
    
    // Make surges extremely subtle, almost imperceptible
    // More selective timing with higher threshold
    let subtle_surge_timing = smoothstep(0.8, 0.95, sin(time * surge_frequency * 6.28 + surge_phase * 10.0));
    
    // Drastically reduced intensity and only active during very brief moments
    let intensity_surge = max(surge_pattern1, max(surge_pattern2, surge_pattern3)) 
                        * intensity * 0.12 * subtle_surge_timing; // Reduced by ~3x
    
    // Time-varying parameters for color animation
    let t1 = sin(time * 0.3) * 0.5 + 0.5;
    let t2 = cos(time * 0.2) * 0.5 + 0.5;
    let t3 = sin(time * 0.4 + dist_center) * 0.5 + 0.5;
    
    // Aurora borealis colors in OKLab space for better blending
    // Vibrant greens and teals are common in auroras
    let green = vec3<f32>(0.86644, -0.233887, 0.179498);  // Vibrant green
    let teal = vec3<f32>(0.7, -0.1, 0.1);                 // Bluish-green
    let blue = vec3<f32>(0.701674, 0.174566, -0.269156);  // Cold blue
    let purple = vec3<f32>(0.7, 0.3, -0.1);               // Purplish hue
    let red = vec3<f32>(0.75, 0.35, 0.15);               // Reddish aurora
    let pink = vec3<f32>(0.78, 0.27, 0.05);              // Pinkish aurora
    
    // Final color is a complex mix based on multiple parameters
    let color1 = mix(green, teal, t1);
    let color2 = mix(blue, purple, t2);
    // Add fine detail to the color mixing for more complexity
    let mixed_color = mix(color1, color2, t3 * dist_factor + detail_intensity + fine_detail * 0.5);
    
    // Barely visible microdetail pattern
    let micro_noise_coord = noise_coord * 12.0 + vec2<f32>(time * 0.4, time * -0.3); // Lower frequency
    let micro_noise = fbm(micro_noise_coord, 2) * 0.07 * intensity; // Drastically reduced
    let micro_detail = smoothstep(0.35, 0.65, sin(coord.y * 30.0 + large_noise * 10.0)) * // Lower frequency
                      intensity * 0.08; // Drastically reduced
    
    // Apply intensity to color and convert back to linear RGB
    // Include all detail layers and intensity surge in the final intensity
    let aurora_lab_color = mixed_color * (intensity + detail_intensity * 0.7 + fine_detail * 0.6 + 
                                         micro_noise + micro_detail + intensity_surge);
    
    // Create a specific surge color that's more vibrant
    let surge_color = mix(green, vec3<f32>(0.9, -0.1, 0.2), 0.3); // Brighter, slightly more yellowish
    let surge_contribution = surge_color * intensity_surge * 0.7;
    
    // Final color with surge
    let aurora_with_surge = aurora_lab_color + surge_contribution;
    let rgb_color = oklab_to_linear_srgb(aurora_with_surge);
    
    // Add a subtle glow effect
    let glow = intensity * 0.4;
    let glow_color = mix(vec3<f32>(0.05, 0.1, 0.2), rgb_color, intensity);
    
    // Add a second fine detail pattern with different orientation
    let cross_detail = fbm(vec2<f32>(coord.y * 6.0, coord.x * 12.0) + vec2<f32>(time * 0.15, -time * 0.05), 2) * 0.2;
    let cross_effect = cross_detail * smoothstep(0.0, 0.6, intensity) * 0.3;
    
    // Fine wisps in the aurora
    let wisps = smoothstep(0.3, 0.7, sin(coord.y * 30.0 + large_noise * 10.0 + time * 0.2)) * intensity * 0.15;
    
    // Create a second, reddish aurora layer with offset
    // Use a different offset and scale for the second aurora
    let aurora2_offset = vec2<f32>(0.2, -0.1); // Offset for the second aurora
    let aurora2_noise_coord = vec2<f32>(
        coord.x * 1.5 + aurora2_offset.x,
        coord.y * 2.5 + aurora2_offset.y
    ) + vec2<f32>(time * 0.08, time * 0.02); // Different speed
    
    // Additional high-frequency detail for second aurora
    let aurora2_detail_coord = aurora2_noise_coord * 3.0 + vec2<f32>(time * -0.2, time * 0.1);
    let aurora2_detail = fbm(aurora2_detail_coord, 2) * 0.5;
    
    // Create wave patterns for second aurora
    let aurora2_large_noise = fbm(aurora2_noise_coord, 3) * 0.7;
    let aurora2_wave = sin(coord.y * 10.0 + time * 0.8 + aurora2_large_noise * 4.0) * 0.07;
    let aurora2_displaced_x = coord.x + aurora2_wave;
    
    // Different flow pattern
    let aurora2_flow = sin(aurora2_displaced_x * 8.0 + aurora2_large_noise * 2.0 + time * 0.5) * 0.5 + 0.5;
    // Concentrated more toward the top
    let aurora2_height_mask = smoothstep(0.0, 0.6, 1.0 - abs(coord.y - 0.6) * 2.0);
    let aurora2_intensity = aurora2_flow * aurora2_height_mask * smoothstep(0.0, 0.4, aurora2_large_noise);
    
    // Reddish color mix for second aurora
    let t4 = sin(time * 0.25) * 0.5 + 0.5;
    let aurora2_color_mix = mix(red, pink, t4);
    
    // Convert to RGB with reduced intensity compared to main aurora
    // Incorporate the high-frequency detail into the aurora
    let aurora2_lab_color = aurora2_color_mix * (aurora2_intensity * 0.7 + aurora2_detail * aurora2_intensity * 0.4);
    let aurora2_rgb = oklab_to_linear_srgb(aurora2_lab_color);
    
    // Add some fine wispy structures to the second aurora
    let aurora2_wisps = smoothstep(0.4, 0.6, sin(coord.y * 40.0 + aurora2_large_noise * 15.0 + time * 0.3)) * 
                       aurora2_intensity * 0.2;
    let aurora2_wisp_color = oklab_to_linear_srgb(mix(red, pink, 0.3) * aurora2_wisps);
    
    // Add vertical curtain-like structures with fine detail that appear randomly
    // Create a pulsing pattern with approximately 2-3 second intervals
    // At 60fps, we need roughly 120-180 frames per cycle, so a frequency of about 0.03-0.05 Hz
    let pulse_frequency = 0.04; // Adjust for cycle time (lower = longer cycle)
    let pulse_phase = hash(floor(vec2<f32>(time * pulse_frequency, 0.0))); // Random phase per interval
    let pulse_intensity = smoothstep(0.75, 0.95, sin(time * pulse_frequency * 6.28 + pulse_phase * 6.28));
    
    // Make curtains even more subtle and rare
    let curtain_x = coord.x * 15.0 + large_noise * 5.0; // Lower frequency
    // Much rarer pulses by using higher threshold
    let pulse_visibility = smoothstep(0.85, 0.95, sin(time * pulse_frequency * 6.28 + pulse_phase * 6.28));
    let curtain_pattern = smoothstep(0.45, 0.65, sin(curtain_x + time * 0.7)) * // Wider smoothstep
                        smoothstep(0.3, 0.7, intensity) * 0.03 * // Drastically reduced intensity
                        pulse_visibility; // Extremely selective timing
    
    // Drastically reduce the ray pattern to near-imperceptible levels
    // Fine rays emanating from curtains with extremely subtle presence
    let rays_pattern = smoothstep(0.40, 0.70, // Very wide smoothstep for soft edges
                                 sin(coord.y * 40.0 + large_noise * 8.0 + time * 0.1) * // Lower frequency
                                 smoothstep(0.45, 0.65, sin(curtain_x + time * 0.7))) // Wider smoothstep
                     * intensity * 0.04 * // Drastically reduced intensity
                     pulse_visibility; // Only during the rare pulse moments
    
    let curtain_color = oklab_to_linear_srgb(mix(green, teal, 0.5) * curtain_pattern);
    let rays_color = oklab_to_linear_srgb(mix(green, blue, 0.4) * rays_pattern);
    
    // Combine all effects for final aurora color
    let aurora_color = rgb_color + glow_color * glow + 
                      vec3<f32>(0.1, 0.15, 0.2) * cross_effect + 
                      rgb_color * wisps +
                      aurora2_rgb + // Add the second aurora layer
                      aurora2_wisp_color + // Additional wisps to second aurora
                      curtain_color + // Add curtain structures
                      rays_color; // Add vertical rays
    
    // Generate celestial elements
    let nebula_color = nebula(coord, time);
    
    // Create a subtle vignette effect
    let vignette = smoothstep(1.2, 0.5, length(coord - vec2<f32>(0.5)));
    
    // Background gradient for night sky
    let bg_gradient = mix(
        vec3<f32>(0.0, 0.01, 0.03),  // Bottom - darker
        vec3<f32>(0.01, 0.03, 0.07), // Top - slightly lighter
        coord.y * 0.7
    ) * vignette;
    
    // Create the night sky background first
    let sky_color = bg_gradient + nebula_color;
    
    // Aurora overlay 
    let aurora_blend_factor = clamp(intensity * 0.7, 0.0, 0.8); 
    
    // Blend aurora over the sky
    let with_aurora = mix(sky_color, aurora_color, aurora_blend_factor);
    
    return vec4<f32>(with_aurora, intensity * 0.9 + 0.1);
}

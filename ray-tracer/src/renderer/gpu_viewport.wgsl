struct Params {
    camera_origin: vec4<f32>,
    camera_forward: vec4<f32>,
    camera_right: vec4<f32>,
    camera_up: vec4<f32>,
    viewport: vec4<f32>,
    scene: vec4<u32>,
    light_dir: vec4<f32>,
    plane_point: vec4<f32>,
    plane_normal: vec4<f32>,
    plane_base_ambient: vec4<f32>,
    plane_properties: vec4<f32>,
    plane_shadow_ambient: vec4<f32>,
};

struct GpuSphere {
    center_radius: vec4<f32>,
    base_ambient: vec4<f32>,
    properties: vec4<f32>,
    shadow_ambient: vec4<f32>,
};

struct SurfaceHit {
    t: f32,
    point: vec3<f32>,
    normal: vec3<f32>,
    base: vec3<f32>,
    ambient: f32,
    shadow_ambient: f32,
    diffuse: f32,
    specular: f32,
    shininess: f32,
    reflectivity: f32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> spheres: array<GpuSphere>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn is_occluded(point: vec3<f32>, normal: vec3<f32>) -> bool {
    let shadow_origin = point + normal * 0.001;
    let direction = params.light_dir.xyz;
    let sphere_count = min(params.scene.x, arrayLength(&spheres));
    for (var i = 0u; i < sphere_count; i += 1u) {
        let sphere = spheres[i];
        let offset = shadow_origin - sphere.center_radius.xyz;
        let half_b = dot(offset, direction);
        let c = dot(offset, offset) - sphere.center_radius.w * sphere.center_radius.w;
        let discriminant = half_b * half_b - c;
        if discriminant >= 0.0 {
            let root = sqrt(discriminant);
            var t = -half_b - root;
            if t <= 0.001 {
                t = -half_b + root;
            }
            if t > 0.001 {
                return true;
            }
        }
    }

    let plane_denom = dot(direction, params.plane_normal.xyz);
    if abs(plane_denom) > 1.0e-6 {
        let plane_t = dot(
            params.plane_point.xyz - shadow_origin,
            params.plane_normal.xyz,
        ) / plane_denom;
        if plane_t > 0.001 {
            return true;
        }
    }
    return false;
}

fn shade_surface(base: vec3<f32>, ambient: f32, shadow_ambient: f32,
                 diffuse: f32, specular: f32, shininess: f32,
                 point: vec3<f32>, normal: vec3<f32>,
                 ray_direction: vec3<f32>) -> vec3<f32> {
    let n_dot_l = max(dot(normal, params.light_dir.xyz), 0.0);
    let shadowed = params.scene.y != 0u && n_dot_l > 0.0 && is_occluded(point, normal);
    let direct_visibility = select(1.0, 0.0, shadowed);
    let ambient_light = select(ambient, shadow_ambient, shadowed);
    let light_reflection = reflect(-params.light_dir.xyz, normal);
    let view_direction = -ray_direction;
    let spec = pow(max(dot(light_reflection, view_direction), 0.0), shininess)
        * specular * direct_visibility;
    return clamp(base * (ambient_light + diffuse * n_dot_l * direct_visibility)
        + vec3<f32>(spec), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn trace_scene(origin: vec3<f32>, ray_direction: vec3<f32>) -> SurfaceHit {
    var hit = SurfaceHit(
        1.0e30,
        vec3<f32>(0.0),
        vec3<f32>(0.0),
        vec3<f32>(0.0),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
    );
    let sphere_count = min(params.scene.x, arrayLength(&spheres));
    for (var i = 0u; i < sphere_count; i += 1u) {
        let sphere = spheres[i];
        let offset = origin - sphere.center_radius.xyz;
        let half_b = dot(offset, ray_direction);
        let c = dot(offset, offset) - sphere.center_radius.w * sphere.center_radius.w;
        let discriminant = half_b * half_b - c;
        if discriminant >= 0.0 {
            let root = sqrt(discriminant);
            var t = -half_b - root;
            if t <= 0.001 {
                t = -half_b + root;
            }
            if t > 0.001 && t < hit.t {
                let point = origin + t * ray_direction;
                hit = SurfaceHit(
                    t,
                    point,
                    normalize(point - sphere.center_radius.xyz),
                    sphere.base_ambient.xyz,
                    sphere.base_ambient.w,
                    sphere.shadow_ambient.x,
                    sphere.properties.x,
                    sphere.properties.y,
                    sphere.properties.z,
                    sphere.properties.w,
                );
            }
        }
    }

    let plane_denom = dot(ray_direction, params.plane_normal.xyz);
    if abs(plane_denom) > 1.0e-6 {
        let t = dot(params.plane_point.xyz - origin, params.plane_normal.xyz) / plane_denom;
        if t > 0.001 && t < hit.t {
            hit = SurfaceHit(
                t,
                origin + t * ray_direction,
                params.plane_normal.xyz,
                params.plane_base_ambient.xyz,
                params.plane_base_ambient.w,
                params.plane_shadow_ambient.x,
                params.plane_properties.x,
                params.plane_properties.y,
                params.plane_properties.z,
                params.plane_properties.w,
            );
        }
    }
    return hit;
}

fn sample_sky(ray_direction: vec3<f32>) -> vec3<f32> {
    let sky_t = 0.5 * (ray_direction.y + 1.0);
    return mix(vec3<f32>(0.015, 0.02, 0.04), vec3<f32>(0.35, 0.55, 0.85), sky_t);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let local_pixel = input.position.xy - params.viewport.xy;
    let uv = local_pixel / params.viewport.zw;
    let screen = vec2<f32>(2.0 * uv.x - 1.0, 1.0 - 2.0 * uv.y);
    var ray_origin = params.camera_origin.xyz;
    var ray_direction = normalize(params.camera_forward.xyz
        + params.camera_right.xyz * screen.x
        + params.camera_up.xyz * screen.y);

    var radiance = vec3<f32>(0.0);
    var throughput = vec3<f32>(1.0);
    for (var bounce = 0u; bounce < 3u; bounce += 1u) {
        let hit = trace_scene(ray_origin, ray_direction);
        if hit.t >= 1.0e30 {
            radiance += throughput * sample_sky(ray_direction);
            throughput = vec3<f32>(0.0);
            break;
        }

        let local_color = shade_surface(
            hit.base,
            hit.ambient,
            hit.shadow_ambient,
            hit.diffuse,
            hit.specular,
            hit.shininess,
            hit.point,
            hit.normal,
            ray_direction,
        );
        let reflectivity = clamp(hit.reflectivity, 0.0, 1.0);
        radiance += throughput * (1.0 - reflectivity) * local_color;
        throughput *= reflectivity;
        if reflectivity <= 0.0 {
            break;
        }
        ray_origin = hit.point + hit.normal * 0.001;
        ray_direction = reflect(ray_direction, hit.normal);
    }

    if any(throughput > vec3<f32>(0.0)) {
        radiance += throughput * sample_sky(ray_direction);
    }
    return vec4<f32>(radiance, 1.0);
}

[[group(0), binding(0)]] 
var out_texture: [[access(write)]] texture_storage_2d<rgba32float>;
[[group(0), binding(1)]] 
var in_texture: [[access(read)]] texture_storage_2d<rgba32float>;

[[stage(compute), workgroup_size(32, 32)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let window_size = vec2<f32>(800.0, 600.0);
    let pixel_coordinates: vec2<i32> = vec2<i32>(global_id.xy);
    let uv = vec2<f32>(pixel_coordinates) / (window_size - 1.0);

    // let aspect_ratio = window_size.x / window_size.y;
    // let viewport_height = 2.0;
    // let viewport_width = aspect_ratio * viewport_height;
    // let focal_length = 1.0;

    // let origin = Vector(0.0);
    // let horizontal = Vector(viewport_width, 0.0, 0.0);
    // let vertical = Vector(0.0, viewport_height, 0.0);
    // let camera = Camera(origin, horizontal, vertical, origin - 0.5*horizontal + 0.5*vertical - Vector(0.0, 0.0, focal_length));

    // let r = get_ray(camera, uv.x, uv.y);

    // let pixel_Color = ray_Color(r);

    let pixel_color = textureLoad(in_texture, pixel_coordinates);
    textureStore(out_texture, pixel_coordinates, vec4<f32>(uv.x, uv.y, 0.25, 1.0));
}

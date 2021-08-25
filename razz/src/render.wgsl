[[stage(vertex)]]
fn main([[builtin(vertex_index)]] in_vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let x = -1.0 + f32((in_vertex_index & u32(1)) << u32(2));
    let y = -1.0 + f32((in_vertex_index & u32(2)) << u32(1));
    return vec4<f32>(x, y, 0.0, 1.0);
}

[[group(0), binding(0)]] 
var in_texture: [[access(read)]] texture_storage_2d<rgba32float>;

[[stage(fragment)]]
fn main([[builtin(position)]] coord_in: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let pixel_color = textureLoad(in_texture, vec2<i32>(coord_in.xy));
    return pixel_color;
}

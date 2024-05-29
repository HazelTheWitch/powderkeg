#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var chunk_texture: texture_2d<f32>;
@group(2) @binding(1) var chunk_texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = vec2<f32>(mesh.uv.x, 1.0 - mesh.uv.y);
    return textureSample(chunk_texture, chunk_texture_sampler, uv);
}

struct CameraUniform {
	view_pos: vec3<f32>,
  view_proj: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
		@location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(0) var texture: texture_2d<f32>; 
@group(1) @binding(1) var texture_sampler: sampler; 

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	var view_pos = camera.view_proj * vec4<f32>(input.position, 1.0);

	out.clip_position = view_pos;
	out.tex_coords = input.tex_coords;

	return out;

}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
		var sample = textureSample(texture, texture_sampler, in.tex_coords);
    return vec4<f32>(sample);
}



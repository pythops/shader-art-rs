struct Infos {
    dimensions: vec2<f32>,
    time: f32,
}

@group(0) @binding(0) var<uniform> infos: Infos;

struct VertexOutput {
    @builtin(position) foo: vec4f,
    @location(0) pos: vec2f,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.foo = vec4f(position, 0.0, 1.0);
    out.pos = position;
    return out;
}

fn color(t: f32) -> vec3f {
    let a = vec3f(0.5, 0.5, 0.5);
    let b = vec3f(0.5, 0.5, 0.5);
    let c = vec3f(1.0, 1.0, 1.0);
    let d = vec3f(0.263, 0.416, 0.557);
    return a + b * cos(6.28318 * (c * t + d));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let aspect_ratio = infos.dimensions.y / infos.dimensions.x;
    var uv = in.pos;
    uv.x = uv.x * aspect_ratio;

    let uv0 = uv;
    var final_color = vec3f(0.0, 0.0, 0.0);

    for (var i = 0; i < 6; i++) {
        uv = fract(uv * 1.5) - 0.5;

        var d = length(uv) * exp(-length(uv0));

        var col = color(length(uv0) + f32(i) * 0.4 + infos.time);

        d = sin(d * 8.0 + infos.time) / 8.0;
        d = abs(d);

        d = pow(0.005 / d, 3.0);
        final_color = final_color + col * d;
    }

    return vec4f(final_color, 1.0);
}

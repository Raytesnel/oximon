// blended_mist_procedural.glsl
uniform vec2 iCamera;

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float noise(vec2 p){
    vec2 i = floor(p);
    vec2 f = fract(p);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    vec2 u = f*f*(3.0-2.0*f);
    return mix(a, b, u.x) + (c - a)* u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

void mainImage(out vec4 fragColor, in vec2 fragCoord){
    vec2 uv = fragCoord / iResolution.xy;
    uv += iCamera / iResolution.xy;

    uv -= 0.5;

    // Create swirling mist effect
    float n = noise(uv * 5.0 + vec2(iTime*0.2, -iTime*0.1));
    float m = noise(uv * 7.0 - vec2(iTime*0.3, iTime*0.2));
    float fog = clamp((n + m) * 0.5, 0.0, 1.0);

    vec3 mist_color = vec3(0.85, 0.88, 0.9);
    float alpha = fog * 0.9;// lager is minder wolken te zen alleen gaten van licht

    //    fragColor = vec4(mist_color * alpha,alpha); // wolken met lich gaten
    fragColor = vec4(mist_color, alpha);// pure mist
}

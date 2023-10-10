fn vertex([[location(0)]] pos: vec3) -> [[builtin(position)]] vec4 {
    return vec4(pos, 1.0);
}

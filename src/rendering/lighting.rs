use crate::color_ext::ColorExt;
use crate::geometry::Vec3;
use pixels::wgpu::Color;

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct AmbientLight {
    pub color: Color,
    pub intensity: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct PhongMaterial {
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub specular_intensity: f32,
    pub shininess: f32,
}

pub fn phong_lighting(
    camera_position: Vec3,
    surface_point: Vec3,
    surface_normal: Vec3,
    light: PointLight,
    material: PhongMaterial,
    ambient_light: AmbientLight,
) -> Color {
    let light_dir = (light.position - surface_point).normalize();

    let reflected_light_dir = (light_dir * -1.0).reflect(surface_normal);
    let view_dir = (camera_position - surface_point).normalize();

    let ambient_color = material
        .diffuse_color
        .multiply_rgb(ambient_light.color)
        .scale_rgb(ambient_light.intensity);

    let diffuse_strength = surface_normal.dot(light_dir).max(0.0);
    let diffuse_color = material
        .diffuse_color
        .multiply_rgb(light.color)
        .scale_rgb(diffuse_strength * light.intensity);

    let specular_strength = if diffuse_strength > 0.0 {
        reflected_light_dir.dot(view_dir).max(0.0).powf(material.shininess)
    } else {
        0.0
    };

    let specular_color = material
        .specular_color
        .multiply_rgb(light.color)
        .scale_rgb(specular_strength * material.specular_intensity * light.intensity);

    ambient_color.add_rgb(diffuse_color).add_rgb(specular_color)
}

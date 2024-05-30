use std::marker::PhantomData;

use bevy::{asset::load_internal_asset, prelude::*, render::{render_asset::RenderAssetUsages, render_resource::AsBindGroup}, sprite::{Material2d, Material2dPlugin, Mesh2dHandle}};
use image::{DynamicImage, RgbaImage};

use crate::{cell::Renderable, chunk::Chunk, grid::Grid, stain::{Stain, Stainable}, PowderkegSet};

#[rustfmt::skip]
pub const CHUNK_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(33721791328259611974385727409331747184);

pub(crate) struct PowderkegViewPlugin<T: Renderable + Send + Sync + 'static, const N: i32>(PhantomData<T>);

impl<T, const N: i32> Default for PowderkegViewPlugin<T, N>
where
    T: Renderable + Send + Sync + 'static,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T, const N: i32> Plugin for PowderkegViewPlugin<T, N>
where
    T: Renderable,
{
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, CHUNK_SHADER_HANDLE, "chunk.wgsl", Shader::from_wgsl);
        
        app
            .add_plugins(Material2dPlugin::<ChunkMaterial>::default())
            .add_systems(Update, (
                instantiate_chunk_images::<T, N>,
                generate_chunk_images::<T, N>,
            ).chain().in_set(PowderkegSet::Render))
            .add_systems(Update, draw_stained::<T, N>);
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material2d for ChunkMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        CHUNK_SHADER_HANDLE.into()
    }
}

fn instantiate_chunk_images<T: Renderable + Send + Sync + 'static, const N: i32>(
    mut commands: Commands,
    query: Query<(Entity, &Chunk<T, N>), (Without<Mesh2dHandle>, Without<Handle<ChunkMaterial>>)>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, chunk) in query.iter() {
        let image_buffer = RgbaImage::new(N as u32, N as u32);
        let dynamic = DynamicImage::from(image_buffer);
        let mut image = Image::from_dynamic(dynamic, true, RenderAssetUsages::all());

        for y in 0..N {
            for x in 0..N {
                let point = IVec2::new(x, y);

                if let Some(index) = chunk.index(point) {
                    let [r, g, b, a] = chunk.at(point).to_color(point).as_rgba_u8();

                    image.data[4 * index + 0] = r;
                    image.data[4 * index + 1] = g;
                    image.data[4 * index + 2] = b;
                    image.data[4 * index + 3] = a;
                }
            }
        }

        let material = ChunkMaterial {
            texture: images.add(image),
        };

        commands
            .entity(entity)
            .insert((
                Mesh2dHandle::from(meshes.add(Rectangle::new(N as f32, N as f32))),
                materials.add(material),
            ));
    }
}

fn generate_chunk_images<T, const N: i32>(
    mut chunks: Query<(
        &Chunk<T, N>,
        &mut Handle<ChunkMaterial>,
        &ViewVisibility
    )>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
) where
    T: Renderable,
{
    for (chunk, material_handle, visible) in chunks.iter_mut() {
        if !visible.get() {
            continue;
        }

        let stain = chunk.stained();

        if stain.is_empty() {
            continue;
        }

        let Some(material) = materials.get_mut(&*material_handle) else {
            continue;
        };

        let Some(image) = images.get_mut(&material.texture) else {
            continue;
        };

        stain.apply(|point| {
            if let Some(index) = chunk.index(point) {
                let cell = chunk.at(point);
                let [r, g, b, a] = cell.to_color(point).as_rgba_u8();

                image.data[4 * index + 0] = r;
                image.data[4 * index + 1] = g;
                image.data[4 * index + 2] = b;
                image.data[4 * index + 3] = a;
            }
        });
    }
}

#[derive(Component)]
pub struct DrawStained;

fn draw_stained<T, const N: i32>(
    mut gizmos: Gizmos,
    chunks: Query<(&GlobalTransform, &Chunk<T, N>), With<DrawStained>>,
) where
    T: Renderable,
{
    for (transform, chunk) in chunks.iter() {
        let (s, _, t) = transform.to_scale_rotation_translation();

        let s = s.truncate();
        let t = t.truncate();

        match chunk.stained() {
            Stain::Empty => {},
            Stain::Area(area) => {
                let min = (area.min.as_vec2() - Vec2::splat(N as f32 / 2.0)) * s + t;
                let max = (area.max.as_vec2() - Vec2::splat(N as f32 / 2.0)) * s + t;

                gizmos.rect_2d((max + min) / 2.0, 0.0, max - min, Color::RED);
            },
            Stain::Many(areas) => {
                for area in areas.iter() {
                    let min = (area.min.as_vec2() - Vec2::splat(N as f32 / 2.0)) * s + t;
                    let max = (area.max.as_vec2() - Vec2::splat(N as f32 / 2.0)) * s + t;
    
                    gizmos.rect_2d((max + min) / 2.0, 0.0, max - min, Color::RED);
                }
            },
        }
    }
}

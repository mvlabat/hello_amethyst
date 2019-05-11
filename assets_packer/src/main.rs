use amethyst::{
    animation::{
        AnimationPrefab, AnimationSetPrefab, InterpolationFunction, Sampler, SpriteRenderChannel,
        SpriteRenderPrimitive,
    },
    assets::Prefab,
    core::Named,
    renderer::{
        SpriteList, SpritePosition, SpriteScenePrefab, SpriteSheetPrefab, Sprites, TextureFormat,
        TextureMetadata, TexturePrefab,
    },
};
use failure;
use image;
use ron::{
    de::from_str,
    ser::{to_string_pretty, PrettyConfig},
};
use texture_packer::{
    exporter::ImageExporter, importer::ImageImporter, Frame, TexturePacker, TexturePackerConfig,
};

use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use animation_prefabs::{AnimationId, GameSpriteAnimationPrefab};
use texture_packer::texture::Texture;

struct SpriteSceneData {
    sprite_sheet: SpriteSheetPrefab,
    torso_indices: Vec<SpriteRenderPrimitive>,
    legs_indices: Vec<SpriteRenderPrimitive>,
}

const FRAMES_COUNT: usize = 20;

fn main() -> Result<(), failure::Error> {
    let config = TexturePackerConfig {
        allow_rotation: false,
        ..Default::default()
    };

    let mut packer = TexturePacker::new_skyline(config);

    let import_path = match env::var("GRUMPY_IMPORT_PATH") {
        Ok(path) => path,
        Err(env::VarError::NotPresent) => "assets_packer/input".to_owned(),
        Err(env::VarError::NotUnicode(_)) => panic!("GRUMPY_IMPORT_PATH value is invalid"),
    };
    let import_path = Path::new(&import_path);

    // Pack torso.
    for i in 0..FRAMES_COUNT {
        let name = format!("mage64_{:04}.png", i);
        let path = import_path.join(&name);
        let texture = ImageImporter::import_from_file(&path).unwrap();

        packer.pack_own(name, texture);
    }
    // Pack legs.
    for i in 0..FRAMES_COUNT {
        let name = format!("mage64_legs_{:04}.png", i);
        let path = import_path.join(&name);
        let texture = ImageImporter::import_from_file(&path).unwrap();

        packer.pack_own(name, texture);
    }

    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output");
    let output_file = output_dir.join("atlas.png");

    let SpriteSceneData {
        sprite_sheet,
        torso_indices,
        legs_indices,
    } = construct_sprite_scene(
        packer.get_frames(),
        packer.width(),
        packer.height(),
        &output_file,
    );

    let prefab = {
        let mut prefab = Prefab::new();
        prefab.main(Some(create_prefab(
            "hero_torso",
            Some(sprite_sheet),
            torso_indices,
            AnimationId::Walk,
        )));
        prefab.add(None,
        Some(create_prefab(
            "hero_legs",
            None,
            legs_indices,
            AnimationId::Walk,
        )),
        );
        prefab
    };
    fs::create_dir_all(&output_dir).unwrap();

    let ron_metadata = to_string_pretty(
        &prefab,
        PrettyConfig {
            new_line: "\n".to_owned(),
            ..PrettyConfig::default()
        },
    )?;
    let mut file = File::create(output_dir.join("animation_metadata.ron"))?;
    file.write_all(ron_metadata.as_bytes())?;

    let exporter = ImageExporter::export(&packer).unwrap();

    let mut file = File::create(output_file)?;
    exporter.write_to(&mut file, image::PNG)?;
    Ok(())
}

fn construct_sprite_scene(
    frames: &HashMap<String, Frame>,
    atlas_width: u32,
    atlas_height: u32,
    output_file_path: impl AsRef<Path>,
) -> SpriteSceneData {
    let mut sprites = Vec::with_capacity(frames.len());
    let mut torso_indices = vec![SpriteRenderPrimitive::SpriteIndex(0); 20];
    let mut legs_indices = vec![SpriteRenderPrimitive::SpriteIndex(0); 20];

    for (sprite_index, (filename, frame)) in frames.iter().enumerate() {
        sprites.push(SpritePosition {
            x: frame.frame.x,
            y: frame.frame.y,
            width: frame.frame.w,
            height: frame.frame.h,
            offsets: Some([(frame.source.x as f32), (frame.source.y as f32)]),
        });

        let number = &filename[filename.len() - 8..filename.len() - 4];
        let sprite_index = SpriteRenderPrimitive::SpriteIndex(sprite_index);
        let i = number
            .parse::<usize>()
            .expect("Expected a PNG file with 4 digit index in the filename");
        if filename.contains("legs") {
            legs_indices[i] = sprite_index;
        } else {
            torso_indices[i] = sprite_index;
        }
    }

    SpriteSceneData {
        sprite_sheet: SpriteSheetPrefab::Sheet {
            texture: TexturePrefab::File(
                output_file_path.as_ref().to_str().unwrap().to_owned(),
                TextureFormat::Png,
                TextureMetadata::srgb(),
            ),
            sprites: vec![Sprites::List(SpriteList {
                texture_width: atlas_width,
                texture_height: atlas_height,
                sprites,
            })],
            name: None,
        },
        torso_indices,
        legs_indices,
    }
}

fn create_prefab(
    name: &'static str,
    sprite_sheet: Option<SpriteSheetPrefab>,
    frames_indices: Vec<SpriteRenderPrimitive>,
    animation_id: AnimationId,
) -> GameSpriteAnimationPrefab {
    GameSpriteAnimationPrefab {
        name: Named::new(name),
        sprite_scene: SpriteScenePrefab {
            sheet: sprite_sheet,
            // TODO: fix after https://github.com/amethyst/amethyst/issues/1585.
            render: from_str("Some((sheet: 0, sprite_number: 0))").unwrap(),
            transform: None,
        },
        animation_set: AnimationSetPrefab {
            animations: vec![(animation_id, {
                let mut prefab = AnimationPrefab::default();
                prefab.samplers = vec![(
                    0,
                    SpriteRenderChannel::SpriteIndex,
                    Sampler {
                        input: (0..FRAMES_COUNT).map(|i| i as f32 * 0.05).collect(),
                        output: frames_indices,
                        function: InterpolationFunction::Step,
                    },
                )];
                prefab
            })],
        },
    }
}

use std::{
    cell::RefCell,
    collections::HashMap,
    io::{BufReader, Cursor},
    rc::Rc,
};

use anyhow::anyhow;
use common::store_ext::StoreExt2;
use crosscom::ComRc;
use fileformats::{binrw::BinRead, cam::CameraDataFile, npc::NpcInfoFile};
use mini_fs::{MiniFs, StoreExt};
use radiance::{
    comdef::{IEntity, IScene},
    components::mesh::skinned_mesh::AnimKeyFrame,
    rendering::{ComponentFactory, Sprite},
    scene::CoreScene,
    utils::SeekRead,
};

use crate::{
    loaders::{
        anm::load_anm, bsp::create_entity_from_bsp_model, dff::create_entity_from_dff_model,
        smp::load_smp, Pal4TextureResolver,
    },
    scripting::angelscript::ScriptModule,
};

use super::actor::Pal4Actor;

pub struct AssetLoader {
    vfs: MiniFs,
    component_factory: Rc<dyn ComponentFactory>,
    texture_resolver: Pal4TextureResolver,
    portraits: HashMap<String, ImageSetImage>,
}

impl AssetLoader {
    pub fn new(component_factory: Rc<dyn ComponentFactory>, vfs: MiniFs) -> Rc<Self> {
        let portraits = load_portraits(&component_factory, &vfs);
        Rc::new(Self {
            component_factory,
            vfs,
            texture_resolver: Pal4TextureResolver {},
            portraits,
        })
    }

    pub fn load_script_module(&self, scene: &str) -> anyhow::Result<Rc<RefCell<ScriptModule>>> {
        let content = self
            .vfs
            .read_to_end(&format!("/gamedata/script/{}.csb", scene))?;
        Ok(Rc::new(RefCell::new(
            ScriptModule::read_from_buffer(&content).unwrap(),
        )))
    }

    pub fn load_actor(
        &self,
        entity_name: &str,
        actor_name: &str,
        default_act: Option<&str>,
    ) -> anyhow::Result<ComRc<IEntity>> {
        let model_path = format!("/gamedata/PALActor/{}/{}.dff", actor_name, actor_name);

        let entity = create_entity_from_dff_model(
            &self.component_factory,
            &self.vfs,
            model_path,
            entity_name.to_string(),
            true,
            &self.texture_resolver,
        );

        if let Some(default_act) = default_act {
            let act_path = format!("/gamedata/PALActor/{}/{}.anm", actor_name, default_act);
            let anm = load_anm(&self.vfs, &act_path)?;
            Pal4Actor::set_anim(entity.clone(), &anm);
        }

        Ok(entity)
    }

    pub fn load_anm(
        &self,
        actor_name: &str,
        act_name: &str,
    ) -> anyhow::Result<Vec<Vec<AnimKeyFrame>>> {
        let act_path = format!("/gamedata/PALActor/{}/{}.anm", actor_name, act_name);
        Ok(load_anm(&self.vfs, &act_path)?)
    }

    pub fn load_scene(&self, scene_name: &str, block_name: &str) -> anyhow::Result<ComRc<IScene>> {
        let path = format!(
            "/gamedata/PALWorld/{}/{}/{}.bsp",
            scene_name, block_name, block_name,
        );

        let scene = CoreScene::create();
        let entity = create_entity_from_bsp_model(
            &self.component_factory,
            &self.vfs,
            path,
            "world".to_string(),
            &self.texture_resolver,
        );

        scene.add_entity(entity);
        Ok(scene)
    }

    pub fn load_npc_info(&self, scene_name: &str, block_name: &str) -> anyhow::Result<NpcInfoFile> {
        let path = format!(
            "/gamedata/scenedata/{}/{}/npcInfo.npc",
            scene_name, block_name,
        );

        let data = self.vfs.read_to_end(&path)?;
        let mut cursor = Cursor::new(data);
        Ok(NpcInfoFile::read(&mut cursor)?)
    }

    pub fn load_video(&self, video_name: &str) -> anyhow::Result<Box<dyn SeekRead>> {
        let video_folder = match video_name.to_lowercase().as_str() {
            "1a.bik" | "end2.bik" | "pal4a.bik" => "VideoA",
            _ => "videob",
        };

        let path = format!("/gamedata/{}/{}", video_folder, video_name);
        Ok(Box::new(BufReader::new(self.vfs.open(&path)?)))
    }

    pub fn load_music(&self, music_name: &str) -> anyhow::Result<Vec<u8>> {
        let path = format!("/gamedata/Music/{}.smp", music_name);
        let data = load_smp(self.vfs.read_to_end(path)?)?;
        Ok(data)
    }

    pub fn load_sound(&self, sound_name: &str, ext: &str) -> anyhow::Result<Vec<u8>> {
        let path = format!("/gamedata/PALSound/{}.{}", sound_name, ext);
        let data = self.vfs.read_to_end(path)?;
        Ok(data)
    }

    pub fn load_camera_data(
        &self,
        camera_data_name: &str,
        scene_name: &str,
        block_name: &str,
    ) -> anyhow::Result<CameraDataFile> {
        let path = format!(
            "/gamedata/scenedata/{}/{}/{}.cam",
            scene_name, block_name, camera_data_name,
        );
        let data = CameraDataFile::read(&mut Cursor::new(self.vfs.read_to_end(path)?))?;
        Ok(data)
    }

    pub fn load_portrait(&self, name: &str) -> Option<ImageSetImage> {
        self.portraits.get(&name.to_lowercase()).cloned()
    }
}

#[derive(Clone)]
pub struct ImageSetImage {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub sprite: Rc<Sprite>,
}

fn load_portraits(
    component_factory: &Rc<dyn ComponentFactory>,
    vfs: &MiniFs,
) -> HashMap<String, ImageSetImage> {
    let portrait_files = [
        "/gamedata/ui/portrait/portrait0_0.imageset",
        "/gamedata/ui/portrait/portrait1_0.imageset",
        "/gamedata/ui/portrait/portrait2_0.imageset",
        "/gamedata/ui/portrait/portrait3_0.imageset",
        "/gamedata/ui/portrait/portrait4_0.imageset",
        "/gamedata/ui/portrait/portrait5_0.imageset",
        "/gamedata/ui/portrait/portrait6_0.imageset",
        "/gamedata/ui/portrait/portrait7_0.imageset",
        "/gamedata/ui/portrait/portrait8_0.imageset",
    ];

    let mut portraits = HashMap::new();
    for portrait_file in &portrait_files {
        let ret = load_portraits_single(component_factory, vfs, portrait_file, &mut portraits);
        if let Err(e) = ret {
            log::error!("load_portraits_single failed: {:?}", e);
        }
    }

    portraits
}

fn load_portraits_single(
    component_factory: &Rc<dyn ComponentFactory>,
    vfs: &MiniFs,
    imageset: &str,
    portraits: &mut HashMap<String, ImageSetImage>,
) -> anyhow::Result<()> {
    let data = vfs.read_to_end(imageset)?;
    let content = String::from_utf8_lossy(&data);
    let root = roxmltree::Document::parse(&content)?;
    let root = root.root_element();
    let image_file = root
        .attribute("Imagefile")
        .ok_or(anyhow!("Missing Imagefile attribute in root node"))?;
    let image_file = if !image_file.starts_with(&['/', '\\']) {
        format!("/{}", image_file)
    } else {
        image_file.to_string()
    };

    for image in root
        .children()
        .filter(|n| n.is_element() && n.tag_name().name() == "Image")
    {
        let mut read_node = || -> anyhow::Result<()> {
            let name = image
                .attribute("Name")
                .ok_or(anyhow!("Missing Name in image node"))?
                .to_lowercase();
            let x = image
                .attribute("XPos")
                .ok_or(anyhow!("Missing XPos in image node"))?
                .parse::<u32>()?;
            let y = image
                .attribute("YPos")
                .ok_or(anyhow!("Missing YPos in image node"))?
                .parse::<u32>()?;
            let width = image
                .attribute("Width")
                .ok_or(anyhow!("Missing Width in image node"))?
                .parse::<u32>()?;
            let height = image
                .attribute("Height")
                .ok_or(anyhow!("Missing Height in image node"))?
                .parse::<u32>()?;
            let sprite = Rc::new(Sprite::load_from_buffer(
                &vfs.read_to_end(&image_file)?,
                image::ImageFormat::Png,
                component_factory.as_ref(),
            ));
            portraits.insert(
                name.clone(),
                ImageSetImage {
                    name,
                    x,
                    y,
                    width,
                    height,
                    sprite,
                },
            );

            Ok(())
        };

        let ret = read_node();
        if let Err(e) = ret {
            log::error!("load portrait node failed: {:?}", e);
        }
    }

    Ok(())
}

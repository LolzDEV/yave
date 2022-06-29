use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use bevy_ecs::system::ResMut;
use log::info;
use wgpu::{ShaderModule, ShaderSource};
use crate::client::renderer::Renderer;

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    namespace: String,
    name: String,
}

impl Identifier {
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            name: name.to_string()
        }
    }
}

impl Eq for Identifier {}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.name.hash(state);
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}:{}", self.namespace, self.name))
    }
}

/// The asset manager holds game assets like textures, shaders, models and so on
pub struct AssetManager {
    shaders: HashMap<Identifier, ShaderModule>
}

impl AssetManager {
    pub fn new(renderer: &Renderer) -> Self {
        let mut shaders = HashMap::new();

        for namespace in fs::read_dir("assets").unwrap() {
            let namespace = namespace.unwrap();
            for shader in fs::read_dir(namespace.path().join("shaders")).unwrap() {
                let shader = shader.unwrap();
                let name = shader.file_name().into_string().unwrap().replace(&format!(".{}", shader.path().extension().unwrap().to_str().unwrap()), "");
                let id = Identifier::new(namespace.file_name().to_str().unwrap(), &name);

                info!("Loading shader {}", id);

                let shader = renderer.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some(&id.to_string()),
                    source: ShaderSource::Wgsl(fs::read_to_string(shader.path()).unwrap().into())
                });

                shaders.insert(id, shader);
            }
        }

        Self {
            shaders,
        }
    }

    pub fn get_shader(&self, id: Identifier) -> Option<&ShaderModule> {
        self.shaders.get(&id)
    }
}
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::fs;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use log::{error, info};
use wgpu::{Face, FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology, ShaderModule, ShaderSource, VertexState};
use crate::client::renderer::{PipelineBundle, Renderer, RenderPipelineDescription};
use crate::client::voxel::{BlockDescription, VoxelVertex};

/// An identifier is a structure used to identify objects in game like entities, textures, shaders and everything else
#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    /// The identifier namespace e.g. in base:world the namespace is "base"
    namespace: String,
    /// The identifier name e.g. in base:world the namespace is "world"
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

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(":").collect();
        Ok(Identifier::new(split.get(0).unwrap(), split.get(1).unwrap()))
    }
}

/// The asset manager holds game assets like textures, shaders, models and so on
pub struct AssetManager {
    /// All the shaders beign loaded at startup
    shaders: HashMap<Identifier, ShaderModule>,
    /// All the pipelines beign loaded at startup
    pipelines: HashMap<Identifier, PipelineBundle>
}

impl AssetManager {
    pub fn new(renderer: &Renderer) -> Self {
        let mut shaders = HashMap::new();
        let mut pipelines = HashMap::new();

        for namespace in fs::read_dir("assets").unwrap() {
            let namespace = namespace.unwrap();

            for block in fs::read_dir(namespace.path().join("blocks")).unwrap() {
                let block = block.unwrap();
                let name = block.file_name().into_string().unwrap().replace(&format!(".{}", block.path().extension().unwrap().to_str().unwrap()), "");
                let id = Identifier::new(namespace.file_name().to_str().unwrap(), &name);

                info!("Loading block {}", id);

                let description: Result<BlockDescription, toml::de::Error> = toml::from_str(&fs::read_to_string(block.path()).unwrap());

                match description {
                    Ok(desc) => {

                    }
                    Err(e) => error!("Cannot create block with id {id}: {e}"),
                }
            }

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

            for pipeline in fs::read_dir(namespace.path().join("pipelines")).unwrap() {
                let pipeline = pipeline.unwrap();
                let name = pipeline.file_name().into_string().unwrap().replace(&format!(".{}", pipeline.path().extension().unwrap().to_str().unwrap()), "");
                let id = Identifier::new(namespace.file_name().to_str().unwrap(), &name);

                info!("Loading pipeline {}", id);

                let description: Result<RenderPipelineDescription, toml::de::Error> = toml::from_str(&fs::read_to_string(pipeline.path()).unwrap());

                match description {
                    Ok(desc) => {
                        let pipeline_layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some(&id.to_string()),
                            bind_group_layouts: &[],
                            push_constant_ranges: &[]
                        });

                        let render_pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some(&id.to_string()),
                            layout: Some(&pipeline_layout),
                            vertex: VertexState {
                                module: shaders.get(&Identifier::from_str(&desc.vertex_module).unwrap()).unwrap(),
                                entry_point: &desc.vertex_entry,
                                buffers: &[VoxelVertex::desc()]
                            },
                            primitive: PrimitiveState {
                                topology: match desc.primitive.topology.as_str() {
                                    "triangle_list" => PrimitiveTopology::TriangleList,
                                    "line_list" => PrimitiveTopology::LineList,
                                    "point_list" => PrimitiveTopology::PointList,
                                    "line_strip" => PrimitiveTopology::LineStrip,
                                    _ => PrimitiveTopology::default(),
                                },
                                strip_index_format: None,
                                front_face: match desc.primitive.front_face.as_str() {
                                    "cw" => FrontFace::Cw,
                                    "ccw" => FrontFace::Ccw,
                                    _ => FrontFace::default(),
                                },
                                cull_mode: if let Some(cull_mode) = desc.primitive.cull_mode { match cull_mode.as_str() {
                                    "back" => Some(Face::Back),
                                    "front" => Some(Face::Front),
                                    _ => None
                                } } else { None },
                                unclipped_depth: desc.primitive.unclipped_depth,
                                polygon_mode: match desc.primitive.polygon_mode.as_str() {
                                    "fill" => PolygonMode::Fill,
                                    "point" => PolygonMode::Point,
                                    "line" => PolygonMode::Line,
                                    _ => PolygonMode::default(),
                                },
                                conservative: desc.primitive.conservative
                            },
                            depth_stencil: None,
                            multisample: MultisampleState {
                                count: desc.samples,
                                mask: !0,
                                alpha_to_coverage_enabled: false
                            },
                            fragment: Some(FragmentState {
                                module: shaders.get(&Identifier::from_str(&desc.fragment_module).unwrap()).unwrap(),
                                entry_point: &desc.fragment_entry,
                                targets: &[wgpu::ColorTargetState {
                                    format: renderer.surface_config.format,
                                    blend: Some(wgpu::BlendState::REPLACE),
                                    write_mask: wgpu::ColorWrites::ALL
                                }]
                            }),
                            multiview: None
                        });

                        pipelines.insert(id, PipelineBundle { pipeline_layout, render_pipeline });
                    }
                    Err(e) => error!("Cannot create pipeline with id {id}: {e}"),
                }
            }
        }

        Self {
            shaders,
            pipelines
        }
    }

    /// Get a shader module from its id
    pub fn get_shader(&self, id: Identifier) -> Option<&ShaderModule> {
        self.shaders.get(&id)
    }

    pub fn get_pipeline(&self, id: Identifier) -> Option<&PipelineBundle> {
        self.pipelines.get(&id)
    }
}
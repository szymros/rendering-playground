pub trait Wgsl {
    fn wgsl(&self) -> String;
}

pub trait WgslType {
    fn wgsl_type() -> String;
}

impl Wgsl for f32 {
    fn wgsl(&self) -> String {
        return format!("{:?}", self);
    }
}

impl WgslType for f32 {
    fn wgsl_type() -> String {
        return "f32".to_string();
    }
}

impl Wgsl for u32 {
    fn wgsl(&self) -> String {
        return self.to_string();
    }
}

impl WgslType for u32 {
    fn wgsl_type() -> String {
        return "u32".to_string();
    }
}

impl Wgsl for (f32, f32) {
    fn wgsl(&self) -> String {
        return format!("vec2<f32>({:?},{:?})", self.0, self.1);
    }
}

impl WgslType for (f32, f32) {
    fn wgsl_type() -> String {
        return "vec2<f32>".to_string();
    }
}

#[derive(Hash, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Attribute {
    Pos,
    Velocity,
    Acceleration,
    Color,
    Age,
    Lifetime,
    SpriteIdx,
}

impl Wgsl for Attribute {
    fn wgsl(&self) -> String {
        return match self {
            Attribute::Pos => "pos",
            Attribute::Velocity => "velocity",
            Attribute::Acceleration => "acceleration",
            Attribute::Color => "color",
            Attribute::Age => "age",
            Attribute::Lifetime => "lifetime",
            Attribute::SpriteIdx => "sprite_idx",
        }
        .to_string();
    }
}

pub enum Value<T> {
    Constant(T),
    RandRange { start: T, end: T },
    RandChoice(Vec<T>),
    Attribute(Attribute),
    LocalParam { slot: u32 },
}

impl<T: Wgsl + WgslType> Wgsl for Value<T> {
    fn wgsl(&self) -> String {
        return match self {
            Value::Constant(x) => x.wgsl(),
            Value::RandRange { start, end } => {
                format!("mix({},{},rand(&seed))", start.wgsl(), end.wgsl())
            }
            Value::RandChoice(choices) => {
                let num_choices = choices.len();
                assert!(num_choices > 0);
                let items = choices
                    .iter()
                    .map(|v| v.wgsl())
                    .collect::<Vec<String>>()
                    .join(",");
                let x = format!(
                    "
                        array<{0},{1}>({2})[u32(rand(&seed)*{1})]
                    ",
                    T::wgsl_type(),
                    num_choices,
                    items,
                );
                println!("{}", x);
                x
            }
            Value::Attribute(attribute) => todo!(),
            Value::LocalParam { slot: _ } => todo!(),
        };
    }
}

#[derive(Debug, PartialEq)]
pub enum ModifierStage {
    Init,
    Update,
    Render,
}

pub trait Modifier {
    fn stage(&self) -> &[ModifierStage];
    fn attribute(&self) -> Attribute;
    fn wgsl(&self) -> String;
}

pub struct ConstModifier<T: Wgsl> {
    pub attribute: Attribute,
    pub value: Value<T>,
}

impl<T: Wgsl + WgslType> Modifier for ConstModifier<T> {
    fn stage(&self) -> &[ModifierStage] {
        return &[ModifierStage::Init, ModifierStage::Update];
    }

    fn attribute(&self) -> Attribute {
        return self.attribute;
    }

    fn wgsl(&self) -> String {
        return format!(
            "particle.{} = {};",
            self.attribute().wgsl(),
            self.value.wgsl()
        );
    }
}
pub struct LinearDragModifier {
    pub value: Value<f32>,
}

impl Modifier for LinearDragModifier {
    fn stage(&self) -> &[ModifierStage] {
        return &[ModifierStage::Update];
    }

    fn attribute(&self) -> Attribute {
        return Attribute::Velocity;
    }

    fn wgsl(&self) -> String {
        return format!(
            "particle.velocity *= max(0.0, 1.0 - {} * globals.time_delta);",
            self.value.wgsl()
        );
    }
}

pub struct InitPositionCircleModifier {
    pub radius: Value<f32>,
}

impl Modifier for InitPositionCircleModifier {
    fn stage(&self) -> &[ModifierStage] {
        return &[ModifierStage::Init];
    }

    fn attribute(&self) -> Attribute {
        return Attribute::Pos;
    }

    fn wgsl(&self) -> String {
        let out = format!(
            "
            let theta = 3.1415926 * 2.0 * rand(&seed);
            let radius = {};
            particle.pos = locals.origin.xy + radius * vec2<f32>(cos(theta), sin(theta));
            ",
            self.radius.wgsl(),
        );
        return out;
    }
}

pub enum Direction {
    Inward,
    Outward,
}

pub struct VelocityCircleModifier {
    pub velocity: Value<f32>,
    pub dir: Direction,
}

impl Modifier for VelocityCircleModifier {
    fn stage(&self) -> &[ModifierStage] {
        return &[ModifierStage::Init, ModifierStage::Update];
    }

    fn attribute(&self) -> Attribute {
        return Attribute::Velocity;
    }

    fn wgsl(&self) -> String {
        let dir_str = match self.dir {
            Direction::Outward => "let vel_dir = normalize(particle.pos - locals.origin.xy);",
            Direction::Inward => "let vel_dir = normalize(locals.origin.xy - particle.pos);",
        };
        return format!(
            "{} particle.velocity = vel_dir * {};",
            dir_str,
            self.velocity.wgsl()
        );
    }
}

pub struct AccelerationCircleModifier {
    pub acceleration: Value<f32>,
    pub dir: Direction,
}

impl Modifier for AccelerationCircleModifier {
    fn stage(&self) -> &[ModifierStage] {
        return &[ModifierStage::Init, ModifierStage::Update];
    }

    fn attribute(&self) -> Attribute {
        return Attribute::Acceleration;
    }

    fn wgsl(&self) -> String {
        let dir_str = match self.dir {
            Direction::Outward => "let accel_dir = normalize(particle.pos - locals.origin.xy);",
            Direction::Inward => "let accel_dir = normalize(locals.origin.xy - particle.pos);",
        };
        return format!(
            "{} particle.acceleration = accel_dir * {};",
            dir_str,
            self.acceleration.wgsl()
        );
    }
}

pub struct AccelerationOrbitModifier {
    pub modifier_value: Value<f32>,
    pub clockwise: bool,
}

impl Modifier for AccelerationOrbitModifier {
    fn stage(&self) -> &[ModifierStage] {
        return &[ModifierStage::Init, ModifierStage::Update];
    }

    fn attribute(&self) -> Attribute {
        return Attribute::Acceleration;
    }

    fn wgsl(&self) -> String {
        let rand_id = format!("accel_dir_{}", rand::random::<u32>());
        let tangent_dir = if self.clockwise {
            format!("vec2<f32>(-{0}.y, {0}.x)", rand_id)
        } else {
            format!("vec2<f32>({0}.y, -{0}.x)", rand_id)
        };
        return format!(
            "
            let {} = normalize(particle.pos - locals.origin.xy);
            let tangent = {};
            particle.acceleration = tangent * {};
            ",
            rand_id,
            tangent_dir,
            self.modifier_value.wgsl()
        );
    }
}

pub struct SpriteModifier {
    pub sprite_idx: u32,
    pub atlas_size: (u32, u32),
}

impl Modifier for SpriteModifier {
    fn stage(&self) -> &[ModifierStage] {
        return &[ModifierStage::Render];
    }

    fn attribute(&self) -> Attribute {
        return Attribute::SpriteIdx;
    }

    fn wgsl(&self) -> String {
        return format!(
            "
                let sprite_idx = {};
                let rows_cols = vec2<f32>({},{}); 
                let cell_size = 1.0 / rows_cols;
                let sprite_cord = vec2<f32>(f32(sprite_idx  % {}), f32(sprite_idx / {}));
                let sprite_uv = (v_in.uv + sprite_cord) * cell_size;
                color *= textureSample(tex, tex_sampler, sprite_uv);
            ",
            self.sprite_idx,
            self.atlas_size.0 as f32,
            self.atlas_size.1 as f32,
            self.atlas_size.0 as f32,
            self.atlas_size.1 as f32,
        );
    }
}

pub struct FlipBookModifier {
    pub base_sprite_idx: u32,
    pub frame_time: f32,
    pub num_frames: u32,
    pub atlas_size: (u32, u32),
    pub repeat: bool,
}

impl Modifier for FlipBookModifier {
    fn stage(&self) -> &[ModifierStage] {
        return &[ModifierStage::Render];
    }

    fn attribute(&self) -> Attribute {
        return Attribute::SpriteIdx;
    }

    fn wgsl(&self) -> String {
        let repeat = if self.repeat {
            format!(
                "let frame = floor(particle.age / {}) % {};",
                self.frame_time, self.num_frames
            )
        } else {
            format!(
                "let frame = floor(min(particle.age / {}, {} ));",
                self.frame_time,
                self.num_frames as f32 - 1.0
            )
        };
        return format!(
            "
            {}
            let sprite_idx = {} + frame;
            let rows_cols = vec2<f32>({},{}); 
            let cell_size = 1.0 / rows_cols;
            let sprite_cord = vec2<f32>(f32(sprite_idx  % {}), f32(sprite_idx / {}));
            let sprite_uv = (v_in.uv + sprite_cord) * cell_size;
            color = textureSample(tex, tex_sampler, sprite_uv);
            ",
            repeat,
            self.base_sprite_idx,
            self.atlas_size.0 as f32,
            self.atlas_size.1 as f32,
            self.atlas_size.0 as f32,
            self.atlas_size.1 as f32,
        );
    }
}

impl Wgsl for (f32, f32, f32, f32) {
    fn wgsl(&self) -> String {
        return format!(
            "vec4<f32>({:?},{:?},{:?},{:?})",
            self.0, self.1, self.2, self.3
        );
    }
}

pub struct ColorGradientOverTime {
    pub start: (f32, f32, f32, f32),
    pub end: (f32, f32, f32, f32),
}

impl Modifier for ColorGradientOverTime {
    fn stage(&self) -> &[ModifierStage] {
        return &[ModifierStage::Render];
    }

    fn attribute(&self) -> Attribute {
        return Attribute::Color;
    }

    fn wgsl(&self) -> String {
        return format!(
            "color = mix({},{},particle.age / particle.lifetime);",
            self.start.wgsl(),
            self.end.wgsl()
        );
    }
}

pub struct ParamModifier {
    pub param_slot: u32,
    pub attribute: Attribute,
}

impl Modifier for ParamModifier {
    fn stage(&self) -> &[ModifierStage] {
        return &[ModifierStage::Init, ModifierStage::Update];
    }

    fn attribute(&self) -> Attribute {
        return self.attribute;
    }

    fn wgsl(&self) -> String {
        return format!(
            "particle.{} = locals.params[{}];",
            self.attribute.wgsl(),
            self.param_slot
        );
    }
}

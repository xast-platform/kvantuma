use bytemuck::{Pod, Zeroable};
use flecs_ecs::macros::Component;
use glam::{Mat4, Quat, Vec3};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct JointId(pub usize);

#[derive(Clone, Debug)]
pub struct Joint {
    pub name: String,
    pub parent: Option<JointId>,
    pub inverse_bind_matrix: Mat4,
    pub local_transform: Mat4,
}

#[derive(Clone, Debug, Component)]
pub struct Skeleton {
    pub joints: Vec<Joint>,
    pub joint_map: HashMap<String, JointId>,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            joints: Vec::new(),
            joint_map: HashMap::new(),
        }
    }

    pub fn add_joint(&mut self, name: String, parent: Option<JointId>, inverse_bind_matrix: Mat4) -> JointId {
        let id = JointId(self.joints.len());
        self.joint_map.insert(name.clone(), id);
        self.joints.push(Joint {
            name,
            parent,
            inverse_bind_matrix,
            local_transform: Mat4::IDENTITY,
        });
        id
    }

    pub fn get_joint(&self, id: JointId) -> Option<&Joint> {
        self.joints.get(id.0)
    }

    pub fn get_joint_mut(&mut self, id: JointId) -> Option<&mut Joint> {
        self.joints.get_mut(id.0)
    }

    pub fn find_joint(&self, name: &str) -> Option<JointId> {
        self.joint_map.get(name).copied()
    }

    pub fn compute_skinning_matrices(&self) -> Vec<Mat4> {
        let mut global_transforms = vec![Mat4::IDENTITY; self.joints.len()];
        let mut skinning_matrices = vec![Mat4::IDENTITY; self.joints.len()];

        for (idx, joint) in self.joints.iter().enumerate() {
            let parent_transform = joint.parent
                .and_then(|parent_id| global_transforms.get(parent_id.0))
                .copied()
                .unwrap_or(Mat4::IDENTITY);

            global_transforms[idx] = parent_transform * joint.local_transform;
            skinning_matrices[idx] = global_transforms[idx] * joint.inverse_bind_matrix;
        }

        skinning_matrices
    }
}

#[derive(Clone, Debug)]
pub struct TransformKeyframe {
    pub time: f32,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl TransformKeyframe {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

#[derive(Clone, Debug)]
pub struct JointAnimation {
    pub joint_id: JointId,
    pub keyframes: Vec<TransformKeyframe>,
}

impl JointAnimation {
    pub fn sample(&self, time: f32) -> Mat4 {
        if self.keyframes.is_empty() {
            return Mat4::IDENTITY;
        }

        if self.keyframes.len() == 1 {
            return self.keyframes[0].to_matrix();
        }

        let idx = self.keyframes.iter()
            .position(|kf| kf.time > time)
            .unwrap_or(self.keyframes.len());

        if idx == 0 {
            return self.keyframes[0].to_matrix();
        }

        if idx >= self.keyframes.len() {
            return self.keyframes.last().unwrap().to_matrix();
        }

        let kf0 = &self.keyframes[idx - 1];
        let kf1 = &self.keyframes[idx];

        let t = (time - kf0.time) / (kf1.time - kf0.time);

        let translation = kf0.translation.lerp(kf1.translation, t);
        let rotation = kf0.rotation.slerp(kf1.rotation, t);
        let scale = kf0.scale.lerp(kf1.scale, t);

        Mat4::from_scale_rotation_translation(scale, rotation, translation)
    }
}

#[derive(Clone, Debug, Component)]
pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub joint_animations: Vec<JointAnimation>,
}

impl Animation {
    pub fn new(name: String, duration: f32) -> Self {
        Self {
            name,
            duration,
            joint_animations: Vec::new(),
        }
    }

    pub fn add_joint_animation(&mut self, joint_animation: JointAnimation) {
        self.joint_animations.push(joint_animation);
    }

    pub fn apply_to_skeleton(&self, skeleton: &mut Skeleton, time: f32) {
        let normalized_time = time % self.duration;

        for joint_anim in &self.joint_animations {
            if let Some(joint) = skeleton.get_joint_mut(joint_anim.joint_id) {
                joint.local_transform = joint_anim.sample(normalized_time);
            }
        }
    }
}

#[derive(Clone, Debug, Component)]
pub struct AnimationState {
    pub current_animation: Option<usize>,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            current_animation: None,
            time: 0.0,
            speed: 1.0,
            looping: true,
        }
    }
}

impl AnimationState {
    pub fn play(&mut self, animation_index: usize) {
        self.current_animation = Some(animation_index);
        self.time = 0.0;
    }

    pub fn update(&mut self, delta_time: f32, animations: &[Animation], skeleton: &mut Skeleton) {
        if let Some(anim_idx) = self.current_animation {
            if let Some(animation) = animations.get(anim_idx) {
                self.time += delta_time * self.speed;

                if self.looping {
                    self.time %= animation.duration;
                } else {
                    self.time = self.time.min(animation.duration);
                }

                animation.apply_to_skeleton(skeleton, self.time);
            }
        }
    }
}

#[derive(Pod, Zeroable, Clone, Copy, Debug)]
#[repr(C)]
pub struct JointMatrices {
    pub matrices: [Mat4; 64],
}

impl Default for JointMatrices {
    fn default() -> Self {
        Self {
            matrices: [Mat4::IDENTITY; 64],
        }
    }
}

impl JointMatrices {
    pub const MAX_JOINTS: usize = 64;

    pub fn from_skeleton(skeleton: &Skeleton) -> Self {
        let mut result = Self::default();
        let skinning_matrices = skeleton.compute_skinning_matrices();

        for (i, matrix) in skinning_matrices.iter().take(Self::MAX_JOINTS).enumerate() {
            result.matrices[i] = *matrix;
        }

        result
    }
}

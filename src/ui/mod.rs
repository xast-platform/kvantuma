use std::{collections::HashMap, hash::Hash};

use taffy::{AlignItems, AvailableSpace, Dimension, FlexDirection, JustifyContent, LengthPercentage, NodeId, Size, Style as TaffyStyle, TaffyTree};
use flecs_ecs::prelude::*;
use components::*;

pub mod key;
pub mod msg;
pub mod components;

#[derive(Clone, Default)]
pub struct Style {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
}

pub struct UiManager<K> {
    screens: HashMap<K, UiScreen>,
    current_screen: Option<K>,
    screen_width: f32,
    screen_height: f32,
    dirty: bool,
}

impl<K: Hash + Eq + Copy> UiManager<K> {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screens: HashMap::new(),
            current_screen: None,
            screen_width,
            screen_height,
            dirty: true,
        }
    }
    
    pub fn set_screen(&mut self, screen: K) {
        if self.current_screen != Some(screen) {
            self.current_screen = Some(screen);
            self.dirty = true;
        }
    }
    
    pub fn current_screen(&self) -> Option<K> {
        self.current_screen
    }
    
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
    
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn screen_width(&self) -> f32 {
        self.screen_width
    }

    pub fn screen_height(&self) -> f32 {
        self.screen_height
    }

    pub fn add_screen(&mut self, label: K, screen: UiScreen) {
        self.screens.insert(label, screen);
    }

    pub fn get_screen(&self, label: K) -> Option<&UiScreen> {
        self.screens.get(&label)
    }

    pub fn get_current_screen(&self) -> Option<&UiScreen> {
        self.current_screen.and_then(|key| self.screens.get(&key))
    }

    pub fn get_current_screen_mut(&mut self) -> Option<&mut UiScreen> {
        self.current_screen.and_then(|key| self.screens.get_mut(&key))
    }

    pub fn recompute_layout(&mut self, world: &World, screen_width: f32, screen_height: f32) {
        self.screen_width = screen_width;
        self.screen_height = screen_height;
        
        for screen in self.screens.values_mut() {
            screen.recompute_layout(world, screen_width, screen_height);
        }
        
        self.dirty = true;
    }
}

pub struct UiScreen{
    tree: TaffyTree<()>,
    node: Option<NodeId>,
    root: Entity,
    entity_to_node: HashMap<Entity, NodeId>,
    screen_height: f32,
}

impl UiScreen {
    pub fn new(root: Entity) -> Self {
        Self {
            tree: TaffyTree::new(),
            node: None,
            root,
            entity_to_node: HashMap::new(),
            screen_height: 0.0,
        }
    }

    pub fn recompute_layout(
        &mut self, 
        world: &World,
        screen_width: f32, 
        screen_height: f32,
    ) {
        self.screen_height = screen_height;
        self.tree = TaffyTree::new();
        self.entity_to_node.clear();
        self.node = Some(compute_layout(world, &mut self.tree, self.root, &mut self.entity_to_node));
        
        if let Some(root_node_id) = self.node {
            self.tree.compute_layout(
                root_node_id,
                Size { 
                    width: AvailableSpace::Definite(screen_width), 
                    height: AvailableSpace::Definite(screen_height) 
                },
            ).unwrap();
        }
    }

    pub fn apply_layout_to_entities(&self, world: &World) {
        if let Some(root_node) = self.node {
            self.apply_layout_recursive(world, root_node, 0.0, 0.0);
        }
    }

    fn apply_layout_recursive(&self, world: &World, node_id: NodeId, parent_x: f32, parent_y: f32) {
        if let Ok(layout) = self.tree.layout(node_id) {
            let abs_x = parent_x + layout.location.x;
            let abs_y = parent_y + layout.location.y;
            
            for (entity, entity_node_id) in &self.entity_to_node {
                if *entity_node_id == node_id {
                    let screen_y = self.screen_height - abs_y - layout.size.height;
                    world.entity_from_id(*entity).set(UiPosition {
                        x: abs_x,
                        y: screen_y,
                        width: layout.size.width,
                        height: layout.size.height,
                    });
                    break;
                }
            }
            
            if let Ok(children) = self.tree.children(node_id) {
                for child_id in children {
                    self.apply_layout_recursive(world, child_id, abs_x, abs_y);
                }
            }
        }
    }
}

const UNIFORM_WIDTH: f32 = 120.0;
const UNIFORM_HEIGHT: f32 = 32.0;
const PADDING: f32 = 8.0;

fn compute_layout(
    world: &World,
    tree: &mut TaffyTree<()>, 
    root_node: Entity,
    entity_to_node: &mut HashMap<Entity, NodeId>,
) -> NodeId {
    let entity = world.entity_from_id(root_node);
    
    let mut is_row = false;
    let mut is_col = false;
    let mut is_text = false;
    let mut children_vec = Vec::new();
    let mut col_number = 0u8;
    
    entity.get::<(Option<&EcsRow>, Option<&EcsCol>, Option<&EcsText>)>(|(row_opt, col_opt, text_opt)| {
        if let Some(row) = row_opt {
            is_row = true;
            children_vec = row.children.clone();
        }
        
        if let Some(col) = col_opt {
            is_col = true;
            children_vec = col.children.clone();
            col_number = col.col_number;
        }
        
        if text_opt.is_some() {
            is_text = true;
        }
    });
    
    let node_id = if is_row {
        let child_nodes: Vec<_> = children_vec.iter()
            .map(|child| compute_layout(world, tree, *child, entity_to_node))
            .collect();
        
        let taffy_style = TaffyStyle {
            flex_direction: FlexDirection::Row,
            gap: Size { 
                width: LengthPercentage::length(PADDING), 
                height: LengthPercentage::length(PADDING) 
            },
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        };
        
        tree.new_with_children(taffy_style, &child_nodes).unwrap()
    } else if is_col {
        let child_nodes: Vec<_> = children_vec.iter()
            .map(|child| compute_layout(world, tree, *child, entity_to_node))
            .collect();
        
        let width_percent = (col_number as f32) / 12.0;
        
        tree.new_with_children(
            TaffyStyle {
                flex_direction: FlexDirection::Column,
                gap: Size { 
                    width: LengthPercentage::length(PADDING), 
                    height: LengthPercentage::length(PADDING) 
                },
                size: Size {
                    width: Dimension::percent(width_percent),
                    height: Dimension::percent(1.0),
                },
                align_items: Some(AlignItems::Center),
                justify_content: Some(JustifyContent::Center),
                ..Default::default()
            },
            &child_nodes,
        ).unwrap()
    } else if is_text {
        tree.new_leaf(
            TaffyStyle {
                size: Size {
                    width: Dimension::length(UNIFORM_WIDTH),
                    height: Dimension::length(UNIFORM_HEIGHT),
                },
                ..Default::default()
            },
        ).unwrap()
    } else {
        tree.new_leaf(TaffyStyle::default()).unwrap()
    };
    
    entity_to_node.insert(root_node, node_id);
    node_id
}

pub trait Ui {
    fn build_ui(&self, world: &mut World) -> Entity;
}
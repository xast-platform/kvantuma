use std::{collections::HashMap, hash::Hash};

use taffy::{TaffyTree, Style as TaffyStyle, FlexDirection, Dimension, Size, AvailableSpace, LengthPercentage, AlignItems, JustifyContent};
use glam::Vec2;
use flecs_ecs::macros::Component;

#[derive(Clone)]
pub enum UiNode<Msg: Clone> {
    Column {
        style: Style,
        children: Vec<UiNode<Msg>>,
    },
    Row {
        style: Style,
        children: Vec<UiNode<Msg>>,
    },
    Col {
        style: Style,
        span: u8,
        children: Vec<UiNode<Msg>>,
    },
    Button {
        style: Style,
        text: String,
        on_click: Option<Msg>,
    },
    Text {
        style: Style,
        value: String,
    },
}

#[derive(Component, Clone)]
pub struct UiText {
    pub value: String,
    pub font_size: u32,
}

#[derive(Component, Clone)]
pub struct UiButton {
    pub text: String,
    pub font_size: u32,
}

#[derive(Component)]
pub struct UiPosition {
    pub screen_pos: Vec2,
}

pub fn column<Msg: Clone>(children: Vec<UiNode<Msg>>) -> UiNode<Msg> {
    UiNode::Column {
        style: Style::default(),
        children,
    }
}

pub fn row<Msg: Clone>(children: Vec<UiNode<Msg>>) -> UiNode<Msg> {
    UiNode::Row {
        style: Style::default(),
        children,
    }
}

pub fn col<Msg: Clone>(span: u8, children: Vec<UiNode<Msg>>) -> UiNode<Msg> {
    UiNode::Col {
        style: Style::default(),
        span: span.clamp(1, 12),
        children,
    }
}

pub fn text<Msg: Clone>(value: String) -> UiNode<Msg> {
    UiNode::Text {
        style: Style::default(),
        value,
    }
}

pub fn button<Msg: Clone>(text: String, on_click: Option<Msg>) -> UiNode<Msg> {
    UiNode::Button {
        style: Style::default(),
        text,
        on_click,
    }
}

#[derive(Clone, Default)]
pub struct Style {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
}

pub struct UiManager<K, Msg: Clone> {
    screens: HashMap<K, UiScreen<Msg>>,
    current_screen: K,
}

pub struct UiScreen<Msg: Clone> {
    tree: TaffyTree<()>,
    root: UiNode<Msg>,
    root_node_id: taffy::NodeId,
    cached_nodes: Vec<(UiNode<Msg>, Vec2)>,
}

impl<Msg: Clone> UiScreen<Msg> {
    pub fn new(root: UiNode<Msg>, screen_width: f32, screen_height: f32) -> Self {
        let mut tree = TaffyTree::new();
        let root_node = compute_layout(&mut tree, &root);
        tree.compute_layout(
            root_node,
            Size { 
                width: AvailableSpace::Definite(screen_width), 
                height: AvailableSpace::Definite(screen_height) 
            },
        ).unwrap();

        let mut cached_nodes = Vec::new();
        collect_nodes(&tree, &root, root_node, Vec2::ZERO, &mut cached_nodes);

        Self { tree, root, root_node_id: root_node, cached_nodes }
    }

    pub fn recompute_layout(&mut self, screen_width: f32, screen_height: f32) {
        self.tree.compute_layout(
            self.root_node_id,
            Size { 
                width: AvailableSpace::Definite(screen_width), 
                height: AvailableSpace::Definite(screen_height) 
            },
        ).unwrap();
        
        self.cached_nodes.clear();
        collect_nodes(&self.tree, &self.root, self.root_node_id, Vec2::ZERO, &mut self.cached_nodes);
    }

    pub fn nodes(&self) -> &[(UiNode<Msg>, Vec2)] {
        &self.cached_nodes
    }
}

impl<Msg: Clone, K: Hash + Eq> UiManager<K, Msg> {
    pub fn add_screen(&mut self, label: K, screen: UiScreen<Msg>) {
        self.screens.insert(label, screen);
    }
}

const UNIFORM_WIDTH: f32 = 120.0;
const UNIFORM_HEIGHT: f32 = 32.0;
const PADDING: f32 = 8.0;

fn collect_nodes<Msg: Clone>(
    tree: &TaffyTree<()>,
    node: &UiNode<Msg>,
    node_id: taffy::NodeId,
    offset: Vec2,
    nodes: &mut Vec<(UiNode<Msg>, Vec2)>
) {
    let layout = tree.layout(node_id).unwrap();
    let pos = offset + Vec2::new(layout.location.x, layout.location.y);

    match node {
        UiNode::Column { children, .. } | UiNode::Row { children, .. } | UiNode::Col { children, .. } => {
            let child_ids = tree.children(node_id).unwrap();
            for (child, child_id) in children.iter().zip(child_ids.iter()) {
                collect_nodes(tree, child, *child_id, pos, nodes);
            }
        }
        UiNode::Button { .. } | UiNode::Text { .. } => {
            nodes.push((node.clone(), pos));
        }
    }
}

fn compute_layout<Msg: Clone>(tree: &mut TaffyTree<()>, node: &UiNode<Msg>) -> taffy::NodeId {
    match node {
        UiNode::Column { children, style } => {
            let child_nodes: Vec<_> = children.iter()
                .map(|child| compute_layout(tree, child))
                .collect();
            
            let mut taffy_style = TaffyStyle {
                flex_direction: FlexDirection::Column,
                gap: Size { width: LengthPercentage::length(PADDING), height: LengthPercentage::length(PADDING) },
                ..Default::default()
            };
            
            if let Some(width) = style.width {
                taffy_style.size.width = width;
            } else {
                taffy_style.size.width = Dimension::percent(1.0);
            }
            
            if let Some(height) = style.height {
                taffy_style.size.height = height;
            } else {
                taffy_style.size.height = Dimension::percent(1.0);
            }
            
            taffy_style.align_items = style.align_items.or(Some(AlignItems::Center));
            taffy_style.justify_content = style.justify_content.or(Some(JustifyContent::Center));
            
            tree.new_with_children(taffy_style, &child_nodes).unwrap()
        }
        UiNode::Row { children, style } => {
            let child_nodes: Vec<_> = children.iter()
                .map(|child| compute_layout(tree, child))
                .collect();
            
            let mut taffy_style = TaffyStyle {
                flex_direction: FlexDirection::Row,
                gap: Size { width: LengthPercentage::length(PADDING), height: LengthPercentage::length(PADDING) },
                ..Default::default()
            };
            
            if let Some(width) = style.width {
                taffy_style.size.width = width;
            } else {
                taffy_style.size.width = Dimension::percent(1.0);
            }
            
            if let Some(height) = style.height {
                taffy_style.size.height = height;
            }
            
            taffy_style.align_items = style.align_items;
            taffy_style.justify_content = style.justify_content;
            
            tree.new_with_children(taffy_style, &child_nodes).unwrap()
        }
        UiNode::Col { children, span, .. } => {
            let child_nodes: Vec<_> = children.iter()
                .map(|child| compute_layout(tree, child))
                .collect();
            
            let width_percent = (*span as f32) / 12.0;
            
            tree.new_with_children(
                TaffyStyle {
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: Dimension::percent(width_percent),
                        height: Dimension::auto(),
                    },
                    ..Default::default()
                },
                &child_nodes,
            ).unwrap()
        }
        UiNode::Button { .. } | UiNode::Text { .. } => {
            tree.new_leaf(
                TaffyStyle {
                    size: Size {
                        width: Dimension::length(UNIFORM_WIDTH),
                        height: Dimension::length(UNIFORM_HEIGHT),
                    },
                    ..Default::default()
                },
            ).unwrap()
        }
    }
}
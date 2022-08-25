use bevy::{
    prelude::*,
    render::{
        render_graph::{
            Node, NodeRunError, RenderGraph, RenderGraphContext, RenderGraphError, SlotInfo,
            SlotType,
        },
        render_resource::TextureFormat,
        renderer::RenderContext,
        texture::BevyDefault,
    },
};

use crate::{
    jfa::JfaNode, jfa_init::JfaInitNode, mask::WaterMaskNode, ripples::RipplesNode,
};

pub(crate) mod water_effect {
    pub const NAME: &str = "water_effect_graph";

    pub mod input {
        pub const VIEW_ENTITY: &str = "view_entity"; // entity in view, i think
    }

    pub mod node {
        pub const MASK_PASS: &str = "mask_pass";
        pub const JFA_INIT_PASS: &str = "jfa_init_pass";
        pub const JFA_PASS: &str = "jfa_pass";
        pub const RIPPLES_PASS: &str = "ripples_pass";
    }
}

pub struct WaterEffectDriverNode;

impl WaterEffectDriverNode {
    pub const NAME: &'static str = "water_effect_driver";
    pub const INPUT_VIEW: &'static str = "view_entity";
}

impl Node for WaterEffectDriverNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        _world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::INPUT_VIEW)?;

        dbg!(view_entity);

        graph.run_sub_graph(water_effect::NAME, vec![view_entity.into()])?;

        Ok(())
    }

    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo {
            name: Self::INPUT_VIEW.into(),
            slot_type: SlotType::Entity,
        }]
    }
}

/// Builds the render graph for applying the JFA outline.
pub fn water_effect(render_app: &mut App) -> Result<RenderGraph, RenderGraphError> {
    let mut graph = RenderGraph::default();

    let input_node_id = graph.set_input(vec![SlotInfo {
        name: water_effect::input::VIEW_ENTITY.into(),
        slot_type: SlotType::Entity,
    }]);

    // Graph order:
    // 1. Mask
    // 2. JFA Init
    // 3. JFA
    // 4. Ripples

    let mask_node = WaterMaskNode::new(&mut render_app.world);
    let jfa_init_node = JfaInitNode;
    let jfa_node = JfaNode::from_world(&mut render_app.world);
    // TODO: BevyDefault for surface texture format is an anti-pattern;
    // the target texture format should be queried from the window when
    // Bevy exposes that functionality.
    let ripples_node = RipplesNode::new(&mut render_app.world, TextureFormat::bevy_default());

    graph.add_node(water_effect::node::MASK_PASS, mask_node);
    graph.add_node(water_effect::node::JFA_INIT_PASS, jfa_init_node);
    graph.add_node(water_effect::node::JFA_PASS, jfa_node);
    graph.add_node(water_effect::node::RIPPLES_PASS, ripples_node);

    // Input -> Mask
    graph.add_slot_edge(
        input_node_id,
        water_effect::input::VIEW_ENTITY,
        water_effect::node::MASK_PASS,
        WaterMaskNode::IN_VIEW,
    )?;

    // Mask -> JFA Init
    graph.add_slot_edge(
        water_effect::node::MASK_PASS,
        WaterMaskNode::OUT_MASK,
        water_effect::node::JFA_INIT_PASS,
        JfaInitNode::IN_MASK,
    )?;

    // Input -> JFA
    graph.add_slot_edge(
        input_node_id,
        water_effect::input::VIEW_ENTITY,
        water_effect::node::JFA_PASS,
        JfaNode::IN_VIEW,
    )?;

    // JFA Init -> JFA
    graph.add_slot_edge(
        water_effect::node::JFA_INIT_PASS,
        JfaInitNode::OUT_JFA_INIT,
        water_effect::node::JFA_PASS,
        JfaNode::IN_BASE,
    )?;

    // Input -> Ripples
    graph.add_slot_edge(
        input_node_id,
        water_effect::input::VIEW_ENTITY,
        water_effect::node::RIPPLES_PASS,
        RipplesNode::IN_VIEW,
    )?;

    // JFA -> Ripples
    graph.add_slot_edge(
        water_effect::node::JFA_PASS,
        JfaNode::OUT_JUMP,
        water_effect::node::RIPPLES_PASS,
        RipplesNode::IN_JFA,
    )?;

    dbg!(&graph);

    Ok(graph)
}

// Civil (Agral CAD) — superfícies como no Civil 3D: Create Ground Data, Toolspace e Surface.

use crate::modules::{CadModule, IconKind, ModuleEvent, RibbonGroup, RibbonItem, ToolDef};

pub struct CivilModule;

fn ferramenta(id: &'static str, label: &'static str, icon: &'static [u8]) -> ToolDef {
    ToolDef { id, label, icon: IconKind::Svg(icon), event: ModuleEvent::Command(id.to_string()) }
}

impl CadModule for CivilModule {
    fn id(&self) -> &'static str {
        "civil"
    }
    fn title(&self) -> &'static str {
        "Civil"
    }

    fn ribbon_groups(&self) -> &[RibbonGroup] {
        static GROUPS: std::sync::OnceLock<Vec<RibbonGroup>> = std::sync::OnceLock::new();
        GROUPS.get_or_init(|| {
            vec![
                RibbonGroup {
                    title: "Create Ground Data",
                    tools: vec![RibbonItem::LargeTool(ferramenta(
                        "CREATESURFACE",
                        "Create\nSurface",
                        include_bytes!("../../../assets/icons/civil_surface.svg"),
                    ))],
                },
                RibbonGroup {
                    title: "Surface",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta(
                            "ADDSURFACECONTOURS",
                            "Add\nContours",
                            include_bytes!("../../../assets/icons/civil_contours.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta(
                            "REBUILDSURFACE",
                            "Rebuild",
                            include_bytes!("../../../assets/icons/civil_rebuild.svg"),
                        )),
                    ],
                },
                RibbonGroup {
                    title: "Palettes",
                    tools: vec![RibbonItem::LargeTool(ferramenta(
                        "TOOLSPACE",
                        "Toolspace",
                        include_bytes!("../../../assets/icons/civil_toolspace.svg"),
                    ))],
                },
            ]
        })
    }
}

/// Aba contextual TIN Surface: aparece com uma superfície selecionada (como no Civil 3D).
pub mod pegar_ponto;

pub struct TinSurfaceModule;

impl CadModule for TinSurfaceModule {
    fn id(&self) -> &'static str {
        "tin_surface"
    }
    fn title(&self) -> &'static str {
        "TIN Surface"
    }

    fn ribbon_groups(&self) -> &[RibbonGroup] {
        static GROUPS: std::sync::OnceLock<Vec<RibbonGroup>> = std::sync::OnceLock::new();
        let sup: &'static [u8] = include_bytes!("../../../assets/icons/civil_surface.svg");
        let cur: &'static [u8] = include_bytes!("../../../assets/icons/civil_contours.svg");
        let reb: &'static [u8] = include_bytes!("../../../assets/icons/civil_rebuild.svg");
        let tsp: &'static [u8] = include_bytes!("../../../assets/icons/civil_toolspace.svg");
        GROUPS.get_or_init(|| {
            vec![
                RibbonGroup {
                    title: "Modify",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("SURFACEPROPERTIES", "Surface\nProperties", sup)),
                        RibbonItem::LargeTool(ferramenta("REBUILDSURFACE", "Rebuild", reb)),
                    ],
                },
                RibbonGroup {
                    title: "Add Data",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("ADDSURFACECONTOURS", "Add\nContours", cur)),
                        RibbonItem::LargeTool(ferramenta("ADDSURFACEBOUNDARIES", "Add\nBoundaries", sup)),
                    ],
                },
                RibbonGroup {
                    title: "Edit Surface",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("ADDSURFACEPOINT", "Add\nPoint", sup)),
                        RibbonItem::LargeTool(ferramenta("DELETESURFACEPOINT", "Delete\nPoint", sup)),
                        RibbonItem::LargeTool(ferramenta("DELETESURFACELINE", "Delete\nLine", sup)),
                        RibbonItem::LargeTool(ferramenta("RAISELOWERSURFACE", "Raise/\nLower", reb)),
                    ],
                },
                RibbonGroup {
                    title: "Surface Tools",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("EXTRACTSURFACEOBJECTS", "Extract\nObjects", cur)),
                        RibbonItem::LargeTool(ferramenta("ZOOMSURFACE", "Zoom\nto", sup)),
                        RibbonItem::LargeTool(ferramenta("TOOLSPACE", "Toolspace", tsp)),
                    ],
                },
            ]
        })
    }
}

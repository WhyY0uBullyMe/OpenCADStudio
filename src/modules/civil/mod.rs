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

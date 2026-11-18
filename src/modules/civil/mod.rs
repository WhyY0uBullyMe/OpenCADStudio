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
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta(
                            "CREATESURFACE",
                            "Create\nSurface",
                            include_bytes!("../../../assets/icons/civil_surface.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta(
                            "IMPORTLANDXMLSURFACE",
                            "Import\nLandXML",
                            include_bytes!("../../../assets/icons/civil_landxml_in.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta(
                            "EXPORTLANDXMLSURFACE",
                            "Export\nLandXML",
                            include_bytes!("../../../assets/icons/civil_landxml_out.svg"),
                        )),
                    ],
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
                            "ADDSURFACEBREAKLINES",
                            "Add\nBreaklines",
                            include_bytes!("../../../assets/icons/civil_breaklines.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta(
                            "ADDSURFACEPOINTFILE",
                            "Add Point\nFile",
                            include_bytes!("../../../assets/icons/civil_pointfile.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta(
                            "ADDSURFACEBOUNDARIES",
                            "Add\nBoundaries",
                            include_bytes!("../../../assets/icons/civil_boundary.svg"),
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
pub mod perguntas;
pub mod refazer_curva;

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
        let cur: &'static [u8] = include_bytes!("../../../assets/icons/civil_contours.svg");
        let reb: &'static [u8] = include_bytes!("../../../assets/icons/civil_rebuild.svg");
        let tsp: &'static [u8] = include_bytes!("../../../assets/icons/civil_toolspace.svg");
        let props: &'static [u8] = include_bytes!("../../../assets/icons/civil_props.svg");
        let lim: &'static [u8] = include_bytes!("../../../assets/icons/civil_boundary.svg");
        let maisp: &'static [u8] = include_bytes!("../../../assets/icons/civil_addpoint.svg");
        let menosp: &'static [u8] = include_bytes!("../../../assets/icons/civil_delpoint.svg");
        let menosl: &'static [u8] = include_bytes!("../../../assets/icons/civil_delline.svg");
        let elevar: &'static [u8] = include_bytes!("../../../assets/icons/civil_raise.svg");
        let extrair: &'static [u8] = include_bytes!("../../../assets/icons/civil_extract.svg");
        let zoom: &'static [u8] = include_bytes!("../../../assets/icons/civil_zoomsurf.svg");
        let quebras: &'static [u8] = include_bytes!("../../../assets/icons/civil_breaklines.svg");
        let arqp: &'static [u8] = include_bytes!("../../../assets/icons/civil_pointfile.svg");
        GROUPS.get_or_init(|| {
            vec![
                RibbonGroup {
                    title: "Modify",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("SURFACEPROPERTIES", "Surface\nProperties", props)),
                        RibbonItem::LargeTool(ferramenta("REBUILDSURFACE", "Rebuild", reb)),
                    ],
                },
                RibbonGroup {
                    title: "Add Data",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("ADDSURFACECONTOURS", "Add\nContours", cur)),
                        RibbonItem::LargeTool(ferramenta("ADDSURFACEBOUNDARIES", "Add\nBoundaries", lim)),
                        RibbonItem::LargeTool(ferramenta("ADDSURFACEBREAKLINES", "Add\nBreaklines", quebras)),
                        RibbonItem::LargeTool(ferramenta("ADDSURFACEPOINTFILE", "Add Point\nFile", arqp)),
                    ],
                },
                RibbonGroup {
                    title: "Edit Surface",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("ADDSURFACEPOINT", "Add\nPoint", maisp)),
                        RibbonItem::LargeTool(ferramenta("DELETESURFACEPOINT", "Delete\nPoint", menosp)),
                        RibbonItem::LargeTool(ferramenta("DELETESURFACELINE", "Delete\nLine", menosl)),
                        RibbonItem::LargeTool(ferramenta("RAISELOWERSURFACE", "Raise/\nLower", elevar)),
                    ],
                },
                RibbonGroup {
                    title: "Surface Tools",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("EXTRACTSURFACEOBJECTS", "Extract\nObjects", extrair)),
                        RibbonItem::LargeTool(ferramenta(
                            "EXPORTLANDXMLSURFACE",
                            "Export\nLandXML",
                            include_bytes!("../../../assets/icons/civil_landxml_out.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta("ZOOMSURFACE", "Zoom\nto", zoom)),
                        RibbonItem::LargeTool(ferramenta("TOOLSPACE", "Toolspace", tsp)),
                    ],
                },
            ]
        })
    }
}

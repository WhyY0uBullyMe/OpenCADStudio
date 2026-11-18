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
                    title: "Alignment and Profile",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta(
                            "CREATEALIGNMENTENTITIES",
                            "Alignment\nfrom Objects",
                            include_bytes!("../../../assets/icons/civil_alignobjects.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta(
                            "CREATEALIGNMENTLAYOUT",
                            "Alignment\nLayout",
                            include_bytes!("../../../assets/icons/civil_alignlayout.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta(
                            "CREATEPROFILEVIEW",
                            "Profile\nView",
                            include_bytes!("../../../assets/icons/civil_profileview.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta(
                            "CREATEPROFILE",
                            "Design\nProfile",
                            include_bytes!("../../../assets/icons/civil_profile.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta(
                            "CREATESECTIONS",
                            "Sections",
                            include_bytes!("../../../assets/icons/civil_sections.svg"),
                        )),
                    ],
                },
                RibbonGroup {
                    title: "Corridor",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta(
                            "CREATEASSEMBLY",
                            "Dam\nAssembly",
                            include_bytes!("../../../assets/icons/civil_assembly.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta(
                            "CREATECORRIDOR",
                            "Create\nCorridor",
                            include_bytes!("../../../assets/icons/civil_corridor.svg"),
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
pub mod eixo_fonte;
pub mod pegar_ponto;
pub mod pivs;
pub mod perguntas;
pub mod refazer_curva;
pub mod tracado;

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
                        RibbonItem::LargeTool(ferramenta(
                            "OBJECTVIEWER",
                            "Object\nViewer",
                            include_bytes!("../../../assets/icons/civil_objectviewer.svg"),
                        )),
                        RibbonItem::LargeTool(ferramenta("TOOLSPACE", "Toolspace", tsp)),
                    ],
                },
            ]
        })
    }
}

/// Aba Home do Agral CAD, a primeira da faixa, como a do Civil 3D (pedido do JP, 18/11: "uma abinha Home com meus
/// negócios"): painéis, dados do terreno, projeto (eixo, greide, barragem, corredor), vistas de perfil e seção,
/// e os painéis de desenho, edição, camadas e área de transferência da aba Draw.
pub struct HomeModule;

impl CadModule for HomeModule {
    fn id(&self) -> &'static str {
        "home"
    }
    fn title(&self) -> &'static str {
        "Home"
    }

    fn ribbon_groups(&self) -> &[RibbonGroup] {
        static GROUPS: std::sync::OnceLock<Vec<RibbonGroup>> = std::sync::OnceLock::new();
        GROUPS.get_or_init(|| {
            let ic = |n: &str| -> &'static [u8] {
                match n {
                    "toolspace" => include_bytes!("../../../assets/icons/civil_toolspace.svg"),
                    "surface" => include_bytes!("../../../assets/icons/civil_surface.svg"),
                    "contours" => include_bytes!("../../../assets/icons/civil_contours.svg"),
                    "pointfile" => include_bytes!("../../../assets/icons/civil_pointfile.svg"),
                    "landxml" => include_bytes!("../../../assets/icons/civil_landxml_in.svg"),
                    "alignobjects" => include_bytes!("../../../assets/icons/civil_alignobjects.svg"),
                    "alignlayout" => include_bytes!("../../../assets/icons/civil_alignlayout.svg"),
                    "profile" => include_bytes!("../../../assets/icons/civil_profile.svg"),
                    "assembly" => include_bytes!("../../../assets/icons/civil_assembly.svg"),
                    "corridor" => include_bytes!("../../../assets/icons/civil_corridor.svg"),
                    "profileview" => include_bytes!("../../../assets/icons/civil_profileview.svg"),
                    "sections" => include_bytes!("../../../assets/icons/civil_sections.svg"),
                    _ => include_bytes!("../../../assets/icons/civil_objectviewer.svg"),
                }
            };
            let mut g = vec![
                RibbonGroup {
                    title: "Palettes",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("TOOLSPACE", "Toolspace", ic("toolspace"))),
                        RibbonItem::LargeTool(ferramenta("OBJECTVIEWER", "Object\nViewer", ic("viewer"))),
                    ],
                },
                RibbonGroup {
                    title: "Create Ground Data",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("CREATESURFACE", "Surface", ic("surface"))),
                        RibbonItem::Tool(ferramenta("ADDSURFACECONTOURS", "Contours", ic("contours"))),
                        RibbonItem::Tool(ferramenta("ADDSURFACEPOINTFILE", "Point File", ic("pointfile"))),
                        RibbonItem::Tool(ferramenta("IMPORTLANDXMLSURFACE", "LandXML", ic("landxml"))),
                    ],
                },
                RibbonGroup {
                    title: "Create Design",
                    tools: vec![
                        RibbonItem::LargeTool(ferramenta("CREATEALIGNMENTENTITIES", "Alignment\nfrom Objects", ic("alignobjects"))),
                        RibbonItem::Tool(ferramenta("CREATEALIGNMENTLAYOUT", "Alignment Layout", ic("alignlayout"))),
                        RibbonItem::LargeTool(ferramenta("CREATEPROFILE", "Profile", ic("profile"))),
                        RibbonItem::LargeTool(ferramenta("CREATEASSEMBLY", "Assembly", ic("assembly"))),
                        RibbonItem::LargeTool(ferramenta("CREATECORRIDOR", "Corridor", ic("corridor"))),
                    ],
                },
                RibbonGroup {
                    title: "Profile & Section Views",
                    tools: vec![
                        RibbonItem::Tool(ferramenta("CREATEPROFILEVIEW", "Profile View", ic("profileview"))),
                        RibbonItem::Tool(ferramenta("CREATESECTIONS", "Section Views", ic("sections"))),
                    ],
                },
            ];
            // Os painéis de desenho e edição são os mesmos da aba Draw (um lugar só para mudar).
            for p in crate::modules::draw::DrawModule.ribbon_groups() {
                if matches!(p.title, "Draw" | "Modify" | "Layers" | "Clipboard") {
                    g.push(p.clone());
                }
            }
            g
        })
    }
}

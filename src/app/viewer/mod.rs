//! viewer plugins
use serde::*;
mod color_block;
mod editor;
pub use editor::*;
pub use color_block::*;

type Frame<'a> = tui::Frame<'a, tui::backend::CrosstermBackend<std::io::Stdout>>;

pub trait Viewer {
    fn name(&self) -> String;
    fn render(&self, f: &mut Frame, rect: tui::layout::Rect);
    fn refresh(&mut self, db: &crate::DataBase);
}

macro_rules! declare_viewer_opt {
    ($($Opt:ident)*) => {
        #[derive(Debug, Serialize, Deserialize)]
        pub enum ViewerOpt {
            $($Opt($Opt), )*
        }
        impl Viewer for ViewerOpt {
            fn name(&self) -> String {
                use ViewerOpt::*;
                match self {
                    $($Opt(x) => x.name(), )*
                }
            }
            fn render(&self, f: &mut Frame, rect: tui::layout::Rect) {
                use ViewerOpt::*;
                match self {
                    $($Opt(x) => x.render(f, rect), )*
                }
            }
            fn refresh(&mut self, db: &crate::DataBase) {
                use ViewerOpt::*;
                match self {
                    $($Opt(x) => x.refresh(db), )*
                }
            }
        }
        $(
            impl Into<ViewerOpt> for $Opt {
                fn into(self) -> ViewerOpt {
                    ViewerOpt::$Opt(self)
                }
            }
        )*
    };
}

// do something like automatic dispatch, but make it serializable
declare_viewer_opt!{
    ColorBlock
    EditorView
}
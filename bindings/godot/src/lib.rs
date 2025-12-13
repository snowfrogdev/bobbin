use godot::prelude::*;
use bobbin_runtime::Runtime;

struct BobbinExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BobbinExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, no_init)]
pub struct BobbinRuntime {
    base: Base<RefCounted>,
    inner: Runtime,
}

#[godot_api]
impl BobbinRuntime {
    #[func]
    fn from_string(content: GString) -> Option<Gd<Self>> {
        match Runtime::new(&content.to_string()) {
            Ok(runtime) => Some(Gd::from_init_fn(|base| Self { base, inner: runtime })),
            Err(e) => {
                godot_error!("Failed to load bobbin script: {:?}", e);
                None
            }
        }
    }

    #[func]
    fn advance(&mut self) {
        self.inner.advance();
    }

    #[func]
    fn current_line(&self) -> GString {
        GString::from(self.inner.current_line())
    }

    #[func]
    fn has_more(&self) -> bool {
        self.inner.has_more()
    }
}

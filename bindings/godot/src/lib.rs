use godot::prelude::*;
use bobbin_interpreter::Interpreter;

struct BobbinExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BobbinExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct BobbinInterpreter {
    base: Base<RefCounted>,
    inner: Interpreter,
}

#[godot_api]
impl IRefCounted for BobbinInterpreter {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            base,
            inner: Interpreter::new(),
        }
    }
}

#[godot_api]
impl BobbinInterpreter {
    #[func]
    fn load_content(&mut self, content: GString) {
        self.inner.load_content(&content.to_string());
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

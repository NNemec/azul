#![allow(non_snake_case)]

extern crate azul_core;

#[cfg(target_arch = "wasm32")]
extern crate azul_web as azul_impl;
#[cfg(not(target_arch = "wasm32"))]
extern crate azul_desktop as azul_impl;

use core::ffi::c_void;
use core::mem;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3::exceptions::PyException;



use pyo3::{PyVisit, PyTraverseError, PyGCProtocol};

fn pystring_to_azstring(input: &String) -> AzString {
    AzString {
        vec: AzU8Vec {
            ptr: input.as_ptr(),
            len: input.len(),
            cap: input.capacity(),
            // prevent the String (which is just a reference) from dropping
            destructor: AzU8VecDestructorEnumWrapper::NoDestructor(),
        }
    }
}
fn az_string_to_py_string(input: AzString) -> String { input.into() }
fn pystring_to_refstr(input: &str) -> AzRefstr {
    AzRefstr {
        ptr: input.as_ptr(),
        len: input.len(),
    }
}
fn az_vecu8_to_py_vecu8(input: AzU8Vec) -> Vec<u8> {
    let input: azul_impl::css::U8Vec = unsafe { mem::transmute(input) };
    input.into_library_owned_vec()
}
fn vec_string_to_vec_refstr(input: &Vec<&str>) -> Vec<AzRefstr> {
    input.iter().map(|i| pystring_to_refstr(i)).collect()
}
fn pybytesrefmut_to_vecu8refmut(input: &mut Vec<u8>) -> AzU8VecRefMut {
    AzU8VecRefMut { ptr: input.as_mut_ptr(), len: input.len() }
}
fn pybytesref_to_vecu8_ref(input: &Vec<u8>) -> AzU8VecRef {
    AzU8VecRef { ptr: input.as_ptr(), len: input.len() }
}
fn pylist_f32_to_rust(input: &Vec<f32>) -> AzF32VecRef {
    AzF32VecRef { ptr: input.as_ptr(), len: input.len() }
}
fn pylist_u32_to_rust(input: &Vec<u32>) -> AzGLuintVecRef {
    AzGLuintVecRef { ptr: input.as_ptr(), len: input.len() }
}
fn pylist_i32_to_rust(input: &mut Vec<i32>) -> AzGLintVecRefMut {
    AzGLintVecRefMut { ptr: input.as_mut_ptr(), len: input.len() }
}
fn pylist_i64_to_rust(input: &mut Vec<i64>) -> AzGLint64VecRefMut {
    AzGLint64VecRefMut { ptr: input.as_mut_ptr(), len: input.len() }
}
fn pylist_bool_to_rust(input: &mut Vec<u8>) -> AzGLbooleanVecRefMut {
    AzGLbooleanVecRefMut { ptr: input.as_mut_ptr(), len: input.len() }
}
fn pylist_glfoat_to_rust(input: &mut Vec<f32>) -> AzGLfloatVecRefMut {
    AzGLfloatVecRefMut { ptr: input.as_mut_ptr(), len: input.len() }
}
fn pylist_str_to_rust(input: &Vec<AzRefstr>) -> AzRefstrVecRef {
    AzRefstrVecRef { ptr: input.as_ptr(), len: input.len() }
}
fn pylist_tesselated_svg_node(input: &Vec<AzTesselatedSvgNode>) -> AzTesselatedSvgNodeVecRef {
    AzTesselatedSvgNodeVecRef { ptr: input.as_ptr(), len: input.len() }
}

impl From<String> for AzString {
    fn from(s: String) -> AzString {
        Self { vec: s.into_bytes().into() }
    }
}

impl From<AzString> for String {
    fn from(s: AzString) -> String {
        let s: azul_impl::css::AzString = unsafe { mem::transmute(s) };
        s.into_library_owned_string()
    }
}

// AzU8Vec
impl From<AzU8Vec> for Vec<u8> {
    fn from(input: AzU8Vec) -> Vec<u8> {
        let input: azul_impl::css::U8Vec = unsafe { mem::transmute(input) };
        input.into_library_owned_vec()
    }
}

impl From<Vec<u8>> for AzU8Vec {
    fn from(input: Vec<u8>) -> AzU8Vec {

        let ptr = input.as_ptr();
        let len = input.len();
        let cap = input.capacity();

        let _ = ::core::mem::ManuallyDrop::new(input);

        Self {
            ptr,
            len,
            cap,
            destructor: AzU8VecDestructorEnumWrapper::DefaultRust(),
        }

    }
}

// manually implement App::new, WindowState::new,
// WindowCreateOptions::new and LayoutCallback::new

#[pymethods]
impl AzApp {
    #[staticmethod]
    pub fn new(py: Python, data: PyObject, config: AzAppConfig) -> Result<Self, PyErr> {
        use pyo3::type_object::PyTypeInfo;

        if data.as_ref(py).is_callable() {
            return Err(PyException::new_err(format!("ERROR in App.new: - argument \"data\" is a function callback, expected class")));
        }

        let app_refany = azul_impl::callbacks::RefAny::new(AppDataTy { _py_app_data: Some(data) });
        Ok(unsafe { mem::transmute(crate::AzApp_new(app_refany, mem::transmute(config))) })
    }
}

#[pyproto]
impl PyGCProtocol for AzApp {
    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {

        let data: &azul_impl::app::AzAppPtr = unsafe { mem::transmute(self) };

        // NOTE: should not block - this should only succeed
        // AFTER the App has finished executing
        let mut app_lock = match data.ptr.try_lock().ok() {
            Some(s) => s,
            None => return Ok(()),
        };

        let data_ref = match app_lock.data.downcast_ref::<AppDataTy>() {
            Some(s) => s,
            None => return Ok(()),
        };

        if let Some(obj) = data_ref._py_app_data.as_ref() {
            visit.call(obj)?;
        }

        Ok(())
    }

    fn __clear__(&mut self) {

        let mut data: &mut azul_impl::app::AzAppPtr = unsafe { mem::transmute(self) };

        // NOTE: should not block - this should only succeed
        // AFTER the App has finished executing
        let mut app_lock = match data.ptr.try_lock().ok() {
            Some(s) => s,
            None => return,
        };

        let mut data = match app_lock.data.downcast_mut::<AppDataTy>() {
            Some(s) => s,
            None => return,
        };

        if data._py_app_data.as_mut().is_some() {
            // Clear reference, this decrements Python ref counter.
            data._py_app_data = None;
        }
    }
}

#[pymethods]
impl AzLayoutCallbackEnumWrapper {
    #[staticmethod]
    pub fn new(py: Python, cb: PyObject) -> Result<Self, PyErr> {
        use pyo3::type_object::PyTypeInfo;

        {
            let cb_any = cb.as_ref(py);
            if !cb_any.is_callable() {
                let type_name = cb_any.get_type().name().unwrap_or("<unknown>");
                return Err(PyException::new_err(format!("ERROR in LayoutCallback.new: - argument \"cb\" is of type \"{}\", expected function", type_name)));
            }
        }

        Ok(unsafe { mem::transmute(Self::Marshaled(AzMarshaledLayoutCallback {
            marshal_data: mem::transmute(azul_impl::callbacks::RefAny::new(LayoutCallbackTy {
                _py_layout_callback: Some(cb)
            })),
            cb: AzMarshaledLayoutCallbackInner { cb: invoke_py_marshaled_layout_callback },
        }))})
    }
}

#[repr(C)]
pub struct AppDataTy {
    _py_app_data: Option<PyObject>,
}

#[repr(C)]
pub struct LayoutCallbackTy {
    // acual callable object from python
    _py_layout_callback: Option<PyObject>,
}

extern "C" fn invoke_py_marshaled_layout_callback(
    marshal_data: &mut AzRefAny,
    app_data: &mut AzRefAny,
    info: AzLayoutCallbackInfo
) -> AzStyledDom {

    let mut marshal_data: &mut azul_impl::callbacks::RefAny = unsafe { mem::transmute(marshal_data) };
    let mut app_data: &mut azul_impl::callbacks::RefAny = unsafe { mem::transmute(app_data) };

    let mut app_data_downcast = match app_data.downcast_mut::<AppDataTy>() {
        Some(s) => s,
        None => return AzStyledDom::default(),
    };

    let mut app_data_downcast = match app_data_downcast._py_app_data.as_mut() {
        Some(s) => s,
        None => return AzStyledDom::default(),
    };

    let mut pyfunction = match marshal_data.downcast_mut::<LayoutCallbackTy>() {
        Some(s) => s,
        None => return AzStyledDom::default(),
    };

    let mut pyfunction = match pyfunction._py_layout_callback.as_mut() {
        Some(s) => s,
        None => return AzStyledDom::default(),
    };

    // call layout callback into python
    let s: AzStyledDom = Python::with_gil(|py| {

        match pyfunction.call1(py.clone(), (app_data_downcast.clone_ref(py.clone()), info)) {
            Ok(o) => match o.as_ref(py).extract::<AzStyledDom>() {
                Ok(o) => o.clone(),
                Err(e) => {
                    #[cfg(feature = "logging")] {
                        let cb_any = o.as_ref(py);
                        let type_name = cb_any.get_type().name().unwrap_or("<unknown>");
                        log::error!("ERROR: LayoutCallback returned object of type {}, expected azul.dom.StyledDom", type_name);
                    }
                    AzStyledDom::default()
                }
            },
            Err(e) => {
                #[cfg(feature = "logging")] {
                    log::error!("Exception caught when invoking LayoutCallback: {}", e);
                }
                AzStyledDom::default()
            }
        }
    });

    unsafe { mem::transmute(s) }
}

#[pyproto]
impl PyGCProtocol for AzMarshaledLayoutCallback {
    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {

        let data: &azul_impl::callbacks::MarshaledLayoutCallback = unsafe { mem::transmute(self) };

        // temporary clone since we can't borrow mutable here
        let mut refany = data.marshal_data.clone();

        let data = match refany.downcast_ref::<LayoutCallbackTy>() {
            Some(s) => s,
            None => return Ok(()),
        };

        if let Some(obj) = data._py_layout_callback.as_ref() {
            visit.call(obj)?;
        }

        Ok(())
    }

    fn __clear__(&mut self) {

        let mut data: &mut azul_impl::callbacks::MarshaledLayoutCallback = unsafe { mem::transmute(self) };

        let mut data = match data.marshal_data.downcast_mut::<LayoutCallbackTy>() {
            Some(s) => s,
            None => return,
        };

        if data._py_layout_callback.as_mut().is_some() {
            // Clear reference, this decrements Python ref counter.
            data._py_layout_callback = None;
        }
    }
}

#[pymethods]
impl AzWindowCreateOptions {
    #[staticmethod]
    pub fn new(py: Python, cb: PyObject) -> Result<Self, PyErr> {
        let window = azul_impl::window::WindowCreateOptions {
            state: unsafe { mem::transmute(AzWindowState::new(py, cb)?) },
            .. Default::default()
        };
        Ok(unsafe { mem::transmute(window) })
    }
}

#[pymethods]
impl AzWindowState {
    #[staticmethod]
    pub fn new(py: Python, cb: PyObject) -> Result<Self, PyErr> {
        let layout_callback = AzLayoutCallbackEnumWrapper::new(py, cb)?;
        let window = azul_impl::window::WindowState {
            layout_callback: unsafe { mem::transmute(layout_callback) },
            .. Default::default()
        };
        Ok(unsafe { mem::transmute(window) })
    }
}
/// Main application class
#[repr(C)]
#[pyclass(name = "App")]
pub struct AzApp {
    pub ptr: *const c_void,
}

/// Configuration to set which messages should be logged.
#[repr(C)]
pub enum AzAppLogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Version of the layout solver to use - future binary versions of azul may have more fields here, necessary so that old compiled applications don't break with newer releases of azul. Newer layout versions are opt-in only.
#[repr(C)]
pub enum AzLayoutSolver {
    Default,
}

/// Whether the renderer has VSync enabled
#[repr(C)]
pub enum AzVsync {
    Enabled,
    Disabled,
}

/// Does the renderer render in SRGB color space? By default, azul tries to set it to `Enabled` and falls back to `Disabled` if the OpenGL context can't be initialized properly
#[repr(C)]
pub enum AzSrgb {
    Enabled,
    Disabled,
}

/// Does the renderer render using hardware acceleration? By default, azul tries to set it to `Enabled` and falls back to `Disabled` if the OpenGL context can't be initialized properly
#[repr(C)]
pub enum AzHwAcceleration {
    Enabled,
    Disabled,
}

/// Offset in physical pixels (integer units)
#[repr(C)]
#[pyclass(name = "LayoutPoint")]
pub struct AzLayoutPoint {
    #[pyo3(get, set)]
    pub x: isize,
    #[pyo3(get, set)]
    pub y: isize,
}

/// Size in physical pixels (integer units)
#[repr(C)]
#[pyclass(name = "LayoutSize")]
pub struct AzLayoutSize {
    #[pyo3(get, set)]
    pub width: isize,
    #[pyo3(get, set)]
    pub height: isize,
}

/// Re-export of rust-allocated (stack based) `IOSHandle` struct
#[repr(C)]
#[pyclass(name = "IOSHandle")]
pub struct AzIOSHandle {
    pub ui_window: *mut c_void,
    pub ui_view: *mut c_void,
    pub ui_view_controller: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `MacOSHandle` struct
#[repr(C)]
#[pyclass(name = "MacOSHandle")]
pub struct AzMacOSHandle {
    pub ns_window: *mut c_void,
    pub ns_view: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `XlibHandle` struct
#[repr(C)]
#[pyclass(name = "XlibHandle")]
pub struct AzXlibHandle {
    #[pyo3(get, set)]
    pub window: u64,
    pub display: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `XcbHandle` struct
#[repr(C)]
#[pyclass(name = "XcbHandle")]
pub struct AzXcbHandle {
    #[pyo3(get, set)]
    pub window: u32,
    pub connection: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `WaylandHandle` struct
#[repr(C)]
#[pyclass(name = "WaylandHandle")]
pub struct AzWaylandHandle {
    pub surface: *mut c_void,
    pub display: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `WindowsHandle` struct
#[repr(C)]
#[pyclass(name = "WindowsHandle")]
pub struct AzWindowsHandle {
    pub hwnd: *mut c_void,
    pub hinstance: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `WebHandle` struct
#[repr(C)]
#[pyclass(name = "WebHandle")]
pub struct AzWebHandle {
    #[pyo3(get, set)]
    pub id: u32,
}

/// Re-export of rust-allocated (stack based) `AndroidHandle` struct
#[repr(C)]
#[pyclass(name = "AndroidHandle")]
pub struct AzAndroidHandle {
    pub a_native_window: *mut c_void,
}

/// X11 window hint: Type of window
#[repr(C)]
pub enum AzXWindowType {
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
    Dialog,
    DropdownMenu,
    PopupMenu,
    Tooltip,
    Notification,
    Combo,
    Dnd,
    Normal,
}

/// Same as `LayoutPoint`, but uses `i32` instead of `isize`
#[repr(C)]
#[pyclass(name = "PhysicalPositionI32")]
pub struct AzPhysicalPositionI32 {
    #[pyo3(get, set)]
    pub x: i32,
    #[pyo3(get, set)]
    pub y: i32,
}

/// Same as `LayoutPoint`, but uses `u32` instead of `isize`
#[repr(C)]
#[pyclass(name = "PhysicalSizeU32")]
pub struct AzPhysicalSizeU32 {
    #[pyo3(get, set)]
    pub width: u32,
    #[pyo3(get, set)]
    pub height: u32,
}

/// Logical position (can differ based on HiDPI settings). Usually this is what you'd want for hit-testing and positioning elements.
#[repr(C)]
#[pyclass(name = "LogicalPosition")]
pub struct AzLogicalPosition {
    #[pyo3(get, set)]
    pub x: f32,
    #[pyo3(get, set)]
    pub y: f32,
}

/// A size in "logical" (non-HiDPI-adjusted) pixels in floating-point units
#[repr(C)]
#[pyclass(name = "LogicalSize")]
pub struct AzLogicalSize {
    #[pyo3(get, set)]
    pub width: f32,
    #[pyo3(get, set)]
    pub height: f32,
}

/// Unique hash of a window icon, so that azul does not have to compare the actual bytes to see wether the window icon has changed.
#[repr(C)]
#[pyclass(name = "IconKey")]
pub struct AzIconKey {
    #[pyo3(get, set)]
    pub id: usize,
}

/// Symbolic name for a keyboard key, does **not** take the keyboard locale into account
#[repr(C)]
pub enum AzVirtualKeyCode {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Snapshot,
    Scroll,
    Pause,
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,
    Left,
    Up,
    Right,
    Down,
    Back,
    Return,
    Space,
    Compose,
    Caret,
    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,
    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    NavigateForward,
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

/// Boolean flags relating to the current window state
#[repr(C)]
#[pyclass(name = "WindowFlags")]
pub struct AzWindowFlags {
    #[pyo3(get, set)]
    pub is_maximized: bool,
    #[pyo3(get, set)]
    pub is_minimized: bool,
    #[pyo3(get, set)]
    pub is_about_to_close: bool,
    #[pyo3(get, set)]
    pub is_fullscreen: bool,
    #[pyo3(get, set)]
    pub has_decorations: bool,
    #[pyo3(get, set)]
    pub is_visible: bool,
    #[pyo3(get, set)]
    pub is_always_on_top: bool,
    #[pyo3(get, set)]
    pub is_resizable: bool,
    #[pyo3(get, set)]
    pub has_focus: bool,
    #[pyo3(get, set)]
    pub has_extended_window_frame: bool,
    #[pyo3(get, set)]
    pub has_blur_behind_window: bool,
}

/// Debugging information, will be rendered as an overlay on top of the UI
#[repr(C)]
#[pyclass(name = "DebugState")]
pub struct AzDebugState {
    #[pyo3(get, set)]
    pub profiler_dbg: bool,
    #[pyo3(get, set)]
    pub render_target_dbg: bool,
    #[pyo3(get, set)]
    pub texture_cache_dbg: bool,
    #[pyo3(get, set)]
    pub gpu_time_queries: bool,
    #[pyo3(get, set)]
    pub gpu_sample_queries: bool,
    #[pyo3(get, set)]
    pub disable_batching: bool,
    #[pyo3(get, set)]
    pub epochs: bool,
    #[pyo3(get, set)]
    pub echo_driver_messages: bool,
    #[pyo3(get, set)]
    pub show_overdraw: bool,
    #[pyo3(get, set)]
    pub gpu_cache_dbg: bool,
    #[pyo3(get, set)]
    pub texture_cache_dbg_clear_evicted: bool,
    #[pyo3(get, set)]
    pub picture_caching_dbg: bool,
    #[pyo3(get, set)]
    pub primitive_dbg: bool,
    #[pyo3(get, set)]
    pub zoom_dbg: bool,
    #[pyo3(get, set)]
    pub small_screen: bool,
    #[pyo3(get, set)]
    pub disable_opaque_pass: bool,
    #[pyo3(get, set)]
    pub disable_alpha_pass: bool,
    #[pyo3(get, set)]
    pub disable_clip_masks: bool,
    #[pyo3(get, set)]
    pub disable_text_prims: bool,
    #[pyo3(get, set)]
    pub disable_gradient_prims: bool,
    #[pyo3(get, set)]
    pub obscure_images: bool,
    #[pyo3(get, set)]
    pub glyph_flashing: bool,
    #[pyo3(get, set)]
    pub smart_profiler: bool,
    #[pyo3(get, set)]
    pub invalidation_dbg: bool,
    #[pyo3(get, set)]
    pub tile_cache_logging_dbg: bool,
    #[pyo3(get, set)]
    pub profiler_capture: bool,
    #[pyo3(get, set)]
    pub force_picture_invalidation: bool,
}

/// Current icon of the mouse cursor
#[repr(C)]
pub enum AzMouseCursorType {
    Default,
    Crosshair,
    Hand,
    Arrow,
    Move,
    Text,
    Wait,
    Help,
    Progress,
    NotAllowed,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    Grab,
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
}

/// Renderer type of the current windows OpenGL context
#[repr(C)]
pub enum AzRendererType {
    Hardware,
    Software,
}

/// Re-export of rust-allocated (stack based) `MacWindowOptions` struct
#[repr(C)]
#[pyclass(name = "MacWindowOptions")]
pub struct AzMacWindowOptions {
    #[pyo3(get, set)]
    pub _reserved: u8,
}

/// Re-export of rust-allocated (stack based) `WasmWindowOptions` struct
#[repr(C)]
#[pyclass(name = "WasmWindowOptions")]
pub struct AzWasmWindowOptions {
    #[pyo3(get, set)]
    pub _reserved: u8,
}

/// Re-export of rust-allocated (stack based) `FullScreenMode` struct
#[repr(C)]
pub enum AzFullScreenMode {
    SlowFullScreen,
    FastFullScreen,
    SlowWindowed,
    FastWindowed,
}

/// Window theme, set by the operating system or `WindowCreateOptions.theme` on startup
#[repr(C)]
pub enum AzWindowTheme {
    DarkMode,
    LightMode,
}

/// Current state of touch devices / touch inputs
#[repr(C)]
#[pyclass(name = "TouchState")]
pub struct AzTouchState {
    #[pyo3(get, set)]
    pub unused: u8,
}

/// C-ABI stable wrapper over a `MarshaledLayoutCallbackInner`
#[repr(C)]
#[pyclass(name = "MarshaledLayoutCallbackInner")]
pub struct AzMarshaledLayoutCallbackInner {
    pub cb: AzMarshaledLayoutCallbackType,
}

/// `AzMarshaledLayoutCallbackType` struct
pub type AzMarshaledLayoutCallbackType = extern "C" fn(&mut AzRefAny, &mut AzRefAny, AzLayoutCallbackInfo) -> AzStyledDom;

/// C-ABI stable wrapper over a `LayoutCallbackType`
#[repr(C)]
#[pyclass(name = "LayoutCallbackInner")]
pub struct AzLayoutCallbackInner {
    pub cb: AzLayoutCallbackType,
}

/// `AzLayoutCallbackType` struct
pub type AzLayoutCallbackType = extern "C" fn(&mut AzRefAny, AzLayoutCallbackInfo) -> AzStyledDom;

/// C-ABI stable wrapper over a `CallbackType`
#[repr(C)]
#[pyclass(name = "Callback")]
pub struct AzCallback {
    pub cb: AzCallbackType,
}

/// `AzCallbackType` struct
pub type AzCallbackType = extern "C" fn(&mut AzRefAny, AzCallbackInfo) -> AzUpdate;

/// Specifies if the screen should be updated after the callback function has returned
#[repr(C)]
pub enum AzUpdate {
    DoNothing,
    RefreshDom,
    RefreshDomAllWindows,
}

/// Index of a Node in the internal `NodeDataContainer`
#[repr(C)]
#[pyclass(name = "NodeId")]
pub struct AzNodeId {
    #[pyo3(get, set)]
    pub inner: usize,
}

/// ID of a DOM - one window can contain multiple, nested DOMs (such as iframes)
#[repr(C)]
#[pyclass(name = "DomId")]
pub struct AzDomId {
    #[pyo3(get, set)]
    pub inner: usize,
}

/// Re-export of rust-allocated (stack based) `PositionInfoInner` struct
#[repr(C)]
#[pyclass(name = "PositionInfoInner")]
pub struct AzPositionInfoInner {
    #[pyo3(get, set)]
    pub x_offset: f32,
    #[pyo3(get, set)]
    pub y_offset: f32,
    #[pyo3(get, set)]
    pub static_x_offset: f32,
    #[pyo3(get, set)]
    pub static_y_offset: f32,
}

/// How should an animation repeat (loop, ping-pong, etc.)
#[repr(C)]
pub enum AzAnimationRepeat {
    NoRepeat,
    Loop,
    PingPong,
}

/// How many times should an animation repeat
#[repr(C, u8)]
pub enum AzAnimationRepeatCount {
    Times(usize),
    Infinite,
}

/// C-ABI wrapper over an `IFrameCallbackType`
#[repr(C)]
#[pyclass(name = "IFrameCallback")]
pub struct AzIFrameCallback {
    pub cb: AzIFrameCallbackType,
}

/// `AzIFrameCallbackType` struct
pub type AzIFrameCallbackType = extern "C" fn(&mut AzRefAny, AzIFrameCallbackInfo) -> AzIFrameCallbackReturn;

/// Re-export of rust-allocated (stack based) `RenderImageCallback` struct
#[repr(C)]
#[pyclass(name = "RenderImageCallback")]
pub struct AzRenderImageCallback {
    pub cb: AzRenderImageCallbackType,
}

/// `AzRenderImageCallbackType` struct
pub type AzRenderImageCallbackType = extern "C" fn(&mut AzRefAny, AzRenderImageCallbackInfo) -> AzImageRef;

/// Re-export of rust-allocated (stack based) `TimerCallback` struct
#[repr(C)]
#[pyclass(name = "TimerCallback")]
pub struct AzTimerCallback {
    pub cb: AzTimerCallbackType,
}

/// `AzTimerCallbackType` struct
pub type AzTimerCallbackType = extern "C" fn(&mut AzRefAny, &mut AzRefAny, AzTimerCallbackInfo) -> AzTimerCallbackReturn;

/// `AzWriteBackCallbackType` struct
pub type AzWriteBackCallbackType = extern "C" fn(&mut AzRefAny, AzRefAny, AzCallbackInfo) -> AzUpdate;

/// Re-export of rust-allocated (stack based) `WriteBackCallback` struct
#[repr(C)]
#[pyclass(name = "WriteBackCallback")]
pub struct AzWriteBackCallback {
    pub cb: AzWriteBackCallbackType,
}

/// Re-export of rust-allocated (stack based) `ThreadCallback` struct
#[repr(C)]
#[pyclass(name = "ThreadCallback")]
pub struct AzThreadCallback {
    pub cb: AzThreadCallbackType,
}

/// `AzThreadCallbackType` struct
pub type AzThreadCallbackType = extern "C" fn(AzRefAny, AzThreadSender, AzThreadReceiver);

/// `AzRefAnyDestructorType` struct
pub type AzRefAnyDestructorType = extern "C" fn(&mut c_void);

/// Re-export of rust-allocated (stack based) `RefCount` struct
#[repr(C)]
#[pyclass(name = "RefCount")]
pub struct AzRefCount {
    pub ptr: *const c_void,
}

/// When to call a callback action - `On::MouseOver`, `On::MouseOut`, etc.
#[repr(C)]
pub enum AzOn {
    MouseOver,
    MouseDown,
    LeftMouseDown,
    MiddleMouseDown,
    RightMouseDown,
    MouseUp,
    LeftMouseUp,
    MiddleMouseUp,
    RightMouseUp,
    MouseEnter,
    MouseLeave,
    Scroll,
    TextInput,
    VirtualKeyDown,
    VirtualKeyUp,
    HoveredFile,
    DroppedFile,
    HoveredFileCancelled,
    FocusReceived,
    FocusLost,
}

/// Re-export of rust-allocated (stack based) `HoverEventFilter` struct
#[repr(C)]
pub enum AzHoverEventFilter {
    MouseOver,
    MouseDown,
    LeftMouseDown,
    RightMouseDown,
    MiddleMouseDown,
    MouseUp,
    LeftMouseUp,
    RightMouseUp,
    MiddleMouseUp,
    MouseEnter,
    MouseLeave,
    Scroll,
    ScrollStart,
    ScrollEnd,
    TextInput,
    VirtualKeyDown,
    VirtualKeyUp,
    HoveredFile,
    DroppedFile,
    HoveredFileCancelled,
    TouchStart,
    TouchMove,
    TouchEnd,
    TouchCancel,
}

/// Re-export of rust-allocated (stack based) `FocusEventFilter` struct
#[repr(C)]
pub enum AzFocusEventFilter {
    MouseOver,
    MouseDown,
    LeftMouseDown,
    RightMouseDown,
    MiddleMouseDown,
    MouseUp,
    LeftMouseUp,
    RightMouseUp,
    MiddleMouseUp,
    MouseEnter,
    MouseLeave,
    Scroll,
    ScrollStart,
    ScrollEnd,
    TextInput,
    VirtualKeyDown,
    VirtualKeyUp,
    FocusReceived,
    FocusLost,
}

/// Re-export of rust-allocated (stack based) `WindowEventFilter` struct
#[repr(C)]
pub enum AzWindowEventFilter {
    MouseOver,
    MouseDown,
    LeftMouseDown,
    RightMouseDown,
    MiddleMouseDown,
    MouseUp,
    LeftMouseUp,
    RightMouseUp,
    MiddleMouseUp,
    MouseEnter,
    MouseLeave,
    Scroll,
    ScrollStart,
    ScrollEnd,
    TextInput,
    VirtualKeyDown,
    VirtualKeyUp,
    HoveredFile,
    DroppedFile,
    HoveredFileCancelled,
    Resized,
    Moved,
    TouchStart,
    TouchMove,
    TouchEnd,
    TouchCancel,
    FocusReceived,
    FocusLost,
    CloseRequested,
    ThemeChanged,
}

/// Re-export of rust-allocated (stack based) `ComponentEventFilter` struct
#[repr(C)]
pub enum AzComponentEventFilter {
    AfterMount,
    BeforeUnmount,
    NodeResized,
}

/// Re-export of rust-allocated (stack based) `ApplicationEventFilter` struct
#[repr(C)]
pub enum AzApplicationEventFilter {
    DeviceConnected,
    DeviceDisconnected,
}

/// Re-export of rust-allocated (stack based) `TabIndex` struct
#[repr(C, u8)]
pub enum AzTabIndex {
    Auto,
    OverrideInParent(u32),
    NoKeyboardFocus,
}

/// Re-export of rust-allocated (stack based) `NodeTypeKey` struct
#[repr(C)]
pub enum AzNodeTypeKey {
    Body,
    Div,
    Br,
    P,
    Img,
    IFrame,
}

/// Re-export of rust-allocated (stack based) `CssNthChildPattern` struct
#[repr(C)]
#[pyclass(name = "CssNthChildPattern")]
pub struct AzCssNthChildPattern {
    #[pyo3(get, set)]
    pub repeat: u32,
    #[pyo3(get, set)]
    pub offset: u32,
}

/// Re-export of rust-allocated (stack based) `CssPropertyType` struct
#[repr(C)]
pub enum AzCssPropertyType {
    TextColor,
    FontSize,
    FontFamily,
    TextAlign,
    LetterSpacing,
    LineHeight,
    WordSpacing,
    TabWidth,
    Cursor,
    Display,
    Float,
    BoxSizing,
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    Position,
    Top,
    Right,
    Left,
    Bottom,
    FlexWrap,
    FlexDirection,
    FlexGrow,
    FlexShrink,
    JustifyContent,
    AlignItems,
    AlignContent,
    OverflowX,
    OverflowY,
    PaddingTop,
    PaddingLeft,
    PaddingRight,
    PaddingBottom,
    MarginTop,
    MarginLeft,
    MarginRight,
    MarginBottom,
    Background,
    BackgroundImage,
    BackgroundColor,
    BackgroundPosition,
    BackgroundSize,
    BackgroundRepeat,
    BorderTopLeftRadius,
    BorderTopRightRadius,
    BorderBottomLeftRadius,
    BorderBottomRightRadius,
    BorderTopColor,
    BorderRightColor,
    BorderLeftColor,
    BorderBottomColor,
    BorderTopStyle,
    BorderRightStyle,
    BorderLeftStyle,
    BorderBottomStyle,
    BorderTopWidth,
    BorderRightWidth,
    BorderLeftWidth,
    BorderBottomWidth,
    BoxShadowLeft,
    BoxShadowRight,
    BoxShadowTop,
    BoxShadowBottom,
    ScrollbarStyle,
    Opacity,
    Transform,
    PerspectiveOrigin,
    TransformOrigin,
    BackfaceVisibility,
}

/// Re-export of rust-allocated (stack based) `ColorU` struct
#[repr(C)]
#[pyclass(name = "ColorU")]
pub struct AzColorU {
    #[pyo3(get, set)]
    pub r: u8,
    #[pyo3(get, set)]
    pub g: u8,
    #[pyo3(get, set)]
    pub b: u8,
    #[pyo3(get, set)]
    pub a: u8,
}

/// Re-export of rust-allocated (stack based) `SizeMetric` struct
#[repr(C)]
pub enum AzSizeMetric {
    Px,
    Pt,
    Em,
    Percent,
}

/// Re-export of rust-allocated (stack based) `FloatValue` struct
#[repr(C)]
#[pyclass(name = "FloatValue")]
pub struct AzFloatValue {
    #[pyo3(get, set)]
    pub number: isize,
}

/// Re-export of rust-allocated (stack based) `BoxShadowClipMode` struct
#[repr(C)]
pub enum AzBoxShadowClipMode {
    Outset,
    Inset,
}

/// Re-export of rust-allocated (stack based) `LayoutAlignContent` struct
#[repr(C)]
pub enum AzLayoutAlignContent {
    Stretch,
    Center,
    Start,
    End,
    SpaceBetween,
    SpaceAround,
}

/// Re-export of rust-allocated (stack based) `LayoutAlignItems` struct
#[repr(C)]
pub enum AzLayoutAlignItems {
    Stretch,
    Center,
    FlexStart,
    FlexEnd,
}

/// Re-export of rust-allocated (stack based) `LayoutBoxSizing` struct
#[repr(C)]
pub enum AzLayoutBoxSizing {
    ContentBox,
    BorderBox,
}

/// Re-export of rust-allocated (stack based) `LayoutFlexDirection` struct
#[repr(C)]
pub enum AzLayoutFlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Re-export of rust-allocated (stack based) `LayoutDisplay` struct
#[repr(C)]
pub enum AzLayoutDisplay {
    None,
    Flex,
    Block,
    InlineBlock,
}

/// Re-export of rust-allocated (stack based) `LayoutFloat` struct
#[repr(C)]
pub enum AzLayoutFloat {
    Left,
    Right,
}

/// Re-export of rust-allocated (stack based) `LayoutJustifyContent` struct
#[repr(C)]
pub enum AzLayoutJustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Re-export of rust-allocated (stack based) `LayoutPosition` struct
#[repr(C)]
pub enum AzLayoutPosition {
    Static,
    Relative,
    Absolute,
    Fixed,
}

/// Re-export of rust-allocated (stack based) `LayoutFlexWrap` struct
#[repr(C)]
pub enum AzLayoutFlexWrap {
    Wrap,
    NoWrap,
}

/// Re-export of rust-allocated (stack based) `LayoutOverflow` struct
#[repr(C)]
pub enum AzLayoutOverflow {
    Scroll,
    Auto,
    Hidden,
    Visible,
}

/// Re-export of rust-allocated (stack based) `AngleMetric` struct
#[repr(C)]
pub enum AzAngleMetric {
    Degree,
    Radians,
    Grad,
    Turn,
    Percent,
}

/// Re-export of rust-allocated (stack based) `DirectionCorner` struct
#[repr(C)]
pub enum AzDirectionCorner {
    Right,
    Left,
    Top,
    Bottom,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

/// Re-export of rust-allocated (stack based) `ExtendMode` struct
#[repr(C)]
pub enum AzExtendMode {
    Clamp,
    Repeat,
}

/// Re-export of rust-allocated (stack based) `Shape` struct
#[repr(C)]
pub enum AzShape {
    Ellipse,
    Circle,
}

/// Re-export of rust-allocated (stack based) `RadialGradientSize` struct
#[repr(C)]
pub enum AzRadialGradientSize {
    ClosestSide,
    ClosestCorner,
    FarthestSide,
    FarthestCorner,
}

/// Re-export of rust-allocated (stack based) `StyleBackgroundRepeat` struct
#[repr(C)]
pub enum AzStyleBackgroundRepeat {
    NoRepeat,
    Repeat,
    RepeatX,
    RepeatY,
}

/// Re-export of rust-allocated (stack based) `BorderStyle` struct
#[repr(C)]
pub enum AzBorderStyle {
    None,
    Solid,
    Double,
    Dotted,
    Dashed,
    Hidden,
    Groove,
    Ridge,
    Inset,
    Outset,
}

/// Re-export of rust-allocated (stack based) `StyleCursor` struct
#[repr(C)]
pub enum AzStyleCursor {
    Alias,
    AllScroll,
    Cell,
    ColResize,
    ContextMenu,
    Copy,
    Crosshair,
    Default,
    EResize,
    EwResize,
    Grab,
    Grabbing,
    Help,
    Move,
    NResize,
    NsResize,
    NeswResize,
    NwseResize,
    Pointer,
    Progress,
    RowResize,
    SResize,
    SeResize,
    Text,
    Unset,
    VerticalText,
    WResize,
    Wait,
    ZoomIn,
    ZoomOut,
}

/// Re-export of rust-allocated (stack based) `StyleBackfaceVisibility` struct
#[repr(C)]
pub enum AzStyleBackfaceVisibility {
    Hidden,
    Visible,
}

/// Re-export of rust-allocated (stack based) `StyleTextAlign` struct
#[repr(C)]
pub enum AzStyleTextAlign {
    Left,
    Center,
    Right,
}

/// Re-export of rust-allocated (stack based) `Node` struct
#[repr(C)]
#[pyclass(name = "Node")]
pub struct AzNode {
    #[pyo3(get, set)]
    pub parent: usize,
    #[pyo3(get, set)]
    pub previous_sibling: usize,
    #[pyo3(get, set)]
    pub next_sibling: usize,
    #[pyo3(get, set)]
    pub last_child: usize,
}

/// Re-export of rust-allocated (stack based) `CascadeInfo` struct
#[repr(C)]
#[pyclass(name = "CascadeInfo")]
pub struct AzCascadeInfo {
    #[pyo3(get, set)]
    pub index_in_parent: u32,
    #[pyo3(get, set)]
    pub is_last_child: bool,
}

/// Re-export of rust-allocated (stack based) `StyledNodeState` struct
#[repr(C)]
#[pyclass(name = "StyledNodeState")]
pub struct AzStyledNodeState {
    #[pyo3(get, set)]
    pub normal: bool,
    #[pyo3(get, set)]
    pub hover: bool,
    #[pyo3(get, set)]
    pub active: bool,
    #[pyo3(get, set)]
    pub focused: bool,
}

/// Re-export of rust-allocated (stack based) `TagId` struct
#[repr(C)]
#[pyclass(name = "TagId")]
pub struct AzTagId {
    #[pyo3(get, set)]
    pub inner: u64,
}

/// Re-export of rust-allocated (stack based) `CssPropertyCache` struct
#[repr(C)]
#[pyclass(name = "CssPropertyCache")]
pub struct AzCssPropertyCache {
    pub ptr: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `GlVoidPtrConst` struct
#[repr(C)]
#[pyclass(name = "GlVoidPtrConst")]
pub struct AzGlVoidPtrConst {
    pub ptr: *const c_void,
}

/// Re-export of rust-allocated (stack based) `GlVoidPtrMut` struct
#[repr(C)]
#[pyclass(name = "GlVoidPtrMut")]
pub struct AzGlVoidPtrMut {
    pub ptr: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `GlShaderPrecisionFormatReturn` struct
#[repr(C)]
#[pyclass(name = "GlShaderPrecisionFormatReturn")]
pub struct AzGlShaderPrecisionFormatReturn {
    #[pyo3(get, set)]
    pub _0: i32,
    #[pyo3(get, set)]
    pub _1: i32,
    #[pyo3(get, set)]
    pub _2: i32,
}

/// Re-export of rust-allocated (stack based) `VertexAttributeType` struct
#[repr(C)]
pub enum AzVertexAttributeType {
    Float,
    Double,
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}

/// Re-export of rust-allocated (stack based) `IndexBufferFormat` struct
#[repr(C)]
pub enum AzIndexBufferFormat {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

/// Re-export of rust-allocated (stack based) `GlType` struct
#[repr(C)]
pub enum AzGlType {
    Gl,
    Gles,
}

/// C-ABI stable reexport of `&[u8]`
#[repr(C)]
#[pyclass(name = "U8VecRef")]
pub struct AzU8VecRef {
    pub ptr: *const u8,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `&mut [u8]`
#[repr(C)]
#[pyclass(name = "U8VecRefMut")]
pub struct AzU8VecRefMut {
    pub ptr: *mut u8,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `&[f32]`
#[repr(C)]
#[pyclass(name = "F32VecRef")]
pub struct AzF32VecRef {
    pub ptr: *const f32,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `&[i32]`
#[repr(C)]
#[pyclass(name = "I32VecRef")]
pub struct AzI32VecRef {
    pub ptr: *const i32,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `&[GLuint]` aka `&[u32]`
#[repr(C)]
#[pyclass(name = "GLuintVecRef")]
pub struct AzGLuintVecRef {
    pub ptr: *const u32,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `&[GLenum]` aka `&[u32]`
#[repr(C)]
#[pyclass(name = "GLenumVecRef")]
pub struct AzGLenumVecRef {
    pub ptr: *const u32,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `&mut [GLint]` aka `&mut [i32]`
#[repr(C)]
#[pyclass(name = "GLintVecRefMut")]
pub struct AzGLintVecRefMut {
    pub ptr: *mut i32,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `&mut [GLint64]` aka `&mut [i64]`
#[repr(C)]
#[pyclass(name = "GLint64VecRefMut")]
pub struct AzGLint64VecRefMut {
    pub ptr: *mut i64,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `&mut [GLboolean]` aka `&mut [u8]`
#[repr(C)]
#[pyclass(name = "GLbooleanVecRefMut")]
pub struct AzGLbooleanVecRefMut {
    pub ptr: *mut u8,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `&mut [GLfloat]` aka `&mut [f32]`
#[repr(C)]
#[pyclass(name = "GLfloatVecRefMut")]
pub struct AzGLfloatVecRefMut {
    pub ptr: *mut f32,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `&str`
#[repr(C)]
#[pyclass(name = "Refstr")]
pub struct AzRefstr {
    pub ptr: *const u8,
    #[pyo3(get, set)]
    pub len: usize,
}

/// C-ABI stable reexport of `*const gleam::gl::GLsync`
#[repr(C)]
#[pyclass(name = "GLsyncPtr")]
pub struct AzGLsyncPtr {
    pub ptr: *const c_void,
}

/// Re-export of rust-allocated (stack based) `TextureFlags` struct
#[repr(C)]
#[pyclass(name = "TextureFlags")]
pub struct AzTextureFlags {
    #[pyo3(get, set)]
    pub is_opaque: bool,
    #[pyo3(get, set)]
    pub is_video_texture: bool,
}

/// Re-export of rust-allocated (stack based) `ImageRef` struct
#[repr(C)]
#[pyclass(name = "ImageRef")]
pub struct AzImageRef {
    pub data: *const c_void,
    pub copies: *const c_void,
}

/// Re-export of rust-allocated (stack based) `RawImageFormat` struct
#[repr(C)]
pub enum AzRawImageFormat {
    R8,
    R16,
    RG16,
    BGRA8,
    RGBAF32,
    RG8,
    RGBAI32,
    RGBA8,
}

/// Re-export of rust-allocated (stack based) `EncodeImageError` struct
#[repr(C)]
pub enum AzEncodeImageError {
    InsufficientMemory,
    DimensionError,
    InvalidData,
    Unknown,
}

/// Re-export of rust-allocated (stack based) `DecodeImageError` struct
#[repr(C)]
pub enum AzDecodeImageError {
    InsufficientMemory,
    DimensionError,
    UnsupportedImageFormat,
    Unknown,
}

/// `AzParsedFontDestructorFnType` struct
pub type AzParsedFontDestructorFnType = extern "C" fn(&mut c_void);

/// Atomically reference-counted parsed font data
#[repr(C)]
#[pyclass(name = "FontRef")]
pub struct AzFontRef {
    pub data: *const c_void,
    pub copies: *const c_void,
}

/// Re-export of rust-allocated (stack based) `Svg` struct
#[repr(C)]
#[pyclass(name = "Svg")]
pub struct AzSvg {
    pub ptr: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `SvgXmlNode` struct
#[repr(C)]
#[pyclass(name = "SvgXmlNode")]
pub struct AzSvgXmlNode {
    pub ptr: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `SvgCircle` struct
#[repr(C)]
#[pyclass(name = "SvgCircle")]
pub struct AzSvgCircle {
    #[pyo3(get, set)]
    pub center_x: f32,
    #[pyo3(get, set)]
    pub center_y: f32,
    #[pyo3(get, set)]
    pub radius: f32,
}

/// Re-export of rust-allocated (stack based) `SvgPoint` struct
#[repr(C)]
#[pyclass(name = "SvgPoint")]
pub struct AzSvgPoint {
    #[pyo3(get, set)]
    pub x: f32,
    #[pyo3(get, set)]
    pub y: f32,
}

/// Re-export of rust-allocated (stack based) `SvgRect` struct
#[repr(C)]
#[pyclass(name = "SvgRect")]
pub struct AzSvgRect {
    #[pyo3(get, set)]
    pub width: f32,
    #[pyo3(get, set)]
    pub height: f32,
    #[pyo3(get, set)]
    pub x: f32,
    #[pyo3(get, set)]
    pub y: f32,
    #[pyo3(get, set)]
    pub radius_top_left: f32,
    #[pyo3(get, set)]
    pub radius_top_right: f32,
    #[pyo3(get, set)]
    pub radius_bottom_left: f32,
    #[pyo3(get, set)]
    pub radius_bottom_right: f32,
}

/// Re-export of rust-allocated (stack based) `SvgVertex` struct
#[repr(C)]
#[pyclass(name = "SvgVertex")]
pub struct AzSvgVertex {
    #[pyo3(get, set)]
    pub x: f32,
    #[pyo3(get, set)]
    pub y: f32,
}

/// Re-export of rust-allocated (stack based) `ShapeRendering` struct
#[repr(C)]
pub enum AzShapeRendering {
    OptimizeSpeed,
    CrispEdges,
    GeometricPrecision,
}

/// Re-export of rust-allocated (stack based) `TextRendering` struct
#[repr(C)]
pub enum AzTextRendering {
    OptimizeSpeed,
    OptimizeLegibility,
    GeometricPrecision,
}

/// Re-export of rust-allocated (stack based) `ImageRendering` struct
#[repr(C)]
pub enum AzImageRendering {
    OptimizeQuality,
    OptimizeSpeed,
}

/// Re-export of rust-allocated (stack based) `FontDatabase` struct
#[repr(C)]
pub enum AzFontDatabase {
    Empty,
    System,
}

/// Re-export of rust-allocated (stack based) `Indent` struct
#[repr(C, u8)]
pub enum AzIndent {
    None,
    Spaces(u8),
    Tabs,
}

/// Re-export of rust-allocated (stack based) `SvgFitTo` struct
#[repr(C, u8)]
pub enum AzSvgFitTo {
    Original,
    Width(u32),
    Height(u32),
    Zoom(f32),
}

/// Re-export of rust-allocated (stack based) `SvgFillRule` struct
#[repr(C)]
pub enum AzSvgFillRule {
    Winding,
    EvenOdd,
}

/// Re-export of rust-allocated (stack based) `SvgTransform` struct
#[repr(C)]
#[pyclass(name = "SvgTransform")]
pub struct AzSvgTransform {
    #[pyo3(get, set)]
    pub sx: f32,
    #[pyo3(get, set)]
    pub kx: f32,
    #[pyo3(get, set)]
    pub ky: f32,
    #[pyo3(get, set)]
    pub sy: f32,
    #[pyo3(get, set)]
    pub tx: f32,
    #[pyo3(get, set)]
    pub ty: f32,
}

/// Re-export of rust-allocated (stack based) `SvgLineJoin` struct
#[repr(C)]
pub enum AzSvgLineJoin {
    Miter,
    MiterClip,
    Round,
    Bevel,
}

/// Re-export of rust-allocated (stack based) `SvgLineCap` struct
#[repr(C)]
pub enum AzSvgLineCap {
    Butt,
    Square,
    Round,
}

/// Re-export of rust-allocated (stack based) `SvgDashPattern` struct
#[repr(C)]
#[pyclass(name = "SvgDashPattern")]
pub struct AzSvgDashPattern {
    #[pyo3(get, set)]
    pub offset: f32,
    #[pyo3(get, set)]
    pub length_1: f32,
    #[pyo3(get, set)]
    pub gap_1: f32,
    #[pyo3(get, set)]
    pub length_2: f32,
    #[pyo3(get, set)]
    pub gap_2: f32,
    #[pyo3(get, set)]
    pub length_3: f32,
    #[pyo3(get, set)]
    pub gap_3: f32,
}

/// **Reference-counted** file handle
#[repr(C)]
#[pyclass(name = "File")]
pub struct AzFile {
    pub ptr: *const c_void,
}

/// Re-export of rust-allocated (stack based) `MsgBox` struct
#[repr(C)]
#[pyclass(name = "MsgBox")]
pub struct AzMsgBox {
    #[pyo3(get, set)]
    pub _reserved: usize,
}

/// Type of message box icon
#[repr(C)]
pub enum AzMsgBoxIcon {
    Info,
    Warning,
    Error,
    Question,
}

/// Value returned from a yes / no message box
#[repr(C)]
pub enum AzMsgBoxYesNo {
    Yes,
    No,
}

/// Value returned from an ok / cancel message box
#[repr(C)]
pub enum AzMsgBoxOkCancel {
    Ok,
    Cancel,
}

/// File picker dialog
#[repr(C)]
#[pyclass(name = "FileDialog")]
pub struct AzFileDialog {
    #[pyo3(get, set)]
    pub _reserved: usize,
}

/// Re-export of rust-allocated (stack based) `ColorPickerDialog` struct
#[repr(C)]
#[pyclass(name = "ColorPickerDialog")]
pub struct AzColorPickerDialog {
    #[pyo3(get, set)]
    pub _reserved: usize,
}

/// Connection to the system clipboard, on some systems this connection can be cached
#[repr(C)]
#[pyclass(name = "SystemClipboard")]
pub struct AzSystemClipboard {
    pub _native: *const c_void,
}

/// `AzInstantPtrCloneFnType` struct
pub type AzInstantPtrCloneFnType = extern "C" fn(&AzInstantPtr) -> AzInstantPtr;

/// Re-export of rust-allocated (stack based) `InstantPtrCloneFn` struct
#[repr(C)]
#[pyclass(name = "InstantPtrCloneFn")]
pub struct AzInstantPtrCloneFn {
    pub cb: AzInstantPtrCloneFnType,
}

/// `AzInstantPtrDestructorFnType` struct
pub type AzInstantPtrDestructorFnType = extern "C" fn(&mut AzInstantPtr);

/// Re-export of rust-allocated (stack based) `InstantPtrDestructorFn` struct
#[repr(C)]
#[pyclass(name = "InstantPtrDestructorFn")]
pub struct AzInstantPtrDestructorFn {
    pub cb: AzInstantPtrDestructorFnType,
}

/// Re-export of rust-allocated (stack based) `SystemTick` struct
#[repr(C)]
#[pyclass(name = "SystemTick")]
pub struct AzSystemTick {
    #[pyo3(get, set)]
    pub tick_counter: u64,
}

/// Re-export of rust-allocated (stack based) `SystemTimeDiff` struct
#[repr(C)]
#[pyclass(name = "SystemTimeDiff")]
pub struct AzSystemTimeDiff {
    #[pyo3(get, set)]
    pub secs: u64,
    #[pyo3(get, set)]
    pub nanos: u32,
}

/// Re-export of rust-allocated (stack based) `SystemTickDiff` struct
#[repr(C)]
#[pyclass(name = "SystemTickDiff")]
pub struct AzSystemTickDiff {
    #[pyo3(get, set)]
    pub tick_diff: u64,
}

/// Re-export of rust-allocated (stack based) `TimerId` struct
#[repr(C)]
#[pyclass(name = "TimerId")]
pub struct AzTimerId {
    #[pyo3(get, set)]
    pub id: usize,
}

/// Should a timer terminate or not - used to remove active timers
#[repr(C)]
pub enum AzTerminateTimer {
    Terminate,
    Continue,
}

/// Re-export of rust-allocated (stack based) `ThreadId` struct
#[repr(C)]
#[pyclass(name = "ThreadId")]
pub struct AzThreadId {
    #[pyo3(get, set)]
    pub id: usize,
}

/// Re-export of rust-allocated (stack based) `Thread` struct
#[repr(C)]
#[pyclass(name = "Thread")]
pub struct AzThread {
    pub ptr: *const c_void,
}

/// Re-export of rust-allocated (stack based) `ThreadSender` struct
#[repr(C)]
#[pyclass(name = "ThreadSender")]
pub struct AzThreadSender {
    pub ptr: *const c_void,
}

/// Re-export of rust-allocated (stack based) `ThreadReceiver` struct
#[repr(C)]
#[pyclass(name = "ThreadReceiver")]
pub struct AzThreadReceiver {
    pub ptr: *const c_void,
}

/// `AzCreateThreadFnType` struct
pub type AzCreateThreadFnType = extern "C" fn(AzRefAny, AzRefAny, AzThreadCallback) -> AzThread;

/// Re-export of rust-allocated (stack based) `CreateThreadFn` struct
#[repr(C)]
#[pyclass(name = "CreateThreadFn")]
pub struct AzCreateThreadFn {
    pub cb: AzCreateThreadFnType,
}

/// `AzGetSystemTimeFnType` struct
pub type AzGetSystemTimeFnType = extern "C" fn() -> AzInstant;

/// Get the current system time, equivalent to `std::time::Instant::now()`, except it also works on systems that work with "ticks" instead of timers
#[repr(C)]
#[pyclass(name = "GetSystemTimeFn")]
pub struct AzGetSystemTimeFn {
    pub cb: AzGetSystemTimeFnType,
}

/// `AzCheckThreadFinishedFnType` struct
pub type AzCheckThreadFinishedFnType = extern "C" fn(&c_void) -> bool;

/// Function called to check if the thread has finished
#[repr(C)]
#[pyclass(name = "CheckThreadFinishedFn")]
pub struct AzCheckThreadFinishedFn {
    pub cb: AzCheckThreadFinishedFnType,
}

/// `AzLibrarySendThreadMsgFnType` struct
pub type AzLibrarySendThreadMsgFnType = extern "C" fn(&c_void, AzThreadSendMsg) -> bool;

/// Function to send a message to the thread
#[repr(C)]
#[pyclass(name = "LibrarySendThreadMsgFn")]
pub struct AzLibrarySendThreadMsgFn {
    pub cb: AzLibrarySendThreadMsgFnType,
}

/// `AzLibraryReceiveThreadMsgFnType` struct
pub type AzLibraryReceiveThreadMsgFnType = extern "C" fn(&c_void) -> AzOptionThreadReceiveMsg;

/// Function to receive a message from the thread
#[repr(C)]
#[pyclass(name = "LibraryReceiveThreadMsgFn")]
pub struct AzLibraryReceiveThreadMsgFn {
    pub cb: AzLibraryReceiveThreadMsgFnType,
}

/// `AzThreadRecvFnType` struct
pub type AzThreadRecvFnType = extern "C" fn(&c_void) -> AzOptionThreadSendMsg;

/// Function that the running `Thread` can call to receive messages from the main UI thread
#[repr(C)]
#[pyclass(name = "ThreadRecvFn")]
pub struct AzThreadRecvFn {
    pub cb: AzThreadRecvFnType,
}

/// `AzThreadSendFnType` struct
pub type AzThreadSendFnType = extern "C" fn(&c_void, AzThreadReceiveMsg) -> bool;

/// Function that the running `Thread` can call to receive messages from the main UI thread
#[repr(C)]
#[pyclass(name = "ThreadSendFn")]
pub struct AzThreadSendFn {
    pub cb: AzThreadSendFnType,
}

/// `AzThreadDestructorFnType` struct
pub type AzThreadDestructorFnType = extern "C" fn(&mut AzThread);

/// Destructor of the `Thread`
#[repr(C)]
#[pyclass(name = "ThreadDestructorFn")]
pub struct AzThreadDestructorFn {
    pub cb: AzThreadDestructorFnType,
}

/// `AzThreadReceiverDestructorFnType` struct
pub type AzThreadReceiverDestructorFnType = extern "C" fn(&mut AzThreadReceiver);

/// Destructor of the `ThreadReceiver`
#[repr(C)]
#[pyclass(name = "ThreadReceiverDestructorFn")]
pub struct AzThreadReceiverDestructorFn {
    pub cb: AzThreadReceiverDestructorFnType,
}

/// `AzThreadSenderDestructorFnType` struct
pub type AzThreadSenderDestructorFnType = extern "C" fn(&mut AzThreadSender);

/// Destructor of the `ThreadSender`
#[repr(C)]
#[pyclass(name = "ThreadSenderDestructorFn")]
pub struct AzThreadSenderDestructorFn {
    pub cb: AzThreadSenderDestructorFnType,
}

/// Re-export of rust-allocated (stack based) `StyleFontFamilyVecDestructor` struct
#[repr(C, u8)]
pub enum AzStyleFontFamilyVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzStyleFontFamilyVecDestructorType),
}

/// `AzStyleFontFamilyVecDestructorType` struct
pub type AzStyleFontFamilyVecDestructorType = extern "C" fn(&mut AzStyleFontFamilyVec);

/// Re-export of rust-allocated (stack based) `TesselatedSvgNodeVecDestructor` struct
#[repr(C, u8)]
pub enum AzTesselatedSvgNodeVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzTesselatedSvgNodeVecDestructorType),
}

/// `AzTesselatedSvgNodeVecDestructorType` struct
pub type AzTesselatedSvgNodeVecDestructorType = extern "C" fn(&mut AzTesselatedSvgNodeVec);

/// Re-export of rust-allocated (stack based) `XmlNodeVecDestructor` struct
#[repr(C, u8)]
pub enum AzXmlNodeVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzXmlNodeVecDestructorType),
}

/// `AzXmlNodeVecDestructorType` struct
pub type AzXmlNodeVecDestructorType = extern "C" fn(&mut AzXmlNodeVec);

/// Re-export of rust-allocated (stack based) `FmtArgVecDestructor` struct
#[repr(C, u8)]
pub enum AzFmtArgVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzFmtArgVecDestructorType),
}

/// `AzFmtArgVecDestructorType` struct
pub type AzFmtArgVecDestructorType = extern "C" fn(&mut AzFmtArgVec);

/// Re-export of rust-allocated (stack based) `InlineLineVecDestructor` struct
#[repr(C, u8)]
pub enum AzInlineLineVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzInlineLineVecDestructorType),
}

/// `AzInlineLineVecDestructorType` struct
pub type AzInlineLineVecDestructorType = extern "C" fn(&mut AzInlineLineVec);

/// Re-export of rust-allocated (stack based) `InlineWordVecDestructor` struct
#[repr(C, u8)]
pub enum AzInlineWordVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzInlineWordVecDestructorType),
}

/// `AzInlineWordVecDestructorType` struct
pub type AzInlineWordVecDestructorType = extern "C" fn(&mut AzInlineWordVec);

/// Re-export of rust-allocated (stack based) `InlineGlyphVecDestructor` struct
#[repr(C, u8)]
pub enum AzInlineGlyphVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzInlineGlyphVecDestructorType),
}

/// `AzInlineGlyphVecDestructorType` struct
pub type AzInlineGlyphVecDestructorType = extern "C" fn(&mut AzInlineGlyphVec);

/// Re-export of rust-allocated (stack based) `InlineTextHitVecDestructor` struct
#[repr(C, u8)]
pub enum AzInlineTextHitVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzInlineTextHitVecDestructorType),
}

/// `AzInlineTextHitVecDestructorType` struct
pub type AzInlineTextHitVecDestructorType = extern "C" fn(&mut AzInlineTextHitVec);

/// Re-export of rust-allocated (stack based) `MonitorVecDestructor` struct
#[repr(C, u8)]
pub enum AzMonitorVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzMonitorVecDestructorType),
}

/// `AzMonitorVecDestructorType` struct
pub type AzMonitorVecDestructorType = extern "C" fn(&mut AzMonitorVec);

/// Re-export of rust-allocated (stack based) `VideoModeVecDestructor` struct
#[repr(C, u8)]
pub enum AzVideoModeVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzVideoModeVecDestructorType),
}

/// `AzVideoModeVecDestructorType` struct
pub type AzVideoModeVecDestructorType = extern "C" fn(&mut AzVideoModeVec);

/// Re-export of rust-allocated (stack based) `DomVecDestructor` struct
#[repr(C, u8)]
pub enum AzDomVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzDomVecDestructorType),
}

/// `AzDomVecDestructorType` struct
pub type AzDomVecDestructorType = extern "C" fn(&mut AzDomVec);

/// Re-export of rust-allocated (stack based) `IdOrClassVecDestructor` struct
#[repr(C, u8)]
pub enum AzIdOrClassVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzIdOrClassVecDestructorType),
}

/// `AzIdOrClassVecDestructorType` struct
pub type AzIdOrClassVecDestructorType = extern "C" fn(&mut AzIdOrClassVec);

/// Re-export of rust-allocated (stack based) `NodeDataInlineCssPropertyVecDestructor` struct
#[repr(C, u8)]
pub enum AzNodeDataInlineCssPropertyVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzNodeDataInlineCssPropertyVecDestructorType),
}

/// `AzNodeDataInlineCssPropertyVecDestructorType` struct
pub type AzNodeDataInlineCssPropertyVecDestructorType = extern "C" fn(&mut AzNodeDataInlineCssPropertyVec);

/// Re-export of rust-allocated (stack based) `StyleBackgroundContentVecDestructor` struct
#[repr(C, u8)]
pub enum AzStyleBackgroundContentVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzStyleBackgroundContentVecDestructorType),
}

/// `AzStyleBackgroundContentVecDestructorType` struct
pub type AzStyleBackgroundContentVecDestructorType = extern "C" fn(&mut AzStyleBackgroundContentVec);

/// Re-export of rust-allocated (stack based) `StyleBackgroundPositionVecDestructor` struct
#[repr(C, u8)]
pub enum AzStyleBackgroundPositionVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzStyleBackgroundPositionVecDestructorType),
}

/// `AzStyleBackgroundPositionVecDestructorType` struct
pub type AzStyleBackgroundPositionVecDestructorType = extern "C" fn(&mut AzStyleBackgroundPositionVec);

/// Re-export of rust-allocated (stack based) `StyleBackgroundRepeatVecDestructor` struct
#[repr(C, u8)]
pub enum AzStyleBackgroundRepeatVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzStyleBackgroundRepeatVecDestructorType),
}

/// `AzStyleBackgroundRepeatVecDestructorType` struct
pub type AzStyleBackgroundRepeatVecDestructorType = extern "C" fn(&mut AzStyleBackgroundRepeatVec);

/// Re-export of rust-allocated (stack based) `StyleBackgroundSizeVecDestructor` struct
#[repr(C, u8)]
pub enum AzStyleBackgroundSizeVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzStyleBackgroundSizeVecDestructorType),
}

/// `AzStyleBackgroundSizeVecDestructorType` struct
pub type AzStyleBackgroundSizeVecDestructorType = extern "C" fn(&mut AzStyleBackgroundSizeVec);

/// Re-export of rust-allocated (stack based) `StyleTransformVecDestructor` struct
#[repr(C, u8)]
pub enum AzStyleTransformVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzStyleTransformVecDestructorType),
}

/// `AzStyleTransformVecDestructorType` struct
pub type AzStyleTransformVecDestructorType = extern "C" fn(&mut AzStyleTransformVec);

/// Re-export of rust-allocated (stack based) `CssPropertyVecDestructor` struct
#[repr(C, u8)]
pub enum AzCssPropertyVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzCssPropertyVecDestructorType),
}

/// `AzCssPropertyVecDestructorType` struct
pub type AzCssPropertyVecDestructorType = extern "C" fn(&mut AzCssPropertyVec);

/// Re-export of rust-allocated (stack based) `SvgMultiPolygonVecDestructor` struct
#[repr(C, u8)]
pub enum AzSvgMultiPolygonVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzSvgMultiPolygonVecDestructorType),
}

/// `AzSvgMultiPolygonVecDestructorType` struct
pub type AzSvgMultiPolygonVecDestructorType = extern "C" fn(&mut AzSvgMultiPolygonVec);

/// Re-export of rust-allocated (stack based) `SvgPathVecDestructor` struct
#[repr(C, u8)]
pub enum AzSvgPathVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzSvgPathVecDestructorType),
}

/// `AzSvgPathVecDestructorType` struct
pub type AzSvgPathVecDestructorType = extern "C" fn(&mut AzSvgPathVec);

/// Re-export of rust-allocated (stack based) `VertexAttributeVecDestructor` struct
#[repr(C, u8)]
pub enum AzVertexAttributeVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzVertexAttributeVecDestructorType),
}

/// `AzVertexAttributeVecDestructorType` struct
pub type AzVertexAttributeVecDestructorType = extern "C" fn(&mut AzVertexAttributeVec);

/// Re-export of rust-allocated (stack based) `SvgPathElementVecDestructor` struct
#[repr(C, u8)]
pub enum AzSvgPathElementVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzSvgPathElementVecDestructorType),
}

/// `AzSvgPathElementVecDestructorType` struct
pub type AzSvgPathElementVecDestructorType = extern "C" fn(&mut AzSvgPathElementVec);

/// Re-export of rust-allocated (stack based) `SvgVertexVecDestructor` struct
#[repr(C, u8)]
pub enum AzSvgVertexVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzSvgVertexVecDestructorType),
}

/// `AzSvgVertexVecDestructorType` struct
pub type AzSvgVertexVecDestructorType = extern "C" fn(&mut AzSvgVertexVec);

/// Re-export of rust-allocated (stack based) `U32VecDestructor` struct
#[repr(C, u8)]
pub enum AzU32VecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzU32VecDestructorType),
}

/// `AzU32VecDestructorType` struct
pub type AzU32VecDestructorType = extern "C" fn(&mut AzU32Vec);

/// Re-export of rust-allocated (stack based) `XWindowTypeVecDestructor` struct
#[repr(C, u8)]
pub enum AzXWindowTypeVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzXWindowTypeVecDestructorType),
}

/// `AzXWindowTypeVecDestructorType` struct
pub type AzXWindowTypeVecDestructorType = extern "C" fn(&mut AzXWindowTypeVec);

/// Re-export of rust-allocated (stack based) `VirtualKeyCodeVecDestructor` struct
#[repr(C, u8)]
pub enum AzVirtualKeyCodeVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzVirtualKeyCodeVecDestructorType),
}

/// `AzVirtualKeyCodeVecDestructorType` struct
pub type AzVirtualKeyCodeVecDestructorType = extern "C" fn(&mut AzVirtualKeyCodeVec);

/// Re-export of rust-allocated (stack based) `CascadeInfoVecDestructor` struct
#[repr(C, u8)]
pub enum AzCascadeInfoVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzCascadeInfoVecDestructorType),
}

/// `AzCascadeInfoVecDestructorType` struct
pub type AzCascadeInfoVecDestructorType = extern "C" fn(&mut AzCascadeInfoVec);

/// Re-export of rust-allocated (stack based) `ScanCodeVecDestructor` struct
#[repr(C, u8)]
pub enum AzScanCodeVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzScanCodeVecDestructorType),
}

/// `AzScanCodeVecDestructorType` struct
pub type AzScanCodeVecDestructorType = extern "C" fn(&mut AzScanCodeVec);

/// Re-export of rust-allocated (stack based) `CssDeclarationVecDestructor` struct
#[repr(C, u8)]
pub enum AzCssDeclarationVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzCssDeclarationVecDestructorType),
}

/// `AzCssDeclarationVecDestructorType` struct
pub type AzCssDeclarationVecDestructorType = extern "C" fn(&mut AzCssDeclarationVec);

/// Re-export of rust-allocated (stack based) `CssPathSelectorVecDestructor` struct
#[repr(C, u8)]
pub enum AzCssPathSelectorVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzCssPathSelectorVecDestructorType),
}

/// `AzCssPathSelectorVecDestructorType` struct
pub type AzCssPathSelectorVecDestructorType = extern "C" fn(&mut AzCssPathSelectorVec);

/// Re-export of rust-allocated (stack based) `StylesheetVecDestructor` struct
#[repr(C, u8)]
pub enum AzStylesheetVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzStylesheetVecDestructorType),
}

/// `AzStylesheetVecDestructorType` struct
pub type AzStylesheetVecDestructorType = extern "C" fn(&mut AzStylesheetVec);

/// Re-export of rust-allocated (stack based) `CssRuleBlockVecDestructor` struct
#[repr(C, u8)]
pub enum AzCssRuleBlockVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzCssRuleBlockVecDestructorType),
}

/// `AzCssRuleBlockVecDestructorType` struct
pub type AzCssRuleBlockVecDestructorType = extern "C" fn(&mut AzCssRuleBlockVec);

/// Re-export of rust-allocated (stack based) `F32VecDestructor` struct
#[repr(C, u8)]
pub enum AzF32VecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzF32VecDestructorType),
}

/// `AzF32VecDestructorType` struct
pub type AzF32VecDestructorType = extern "C" fn(&mut AzF32Vec);

/// Re-export of rust-allocated (stack based) `U16VecDestructor` struct
#[repr(C, u8)]
pub enum AzU16VecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzU16VecDestructorType),
}

/// `AzU16VecDestructorType` struct
pub type AzU16VecDestructorType = extern "C" fn(&mut AzU16Vec);

/// Re-export of rust-allocated (stack based) `U8VecDestructor` struct
#[repr(C, u8)]
pub enum AzU8VecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzU8VecDestructorType),
}

/// `AzU8VecDestructorType` struct
pub type AzU8VecDestructorType = extern "C" fn(&mut AzU8Vec);

/// Re-export of rust-allocated (stack based) `CallbackDataVecDestructor` struct
#[repr(C, u8)]
pub enum AzCallbackDataVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzCallbackDataVecDestructorType),
}

/// `AzCallbackDataVecDestructorType` struct
pub type AzCallbackDataVecDestructorType = extern "C" fn(&mut AzCallbackDataVec);

/// Re-export of rust-allocated (stack based) `DebugMessageVecDestructor` struct
#[repr(C, u8)]
pub enum AzDebugMessageVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzDebugMessageVecDestructorType),
}

/// `AzDebugMessageVecDestructorType` struct
pub type AzDebugMessageVecDestructorType = extern "C" fn(&mut AzDebugMessageVec);

/// Re-export of rust-allocated (stack based) `GLuintVecDestructor` struct
#[repr(C, u8)]
pub enum AzGLuintVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzGLuintVecDestructorType),
}

/// `AzGLuintVecDestructorType` struct
pub type AzGLuintVecDestructorType = extern "C" fn(&mut AzGLuintVec);

/// Re-export of rust-allocated (stack based) `GLintVecDestructor` struct
#[repr(C, u8)]
pub enum AzGLintVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzGLintVecDestructorType),
}

/// `AzGLintVecDestructorType` struct
pub type AzGLintVecDestructorType = extern "C" fn(&mut AzGLintVec);

/// Re-export of rust-allocated (stack based) `StringVecDestructor` struct
#[repr(C, u8)]
pub enum AzStringVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzStringVecDestructorType),
}

/// `AzStringVecDestructorType` struct
pub type AzStringVecDestructorType = extern "C" fn(&mut AzStringVec);

/// Re-export of rust-allocated (stack based) `StringPairVecDestructor` struct
#[repr(C, u8)]
pub enum AzStringPairVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzStringPairVecDestructorType),
}

/// `AzStringPairVecDestructorType` struct
pub type AzStringPairVecDestructorType = extern "C" fn(&mut AzStringPairVec);

/// Re-export of rust-allocated (stack based) `NormalizedLinearColorStopVecDestructor` struct
#[repr(C, u8)]
pub enum AzNormalizedLinearColorStopVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzNormalizedLinearColorStopVecDestructorType),
}

/// `AzNormalizedLinearColorStopVecDestructorType` struct
pub type AzNormalizedLinearColorStopVecDestructorType = extern "C" fn(&mut AzNormalizedLinearColorStopVec);

/// Re-export of rust-allocated (stack based) `NormalizedRadialColorStopVecDestructor` struct
#[repr(C, u8)]
pub enum AzNormalizedRadialColorStopVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzNormalizedRadialColorStopVecDestructorType),
}

/// `AzNormalizedRadialColorStopVecDestructorType` struct
pub type AzNormalizedRadialColorStopVecDestructorType = extern "C" fn(&mut AzNormalizedRadialColorStopVec);

/// Re-export of rust-allocated (stack based) `NodeIdVecDestructor` struct
#[repr(C, u8)]
pub enum AzNodeIdVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzNodeIdVecDestructorType),
}

/// `AzNodeIdVecDestructorType` struct
pub type AzNodeIdVecDestructorType = extern "C" fn(&mut AzNodeIdVec);

/// Re-export of rust-allocated (stack based) `NodeVecDestructor` struct
#[repr(C, u8)]
pub enum AzNodeVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzNodeVecDestructorType),
}

/// `AzNodeVecDestructorType` struct
pub type AzNodeVecDestructorType = extern "C" fn(&mut AzNodeVec);

/// Re-export of rust-allocated (stack based) `StyledNodeVecDestructor` struct
#[repr(C, u8)]
pub enum AzStyledNodeVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzStyledNodeVecDestructorType),
}

/// `AzStyledNodeVecDestructorType` struct
pub type AzStyledNodeVecDestructorType = extern "C" fn(&mut AzStyledNodeVec);

/// Re-export of rust-allocated (stack based) `TagIdsToNodeIdsMappingVecDestructor` struct
#[repr(C, u8)]
pub enum AzTagIdsToNodeIdsMappingVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzTagIdsToNodeIdsMappingVecDestructorType),
}

/// `AzTagIdsToNodeIdsMappingVecDestructorType` struct
pub type AzTagIdsToNodeIdsMappingVecDestructorType = extern "C" fn(&mut AzTagIdsToNodeIdsMappingVec);

/// Re-export of rust-allocated (stack based) `ParentWithNodeDepthVecDestructor` struct
#[repr(C, u8)]
pub enum AzParentWithNodeDepthVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzParentWithNodeDepthVecDestructorType),
}

/// `AzParentWithNodeDepthVecDestructorType` struct
pub type AzParentWithNodeDepthVecDestructorType = extern "C" fn(&mut AzParentWithNodeDepthVec);

/// Re-export of rust-allocated (stack based) `NodeDataVecDestructor` struct
#[repr(C, u8)]
pub enum AzNodeDataVecDestructor {
    DefaultRust,
    NoDestructor,
    External(AzNodeDataVecDestructorType),
}

/// `AzNodeDataVecDestructorType` struct
pub type AzNodeDataVecDestructorType = extern "C" fn(&mut AzNodeDataVec);

/// Re-export of rust-allocated (stack based) `OptionI16` struct
#[repr(C, u8)]
pub enum AzOptionI16 {
    None,
    Some(i16),
}

/// Re-export of rust-allocated (stack based) `OptionU16` struct
#[repr(C, u8)]
pub enum AzOptionU16 {
    None,
    Some(u16),
}

/// Re-export of rust-allocated (stack based) `OptionU32` struct
#[repr(C, u8)]
pub enum AzOptionU32 {
    None,
    Some(u32),
}

/// Re-export of rust-allocated (stack based) `OptionHwndHandle` struct
#[repr(C, u8)]
pub enum AzOptionHwndHandle {
    None,
    Some(*mut c_void),
}

/// Re-export of rust-allocated (stack based) `OptionX11Visual` struct
#[repr(C, u8)]
pub enum AzOptionX11Visual {
    None,
    Some(*const c_void),
}

/// Re-export of rust-allocated (stack based) `OptionI32` struct
#[repr(C, u8)]
pub enum AzOptionI32 {
    None,
    Some(i32),
}

/// Re-export of rust-allocated (stack based) `OptionF32` struct
#[repr(C, u8)]
pub enum AzOptionF32 {
    None,
    Some(f32),
}

/// Option<char> but the char is a u32, for C FFI stability reasons
#[repr(C, u8)]
pub enum AzOptionChar {
    None,
    Some(u32),
}

/// Re-export of rust-allocated (stack based) `OptionUsize` struct
#[repr(C, u8)]
pub enum AzOptionUsize {
    None,
    Some(usize),
}

/// Re-export of rust-allocated (stack based) `SvgParseErrorPosition` struct
#[repr(C)]
#[pyclass(name = "SvgParseErrorPosition")]
pub struct AzSvgParseErrorPosition {
    #[pyo3(get, set)]
    pub row: u32,
    #[pyo3(get, set)]
    pub col: u32,
}

/// External system callbacks to get the system time or create / manage threads
#[repr(C)]
#[pyclass(name = "SystemCallbacks")]
pub struct AzSystemCallbacks {
    #[pyo3(get, set)]
    pub create_thread_fn: AzCreateThreadFn,
    #[pyo3(get, set)]
    pub get_system_time_fn: AzGetSystemTimeFn,
}

/// Force a specific renderer: note that azul will **crash** on startup if the `RendererOptions` are not satisfied.
#[repr(C)]
#[pyclass(name = "RendererOptions")]
pub struct AzRendererOptions {
    #[pyo3(get, set)]
    pub vsync: AzVsyncEnumWrapper,
    #[pyo3(get, set)]
    pub srgb: AzSrgbEnumWrapper,
    #[pyo3(get, set)]
    pub hw_accel: AzHwAccelerationEnumWrapper,
}

/// Represents a rectangle in physical pixels (integer units)
#[repr(C)]
#[pyclass(name = "LayoutRect")]
pub struct AzLayoutRect {
    #[pyo3(get, set)]
    pub origin: AzLayoutPoint,
    #[pyo3(get, set)]
    pub size: AzLayoutSize,
}

/// Raw platform handle, for integration in / with other toolkits and custom non-azul window extensions
#[repr(C, u8)]
pub enum AzRawWindowHandle {
    IOS(AzIOSHandle),
    MacOS(AzMacOSHandle),
    Xlib(AzXlibHandle),
    Xcb(AzXcbHandle),
    Wayland(AzWaylandHandle),
    Windows(AzWindowsHandle),
    Web(AzWebHandle),
    Android(AzAndroidHandle),
    Unsupported,
}

/// Logical rectangle area (can differ based on HiDPI settings). Usually this is what you'd want for hit-testing and positioning elements.
#[repr(C)]
#[pyclass(name = "LogicalRect")]
pub struct AzLogicalRect {
    #[pyo3(get, set)]
    pub origin: AzLogicalPosition,
    #[pyo3(get, set)]
    pub size: AzLogicalSize,
}

/// Symbolic accelerator key (ctrl, alt, shift)
#[repr(C, u8)]
pub enum AzAcceleratorKey {
    Ctrl,
    Alt,
    Shift,
    Key(AzVirtualKeyCode),
}

/// Current position of the mouse cursor, relative to the window. Set to `Uninitialized` on startup (gets initialized on the first frame).
#[repr(C, u8)]
pub enum AzCursorPosition {
    OutOfWindow,
    Uninitialized,
    InWindow(AzLogicalPosition),
}

/// Position of the top left corner of the window relative to the top left of the monitor
#[repr(C, u8)]
pub enum AzWindowPosition {
    Uninitialized,
    Initialized(AzPhysicalPositionI32),
}

/// Position of the virtual keyboard necessary to insert CJK characters
#[repr(C, u8)]
pub enum AzImePosition {
    Uninitialized,
    Initialized(AzLogicalPosition),
}

/// Describes a rendering configuration for a monitor
#[repr(C)]
#[pyclass(name = "VideoMode")]
pub struct AzVideoMode {
    #[pyo3(get, set)]
    pub size: AzLayoutSize,
    #[pyo3(get, set)]
    pub bit_depth: u16,
    #[pyo3(get, set)]
    pub refresh_rate: u16,
}

/// Combination of node ID + DOM ID, both together can identify a node
#[repr(C)]
#[pyclass(name = "DomNodeId")]
pub struct AzDomNodeId {
    #[pyo3(get, set)]
    pub dom: AzDomId,
    #[pyo3(get, set)]
    pub node: AzNodeId,
}

/// Re-export of rust-allocated (stack based) `PositionInfo` struct
#[repr(C, u8)]
pub enum AzPositionInfo {
    Static(AzPositionInfoInner),
    Fixed(AzPositionInfoInner),
    Absolute(AzPositionInfoInner),
    Relative(AzPositionInfoInner),
}

/// Re-export of rust-allocated (stack based) `HidpiAdjustedBounds` struct
#[repr(C)]
#[pyclass(name = "HidpiAdjustedBounds")]
pub struct AzHidpiAdjustedBounds {
    #[pyo3(get, set)]
    pub logical_size: AzLogicalSize,
    #[pyo3(get, set)]
    pub hidpi_factor: f32,
}

/// Re-export of rust-allocated (stack based) `InlineGlyph` struct
#[repr(C)]
#[pyclass(name = "InlineGlyph")]
pub struct AzInlineGlyph {
    #[pyo3(get, set)]
    pub bounds: AzLogicalRect,
    #[pyo3(get, set)]
    pub unicode_codepoint: AzOptionCharEnumWrapper,
    #[pyo3(get, set)]
    pub glyph_index: u32,
}

/// Re-export of rust-allocated (stack based) `InlineTextHit` struct
#[repr(C)]
#[pyclass(name = "InlineTextHit")]
pub struct AzInlineTextHit {
    #[pyo3(get, set)]
    pub unicode_codepoint: AzOptionCharEnumWrapper,
    #[pyo3(get, set)]
    pub hit_relative_to_inline_text: AzLogicalPosition,
    #[pyo3(get, set)]
    pub hit_relative_to_line: AzLogicalPosition,
    #[pyo3(get, set)]
    pub hit_relative_to_text_content: AzLogicalPosition,
    #[pyo3(get, set)]
    pub hit_relative_to_glyph: AzLogicalPosition,
    #[pyo3(get, set)]
    pub line_index_relative_to_text: usize,
    #[pyo3(get, set)]
    pub word_index_relative_to_text: usize,
    #[pyo3(get, set)]
    pub text_content_index_relative_to_text: usize,
    #[pyo3(get, set)]
    pub glyph_index_relative_to_text: usize,
    #[pyo3(get, set)]
    pub char_index_relative_to_text: usize,
    #[pyo3(get, set)]
    pub word_index_relative_to_line: usize,
    #[pyo3(get, set)]
    pub text_content_index_relative_to_line: usize,
    #[pyo3(get, set)]
    pub glyph_index_relative_to_line: usize,
    #[pyo3(get, set)]
    pub char_index_relative_to_line: usize,
    #[pyo3(get, set)]
    pub glyph_index_relative_to_word: usize,
    #[pyo3(get, set)]
    pub char_index_relative_to_word: usize,
}

/// Re-export of rust-allocated (stack based) `IFrameCallbackInfo` struct
#[repr(C)]
#[pyclass(name = "IFrameCallbackInfo")]
pub struct AzIFrameCallbackInfo {
    pub system_fonts: *const c_void,
    pub image_cache: *const c_void,
    #[pyo3(get, set)]
    pub window_theme: AzWindowThemeEnumWrapper,
    #[pyo3(get, set)]
    pub bounds: AzHidpiAdjustedBounds,
    #[pyo3(get, set)]
    pub scroll_size: AzLogicalSize,
    #[pyo3(get, set)]
    pub scroll_offset: AzLogicalPosition,
    #[pyo3(get, set)]
    pub virtual_scroll_size: AzLogicalSize,
    #[pyo3(get, set)]
    pub virtual_scroll_offset: AzLogicalPosition,
    pub _reserved_ref: *const c_void,
    pub _reserved_mut: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `TimerCallbackReturn` struct
#[repr(C)]
#[pyclass(name = "TimerCallbackReturn")]
pub struct AzTimerCallbackReturn {
    #[pyo3(get, set)]
    pub should_update: AzUpdateEnumWrapper,
    #[pyo3(get, set)]
    pub should_terminate: AzTerminateTimerEnumWrapper,
}

/// RefAny is a reference-counted, opaque pointer, which stores a reference to a struct. `RefAny` can be up- and downcasted (this usually done via generics and can't be expressed in the Rust API)
#[repr(C)]
#[pyclass(name = "RefAny")]
pub struct AzRefAny {
    pub _internal_ptr: *const c_void,
    #[pyo3(get, set)]
    pub sharing_info: AzRefCount,
}

/// Re-export of rust-allocated (stack based) `IFrameNode` struct
#[repr(C)]
#[pyclass(name = "IFrameNode")]
pub struct AzIFrameNode {
    #[pyo3(get, set)]
    pub callback: AzIFrameCallback,
    #[pyo3(get, set)]
    pub data: AzRefAny,
}

/// Re-export of rust-allocated (stack based) `NotEventFilter` struct
#[repr(C, u8)]
pub enum AzNotEventFilter {
    Hover(AzHoverEventFilter),
    Focus(AzFocusEventFilter),
}

/// Re-export of rust-allocated (stack based) `CssNthChildSelector` struct
#[repr(C, u8)]
pub enum AzCssNthChildSelector {
    Number(u32),
    Even,
    Odd,
    Pattern(AzCssNthChildPattern),
}

/// Re-export of rust-allocated (stack based) `PixelValue` struct
#[repr(C)]
#[pyclass(name = "PixelValue")]
pub struct AzPixelValue {
    #[pyo3(get, set)]
    pub metric: AzSizeMetricEnumWrapper,
    #[pyo3(get, set)]
    pub number: AzFloatValue,
}

/// Re-export of rust-allocated (stack based) `PixelValueNoPercent` struct
#[repr(C)]
#[pyclass(name = "PixelValueNoPercent")]
pub struct AzPixelValueNoPercent {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleBoxShadow` struct
#[repr(C)]
#[pyclass(name = "StyleBoxShadow")]
pub struct AzStyleBoxShadow {
    pub offset: [AzPixelValueNoPercent;2],
    #[pyo3(get, set)]
    pub color: AzColorU,
    #[pyo3(get, set)]
    pub blur_radius: AzPixelValueNoPercent,
    #[pyo3(get, set)]
    pub spread_radius: AzPixelValueNoPercent,
    #[pyo3(get, set)]
    pub clip_mode: AzBoxShadowClipModeEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `LayoutBottom` struct
#[repr(C)]
#[pyclass(name = "LayoutBottom")]
pub struct AzLayoutBottom {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutFlexGrow` struct
#[repr(C)]
#[pyclass(name = "LayoutFlexGrow")]
pub struct AzLayoutFlexGrow {
    #[pyo3(get, set)]
    pub inner: AzFloatValue,
}

/// Re-export of rust-allocated (stack based) `LayoutFlexShrink` struct
#[repr(C)]
#[pyclass(name = "LayoutFlexShrink")]
pub struct AzLayoutFlexShrink {
    #[pyo3(get, set)]
    pub inner: AzFloatValue,
}

/// Re-export of rust-allocated (stack based) `LayoutHeight` struct
#[repr(C)]
#[pyclass(name = "LayoutHeight")]
pub struct AzLayoutHeight {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutLeft` struct
#[repr(C)]
#[pyclass(name = "LayoutLeft")]
pub struct AzLayoutLeft {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutMarginBottom` struct
#[repr(C)]
#[pyclass(name = "LayoutMarginBottom")]
pub struct AzLayoutMarginBottom {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutMarginLeft` struct
#[repr(C)]
#[pyclass(name = "LayoutMarginLeft")]
pub struct AzLayoutMarginLeft {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutMarginRight` struct
#[repr(C)]
#[pyclass(name = "LayoutMarginRight")]
pub struct AzLayoutMarginRight {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutMarginTop` struct
#[repr(C)]
#[pyclass(name = "LayoutMarginTop")]
pub struct AzLayoutMarginTop {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutMaxHeight` struct
#[repr(C)]
#[pyclass(name = "LayoutMaxHeight")]
pub struct AzLayoutMaxHeight {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutMaxWidth` struct
#[repr(C)]
#[pyclass(name = "LayoutMaxWidth")]
pub struct AzLayoutMaxWidth {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutMinHeight` struct
#[repr(C)]
#[pyclass(name = "LayoutMinHeight")]
pub struct AzLayoutMinHeight {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutMinWidth` struct
#[repr(C)]
#[pyclass(name = "LayoutMinWidth")]
pub struct AzLayoutMinWidth {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutPaddingBottom` struct
#[repr(C)]
#[pyclass(name = "LayoutPaddingBottom")]
pub struct AzLayoutPaddingBottom {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutPaddingLeft` struct
#[repr(C)]
#[pyclass(name = "LayoutPaddingLeft")]
pub struct AzLayoutPaddingLeft {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutPaddingRight` struct
#[repr(C)]
#[pyclass(name = "LayoutPaddingRight")]
pub struct AzLayoutPaddingRight {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutPaddingTop` struct
#[repr(C)]
#[pyclass(name = "LayoutPaddingTop")]
pub struct AzLayoutPaddingTop {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutRight` struct
#[repr(C)]
#[pyclass(name = "LayoutRight")]
pub struct AzLayoutRight {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutTop` struct
#[repr(C)]
#[pyclass(name = "LayoutTop")]
pub struct AzLayoutTop {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `LayoutWidth` struct
#[repr(C)]
#[pyclass(name = "LayoutWidth")]
pub struct AzLayoutWidth {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `PercentageValue` struct
#[repr(C)]
#[pyclass(name = "PercentageValue")]
pub struct AzPercentageValue {
    #[pyo3(get, set)]
    pub number: AzFloatValue,
}

/// Re-export of rust-allocated (stack based) `AngleValue` struct
#[repr(C)]
#[pyclass(name = "AngleValue")]
pub struct AzAngleValue {
    #[pyo3(get, set)]
    pub metric: AzAngleMetricEnumWrapper,
    #[pyo3(get, set)]
    pub number: AzFloatValue,
}

/// Re-export of rust-allocated (stack based) `NormalizedLinearColorStop` struct
#[repr(C)]
#[pyclass(name = "NormalizedLinearColorStop")]
pub struct AzNormalizedLinearColorStop {
    #[pyo3(get, set)]
    pub offset: AzPercentageValue,
    #[pyo3(get, set)]
    pub color: AzColorU,
}

/// Re-export of rust-allocated (stack based) `NormalizedRadialColorStop` struct
#[repr(C)]
#[pyclass(name = "NormalizedRadialColorStop")]
pub struct AzNormalizedRadialColorStop {
    #[pyo3(get, set)]
    pub offset: AzAngleValue,
    #[pyo3(get, set)]
    pub color: AzColorU,
}

/// Re-export of rust-allocated (stack based) `DirectionCorners` struct
#[repr(C)]
#[pyclass(name = "DirectionCorners")]
pub struct AzDirectionCorners {
    #[pyo3(get, set)]
    pub from: AzDirectionCornerEnumWrapper,
    #[pyo3(get, set)]
    pub to: AzDirectionCornerEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `Direction` struct
#[repr(C, u8)]
pub enum AzDirection {
    Angle(AzAngleValue),
    FromTo(AzDirectionCorners),
}

/// Re-export of rust-allocated (stack based) `BackgroundPositionHorizontal` struct
#[repr(C, u8)]
pub enum AzBackgroundPositionHorizontal {
    Left,
    Center,
    Right,
    Exact(AzPixelValue),
}

/// Re-export of rust-allocated (stack based) `BackgroundPositionVertical` struct
#[repr(C, u8)]
pub enum AzBackgroundPositionVertical {
    Top,
    Center,
    Bottom,
    Exact(AzPixelValue),
}

/// Re-export of rust-allocated (stack based) `StyleBackgroundPosition` struct
#[repr(C)]
#[pyclass(name = "StyleBackgroundPosition")]
pub struct AzStyleBackgroundPosition {
    #[pyo3(get, set)]
    pub horizontal: AzBackgroundPositionHorizontalEnumWrapper,
    #[pyo3(get, set)]
    pub vertical: AzBackgroundPositionVerticalEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `StyleBackgroundSize` struct
#[repr(C, u8)]
pub enum AzStyleBackgroundSize {
    ExactSize([AzPixelValue;2]),
    Contain,
    Cover,
}

/// Re-export of rust-allocated (stack based) `StyleBorderBottomColor` struct
#[repr(C)]
#[pyclass(name = "StyleBorderBottomColor")]
pub struct AzStyleBorderBottomColor {
    #[pyo3(get, set)]
    pub inner: AzColorU,
}

/// Re-export of rust-allocated (stack based) `StyleBorderBottomLeftRadius` struct
#[repr(C)]
#[pyclass(name = "StyleBorderBottomLeftRadius")]
pub struct AzStyleBorderBottomLeftRadius {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleBorderBottomRightRadius` struct
#[repr(C)]
#[pyclass(name = "StyleBorderBottomRightRadius")]
pub struct AzStyleBorderBottomRightRadius {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleBorderBottomStyle` struct
#[repr(C)]
#[pyclass(name = "StyleBorderBottomStyle")]
pub struct AzStyleBorderBottomStyle {
    #[pyo3(get, set)]
    pub inner: AzBorderStyleEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `LayoutBorderBottomWidth` struct
#[repr(C)]
#[pyclass(name = "LayoutBorderBottomWidth")]
pub struct AzLayoutBorderBottomWidth {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleBorderLeftColor` struct
#[repr(C)]
#[pyclass(name = "StyleBorderLeftColor")]
pub struct AzStyleBorderLeftColor {
    #[pyo3(get, set)]
    pub inner: AzColorU,
}

/// Re-export of rust-allocated (stack based) `StyleBorderLeftStyle` struct
#[repr(C)]
#[pyclass(name = "StyleBorderLeftStyle")]
pub struct AzStyleBorderLeftStyle {
    #[pyo3(get, set)]
    pub inner: AzBorderStyleEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `LayoutBorderLeftWidth` struct
#[repr(C)]
#[pyclass(name = "LayoutBorderLeftWidth")]
pub struct AzLayoutBorderLeftWidth {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleBorderRightColor` struct
#[repr(C)]
#[pyclass(name = "StyleBorderRightColor")]
pub struct AzStyleBorderRightColor {
    #[pyo3(get, set)]
    pub inner: AzColorU,
}

/// Re-export of rust-allocated (stack based) `StyleBorderRightStyle` struct
#[repr(C)]
#[pyclass(name = "StyleBorderRightStyle")]
pub struct AzStyleBorderRightStyle {
    #[pyo3(get, set)]
    pub inner: AzBorderStyleEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `LayoutBorderRightWidth` struct
#[repr(C)]
#[pyclass(name = "LayoutBorderRightWidth")]
pub struct AzLayoutBorderRightWidth {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleBorderTopColor` struct
#[repr(C)]
#[pyclass(name = "StyleBorderTopColor")]
pub struct AzStyleBorderTopColor {
    #[pyo3(get, set)]
    pub inner: AzColorU,
}

/// Re-export of rust-allocated (stack based) `StyleBorderTopLeftRadius` struct
#[repr(C)]
#[pyclass(name = "StyleBorderTopLeftRadius")]
pub struct AzStyleBorderTopLeftRadius {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleBorderTopRightRadius` struct
#[repr(C)]
#[pyclass(name = "StyleBorderTopRightRadius")]
pub struct AzStyleBorderTopRightRadius {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleBorderTopStyle` struct
#[repr(C)]
#[pyclass(name = "StyleBorderTopStyle")]
pub struct AzStyleBorderTopStyle {
    #[pyo3(get, set)]
    pub inner: AzBorderStyleEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `LayoutBorderTopWidth` struct
#[repr(C)]
#[pyclass(name = "LayoutBorderTopWidth")]
pub struct AzLayoutBorderTopWidth {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleFontSize` struct
#[repr(C)]
#[pyclass(name = "StyleFontSize")]
pub struct AzStyleFontSize {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleLetterSpacing` struct
#[repr(C)]
#[pyclass(name = "StyleLetterSpacing")]
pub struct AzStyleLetterSpacing {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleLineHeight` struct
#[repr(C)]
#[pyclass(name = "StyleLineHeight")]
pub struct AzStyleLineHeight {
    #[pyo3(get, set)]
    pub inner: AzPercentageValue,
}

/// Re-export of rust-allocated (stack based) `StyleTabWidth` struct
#[repr(C)]
#[pyclass(name = "StyleTabWidth")]
pub struct AzStyleTabWidth {
    #[pyo3(get, set)]
    pub inner: AzPercentageValue,
}

/// Re-export of rust-allocated (stack based) `StyleOpacity` struct
#[repr(C)]
#[pyclass(name = "StyleOpacity")]
pub struct AzStyleOpacity {
    #[pyo3(get, set)]
    pub inner: AzPercentageValue,
}

/// Re-export of rust-allocated (stack based) `StyleTransformOrigin` struct
#[repr(C)]
#[pyclass(name = "StyleTransformOrigin")]
pub struct AzStyleTransformOrigin {
    #[pyo3(get, set)]
    pub x: AzPixelValue,
    #[pyo3(get, set)]
    pub y: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StylePerspectiveOrigin` struct
#[repr(C)]
#[pyclass(name = "StylePerspectiveOrigin")]
pub struct AzStylePerspectiveOrigin {
    #[pyo3(get, set)]
    pub x: AzPixelValue,
    #[pyo3(get, set)]
    pub y: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleTransformMatrix2D` struct
#[repr(C)]
#[pyclass(name = "StyleTransformMatrix2D")]
pub struct AzStyleTransformMatrix2D {
    #[pyo3(get, set)]
    pub a: AzPixelValue,
    #[pyo3(get, set)]
    pub b: AzPixelValue,
    #[pyo3(get, set)]
    pub c: AzPixelValue,
    #[pyo3(get, set)]
    pub d: AzPixelValue,
    #[pyo3(get, set)]
    pub tx: AzPixelValue,
    #[pyo3(get, set)]
    pub ty: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleTransformMatrix3D` struct
#[repr(C)]
#[pyclass(name = "StyleTransformMatrix3D")]
pub struct AzStyleTransformMatrix3D {
    #[pyo3(get, set)]
    pub m11: AzPixelValue,
    #[pyo3(get, set)]
    pub m12: AzPixelValue,
    #[pyo3(get, set)]
    pub m13: AzPixelValue,
    #[pyo3(get, set)]
    pub m14: AzPixelValue,
    #[pyo3(get, set)]
    pub m21: AzPixelValue,
    #[pyo3(get, set)]
    pub m22: AzPixelValue,
    #[pyo3(get, set)]
    pub m23: AzPixelValue,
    #[pyo3(get, set)]
    pub m24: AzPixelValue,
    #[pyo3(get, set)]
    pub m31: AzPixelValue,
    #[pyo3(get, set)]
    pub m32: AzPixelValue,
    #[pyo3(get, set)]
    pub m33: AzPixelValue,
    #[pyo3(get, set)]
    pub m34: AzPixelValue,
    #[pyo3(get, set)]
    pub m41: AzPixelValue,
    #[pyo3(get, set)]
    pub m42: AzPixelValue,
    #[pyo3(get, set)]
    pub m43: AzPixelValue,
    #[pyo3(get, set)]
    pub m44: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleTransformTranslate2D` struct
#[repr(C)]
#[pyclass(name = "StyleTransformTranslate2D")]
pub struct AzStyleTransformTranslate2D {
    #[pyo3(get, set)]
    pub x: AzPixelValue,
    #[pyo3(get, set)]
    pub y: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleTransformTranslate3D` struct
#[repr(C)]
#[pyclass(name = "StyleTransformTranslate3D")]
pub struct AzStyleTransformTranslate3D {
    #[pyo3(get, set)]
    pub x: AzPixelValue,
    #[pyo3(get, set)]
    pub y: AzPixelValue,
    #[pyo3(get, set)]
    pub z: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleTransformRotate3D` struct
#[repr(C)]
#[pyclass(name = "StyleTransformRotate3D")]
pub struct AzStyleTransformRotate3D {
    #[pyo3(get, set)]
    pub x: AzPercentageValue,
    #[pyo3(get, set)]
    pub y: AzPercentageValue,
    #[pyo3(get, set)]
    pub z: AzPercentageValue,
    #[pyo3(get, set)]
    pub angle: AzAngleValue,
}

/// Re-export of rust-allocated (stack based) `StyleTransformScale2D` struct
#[repr(C)]
#[pyclass(name = "StyleTransformScale2D")]
pub struct AzStyleTransformScale2D {
    #[pyo3(get, set)]
    pub x: AzPercentageValue,
    #[pyo3(get, set)]
    pub y: AzPercentageValue,
}

/// Re-export of rust-allocated (stack based) `StyleTransformScale3D` struct
#[repr(C)]
#[pyclass(name = "StyleTransformScale3D")]
pub struct AzStyleTransformScale3D {
    #[pyo3(get, set)]
    pub x: AzPercentageValue,
    #[pyo3(get, set)]
    pub y: AzPercentageValue,
    #[pyo3(get, set)]
    pub z: AzPercentageValue,
}

/// Re-export of rust-allocated (stack based) `StyleTransformSkew2D` struct
#[repr(C)]
#[pyclass(name = "StyleTransformSkew2D")]
pub struct AzStyleTransformSkew2D {
    #[pyo3(get, set)]
    pub x: AzPercentageValue,
    #[pyo3(get, set)]
    pub y: AzPercentageValue,
}

/// Re-export of rust-allocated (stack based) `StyleTextColor` struct
#[repr(C)]
#[pyclass(name = "StyleTextColor")]
pub struct AzStyleTextColor {
    #[pyo3(get, set)]
    pub inner: AzColorU,
}

/// Re-export of rust-allocated (stack based) `StyleWordSpacing` struct
#[repr(C)]
#[pyclass(name = "StyleWordSpacing")]
pub struct AzStyleWordSpacing {
    #[pyo3(get, set)]
    pub inner: AzPixelValue,
}

/// Re-export of rust-allocated (stack based) `StyleBoxShadowValue` struct
#[repr(C, u8)]
pub enum AzStyleBoxShadowValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBoxShadow),
}

/// Re-export of rust-allocated (stack based) `LayoutAlignContentValue` struct
#[repr(C, u8)]
pub enum AzLayoutAlignContentValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutAlignContent),
}

/// Re-export of rust-allocated (stack based) `LayoutAlignItemsValue` struct
#[repr(C, u8)]
pub enum AzLayoutAlignItemsValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutAlignItems),
}

/// Re-export of rust-allocated (stack based) `LayoutBottomValue` struct
#[repr(C, u8)]
pub enum AzLayoutBottomValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutBottom),
}

/// Re-export of rust-allocated (stack based) `LayoutBoxSizingValue` struct
#[repr(C, u8)]
pub enum AzLayoutBoxSizingValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutBoxSizing),
}

/// Re-export of rust-allocated (stack based) `LayoutFlexDirectionValue` struct
#[repr(C, u8)]
pub enum AzLayoutFlexDirectionValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutFlexDirection),
}

/// Re-export of rust-allocated (stack based) `LayoutDisplayValue` struct
#[repr(C, u8)]
pub enum AzLayoutDisplayValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutDisplay),
}

/// Re-export of rust-allocated (stack based) `LayoutFlexGrowValue` struct
#[repr(C, u8)]
pub enum AzLayoutFlexGrowValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutFlexGrow),
}

/// Re-export of rust-allocated (stack based) `LayoutFlexShrinkValue` struct
#[repr(C, u8)]
pub enum AzLayoutFlexShrinkValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutFlexShrink),
}

/// Re-export of rust-allocated (stack based) `LayoutFloatValue` struct
#[repr(C, u8)]
pub enum AzLayoutFloatValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutFloat),
}

/// Re-export of rust-allocated (stack based) `LayoutHeightValue` struct
#[repr(C, u8)]
pub enum AzLayoutHeightValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutHeight),
}

/// Re-export of rust-allocated (stack based) `LayoutJustifyContentValue` struct
#[repr(C, u8)]
pub enum AzLayoutJustifyContentValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutJustifyContent),
}

/// Re-export of rust-allocated (stack based) `LayoutLeftValue` struct
#[repr(C, u8)]
pub enum AzLayoutLeftValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutLeft),
}

/// Re-export of rust-allocated (stack based) `LayoutMarginBottomValue` struct
#[repr(C, u8)]
pub enum AzLayoutMarginBottomValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutMarginBottom),
}

/// Re-export of rust-allocated (stack based) `LayoutMarginLeftValue` struct
#[repr(C, u8)]
pub enum AzLayoutMarginLeftValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutMarginLeft),
}

/// Re-export of rust-allocated (stack based) `LayoutMarginRightValue` struct
#[repr(C, u8)]
pub enum AzLayoutMarginRightValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutMarginRight),
}

/// Re-export of rust-allocated (stack based) `LayoutMarginTopValue` struct
#[repr(C, u8)]
pub enum AzLayoutMarginTopValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutMarginTop),
}

/// Re-export of rust-allocated (stack based) `LayoutMaxHeightValue` struct
#[repr(C, u8)]
pub enum AzLayoutMaxHeightValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutMaxHeight),
}

/// Re-export of rust-allocated (stack based) `LayoutMaxWidthValue` struct
#[repr(C, u8)]
pub enum AzLayoutMaxWidthValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutMaxWidth),
}

/// Re-export of rust-allocated (stack based) `LayoutMinHeightValue` struct
#[repr(C, u8)]
pub enum AzLayoutMinHeightValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutMinHeight),
}

/// Re-export of rust-allocated (stack based) `LayoutMinWidthValue` struct
#[repr(C, u8)]
pub enum AzLayoutMinWidthValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutMinWidth),
}

/// Re-export of rust-allocated (stack based) `LayoutPaddingBottomValue` struct
#[repr(C, u8)]
pub enum AzLayoutPaddingBottomValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutPaddingBottom),
}

/// Re-export of rust-allocated (stack based) `LayoutPaddingLeftValue` struct
#[repr(C, u8)]
pub enum AzLayoutPaddingLeftValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutPaddingLeft),
}

/// Re-export of rust-allocated (stack based) `LayoutPaddingRightValue` struct
#[repr(C, u8)]
pub enum AzLayoutPaddingRightValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutPaddingRight),
}

/// Re-export of rust-allocated (stack based) `LayoutPaddingTopValue` struct
#[repr(C, u8)]
pub enum AzLayoutPaddingTopValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutPaddingTop),
}

/// Re-export of rust-allocated (stack based) `LayoutPositionValue` struct
#[repr(C, u8)]
pub enum AzLayoutPositionValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutPosition),
}

/// Re-export of rust-allocated (stack based) `LayoutRightValue` struct
#[repr(C, u8)]
pub enum AzLayoutRightValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutRight),
}

/// Re-export of rust-allocated (stack based) `LayoutTopValue` struct
#[repr(C, u8)]
pub enum AzLayoutTopValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutTop),
}

/// Re-export of rust-allocated (stack based) `LayoutWidthValue` struct
#[repr(C, u8)]
pub enum AzLayoutWidthValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutWidth),
}

/// Re-export of rust-allocated (stack based) `LayoutFlexWrapValue` struct
#[repr(C, u8)]
pub enum AzLayoutFlexWrapValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutFlexWrap),
}

/// Re-export of rust-allocated (stack based) `LayoutOverflowValue` struct
#[repr(C, u8)]
pub enum AzLayoutOverflowValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutOverflow),
}

/// Re-export of rust-allocated (stack based) `StyleBorderBottomColorValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderBottomColorValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderBottomColor),
}

/// Re-export of rust-allocated (stack based) `StyleBorderBottomLeftRadiusValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderBottomLeftRadiusValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderBottomLeftRadius),
}

/// Re-export of rust-allocated (stack based) `StyleBorderBottomRightRadiusValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderBottomRightRadiusValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderBottomRightRadius),
}

/// Re-export of rust-allocated (stack based) `StyleBorderBottomStyleValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderBottomStyleValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderBottomStyle),
}

/// Re-export of rust-allocated (stack based) `LayoutBorderBottomWidthValue` struct
#[repr(C, u8)]
pub enum AzLayoutBorderBottomWidthValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutBorderBottomWidth),
}

/// Re-export of rust-allocated (stack based) `StyleBorderLeftColorValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderLeftColorValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderLeftColor),
}

/// Re-export of rust-allocated (stack based) `StyleBorderLeftStyleValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderLeftStyleValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderLeftStyle),
}

/// Re-export of rust-allocated (stack based) `LayoutBorderLeftWidthValue` struct
#[repr(C, u8)]
pub enum AzLayoutBorderLeftWidthValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutBorderLeftWidth),
}

/// Re-export of rust-allocated (stack based) `StyleBorderRightColorValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderRightColorValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderRightColor),
}

/// Re-export of rust-allocated (stack based) `StyleBorderRightStyleValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderRightStyleValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderRightStyle),
}

/// Re-export of rust-allocated (stack based) `LayoutBorderRightWidthValue` struct
#[repr(C, u8)]
pub enum AzLayoutBorderRightWidthValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutBorderRightWidth),
}

/// Re-export of rust-allocated (stack based) `StyleBorderTopColorValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderTopColorValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderTopColor),
}

/// Re-export of rust-allocated (stack based) `StyleBorderTopLeftRadiusValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderTopLeftRadiusValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderTopLeftRadius),
}

/// Re-export of rust-allocated (stack based) `StyleBorderTopRightRadiusValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderTopRightRadiusValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderTopRightRadius),
}

/// Re-export of rust-allocated (stack based) `StyleBorderTopStyleValue` struct
#[repr(C, u8)]
pub enum AzStyleBorderTopStyleValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBorderTopStyle),
}

/// Re-export of rust-allocated (stack based) `LayoutBorderTopWidthValue` struct
#[repr(C, u8)]
pub enum AzLayoutBorderTopWidthValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzLayoutBorderTopWidth),
}

/// Re-export of rust-allocated (stack based) `StyleCursorValue` struct
#[repr(C, u8)]
pub enum AzStyleCursorValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleCursor),
}

/// Re-export of rust-allocated (stack based) `StyleFontSizeValue` struct
#[repr(C, u8)]
pub enum AzStyleFontSizeValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleFontSize),
}

/// Re-export of rust-allocated (stack based) `StyleLetterSpacingValue` struct
#[repr(C, u8)]
pub enum AzStyleLetterSpacingValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleLetterSpacing),
}

/// Re-export of rust-allocated (stack based) `StyleLineHeightValue` struct
#[repr(C, u8)]
pub enum AzStyleLineHeightValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleLineHeight),
}

/// Re-export of rust-allocated (stack based) `StyleTabWidthValue` struct
#[repr(C, u8)]
pub enum AzStyleTabWidthValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleTabWidth),
}

/// Re-export of rust-allocated (stack based) `StyleTextAlignValue` struct
#[repr(C, u8)]
pub enum AzStyleTextAlignValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleTextAlign),
}

/// Re-export of rust-allocated (stack based) `StyleTextColorValue` struct
#[repr(C, u8)]
pub enum AzStyleTextColorValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleTextColor),
}

/// Re-export of rust-allocated (stack based) `StyleWordSpacingValue` struct
#[repr(C, u8)]
pub enum AzStyleWordSpacingValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleWordSpacing),
}

/// Re-export of rust-allocated (stack based) `StyleOpacityValue` struct
#[repr(C, u8)]
pub enum AzStyleOpacityValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleOpacity),
}

/// Re-export of rust-allocated (stack based) `StyleTransformOriginValue` struct
#[repr(C, u8)]
pub enum AzStyleTransformOriginValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleTransformOrigin),
}

/// Re-export of rust-allocated (stack based) `StylePerspectiveOriginValue` struct
#[repr(C, u8)]
pub enum AzStylePerspectiveOriginValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStylePerspectiveOrigin),
}

/// Re-export of rust-allocated (stack based) `StyleBackfaceVisibilityValue` struct
#[repr(C, u8)]
pub enum AzStyleBackfaceVisibilityValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBackfaceVisibility),
}

/// Re-export of rust-allocated (stack based) `ParentWithNodeDepth` struct
#[repr(C)]
#[pyclass(name = "ParentWithNodeDepth")]
pub struct AzParentWithNodeDepth {
    #[pyo3(get, set)]
    pub depth: usize,
    #[pyo3(get, set)]
    pub node_id: AzNodeId,
}

/// Re-export of rust-allocated (stack based) `Gl` struct
#[repr(C)]
#[pyclass(name = "Gl")]
pub struct AzGl {
    pub ptr: *const c_void,
    #[pyo3(get, set)]
    pub renderer_type: AzRendererTypeEnumWrapper,
}

/// C-ABI stable reexport of `&[Refstr]` aka `&mut [&str]`
#[repr(C)]
#[pyclass(name = "RefstrVecRef")]
pub struct AzRefstrVecRef {
    pub(crate) ptr: *const AzRefstr,
    #[pyo3(get, set)]
    pub len: usize,
}

/// Re-export of rust-allocated (stack based) `ImageMask` struct
#[repr(C)]
#[pyclass(name = "ImageMask")]
pub struct AzImageMask {
    #[pyo3(get, set)]
    pub image: AzImageRef,
    #[pyo3(get, set)]
    pub rect: AzLogicalRect,
    #[pyo3(get, set)]
    pub repeat: bool,
}

/// Re-export of rust-allocated (stack based) `FontMetrics` struct
#[repr(C)]
#[pyclass(name = "FontMetrics")]
pub struct AzFontMetrics {
    #[pyo3(get, set)]
    pub units_per_em: u16,
    #[pyo3(get, set)]
    pub font_flags: u16,
    #[pyo3(get, set)]
    pub x_min: i16,
    #[pyo3(get, set)]
    pub y_min: i16,
    #[pyo3(get, set)]
    pub x_max: i16,
    #[pyo3(get, set)]
    pub y_max: i16,
    #[pyo3(get, set)]
    pub ascender: i16,
    #[pyo3(get, set)]
    pub descender: i16,
    #[pyo3(get, set)]
    pub line_gap: i16,
    #[pyo3(get, set)]
    pub advance_width_max: u16,
    #[pyo3(get, set)]
    pub min_left_side_bearing: i16,
    #[pyo3(get, set)]
    pub min_right_side_bearing: i16,
    #[pyo3(get, set)]
    pub x_max_extent: i16,
    #[pyo3(get, set)]
    pub caret_slope_rise: i16,
    #[pyo3(get, set)]
    pub caret_slope_run: i16,
    #[pyo3(get, set)]
    pub caret_offset: i16,
    #[pyo3(get, set)]
    pub num_h_metrics: u16,
    #[pyo3(get, set)]
    pub x_avg_char_width: i16,
    #[pyo3(get, set)]
    pub us_weight_class: u16,
    #[pyo3(get, set)]
    pub us_width_class: u16,
    #[pyo3(get, set)]
    pub fs_type: u16,
    #[pyo3(get, set)]
    pub y_subscript_x_size: i16,
    #[pyo3(get, set)]
    pub y_subscript_y_size: i16,
    #[pyo3(get, set)]
    pub y_subscript_x_offset: i16,
    #[pyo3(get, set)]
    pub y_subscript_y_offset: i16,
    #[pyo3(get, set)]
    pub y_superscript_x_size: i16,
    #[pyo3(get, set)]
    pub y_superscript_y_size: i16,
    #[pyo3(get, set)]
    pub y_superscript_x_offset: i16,
    #[pyo3(get, set)]
    pub y_superscript_y_offset: i16,
    #[pyo3(get, set)]
    pub y_strikeout_size: i16,
    #[pyo3(get, set)]
    pub y_strikeout_position: i16,
    #[pyo3(get, set)]
    pub s_family_class: i16,
    pub panose: [u8; 10],
    #[pyo3(get, set)]
    pub ul_unicode_range1: u32,
    #[pyo3(get, set)]
    pub ul_unicode_range2: u32,
    #[pyo3(get, set)]
    pub ul_unicode_range3: u32,
    #[pyo3(get, set)]
    pub ul_unicode_range4: u32,
    #[pyo3(get, set)]
    pub ach_vend_id: u32,
    #[pyo3(get, set)]
    pub fs_selection: u16,
    #[pyo3(get, set)]
    pub us_first_char_index: u16,
    #[pyo3(get, set)]
    pub us_last_char_index: u16,
    #[pyo3(get, set)]
    pub s_typo_ascender: AzOptionI16EnumWrapper,
    #[pyo3(get, set)]
    pub s_typo_descender: AzOptionI16EnumWrapper,
    #[pyo3(get, set)]
    pub s_typo_line_gap: AzOptionI16EnumWrapper,
    #[pyo3(get, set)]
    pub us_win_ascent: AzOptionU16EnumWrapper,
    #[pyo3(get, set)]
    pub us_win_descent: AzOptionU16EnumWrapper,
    #[pyo3(get, set)]
    pub ul_code_page_range1: AzOptionU32EnumWrapper,
    #[pyo3(get, set)]
    pub ul_code_page_range2: AzOptionU32EnumWrapper,
    #[pyo3(get, set)]
    pub sx_height: AzOptionI16EnumWrapper,
    #[pyo3(get, set)]
    pub s_cap_height: AzOptionI16EnumWrapper,
    #[pyo3(get, set)]
    pub us_default_char: AzOptionU16EnumWrapper,
    #[pyo3(get, set)]
    pub us_break_char: AzOptionU16EnumWrapper,
    #[pyo3(get, set)]
    pub us_max_context: AzOptionU16EnumWrapper,
    #[pyo3(get, set)]
    pub us_lower_optical_point_size: AzOptionU16EnumWrapper,
    #[pyo3(get, set)]
    pub us_upper_optical_point_size: AzOptionU16EnumWrapper,
}

/// Re-export of rust-allocated (stack based) `SvgLine` struct
#[repr(C)]
#[pyclass(name = "SvgLine")]
pub struct AzSvgLine {
    #[pyo3(get, set)]
    pub start: AzSvgPoint,
    #[pyo3(get, set)]
    pub end: AzSvgPoint,
}

/// Re-export of rust-allocated (stack based) `SvgQuadraticCurve` struct
#[repr(C)]
#[pyclass(name = "SvgQuadraticCurve")]
pub struct AzSvgQuadraticCurve {
    #[pyo3(get, set)]
    pub start: AzSvgPoint,
    #[pyo3(get, set)]
    pub ctrl: AzSvgPoint,
    #[pyo3(get, set)]
    pub end: AzSvgPoint,
}

/// Re-export of rust-allocated (stack based) `SvgCubicCurve` struct
#[repr(C)]
#[pyclass(name = "SvgCubicCurve")]
pub struct AzSvgCubicCurve {
    #[pyo3(get, set)]
    pub start: AzSvgPoint,
    #[pyo3(get, set)]
    pub ctrl_1: AzSvgPoint,
    #[pyo3(get, set)]
    pub ctrl_2: AzSvgPoint,
    #[pyo3(get, set)]
    pub end: AzSvgPoint,
}

/// Re-export of rust-allocated (stack based) `SvgStringFormatOptions` struct
#[repr(C)]
#[pyclass(name = "SvgStringFormatOptions")]
pub struct AzSvgStringFormatOptions {
    #[pyo3(get, set)]
    pub use_single_quote: bool,
    #[pyo3(get, set)]
    pub indent: AzIndentEnumWrapper,
    #[pyo3(get, set)]
    pub attributes_indent: AzIndentEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `SvgFillStyle` struct
#[repr(C)]
#[pyclass(name = "SvgFillStyle")]
pub struct AzSvgFillStyle {
    #[pyo3(get, set)]
    pub line_join: AzSvgLineJoinEnumWrapper,
    #[pyo3(get, set)]
    pub miter_limit: f32,
    #[pyo3(get, set)]
    pub tolerance: f32,
    #[pyo3(get, set)]
    pub fill_rule: AzSvgFillRuleEnumWrapper,
    #[pyo3(get, set)]
    pub transform: AzSvgTransform,
    #[pyo3(get, set)]
    pub anti_alias: bool,
    #[pyo3(get, set)]
    pub high_quality_aa: bool,
}

/// Re-export of rust-allocated (stack based) `InstantPtr` struct
#[repr(C)]
#[pyclass(name = "InstantPtr")]
pub struct AzInstantPtr {
    pub ptr: *const c_void,
    #[pyo3(get, set)]
    pub clone_fn: AzInstantPtrCloneFn,
    #[pyo3(get, set)]
    pub destructor: AzInstantPtrDestructorFn,
}

/// Re-export of rust-allocated (stack based) `Duration` struct
#[repr(C, u8)]
pub enum AzDuration {
    System(AzSystemTimeDiff),
    Tick(AzSystemTickDiff),
}

/// Re-export of rust-allocated (stack based) `ThreadSendMsg` struct
#[repr(C, u8)]
pub enum AzThreadSendMsg {
    TerminateThread,
    Tick,
    Custom(AzRefAny),
}

/// Re-export of rust-allocated (stack based) `ThreadWriteBackMsg` struct
#[repr(C)]
#[pyclass(name = "ThreadWriteBackMsg")]
pub struct AzThreadWriteBackMsg {
    #[pyo3(get, set)]
    pub data: AzRefAny,
    #[pyo3(get, set)]
    pub callback: AzWriteBackCallback,
}

/// Wrapper over a Rust-allocated `Vec<XmlNode>`
#[repr(C)]
#[pyclass(name = "XmlNodeVec")]
pub struct AzXmlNodeVec {
    pub(crate) ptr: *const AzXmlNode,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzXmlNodeVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<InlineGlyph>`
#[repr(C)]
#[pyclass(name = "InlineGlyphVec")]
pub struct AzInlineGlyphVec {
    pub(crate) ptr: *const AzInlineGlyph,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzInlineGlyphVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<InlineTextHit>`
#[repr(C)]
#[pyclass(name = "InlineTextHitVec")]
pub struct AzInlineTextHitVec {
    pub(crate) ptr: *const AzInlineTextHit,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzInlineTextHitVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<VideoMode>`
#[repr(C)]
#[pyclass(name = "VideoModeVec")]
pub struct AzVideoModeVec {
    pub(crate) ptr: *const AzVideoMode,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzVideoModeVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<Dom>`
#[repr(C)]
#[pyclass(name = "DomVec")]
pub struct AzDomVec {
    pub(crate) ptr: *const AzDom,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzDomVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<StyleBackgroundPosition>`
#[repr(C)]
#[pyclass(name = "StyleBackgroundPositionVec")]
pub struct AzStyleBackgroundPositionVec {
    pub(crate) ptr: *const AzStyleBackgroundPosition,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzStyleBackgroundPositionVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<StyleBackgroundRepeat>`
#[repr(C)]
#[pyclass(name = "StyleBackgroundRepeatVec")]
pub struct AzStyleBackgroundRepeatVec {
    pub(crate) ptr: *const AzStyleBackgroundRepeatEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzStyleBackgroundRepeatVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<StyleBackgroundSize>`
#[repr(C)]
#[pyclass(name = "StyleBackgroundSizeVec")]
pub struct AzStyleBackgroundSizeVec {
    pub(crate) ptr: *const AzStyleBackgroundSizeEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzStyleBackgroundSizeVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `SvgVertex`
#[repr(C)]
#[pyclass(name = "SvgVertexVec")]
pub struct AzSvgVertexVec {
    pub(crate) ptr: *const AzSvgVertex,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzSvgVertexVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<u32>`
#[repr(C)]
#[pyclass(name = "U32Vec")]
pub struct AzU32Vec {
    pub ptr: *const u32,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzU32VecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `XWindowType`
#[repr(C)]
#[pyclass(name = "XWindowTypeVec")]
pub struct AzXWindowTypeVec {
    pub(crate) ptr: *const AzXWindowTypeEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzXWindowTypeVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `VirtualKeyCode`
#[repr(C)]
#[pyclass(name = "VirtualKeyCodeVec")]
pub struct AzVirtualKeyCodeVec {
    pub(crate) ptr: *const AzVirtualKeyCodeEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzVirtualKeyCodeVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `CascadeInfo`
#[repr(C)]
#[pyclass(name = "CascadeInfoVec")]
pub struct AzCascadeInfoVec {
    pub(crate) ptr: *const AzCascadeInfo,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzCascadeInfoVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `ScanCode`
#[repr(C)]
#[pyclass(name = "ScanCodeVec")]
pub struct AzScanCodeVec {
    pub ptr: *const u32,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzScanCodeVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<u16>`
#[repr(C)]
#[pyclass(name = "U16Vec")]
pub struct AzU16Vec {
    pub ptr: *const u16,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzU16VecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<f32>`
#[repr(C)]
#[pyclass(name = "F32Vec")]
pub struct AzF32Vec {
    pub ptr: *const f32,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzF32VecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `U8Vec`
#[repr(C)]
#[pyclass(name = "U8Vec")]
pub struct AzU8Vec {
    pub ptr: *const u8,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzU8VecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `U32Vec`
#[repr(C)]
#[pyclass(name = "GLuintVec")]
pub struct AzGLuintVec {
    pub ptr: *const u32,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzGLuintVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `GLintVec`
#[repr(C)]
#[pyclass(name = "GLintVec")]
pub struct AzGLintVec {
    pub ptr: *const i32,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzGLintVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `NormalizedLinearColorStopVec`
#[repr(C)]
#[pyclass(name = "NormalizedLinearColorStopVec")]
pub struct AzNormalizedLinearColorStopVec {
    pub(crate) ptr: *const AzNormalizedLinearColorStop,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzNormalizedLinearColorStopVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `NormalizedRadialColorStopVec`
#[repr(C)]
#[pyclass(name = "NormalizedRadialColorStopVec")]
pub struct AzNormalizedRadialColorStopVec {
    pub(crate) ptr: *const AzNormalizedRadialColorStop,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzNormalizedRadialColorStopVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `NodeIdVec`
#[repr(C)]
#[pyclass(name = "NodeIdVec")]
pub struct AzNodeIdVec {
    pub(crate) ptr: *const AzNodeId,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzNodeIdVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `NodeVec`
#[repr(C)]
#[pyclass(name = "NodeVec")]
pub struct AzNodeVec {
    pub(crate) ptr: *const AzNode,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzNodeVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `ParentWithNodeDepthVec`
#[repr(C)]
#[pyclass(name = "ParentWithNodeDepthVec")]
pub struct AzParentWithNodeDepthVec {
    pub(crate) ptr: *const AzParentWithNodeDepth,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzParentWithNodeDepthVecDestructorEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `OptionPositionInfo` struct
#[repr(C, u8)]
pub enum AzOptionPositionInfo {
    None,
    Some(AzPositionInfo),
}

/// Re-export of rust-allocated (stack based) `OptionTimerId` struct
#[repr(C, u8)]
pub enum AzOptionTimerId {
    None,
    Some(AzTimerId),
}

/// Re-export of rust-allocated (stack based) `OptionThreadId` struct
#[repr(C, u8)]
pub enum AzOptionThreadId {
    None,
    Some(AzThreadId),
}

/// Re-export of rust-allocated (stack based) `OptionImageRef` struct
#[repr(C, u8)]
pub enum AzOptionImageRef {
    None,
    Some(AzImageRef),
}

/// Re-export of rust-allocated (stack based) `OptionFontRef` struct
#[repr(C, u8)]
pub enum AzOptionFontRef {
    None,
    Some(AzFontRef),
}

/// Re-export of rust-allocated (stack based) `OptionSystemClipboard` struct
#[repr(C, u8)]
pub enum AzOptionSystemClipboard {
    None,
    Some(AzSystemClipboard),
}

/// Re-export of rust-allocated (stack based) `OptionFile` struct
#[repr(C, u8)]
pub enum AzOptionFile {
    None,
    Some(AzFile),
}

/// Re-export of rust-allocated (stack based) `OptionGl` struct
#[repr(C, u8)]
pub enum AzOptionGl {
    None,
    Some(AzGl),
}

/// Re-export of rust-allocated (stack based) `OptionPercentageValue` struct
#[repr(C, u8)]
pub enum AzOptionPercentageValue {
    None,
    Some(AzPercentageValue),
}

/// Re-export of rust-allocated (stack based) `OptionAngleValue` struct
#[repr(C, u8)]
pub enum AzOptionAngleValue {
    None,
    Some(AzAngleValue),
}

/// Re-export of rust-allocated (stack based) `OptionRendererOptions` struct
#[repr(C, u8)]
pub enum AzOptionRendererOptions {
    None,
    Some(AzRendererOptions),
}

/// Re-export of rust-allocated (stack based) `OptionCallback` struct
#[repr(C, u8)]
pub enum AzOptionCallback {
    None,
    Some(AzCallback),
}

/// Re-export of rust-allocated (stack based) `OptionThreadSendMsg` struct
#[repr(C, u8)]
pub enum AzOptionThreadSendMsg {
    None,
    Some(AzThreadSendMsg),
}

/// Re-export of rust-allocated (stack based) `OptionLayoutRect` struct
#[repr(C, u8)]
pub enum AzOptionLayoutRect {
    None,
    Some(AzLayoutRect),
}

/// Re-export of rust-allocated (stack based) `OptionRefAny` struct
#[repr(C, u8)]
pub enum AzOptionRefAny {
    None,
    Some(AzRefAny),
}

/// Re-export of rust-allocated (stack based) `OptionLayoutPoint` struct
#[repr(C, u8)]
pub enum AzOptionLayoutPoint {
    None,
    Some(AzLayoutPoint),
}

/// Re-export of rust-allocated (stack based) `OptionLayoutSize` struct
#[repr(C, u8)]
pub enum AzOptionLayoutSize {
    None,
    Some(AzLayoutSize),
}

/// Re-export of rust-allocated (stack based) `OptionWindowTheme` struct
#[repr(C, u8)]
pub enum AzOptionWindowTheme {
    None,
    Some(AzWindowTheme),
}

/// Re-export of rust-allocated (stack based) `OptionNodeId` struct
#[repr(C, u8)]
pub enum AzOptionNodeId {
    None,
    Some(AzNodeId),
}

/// Re-export of rust-allocated (stack based) `OptionDomNodeId` struct
#[repr(C, u8)]
pub enum AzOptionDomNodeId {
    None,
    Some(AzDomNodeId),
}

/// Re-export of rust-allocated (stack based) `OptionColorU` struct
#[repr(C, u8)]
pub enum AzOptionColorU {
    None,
    Some(AzColorU),
}

/// Re-export of rust-allocated (stack based) `OptionSvgDashPattern` struct
#[repr(C, u8)]
pub enum AzOptionSvgDashPattern {
    None,
    Some(AzSvgDashPattern),
}

/// Re-export of rust-allocated (stack based) `OptionLogicalPosition` struct
#[repr(C, u8)]
pub enum AzOptionLogicalPosition {
    None,
    Some(AzLogicalPosition),
}

/// Re-export of rust-allocated (stack based) `OptionPhysicalPositionI32` struct
#[repr(C, u8)]
pub enum AzOptionPhysicalPositionI32 {
    None,
    Some(AzPhysicalPositionI32),
}

/// Re-export of rust-allocated (stack based) `OptionMouseCursorType` struct
#[repr(C, u8)]
pub enum AzOptionMouseCursorType {
    None,
    Some(AzMouseCursorType),
}

/// Re-export of rust-allocated (stack based) `OptionLogicalSize` struct
#[repr(C, u8)]
pub enum AzOptionLogicalSize {
    None,
    Some(AzLogicalSize),
}

/// Re-export of rust-allocated (stack based) `OptionVirtualKeyCode` struct
#[repr(C, u8)]
pub enum AzOptionVirtualKeyCode {
    None,
    Some(AzVirtualKeyCode),
}

/// Re-export of rust-allocated (stack based) `OptionImageMask` struct
#[repr(C, u8)]
pub enum AzOptionImageMask {
    None,
    Some(AzImageMask),
}

/// Re-export of rust-allocated (stack based) `OptionTabIndex` struct
#[repr(C, u8)]
pub enum AzOptionTabIndex {
    None,
    Some(AzTabIndex),
}

/// Re-export of rust-allocated (stack based) `OptionTagId` struct
#[repr(C, u8)]
pub enum AzOptionTagId {
    None,
    Some(AzTagId),
}

/// Re-export of rust-allocated (stack based) `OptionDuration` struct
#[repr(C, u8)]
pub enum AzOptionDuration {
    None,
    Some(AzDuration),
}

/// Re-export of rust-allocated (stack based) `OptionU8Vec` struct
#[repr(C, u8)]
pub enum AzOptionU8Vec {
    None,
    Some(AzU8Vec),
}

/// Re-export of rust-allocated (stack based) `OptionU8VecRef` struct
#[repr(C, u8)]
pub enum AzOptionU8VecRef {
    None,
    Some(AzU8VecRef),
}

/// Re-export of rust-allocated (stack based) `ResultU8VecEncodeImageError` struct
#[repr(C, u8)]
pub enum AzResultU8VecEncodeImageError {
    Ok(AzU8Vec),
    Err(AzEncodeImageError),
}

/// Re-export of rust-allocated (stack based) `NonXmlCharError` struct
#[repr(C)]
#[pyclass(name = "NonXmlCharError")]
pub struct AzNonXmlCharError {
    #[pyo3(get, set)]
    pub ch: u32,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Re-export of rust-allocated (stack based) `InvalidCharError` struct
#[repr(C)]
#[pyclass(name = "InvalidCharError")]
pub struct AzInvalidCharError {
    #[pyo3(get, set)]
    pub expected: u8,
    #[pyo3(get, set)]
    pub got: u8,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Re-export of rust-allocated (stack based) `InvalidCharMultipleError` struct
#[repr(C)]
#[pyclass(name = "InvalidCharMultipleError")]
pub struct AzInvalidCharMultipleError {
    #[pyo3(get, set)]
    pub expected: u8,
    #[pyo3(get, set)]
    pub got: AzU8Vec,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Re-export of rust-allocated (stack based) `InvalidQuoteError` struct
#[repr(C)]
#[pyclass(name = "InvalidQuoteError")]
pub struct AzInvalidQuoteError {
    #[pyo3(get, set)]
    pub got: u8,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Re-export of rust-allocated (stack based) `InvalidSpaceError` struct
#[repr(C)]
#[pyclass(name = "InvalidSpaceError")]
pub struct AzInvalidSpaceError {
    #[pyo3(get, set)]
    pub got: u8,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Configuration for optional features, such as whether to enable logging or panic hooks
#[repr(C)]
#[pyclass(name = "AppConfig")]
pub struct AzAppConfig {
    #[pyo3(get, set)]
    pub layout_solver: AzLayoutSolverEnumWrapper,
    #[pyo3(get, set)]
    pub log_level: AzAppLogLevelEnumWrapper,
    #[pyo3(get, set)]
    pub enable_visual_panic_hook: bool,
    #[pyo3(get, set)]
    pub enable_logging_on_panic: bool,
    #[pyo3(get, set)]
    pub enable_tab_navigation: bool,
    #[pyo3(get, set)]
    pub system_callbacks: AzSystemCallbacks,
}

/// Small (16x16x4) window icon, usually shown in the window titlebar
#[repr(C)]
#[pyclass(name = "SmallWindowIconBytes")]
pub struct AzSmallWindowIconBytes {
    #[pyo3(get, set)]
    pub key: AzIconKey,
    #[pyo3(get, set)]
    pub rgba_bytes: AzU8Vec,
}

/// Large (32x32x4) window icon, usually used on high-resolution displays (instead of `SmallWindowIcon`)
#[repr(C)]
#[pyclass(name = "LargeWindowIconBytes")]
pub struct AzLargeWindowIconBytes {
    #[pyo3(get, set)]
    pub key: AzIconKey,
    #[pyo3(get, set)]
    pub rgba_bytes: AzU8Vec,
}

/// Window "favicon", usually shown in the top left of the window on Windows
#[repr(C, u8)]
pub enum AzWindowIcon {
    Small(AzSmallWindowIconBytes),
    Large(AzLargeWindowIconBytes),
}

/// Application taskbar icon, 256x256x4 bytes in size
#[repr(C)]
#[pyclass(name = "TaskBarIcon")]
pub struct AzTaskBarIcon {
    #[pyo3(get, set)]
    pub key: AzIconKey,
    #[pyo3(get, set)]
    pub rgba_bytes: AzU8Vec,
}

/// Minimum / maximum / current size of the window in logical dimensions
#[repr(C)]
#[pyclass(name = "WindowSize")]
pub struct AzWindowSize {
    #[pyo3(get, set)]
    pub dimensions: AzLogicalSize,
    #[pyo3(get, set)]
    pub hidpi_factor: f32,
    #[pyo3(get, set)]
    pub system_hidpi_factor: f32,
    #[pyo3(get, set)]
    pub min_dimensions: AzOptionLogicalSizeEnumWrapper,
    #[pyo3(get, set)]
    pub max_dimensions: AzOptionLogicalSizeEnumWrapper,
}

/// Current keyboard state, stores what keys / characters have been pressed
#[repr(C)]
#[pyclass(name = "KeyboardState")]
pub struct AzKeyboardState {
    #[pyo3(get, set)]
    pub shift_down: bool,
    #[pyo3(get, set)]
    pub ctrl_down: bool,
    #[pyo3(get, set)]
    pub alt_down: bool,
    #[pyo3(get, set)]
    pub super_down: bool,
    #[pyo3(get, set)]
    pub current_char: AzOptionCharEnumWrapper,
    #[pyo3(get, set)]
    pub current_virtual_keycode: AzOptionVirtualKeyCodeEnumWrapper,
    #[pyo3(get, set)]
    pub pressed_virtual_keycodes: AzVirtualKeyCodeVec,
    #[pyo3(get, set)]
    pub pressed_scancodes: AzScanCodeVec,
}

/// Current mouse / cursor state
#[repr(C)]
#[pyclass(name = "MouseState")]
pub struct AzMouseState {
    #[pyo3(get, set)]
    pub mouse_cursor_type: AzOptionMouseCursorTypeEnumWrapper,
    #[pyo3(get, set)]
    pub cursor_position: AzCursorPositionEnumWrapper,
    #[pyo3(get, set)]
    pub is_cursor_locked: bool,
    #[pyo3(get, set)]
    pub left_down: bool,
    #[pyo3(get, set)]
    pub right_down: bool,
    #[pyo3(get, set)]
    pub middle_down: bool,
    #[pyo3(get, set)]
    pub scroll_x: AzOptionF32EnumWrapper,
    #[pyo3(get, set)]
    pub scroll_y: AzOptionF32EnumWrapper,
}

/// C-ABI stable wrapper over a `MarshaledLayoutCallback`
#[repr(C)]
#[pyclass(name = "MarshaledLayoutCallback")]
pub struct AzMarshaledLayoutCallback {
    #[pyo3(get, set)]
    pub marshal_data: AzRefAny,
    pub cb: AzMarshaledLayoutCallbackInner,
}

/// Re-export of rust-allocated (stack based) `InlineTextContents` struct
#[repr(C)]
#[pyclass(name = "InlineTextContents")]
pub struct AzInlineTextContents {
    #[pyo3(get, set)]
    pub glyphs: AzInlineGlyphVec,
    #[pyo3(get, set)]
    pub bounds: AzLogicalRect,
}

/// Easing function of the animation (ease-in, ease-out, ease-in-out, custom)
#[repr(C, u8)]
pub enum AzAnimationEasing {
    Ease,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(AzSvgCubicCurve),
}

/// Re-export of rust-allocated (stack based) `RenderImageCallbackInfo` struct
#[repr(C)]
#[pyclass(name = "RenderImageCallbackInfo")]
pub struct AzRenderImageCallbackInfo {
    #[pyo3(get, set)]
    pub callback_node_id: AzDomNodeId,
    #[pyo3(get, set)]
    pub bounds: AzHidpiAdjustedBounds,
    pub gl_context: *const AzOptionGlEnumWrapper,
    pub image_cache: *const c_void,
    pub system_fonts: *const c_void,
    pub node_hierarchy: *const AzNodeVec,
    pub words_cache: *const c_void,
    pub shaped_words_cache: *const c_void,
    pub positioned_words_cache: *const c_void,
    pub positioned_rects: *const c_void,
    pub _reserved_ref: *const c_void,
    pub _reserved_mut: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `LayoutCallbackInfo` struct
#[repr(C)]
#[pyclass(name = "LayoutCallbackInfo")]
pub struct AzLayoutCallbackInfo {
    #[pyo3(get, set)]
    pub window_size: AzWindowSize,
    #[pyo3(get, set)]
    pub theme: AzWindowThemeEnumWrapper,
    pub image_cache: *const c_void,
    pub gl_context: *const AzOptionGlEnumWrapper,
    pub system_fonts: *const c_void,
    pub _reserved_ref: *const c_void,
    pub _reserved_mut: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `EventFilter` struct
#[repr(C, u8)]
pub enum AzEventFilter {
    Hover(AzHoverEventFilter),
    Not(AzNotEventFilter),
    Focus(AzFocusEventFilter),
    Window(AzWindowEventFilter),
    Component(AzComponentEventFilter),
    Application(AzApplicationEventFilter),
}

/// Re-export of rust-allocated (stack based) `CssPathPseudoSelector` struct
#[repr(C, u8)]
pub enum AzCssPathPseudoSelector {
    First,
    Last,
    NthChild(AzCssNthChildSelector),
    Hover,
    Active,
    Focus,
}

/// Re-export of rust-allocated (stack based) `AnimationInterpolationFunction` struct
#[repr(C, u8)]
pub enum AzAnimationInterpolationFunction {
    Ease,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(AzSvgCubicCurve),
}

/// Re-export of rust-allocated (stack based) `InterpolateContext` struct
#[repr(C)]
#[pyclass(name = "InterpolateContext")]
pub struct AzInterpolateContext {
    #[pyo3(get, set)]
    pub animation_func: AzAnimationInterpolationFunctionEnumWrapper,
    #[pyo3(get, set)]
    pub parent_rect_width: f32,
    #[pyo3(get, set)]
    pub parent_rect_height: f32,
    #[pyo3(get, set)]
    pub current_rect_width: f32,
    #[pyo3(get, set)]
    pub current_rect_height: f32,
}

/// Re-export of rust-allocated (stack based) `LinearGradient` struct
#[repr(C)]
#[pyclass(name = "LinearGradient")]
pub struct AzLinearGradient {
    #[pyo3(get, set)]
    pub direction: AzDirectionEnumWrapper,
    #[pyo3(get, set)]
    pub extend_mode: AzExtendModeEnumWrapper,
    #[pyo3(get, set)]
    pub stops: AzNormalizedLinearColorStopVec,
}

/// Re-export of rust-allocated (stack based) `RadialGradient` struct
#[repr(C)]
#[pyclass(name = "RadialGradient")]
pub struct AzRadialGradient {
    #[pyo3(get, set)]
    pub shape: AzShapeEnumWrapper,
    #[pyo3(get, set)]
    pub size: AzRadialGradientSizeEnumWrapper,
    #[pyo3(get, set)]
    pub position: AzStyleBackgroundPosition,
    #[pyo3(get, set)]
    pub extend_mode: AzExtendModeEnumWrapper,
    #[pyo3(get, set)]
    pub stops: AzNormalizedLinearColorStopVec,
}

/// Re-export of rust-allocated (stack based) `ConicGradient` struct
#[repr(C)]
#[pyclass(name = "ConicGradient")]
pub struct AzConicGradient {
    #[pyo3(get, set)]
    pub extend_mode: AzExtendModeEnumWrapper,
    #[pyo3(get, set)]
    pub center: AzStyleBackgroundPosition,
    #[pyo3(get, set)]
    pub angle: AzAngleValue,
    #[pyo3(get, set)]
    pub stops: AzNormalizedRadialColorStopVec,
}

/// Re-export of rust-allocated (stack based) `StyleTransform` struct
#[repr(C, u8)]
pub enum AzStyleTransform {
    Matrix(AzStyleTransformMatrix2D),
    Matrix3D(AzStyleTransformMatrix3D),
    Translate(AzStyleTransformTranslate2D),
    Translate3D(AzStyleTransformTranslate3D),
    TranslateX(AzPixelValue),
    TranslateY(AzPixelValue),
    TranslateZ(AzPixelValue),
    Rotate(AzAngleValue),
    Rotate3D(AzStyleTransformRotate3D),
    RotateX(AzAngleValue),
    RotateY(AzAngleValue),
    RotateZ(AzAngleValue),
    Scale(AzStyleTransformScale2D),
    Scale3D(AzStyleTransformScale3D),
    ScaleX(AzPercentageValue),
    ScaleY(AzPercentageValue),
    ScaleZ(AzPercentageValue),
    Skew(AzStyleTransformSkew2D),
    SkewX(AzPercentageValue),
    SkewY(AzPercentageValue),
    Perspective(AzPixelValue),
}

/// Re-export of rust-allocated (stack based) `StyleBackgroundPositionVecValue` struct
#[repr(C, u8)]
pub enum AzStyleBackgroundPositionVecValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBackgroundPositionVec),
}

/// Re-export of rust-allocated (stack based) `StyleBackgroundRepeatVecValue` struct
#[repr(C, u8)]
pub enum AzStyleBackgroundRepeatVecValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBackgroundRepeatVec),
}

/// Re-export of rust-allocated (stack based) `StyleBackgroundSizeVecValue` struct
#[repr(C, u8)]
pub enum AzStyleBackgroundSizeVecValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBackgroundSizeVec),
}

/// Re-export of rust-allocated (stack based) `StyledNode` struct
#[repr(C)]
#[pyclass(name = "StyledNode")]
pub struct AzStyledNode {
    #[pyo3(get, set)]
    pub state: AzStyledNodeState,
    #[pyo3(get, set)]
    pub tag_id: AzOptionTagIdEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `TagIdToNodeIdMapping` struct
#[repr(C)]
#[pyclass(name = "TagIdToNodeIdMapping")]
pub struct AzTagIdToNodeIdMapping {
    #[pyo3(get, set)]
    pub tag_id: AzTagId,
    #[pyo3(get, set)]
    pub node_id: AzNodeId,
    #[pyo3(get, set)]
    pub tab_index: AzOptionTabIndexEnumWrapper,
    #[pyo3(get, set)]
    pub parents: AzNodeIdVec,
}

/// Re-export of rust-allocated (stack based) `Texture` struct
#[repr(C)]
#[pyclass(name = "Texture")]
pub struct AzTexture {
    #[pyo3(get, set)]
    pub texture_id: u32,
    #[pyo3(get, set)]
    pub format: AzRawImageFormatEnumWrapper,
    #[pyo3(get, set)]
    pub flags: AzTextureFlags,
    #[pyo3(get, set)]
    pub size: AzPhysicalSizeU32,
    #[pyo3(get, set)]
    pub gl_context: AzGl,
}

/// C-ABI stable reexport of `(U8Vec, u32)`
#[repr(C)]
#[pyclass(name = "GetProgramBinaryReturn")]
pub struct AzGetProgramBinaryReturn {
    #[pyo3(get, set)]
    pub _0: AzU8Vec,
    #[pyo3(get, set)]
    pub _1: u32,
}

/// Re-export of rust-allocated (stack based) `RawImageData` struct
#[repr(C, u8)]
pub enum AzRawImageData {
    U8(AzU8Vec),
    U16(AzU16Vec),
    F32(AzF32Vec),
}

/// Source data of a font file (bytes)
#[repr(C)]
#[pyclass(name = "FontSource")]
pub struct AzFontSource {
    #[pyo3(get, set)]
    pub data: AzU8Vec,
    #[pyo3(get, set)]
    pub font_index: u32,
    #[pyo3(get, set)]
    pub parse_glyph_outlines: bool,
}

/// Re-export of rust-allocated (stack based) `SvgPathElement` struct
#[repr(C, u8)]
pub enum AzSvgPathElement {
    Line(AzSvgLine),
    QuadraticCurve(AzSvgQuadraticCurve),
    CubicCurve(AzSvgCubicCurve),
}

/// Re-export of rust-allocated (stack based) `TesselatedSvgNode` struct
#[repr(C)]
#[pyclass(name = "TesselatedSvgNode")]
pub struct AzTesselatedSvgNode {
    #[pyo3(get, set)]
    pub vertices: AzSvgVertexVec,
    #[pyo3(get, set)]
    pub indices: AzU32Vec,
}

/// Rust wrapper over a `&[TesselatedSvgNode]` or `&Vec<TesselatedSvgNode>`
#[repr(C)]
#[pyclass(name = "TesselatedSvgNodeVecRef")]
pub struct AzTesselatedSvgNodeVecRef {
    pub(crate) ptr: *const AzTesselatedSvgNode,
    #[pyo3(get, set)]
    pub len: usize,
}

/// Re-export of rust-allocated (stack based) `SvgRenderOptions` struct
#[repr(C)]
#[pyclass(name = "SvgRenderOptions")]
pub struct AzSvgRenderOptions {
    #[pyo3(get, set)]
    pub target_size: AzOptionLayoutSizeEnumWrapper,
    #[pyo3(get, set)]
    pub background_color: AzOptionColorUEnumWrapper,
    #[pyo3(get, set)]
    pub fit: AzSvgFitToEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `SvgStrokeStyle` struct
#[repr(C)]
#[pyclass(name = "SvgStrokeStyle")]
pub struct AzSvgStrokeStyle {
    #[pyo3(get, set)]
    pub start_cap: AzSvgLineCapEnumWrapper,
    #[pyo3(get, set)]
    pub end_cap: AzSvgLineCapEnumWrapper,
    #[pyo3(get, set)]
    pub line_join: AzSvgLineJoinEnumWrapper,
    #[pyo3(get, set)]
    pub dash_pattern: AzOptionSvgDashPatternEnumWrapper,
    #[pyo3(get, set)]
    pub line_width: f32,
    #[pyo3(get, set)]
    pub miter_limit: f32,
    #[pyo3(get, set)]
    pub tolerance: f32,
    #[pyo3(get, set)]
    pub apply_line_width: bool,
    #[pyo3(get, set)]
    pub transform: AzSvgTransform,
    #[pyo3(get, set)]
    pub anti_alias: bool,
    #[pyo3(get, set)]
    pub high_quality_aa: bool,
}

/// Re-export of rust-allocated (stack based) `Xml` struct
#[repr(C)]
#[pyclass(name = "Xml")]
pub struct AzXml {
    #[pyo3(get, set)]
    pub root: AzXmlNodeVec,
}

/// Re-export of rust-allocated (stack based) `Instant` struct
#[repr(C, u8)]
pub enum AzInstant {
    System(AzInstantPtr),
    Tick(AzSystemTick),
}

/// Re-export of rust-allocated (stack based) `ThreadReceiveMsg` struct
#[repr(C, u8)]
pub enum AzThreadReceiveMsg {
    WriteBack(AzThreadWriteBackMsg),
    Update(AzUpdate),
}

/// Re-export of rust-allocated (stack based) `String` struct
#[repr(C)]
#[pyclass(name = "String")]
pub struct AzString {
    #[pyo3(get, set)]
    pub vec: AzU8Vec,
}

/// Wrapper over a Rust-allocated `Vec<TesselatedSvgNode>`
#[repr(C)]
#[pyclass(name = "TesselatedSvgNodeVec")]
pub struct AzTesselatedSvgNodeVec {
    pub(crate) ptr: *const AzTesselatedSvgNode,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzTesselatedSvgNodeVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<StyleTransform>`
#[repr(C)]
#[pyclass(name = "StyleTransformVec")]
pub struct AzStyleTransformVec {
    pub(crate) ptr: *const AzStyleTransformEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzStyleTransformVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `VertexAttribute`
#[repr(C)]
#[pyclass(name = "SvgPathElementVec")]
pub struct AzSvgPathElementVec {
    pub(crate) ptr: *const AzSvgPathElementEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzSvgPathElementVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `StringVec`
#[repr(C)]
#[pyclass(name = "StringVec")]
pub struct AzStringVec {
    pub(crate) ptr: *const AzString,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzStringVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `StyledNodeVec`
#[repr(C)]
#[pyclass(name = "StyledNodeVec")]
pub struct AzStyledNodeVec {
    pub(crate) ptr: *const AzStyledNode,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzStyledNodeVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `TagIdsToNodeIdsMappingVec`
#[repr(C)]
#[pyclass(name = "TagIdsToNodeIdsMappingVec")]
pub struct AzTagIdsToNodeIdsMappingVec {
    pub(crate) ptr: *const AzTagIdToNodeIdMapping,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzTagIdsToNodeIdsMappingVecDestructorEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `OptionMouseState` struct
#[repr(C, u8)]
pub enum AzOptionMouseState {
    None,
    Some(AzMouseState),
}

/// Re-export of rust-allocated (stack based) `OptionKeyboardState` struct
#[repr(C, u8)]
pub enum AzOptionKeyboardState {
    None,
    Some(AzKeyboardState),
}

/// Re-export of rust-allocated (stack based) `OptionStringVec` struct
#[repr(C, u8)]
pub enum AzOptionStringVec {
    None,
    Some(AzStringVec),
}

/// Re-export of rust-allocated (stack based) `OptionThreadReceiveMsg` struct
#[repr(C, u8)]
pub enum AzOptionThreadReceiveMsg {
    None,
    Some(AzThreadReceiveMsg),
}

/// Re-export of rust-allocated (stack based) `OptionTaskBarIcon` struct
#[repr(C, u8)]
pub enum AzOptionTaskBarIcon {
    None,
    Some(AzTaskBarIcon),
}

/// Re-export of rust-allocated (stack based) `OptionWindowIcon` struct
#[repr(C, u8)]
pub enum AzOptionWindowIcon {
    None,
    Some(AzWindowIcon),
}

/// Re-export of rust-allocated (stack based) `OptionString` struct
#[repr(C, u8)]
pub enum AzOptionString {
    None,
    Some(AzString),
}

/// Re-export of rust-allocated (stack based) `OptionTexture` struct
#[repr(C, u8)]
pub enum AzOptionTexture {
    None,
    Some(AzTexture),
}

/// Re-export of rust-allocated (stack based) `OptionInstant` struct
#[repr(C, u8)]
pub enum AzOptionInstant {
    None,
    Some(AzInstant),
}

/// Re-export of rust-allocated (stack based) `DuplicatedNamespaceError` struct
#[repr(C)]
#[pyclass(name = "DuplicatedNamespaceError")]
pub struct AzDuplicatedNamespaceError {
    #[pyo3(get, set)]
    pub ns: AzString,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Re-export of rust-allocated (stack based) `UnknownNamespaceError` struct
#[repr(C)]
#[pyclass(name = "UnknownNamespaceError")]
pub struct AzUnknownNamespaceError {
    #[pyo3(get, set)]
    pub ns: AzString,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Re-export of rust-allocated (stack based) `UnexpectedCloseTagError` struct
#[repr(C)]
#[pyclass(name = "UnexpectedCloseTagError")]
pub struct AzUnexpectedCloseTagError {
    #[pyo3(get, set)]
    pub expected: AzString,
    #[pyo3(get, set)]
    pub actual: AzString,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Re-export of rust-allocated (stack based) `UnknownEntityReferenceError` struct
#[repr(C)]
#[pyclass(name = "UnknownEntityReferenceError")]
pub struct AzUnknownEntityReferenceError {
    #[pyo3(get, set)]
    pub entity: AzString,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Re-export of rust-allocated (stack based) `DuplicatedAttributeError` struct
#[repr(C)]
#[pyclass(name = "DuplicatedAttributeError")]
pub struct AzDuplicatedAttributeError {
    #[pyo3(get, set)]
    pub attribute: AzString,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Re-export of rust-allocated (stack based) `InvalidStringError` struct
#[repr(C)]
#[pyclass(name = "InvalidStringError")]
pub struct AzInvalidStringError {
    #[pyo3(get, set)]
    pub got: AzString,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Window configuration specific to Win32
#[repr(C)]
#[pyclass(name = "WindowsWindowOptions")]
pub struct AzWindowsWindowOptions {
    #[pyo3(get, set)]
    pub allow_drag_drop: bool,
    #[pyo3(get, set)]
    pub no_redirection_bitmap: bool,
    #[pyo3(get, set)]
    pub window_icon: AzOptionWindowIconEnumWrapper,
    #[pyo3(get, set)]
    pub taskbar_icon: AzOptionTaskBarIconEnumWrapper,
    #[pyo3(get, set)]
    pub parent_window: AzOptionHwndHandleEnumWrapper,
}

/// CSD theme of the window title / button controls
#[repr(C)]
#[pyclass(name = "WaylandTheme")]
pub struct AzWaylandTheme {
    pub title_bar_active_background_color: [u8;4],
    pub title_bar_active_separator_color: [u8;4],
    pub title_bar_active_text_color: [u8;4],
    pub title_bar_inactive_background_color: [u8;4],
    pub title_bar_inactive_separator_color: [u8;4],
    pub title_bar_inactive_text_color: [u8;4],
    pub maximize_idle_foreground_inactive_color: [u8;4],
    pub minimize_idle_foreground_inactive_color: [u8;4],
    pub close_idle_foreground_inactive_color: [u8;4],
    pub maximize_hovered_foreground_inactive_color: [u8;4],
    pub minimize_hovered_foreground_inactive_color: [u8;4],
    pub close_hovered_foreground_inactive_color: [u8;4],
    pub maximize_disabled_foreground_inactive_color: [u8;4],
    pub minimize_disabled_foreground_inactive_color: [u8;4],
    pub close_disabled_foreground_inactive_color: [u8;4],
    pub maximize_idle_background_inactive_color: [u8;4],
    pub minimize_idle_background_inactive_color: [u8;4],
    pub close_idle_background_inactive_color: [u8;4],
    pub maximize_hovered_background_inactive_color: [u8;4],
    pub minimize_hovered_background_inactive_color: [u8;4],
    pub close_hovered_background_inactive_color: [u8;4],
    pub maximize_disabled_background_inactive_color: [u8;4],
    pub minimize_disabled_background_inactive_color: [u8;4],
    pub close_disabled_background_inactive_color: [u8;4],
    pub maximize_idle_foreground_active_color: [u8;4],
    pub minimize_idle_foreground_active_color: [u8;4],
    pub close_idle_foreground_active_color: [u8;4],
    pub maximize_hovered_foreground_active_color: [u8;4],
    pub minimize_hovered_foreground_active_color: [u8;4],
    pub close_hovered_foreground_active_color: [u8;4],
    pub maximize_disabled_foreground_active_color: [u8;4],
    pub minimize_disabled_foreground_active_color: [u8;4],
    pub close_disabled_foreground_active_color: [u8;4],
    pub maximize_idle_background_active_color: [u8;4],
    pub minimize_idle_background_active_color: [u8;4],
    pub close_idle_background_active_color: [u8;4],
    pub maximize_hovered_background_active_color: [u8;4],
    pub minimize_hovered_background_active_color: [u8;4],
    pub close_hovered_background_active_color: [u8;4],
    pub maximize_disabled_background_active_color: [u8;4],
    pub minimize_disabled_background_active_color: [u8;4],
    pub close_disabled_background_active_color: [u8;4],
    #[pyo3(get, set)]
    pub title_bar_font: AzString,
    #[pyo3(get, set)]
    pub title_bar_font_size: f32,
}

/// Key-value pair, used for setting WM hints values specific to GNOME
#[repr(C)]
#[pyclass(name = "StringPair")]
pub struct AzStringPair {
    #[pyo3(get, set)]
    pub key: AzString,
    #[pyo3(get, set)]
    pub value: AzString,
}

/// Information about a single (or many) monitors, useful for dock widgets
#[repr(C)]
#[pyclass(name = "Monitor")]
pub struct AzMonitor {
    #[pyo3(get, set)]
    pub id: usize,
    #[pyo3(get, set)]
    pub name: AzOptionStringEnumWrapper,
    #[pyo3(get, set)]
    pub size: AzLayoutSize,
    #[pyo3(get, set)]
    pub position: AzLayoutPoint,
    #[pyo3(get, set)]
    pub scale_factor: f64,
    #[pyo3(get, set)]
    pub video_modes: AzVideoModeVec,
    #[pyo3(get, set)]
    pub is_primary_monitor: bool,
}

/// Re-export of rust-allocated (stack based) `LayoutCallback` struct
#[repr(C, u8)]
pub enum AzLayoutCallback {
    Raw(AzLayoutCallbackInner),
    Marshaled(AzMarshaledLayoutCallback),
}

/// Re-export of rust-allocated (stack based) `InlineWord` struct
#[repr(C, u8)]
pub enum AzInlineWord {
    Tab,
    Return,
    Space,
    Word(AzInlineTextContents),
}

/// Re-export of rust-allocated (stack based) `CallbackData` struct
#[repr(C)]
#[pyclass(name = "CallbackData")]
pub struct AzCallbackData {
    #[pyo3(get, set)]
    pub event: AzEventFilterEnumWrapper,
    #[pyo3(get, set)]
    pub callback: AzCallback,
    #[pyo3(get, set)]
    pub data: AzRefAny,
}

/// List of core DOM node types built-into by `azul`
#[repr(C, u8)]
pub enum AzNodeType {
    Body,
    Div,
    Br,
    Text(AzString),
    Image(AzImageRef),
    IFrame(AzIFrameNode),
}

/// Re-export of rust-allocated (stack based) `IdOrClass` struct
#[repr(C, u8)]
pub enum AzIdOrClass {
    Id(AzString),
    Class(AzString),
}

/// Re-export of rust-allocated (stack based) `CssPathSelector` struct
#[repr(C, u8)]
pub enum AzCssPathSelector {
    Global,
    Type(AzNodeTypeKey),
    Class(AzString),
    Id(AzString),
    PseudoSelector(AzCssPathPseudoSelector),
    DirectChildren,
    Children,
}

/// Re-export of rust-allocated (stack based) `StyleBackgroundContent` struct
#[repr(C, u8)]
pub enum AzStyleBackgroundContent {
    LinearGradient(AzLinearGradient),
    RadialGradient(AzRadialGradient),
    ConicGradient(AzConicGradient),
    Image(AzString),
    Color(AzColorU),
}

/// Re-export of rust-allocated (stack based) `ScrollbarInfo` struct
#[repr(C)]
#[pyclass(name = "ScrollbarInfo")]
pub struct AzScrollbarInfo {
    #[pyo3(get, set)]
    pub width: AzLayoutWidth,
    #[pyo3(get, set)]
    pub padding_left: AzLayoutPaddingLeft,
    #[pyo3(get, set)]
    pub padding_right: AzLayoutPaddingRight,
    #[pyo3(get, set)]
    pub track: AzStyleBackgroundContentEnumWrapper,
    #[pyo3(get, set)]
    pub thumb: AzStyleBackgroundContentEnumWrapper,
    #[pyo3(get, set)]
    pub button: AzStyleBackgroundContentEnumWrapper,
    #[pyo3(get, set)]
    pub corner: AzStyleBackgroundContentEnumWrapper,
    #[pyo3(get, set)]
    pub resizer: AzStyleBackgroundContentEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `ScrollbarStyle` struct
#[repr(C)]
#[pyclass(name = "ScrollbarStyle")]
pub struct AzScrollbarStyle {
    #[pyo3(get, set)]
    pub horizontal: AzScrollbarInfo,
    #[pyo3(get, set)]
    pub vertical: AzScrollbarInfo,
}

/// Re-export of rust-allocated (stack based) `StyleFontFamily` struct
#[repr(C, u8)]
pub enum AzStyleFontFamily {
    System(AzString),
    File(AzString),
    Ref(AzFontRef),
}

/// Re-export of rust-allocated (stack based) `ScrollbarStyleValue` struct
#[repr(C, u8)]
pub enum AzScrollbarStyleValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzScrollbarStyle),
}

/// Re-export of rust-allocated (stack based) `StyleTransformVecValue` struct
#[repr(C, u8)]
pub enum AzStyleTransformVecValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleTransformVec),
}

/// Re-export of rust-allocated (stack based) `VertexAttribute` struct
#[repr(C)]
#[pyclass(name = "VertexAttribute")]
pub struct AzVertexAttribute {
    #[pyo3(get, set)]
    pub name: AzString,
    #[pyo3(get, set)]
    pub layout_location: AzOptionUsizeEnumWrapper,
    #[pyo3(get, set)]
    pub attribute_type: AzVertexAttributeTypeEnumWrapper,
    #[pyo3(get, set)]
    pub item_count: usize,
}

/// Re-export of rust-allocated (stack based) `DebugMessage` struct
#[repr(C)]
#[pyclass(name = "DebugMessage")]
pub struct AzDebugMessage {
    #[pyo3(get, set)]
    pub message: AzString,
    #[pyo3(get, set)]
    pub source: u32,
    #[pyo3(get, set)]
    pub ty: u32,
    #[pyo3(get, set)]
    pub id: u32,
    #[pyo3(get, set)]
    pub severity: u32,
}

/// C-ABI stable reexport of `(i32, u32, AzString)`
#[repr(C)]
#[pyclass(name = "GetActiveAttribReturn")]
pub struct AzGetActiveAttribReturn {
    #[pyo3(get, set)]
    pub _0: i32,
    #[pyo3(get, set)]
    pub _1: u32,
    #[pyo3(get, set)]
    pub _2: AzString,
}

/// C-ABI stable reexport of `(i32, u32, AzString)`
#[repr(C)]
#[pyclass(name = "GetActiveUniformReturn")]
pub struct AzGetActiveUniformReturn {
    #[pyo3(get, set)]
    pub _0: i32,
    #[pyo3(get, set)]
    pub _1: u32,
    #[pyo3(get, set)]
    pub _2: AzString,
}

/// Re-export of rust-allocated (stack based) `RawImage` struct
#[repr(C)]
#[pyclass(name = "RawImage")]
pub struct AzRawImage {
    #[pyo3(get, set)]
    pub pixels: AzRawImageDataEnumWrapper,
    #[pyo3(get, set)]
    pub width: usize,
    #[pyo3(get, set)]
    pub height: usize,
    #[pyo3(get, set)]
    pub alpha_premultiplied: bool,
    #[pyo3(get, set)]
    pub data_format: AzRawImageFormatEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `SvgPath` struct
#[repr(C)]
#[pyclass(name = "SvgPath")]
pub struct AzSvgPath {
    #[pyo3(get, set)]
    pub items: AzSvgPathElementVec,
}

/// Re-export of rust-allocated (stack based) `SvgParseOptions` struct
#[repr(C)]
#[pyclass(name = "SvgParseOptions")]
pub struct AzSvgParseOptions {
    #[pyo3(get, set)]
    pub relative_image_path: AzOptionStringEnumWrapper,
    #[pyo3(get, set)]
    pub dpi: f32,
    #[pyo3(get, set)]
    pub default_font_family: AzString,
    #[pyo3(get, set)]
    pub font_size: f32,
    #[pyo3(get, set)]
    pub languages: AzStringVec,
    #[pyo3(get, set)]
    pub shape_rendering: AzShapeRenderingEnumWrapper,
    #[pyo3(get, set)]
    pub text_rendering: AzTextRenderingEnumWrapper,
    #[pyo3(get, set)]
    pub image_rendering: AzImageRenderingEnumWrapper,
    #[pyo3(get, set)]
    pub keep_named_groups: bool,
    #[pyo3(get, set)]
    pub fontdb: AzFontDatabaseEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `SvgStyle` struct
#[repr(C, u8)]
pub enum AzSvgStyle {
    Fill(AzSvgFillStyle),
    Stroke(AzSvgStrokeStyle),
}

/// Re-export of rust-allocated (stack based) `FileTypeList` struct
#[repr(C)]
#[pyclass(name = "FileTypeList")]
pub struct AzFileTypeList {
    #[pyo3(get, set)]
    pub document_types: AzStringVec,
    #[pyo3(get, set)]
    pub document_descriptor: AzString,
}

/// Re-export of rust-allocated (stack based) `Timer` struct
#[repr(C)]
#[pyclass(name = "Timer")]
pub struct AzTimer {
    #[pyo3(get, set)]
    pub data: AzRefAny,
    #[pyo3(get, set)]
    pub node_id: AzOptionDomNodeIdEnumWrapper,
    #[pyo3(get, set)]
    pub created: AzInstantEnumWrapper,
    #[pyo3(get, set)]
    pub last_run: AzOptionInstantEnumWrapper,
    #[pyo3(get, set)]
    pub run_count: usize,
    #[pyo3(get, set)]
    pub delay: AzOptionDurationEnumWrapper,
    #[pyo3(get, set)]
    pub interval: AzOptionDurationEnumWrapper,
    #[pyo3(get, set)]
    pub timeout: AzOptionDurationEnumWrapper,
    #[pyo3(get, set)]
    pub callback: AzTimerCallback,
}

/// Re-export of rust-allocated (stack based) `FmtValue` struct
#[repr(C, u8)]
pub enum AzFmtValue {
    Bool(bool),
    Uchar(u8),
    Schar(i8),
    Ushort(u16),
    Sshort(i16),
    Uint(u32),
    Sint(i32),
    Ulong(u64),
    Slong(i64),
    Isize(isize),
    Usize(usize),
    Float(f32),
    Double(f64),
    Str(AzString),
    StrVec(AzStringVec),
}

/// Re-export of rust-allocated (stack based) `FmtArg` struct
#[repr(C)]
#[pyclass(name = "FmtArg")]
pub struct AzFmtArg {
    #[pyo3(get, set)]
    pub key: AzString,
    #[pyo3(get, set)]
    pub value: AzFmtValueEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<StyleFontFamily>`
#[repr(C)]
#[pyclass(name = "StyleFontFamilyVec")]
pub struct AzStyleFontFamilyVec {
    pub(crate) ptr: *const AzStyleFontFamilyEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzStyleFontFamilyVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<FmtArg>`
#[repr(C)]
#[pyclass(name = "FmtArgVec")]
pub struct AzFmtArgVec {
    pub(crate) ptr: *const AzFmtArg,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzFmtArgVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<InlineWord>`
#[repr(C)]
#[pyclass(name = "InlineWordVec")]
pub struct AzInlineWordVec {
    pub(crate) ptr: *const AzInlineWordEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzInlineWordVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<Monitor>`
#[repr(C)]
#[pyclass(name = "MonitorVec")]
pub struct AzMonitorVec {
    pub(crate) ptr: *const AzMonitor,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzMonitorVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<IdOrClass>`
#[repr(C)]
#[pyclass(name = "IdOrClassVec")]
pub struct AzIdOrClassVec {
    pub(crate) ptr: *const AzIdOrClassEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzIdOrClassVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<StyleBackgroundContent>`
#[repr(C)]
#[pyclass(name = "StyleBackgroundContentVec")]
pub struct AzStyleBackgroundContentVec {
    pub(crate) ptr: *const AzStyleBackgroundContentEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzStyleBackgroundContentVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<SvgPath>`
#[repr(C)]
#[pyclass(name = "SvgPathVec")]
pub struct AzSvgPathVec {
    pub(crate) ptr: *const AzSvgPath,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzSvgPathVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<VertexAttribute>`
#[repr(C)]
#[pyclass(name = "VertexAttributeVec")]
pub struct AzVertexAttributeVec {
    pub(crate) ptr: *const AzVertexAttribute,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzVertexAttributeVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `CssPathSelector`
#[repr(C)]
#[pyclass(name = "CssPathSelectorVec")]
pub struct AzCssPathSelectorVec {
    pub(crate) ptr: *const AzCssPathSelectorEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzCssPathSelectorVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `CallbackData`
#[repr(C)]
#[pyclass(name = "CallbackDataVec")]
pub struct AzCallbackDataVec {
    pub(crate) ptr: *const AzCallbackData,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzCallbackDataVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<DebugMessage>`
#[repr(C)]
#[pyclass(name = "DebugMessageVec")]
pub struct AzDebugMessageVec {
    pub(crate) ptr: *const AzDebugMessage,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzDebugMessageVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `StringPairVec`
#[repr(C)]
#[pyclass(name = "StringPairVec")]
pub struct AzStringPairVec {
    pub(crate) ptr: *const AzStringPair,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzStringPairVecDestructorEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `OptionFileTypeList` struct
#[repr(C, u8)]
pub enum AzOptionFileTypeList {
    None,
    Some(AzFileTypeList),
}

/// Re-export of rust-allocated (stack based) `OptionRawImage` struct
#[repr(C, u8)]
pub enum AzOptionRawImage {
    None,
    Some(AzRawImage),
}

/// Re-export of rust-allocated (stack based) `OptionWaylandTheme` struct
#[repr(C, u8)]
pub enum AzOptionWaylandTheme {
    None,
    Some(AzWaylandTheme),
}

/// Re-export of rust-allocated (stack based) `ResultRawImageDecodeImageError` struct
#[repr(C, u8)]
pub enum AzResultRawImageDecodeImageError {
    Ok(AzRawImage),
    Err(AzDecodeImageError),
}

/// Re-export of rust-allocated (stack based) `XmlStreamError` struct
#[repr(C, u8)]
pub enum AzXmlStreamError {
    UnexpectedEndOfStream,
    InvalidName,
    NonXmlChar(AzNonXmlCharError),
    InvalidChar(AzInvalidCharError),
    InvalidCharMultiple(AzInvalidCharMultipleError),
    InvalidQuote(AzInvalidQuoteError),
    InvalidSpace(AzInvalidSpaceError),
    InvalidString(AzInvalidStringError),
    InvalidReference,
    InvalidExternalID,
    InvalidCommentData,
    InvalidCommentEnd,
    InvalidCharacterData,
}

/// Re-export of rust-allocated (stack based) `LinuxWindowOptions` struct
#[repr(C)]
#[pyclass(name = "LinuxWindowOptions")]
pub struct AzLinuxWindowOptions {
    #[pyo3(get, set)]
    pub x11_visual: AzOptionX11VisualEnumWrapper,
    #[pyo3(get, set)]
    pub x11_screen: AzOptionI32EnumWrapper,
    #[pyo3(get, set)]
    pub x11_wm_classes: AzStringPairVec,
    #[pyo3(get, set)]
    pub x11_override_redirect: bool,
    #[pyo3(get, set)]
    pub x11_window_types: AzXWindowTypeVec,
    #[pyo3(get, set)]
    pub x11_gtk_theme_variant: AzOptionStringEnumWrapper,
    #[pyo3(get, set)]
    pub x11_resize_increments: AzOptionLogicalSizeEnumWrapper,
    #[pyo3(get, set)]
    pub x11_base_size: AzOptionLogicalSizeEnumWrapper,
    #[pyo3(get, set)]
    pub wayland_app_id: AzOptionStringEnumWrapper,
    #[pyo3(get, set)]
    pub wayland_theme: AzOptionWaylandThemeEnumWrapper,
    #[pyo3(get, set)]
    pub request_user_attention: bool,
    #[pyo3(get, set)]
    pub window_icon: AzOptionWindowIconEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `InlineLine` struct
#[repr(C)]
#[pyclass(name = "InlineLine")]
pub struct AzInlineLine {
    #[pyo3(get, set)]
    pub words: AzInlineWordVec,
    #[pyo3(get, set)]
    pub bounds: AzLogicalRect,
}

/// Re-export of rust-allocated (stack based) `CssPath` struct
#[repr(C)]
#[pyclass(name = "CssPath")]
pub struct AzCssPath {
    #[pyo3(get, set)]
    pub selectors: AzCssPathSelectorVec,
}

/// Re-export of rust-allocated (stack based) `StyleBackgroundContentVecValue` struct
#[repr(C, u8)]
pub enum AzStyleBackgroundContentVecValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleBackgroundContentVec),
}

/// Re-export of rust-allocated (stack based) `StyleFontFamilyVecValue` struct
#[repr(C, u8)]
pub enum AzStyleFontFamilyVecValue {
    Auto,
    None,
    Inherit,
    Initial,
    Exact(AzStyleFontFamilyVec),
}

/// Parsed CSS key-value pair
#[repr(C, u8)]
pub enum AzCssProperty {
    TextColor(AzStyleTextColorValue),
    FontSize(AzStyleFontSizeValue),
    FontFamily(AzStyleFontFamilyVecValue),
    TextAlign(AzStyleTextAlignValue),
    LetterSpacing(AzStyleLetterSpacingValue),
    LineHeight(AzStyleLineHeightValue),
    WordSpacing(AzStyleWordSpacingValue),
    TabWidth(AzStyleTabWidthValue),
    Cursor(AzStyleCursorValue),
    Display(AzLayoutDisplayValue),
    Float(AzLayoutFloatValue),
    BoxSizing(AzLayoutBoxSizingValue),
    Width(AzLayoutWidthValue),
    Height(AzLayoutHeightValue),
    MinWidth(AzLayoutMinWidthValue),
    MinHeight(AzLayoutMinHeightValue),
    MaxWidth(AzLayoutMaxWidthValue),
    MaxHeight(AzLayoutMaxHeightValue),
    Position(AzLayoutPositionValue),
    Top(AzLayoutTopValue),
    Right(AzLayoutRightValue),
    Left(AzLayoutLeftValue),
    Bottom(AzLayoutBottomValue),
    FlexWrap(AzLayoutFlexWrapValue),
    FlexDirection(AzLayoutFlexDirectionValue),
    FlexGrow(AzLayoutFlexGrowValue),
    FlexShrink(AzLayoutFlexShrinkValue),
    JustifyContent(AzLayoutJustifyContentValue),
    AlignItems(AzLayoutAlignItemsValue),
    AlignContent(AzLayoutAlignContentValue),
    BackgroundContent(AzStyleBackgroundContentVecValue),
    BackgroundPosition(AzStyleBackgroundPositionVecValue),
    BackgroundSize(AzStyleBackgroundSizeVecValue),
    BackgroundRepeat(AzStyleBackgroundRepeatVecValue),
    OverflowX(AzLayoutOverflowValue),
    OverflowY(AzLayoutOverflowValue),
    PaddingTop(AzLayoutPaddingTopValue),
    PaddingLeft(AzLayoutPaddingLeftValue),
    PaddingRight(AzLayoutPaddingRightValue),
    PaddingBottom(AzLayoutPaddingBottomValue),
    MarginTop(AzLayoutMarginTopValue),
    MarginLeft(AzLayoutMarginLeftValue),
    MarginRight(AzLayoutMarginRightValue),
    MarginBottom(AzLayoutMarginBottomValue),
    BorderTopLeftRadius(AzStyleBorderTopLeftRadiusValue),
    BorderTopRightRadius(AzStyleBorderTopRightRadiusValue),
    BorderBottomLeftRadius(AzStyleBorderBottomLeftRadiusValue),
    BorderBottomRightRadius(AzStyleBorderBottomRightRadiusValue),
    BorderTopColor(AzStyleBorderTopColorValue),
    BorderRightColor(AzStyleBorderRightColorValue),
    BorderLeftColor(AzStyleBorderLeftColorValue),
    BorderBottomColor(AzStyleBorderBottomColorValue),
    BorderTopStyle(AzStyleBorderTopStyleValue),
    BorderRightStyle(AzStyleBorderRightStyleValue),
    BorderLeftStyle(AzStyleBorderLeftStyleValue),
    BorderBottomStyle(AzStyleBorderBottomStyleValue),
    BorderTopWidth(AzLayoutBorderTopWidthValue),
    BorderRightWidth(AzLayoutBorderRightWidthValue),
    BorderLeftWidth(AzLayoutBorderLeftWidthValue),
    BorderBottomWidth(AzLayoutBorderBottomWidthValue),
    BoxShadowLeft(AzStyleBoxShadowValue),
    BoxShadowRight(AzStyleBoxShadowValue),
    BoxShadowTop(AzStyleBoxShadowValue),
    BoxShadowBottom(AzStyleBoxShadowValue),
    ScrollbarStyle(AzScrollbarStyleValue),
    Opacity(AzStyleOpacityValue),
    Transform(AzStyleTransformVecValue),
    TransformOrigin(AzStyleTransformOriginValue),
    PerspectiveOrigin(AzStylePerspectiveOriginValue),
    BackfaceVisibility(AzStyleBackfaceVisibilityValue),
}

/// Re-export of rust-allocated (stack based) `CssPropertySource` struct
#[repr(C, u8)]
pub enum AzCssPropertySource {
    Css(AzCssPath),
    Inline,
}

/// Re-export of rust-allocated (stack based) `VertexLayout` struct
#[repr(C)]
#[pyclass(name = "VertexLayout")]
pub struct AzVertexLayout {
    #[pyo3(get, set)]
    pub fields: AzVertexAttributeVec,
}

/// Re-export of rust-allocated (stack based) `VertexArrayObject` struct
#[repr(C)]
#[pyclass(name = "VertexArrayObject")]
pub struct AzVertexArrayObject {
    #[pyo3(get, set)]
    pub vertex_layout: AzVertexLayout,
    #[pyo3(get, set)]
    pub vao_id: u32,
    #[pyo3(get, set)]
    pub gl_context: AzGl,
}

/// Re-export of rust-allocated (stack based) `VertexBuffer` struct
#[repr(C)]
#[pyclass(name = "VertexBuffer")]
pub struct AzVertexBuffer {
    #[pyo3(get, set)]
    pub vertex_buffer_id: u32,
    #[pyo3(get, set)]
    pub vertex_buffer_len: usize,
    #[pyo3(get, set)]
    pub vao: AzVertexArrayObject,
    #[pyo3(get, set)]
    pub index_buffer_id: u32,
    #[pyo3(get, set)]
    pub index_buffer_len: usize,
    #[pyo3(get, set)]
    pub index_buffer_format: AzIndexBufferFormatEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `SvgMultiPolygon` struct
#[repr(C)]
#[pyclass(name = "SvgMultiPolygon")]
pub struct AzSvgMultiPolygon {
    #[pyo3(get, set)]
    pub rings: AzSvgPathVec,
}

/// Re-export of rust-allocated (stack based) `XmlNode` struct
#[repr(C)]
#[pyclass(name = "XmlNode")]
pub struct AzXmlNode {
    #[pyo3(get, set)]
    pub tag: AzString,
    #[pyo3(get, set)]
    pub attributes: AzStringPairVec,
    #[pyo3(get, set)]
    pub children: AzXmlNodeVec,
    #[pyo3(get, set)]
    pub text: AzOptionStringEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<InlineLine>`
#[repr(C)]
#[pyclass(name = "InlineLineVec")]
pub struct AzInlineLineVec {
    pub(crate) ptr: *const AzInlineLine,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzInlineLineVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<CssProperty>`
#[repr(C)]
#[pyclass(name = "CssPropertyVec")]
pub struct AzCssPropertyVec {
    pub(crate) ptr: *const AzCssPropertyEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzCssPropertyVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<SvgMultiPolygon>`
#[repr(C)]
#[pyclass(name = "SvgMultiPolygonVec")]
pub struct AzSvgMultiPolygonVec {
    pub(crate) ptr: *const AzSvgMultiPolygon,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzSvgMultiPolygonVecDestructorEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `OptionCssProperty` struct
#[repr(C, u8)]
pub enum AzOptionCssProperty {
    None,
    Some(AzCssProperty),
}

/// Re-export of rust-allocated (stack based) `XmlTextError` struct
#[repr(C)]
#[pyclass(name = "XmlTextError")]
pub struct AzXmlTextError {
    #[pyo3(get, set)]
    pub stream_error: AzXmlStreamErrorEnumWrapper,
    #[pyo3(get, set)]
    pub pos: AzSvgParseErrorPosition,
}

/// Platform-specific window configuration, i.e. WM options that are not cross-platform
#[repr(C)]
#[pyclass(name = "PlatformSpecificOptions")]
pub struct AzPlatformSpecificOptions {
    #[pyo3(get, set)]
    pub windows_options: AzWindowsWindowOptions,
    #[pyo3(get, set)]
    pub linux_options: AzLinuxWindowOptions,
    #[pyo3(get, set)]
    pub mac_options: AzMacWindowOptions,
    #[pyo3(get, set)]
    pub wasm_options: AzWasmWindowOptions,
}

/// Re-export of rust-allocated (stack based) `WindowState` struct
#[repr(C)]
#[pyclass(name = "WindowState")]
pub struct AzWindowState {
    #[pyo3(get, set)]
    pub title: AzString,
    #[pyo3(get, set)]
    pub theme: AzWindowThemeEnumWrapper,
    #[pyo3(get, set)]
    pub size: AzWindowSize,
    #[pyo3(get, set)]
    pub position: AzWindowPositionEnumWrapper,
    #[pyo3(get, set)]
    pub flags: AzWindowFlags,
    #[pyo3(get, set)]
    pub debug_state: AzDebugState,
    #[pyo3(get, set)]
    pub keyboard_state: AzKeyboardState,
    #[pyo3(get, set)]
    pub mouse_state: AzMouseState,
    #[pyo3(get, set)]
    pub touch_state: AzTouchState,
    #[pyo3(get, set)]
    pub ime_position: AzImePositionEnumWrapper,
    #[pyo3(get, set)]
    pub monitor: AzMonitor,
    #[pyo3(get, set)]
    pub platform_specific_options: AzPlatformSpecificOptions,
    #[pyo3(get, set)]
    pub renderer_options: AzRendererOptions,
    #[pyo3(get, set)]
    pub background_color: AzColorU,
    #[pyo3(get, set)]
    pub layout_callback: AzLayoutCallbackEnumWrapper,
    #[pyo3(get, set)]
    pub close_callback: AzOptionCallbackEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `CallbackInfo` struct
#[repr(C)]
#[pyclass(name = "CallbackInfo")]
pub struct AzCallbackInfo {
    pub css_property_cache: *const c_void,
    pub styled_node_states: *const c_void,
    pub previous_window_state: *const c_void,
    pub current_window_state: *const c_void,
    pub modifiable_window_state: *mut AzWindowState,
    pub gl_context: *const AzOptionGlEnumWrapper,
    pub image_cache: *mut c_void,
    pub system_fonts: *mut c_void,
    pub timers: *mut c_void,
    pub threads: *mut c_void,
    pub new_windows: *mut c_void,
    pub current_window_handle: *const AzRawWindowHandleEnumWrapper,
    pub node_hierarchy: *const c_void,
    pub system_callbacks: *const AzSystemCallbacks,
    pub datasets: *mut c_void,
    pub stop_propagation: *mut bool,
    pub focus_target: *mut c_void,
    pub words_cache: *const c_void,
    pub shaped_words_cache: *const c_void,
    pub positioned_words_cache: *const c_void,
    pub positioned_rects: *const c_void,
    pub words_changed_in_callbacks: *mut c_void,
    pub images_changed_in_callbacks: *mut c_void,
    pub image_masks_changed_in_callbacks: *mut c_void,
    pub css_properties_changed_in_callbacks: *mut c_void,
    pub current_scroll_states: *const c_void,
    pub nodes_scrolled_in_callback: *mut c_void,
    #[pyo3(get, set)]
    pub hit_dom_node: AzDomNodeId,
    #[pyo3(get, set)]
    pub cursor_relative_to_item: AzOptionLogicalPositionEnumWrapper,
    #[pyo3(get, set)]
    pub cursor_in_viewport: AzOptionLogicalPositionEnumWrapper,
    pub _reserved_ref: *const c_void,
    pub _reserved_mut: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `InlineText` struct
#[repr(C)]
#[pyclass(name = "InlineText")]
pub struct AzInlineText {
    #[pyo3(get, set)]
    pub lines: AzInlineLineVec,
    #[pyo3(get, set)]
    pub content_size: AzLogicalSize,
    #[pyo3(get, set)]
    pub font_size_px: f32,
    #[pyo3(get, set)]
    pub last_word_index: usize,
    #[pyo3(get, set)]
    pub baseline_descender_px: f32,
}

/// CSS path to set the keyboard input focus
#[repr(C)]
#[pyclass(name = "FocusTargetPath")]
pub struct AzFocusTargetPath {
    #[pyo3(get, set)]
    pub dom: AzDomId,
    #[pyo3(get, set)]
    pub css_path: AzCssPath,
}

/// Animation struct to start a new animation
#[repr(C)]
#[pyclass(name = "Animation")]
pub struct AzAnimation {
    #[pyo3(get, set)]
    pub from: AzCssPropertyEnumWrapper,
    #[pyo3(get, set)]
    pub to: AzCssPropertyEnumWrapper,
    #[pyo3(get, set)]
    pub duration: AzDurationEnumWrapper,
    #[pyo3(get, set)]
    pub repeat: AzAnimationRepeatEnumWrapper,
    #[pyo3(get, set)]
    pub repeat_count: AzAnimationRepeatCountEnumWrapper,
    #[pyo3(get, set)]
    pub easing: AzAnimationEasingEnumWrapper,
    #[pyo3(get, set)]
    pub relayout_on_finish: bool,
}

/// Re-export of rust-allocated (stack based) `TimerCallbackInfo` struct
#[repr(C)]
#[pyclass(name = "TimerCallbackInfo")]
pub struct AzTimerCallbackInfo {
    #[pyo3(get, set)]
    pub callback_info: AzCallbackInfo,
    #[pyo3(get, set)]
    pub node_id: AzOptionDomNodeIdEnumWrapper,
    #[pyo3(get, set)]
    pub frame_start: AzInstantEnumWrapper,
    #[pyo3(get, set)]
    pub call_count: usize,
    #[pyo3(get, set)]
    pub is_about_to_finish: bool,
    pub _reserved_ref: *const c_void,
    pub _reserved_mut: *mut c_void,
}

/// Re-export of rust-allocated (stack based) `NodeDataInlineCssProperty` struct
#[repr(C, u8)]
pub enum AzNodeDataInlineCssProperty {
    Normal(AzCssProperty),
    Active(AzCssProperty),
    Focus(AzCssProperty),
    Hover(AzCssProperty),
}

/// Re-export of rust-allocated (stack based) `DynamicCssProperty` struct
#[repr(C)]
#[pyclass(name = "DynamicCssProperty")]
pub struct AzDynamicCssProperty {
    #[pyo3(get, set)]
    pub dynamic_id: AzString,
    #[pyo3(get, set)]
    pub default_value: AzCssPropertyEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `SvgNode` struct
#[repr(C, u8)]
pub enum AzSvgNode {
    MultiPolygonCollection(AzSvgMultiPolygonVec),
    MultiPolygon(AzSvgMultiPolygon),
    Path(AzSvgPath),
    Circle(AzSvgCircle),
    Rect(AzSvgRect),
}

/// Re-export of rust-allocated (stack based) `SvgStyledNode` struct
#[repr(C)]
#[pyclass(name = "SvgStyledNode")]
pub struct AzSvgStyledNode {
    #[pyo3(get, set)]
    pub geometry: AzSvgNodeEnumWrapper,
    #[pyo3(get, set)]
    pub style: AzSvgStyleEnumWrapper,
}

/// Wrapper over a Rust-allocated `Vec<NodeDataInlineCssProperty>`
#[repr(C)]
#[pyclass(name = "NodeDataInlineCssPropertyVec")]
pub struct AzNodeDataInlineCssPropertyVec {
    pub(crate) ptr: *const AzNodeDataInlineCssPropertyEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzNodeDataInlineCssPropertyVecDestructorEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `OptionWindowState` struct
#[repr(C, u8)]
pub enum AzOptionWindowState {
    None,
    Some(AzWindowState),
}

/// Re-export of rust-allocated (stack based) `OptionInlineText` struct
#[repr(C, u8)]
pub enum AzOptionInlineText {
    None,
    Some(AzInlineText),
}

/// Re-export of rust-allocated (stack based) `XmlParseError` struct
#[repr(C, u8)]
pub enum AzXmlParseError {
    InvalidDeclaration(AzXmlTextError),
    InvalidComment(AzXmlTextError),
    InvalidPI(AzXmlTextError),
    InvalidDoctype(AzXmlTextError),
    InvalidEntity(AzXmlTextError),
    InvalidElement(AzXmlTextError),
    InvalidAttribute(AzXmlTextError),
    InvalidCdata(AzXmlTextError),
    InvalidCharData(AzXmlTextError),
    UnknownToken(AzSvgParseErrorPosition),
}

/// Options on how to initially create the window
#[repr(C)]
#[pyclass(name = "WindowCreateOptions")]
pub struct AzWindowCreateOptions {
    #[pyo3(get, set)]
    pub state: AzWindowState,
    #[pyo3(get, set)]
    pub renderer_type: AzOptionRendererOptionsEnumWrapper,
    #[pyo3(get, set)]
    pub theme: AzOptionWindowThemeEnumWrapper,
    #[pyo3(get, set)]
    pub create_callback: AzOptionCallbackEnumWrapper,
    #[pyo3(get, set)]
    pub hot_reload: bool,
}

/// Defines the keyboard input focus target
#[repr(C, u8)]
pub enum AzFocusTarget {
    Id(AzDomNodeId),
    Path(AzFocusTargetPath),
    Previous,
    Next,
    First,
    Last,
    NoFocus,
}

/// Represents one single DOM node (node type, classes, ids and callbacks are stored here)
#[repr(C)]
#[pyclass(name = "NodeData")]
pub struct AzNodeData {
    #[pyo3(get, set)]
    pub node_type: AzNodeTypeEnumWrapper,
    #[pyo3(get, set)]
    pub dataset: AzOptionRefAnyEnumWrapper,
    #[pyo3(get, set)]
    pub ids_and_classes: AzIdOrClassVec,
    #[pyo3(get, set)]
    pub callbacks: AzCallbackDataVec,
    #[pyo3(get, set)]
    pub inline_css_props: AzNodeDataInlineCssPropertyVec,
    #[pyo3(get, set)]
    pub clip_mask: AzOptionImageMaskEnumWrapper,
    #[pyo3(get, set)]
    pub tab_index: AzOptionTabIndexEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `CssDeclaration` struct
#[repr(C, u8)]
pub enum AzCssDeclaration {
    Static(AzCssProperty),
    Dynamic(AzDynamicCssProperty),
}

/// Wrapper over a Rust-allocated `CssDeclaration`
#[repr(C)]
#[pyclass(name = "CssDeclarationVec")]
pub struct AzCssDeclarationVec {
    pub(crate) ptr: *const AzCssDeclarationEnumWrapper,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzCssDeclarationVecDestructorEnumWrapper,
}

/// Wrapper over a Rust-allocated `NodeDataVec`
#[repr(C)]
#[pyclass(name = "NodeDataVec")]
pub struct AzNodeDataVec {
    pub(crate) ptr: *const AzNodeData,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzNodeDataVecDestructorEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `XmlError` struct
#[repr(C, u8)]
pub enum AzXmlError {
    InvalidXmlPrefixUri(AzSvgParseErrorPosition),
    UnexpectedXmlUri(AzSvgParseErrorPosition),
    UnexpectedXmlnsUri(AzSvgParseErrorPosition),
    InvalidElementNamePrefix(AzSvgParseErrorPosition),
    DuplicatedNamespace(AzDuplicatedNamespaceError),
    UnknownNamespace(AzUnknownNamespaceError),
    UnexpectedCloseTag(AzUnexpectedCloseTagError),
    UnexpectedEntityCloseTag(AzSvgParseErrorPosition),
    UnknownEntityReference(AzUnknownEntityReferenceError),
    MalformedEntityReference(AzSvgParseErrorPosition),
    EntityReferenceLoop(AzSvgParseErrorPosition),
    InvalidAttributeValue(AzSvgParseErrorPosition),
    DuplicatedAttribute(AzDuplicatedAttributeError),
    NoRootNode,
    SizeLimit,
    ParserError(AzXmlParseError),
}

/// Re-export of rust-allocated (stack based) `Dom` struct
#[repr(C)]
#[pyclass(name = "Dom")]
pub struct AzDom {
    #[pyo3(get, set)]
    pub root: AzNodeData,
    #[pyo3(get, set)]
    pub children: AzDomVec,
    #[pyo3(get, set)]
    pub total_children: usize,
}

/// Re-export of rust-allocated (stack based) `CssRuleBlock` struct
#[repr(C)]
#[pyclass(name = "CssRuleBlock")]
pub struct AzCssRuleBlock {
    #[pyo3(get, set)]
    pub path: AzCssPath,
    #[pyo3(get, set)]
    pub declarations: AzCssDeclarationVec,
}

/// Re-export of rust-allocated (stack based) `StyledDom` struct
#[repr(C)]
#[pyclass(name = "StyledDom")]
pub struct AzStyledDom {
    #[pyo3(get, set)]
    pub root: AzNodeId,
    #[pyo3(get, set)]
    pub node_hierarchy: AzNodeVec,
    #[pyo3(get, set)]
    pub node_data: AzNodeDataVec,
    #[pyo3(get, set)]
    pub styled_nodes: AzStyledNodeVec,
    #[pyo3(get, set)]
    pub cascade_info: AzCascadeInfoVec,
    #[pyo3(get, set)]
    pub nodes_with_window_callbacks: AzNodeIdVec,
    #[pyo3(get, set)]
    pub nodes_with_not_callbacks: AzNodeIdVec,
    #[pyo3(get, set)]
    pub nodes_with_datasets_and_callbacks: AzNodeIdVec,
    #[pyo3(get, set)]
    pub tag_ids_to_node_ids: AzTagIdsToNodeIdsMappingVec,
    #[pyo3(get, set)]
    pub non_leaf_nodes: AzParentWithNodeDepthVec,
    #[pyo3(get, set)]
    pub css_property_cache: AzCssPropertyCache,
}

/// Wrapper over a Rust-allocated `CssRuleBlock`
#[repr(C)]
#[pyclass(name = "CssRuleBlockVec")]
pub struct AzCssRuleBlockVec {
    pub(crate) ptr: *const AzCssRuleBlock,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzCssRuleBlockVecDestructorEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `OptionDom` struct
#[repr(C, u8)]
pub enum AzOptionDom {
    None,
    Some(AzDom),
}

/// Re-export of rust-allocated (stack based) `ResultXmlXmlError` struct
#[repr(C, u8)]
pub enum AzResultXmlXmlError {
    Ok(AzXml),
    Err(AzXmlError),
}

/// Re-export of rust-allocated (stack based) `SvgParseError` struct
#[repr(C, u8)]
pub enum AzSvgParseError {
    InvalidFileSuffix,
    FileOpenFailed,
    NotAnUtf8Str,
    MalformedGZip,
    InvalidSize,
    ParsingFailed(AzXmlError),
}

/// <img src="../images/scrollbounds.png"/>
#[repr(C)]
#[pyclass(name = "IFrameCallbackReturn")]
pub struct AzIFrameCallbackReturn {
    #[pyo3(get, set)]
    pub dom: AzStyledDom,
    #[pyo3(get, set)]
    pub scroll_size: AzLogicalSize,
    #[pyo3(get, set)]
    pub scroll_offset: AzLogicalPosition,
    #[pyo3(get, set)]
    pub virtual_scroll_size: AzLogicalSize,
    #[pyo3(get, set)]
    pub virtual_scroll_offset: AzLogicalPosition,
}

/// Re-export of rust-allocated (stack based) `Stylesheet` struct
#[repr(C)]
#[pyclass(name = "Stylesheet")]
pub struct AzStylesheet {
    #[pyo3(get, set)]
    pub rules: AzCssRuleBlockVec,
}

/// Wrapper over a Rust-allocated `Stylesheet`
#[repr(C)]
#[pyclass(name = "StylesheetVec")]
pub struct AzStylesheetVec {
    pub(crate) ptr: *const AzStylesheet,
    #[pyo3(get, set)]
    pub len: usize,
    #[pyo3(get, set)]
    pub cap: usize,
    #[pyo3(get, set)]
    pub destructor: AzStylesheetVecDestructorEnumWrapper,
}

/// Re-export of rust-allocated (stack based) `ResultSvgXmlNodeSvgParseError` struct
#[repr(C, u8)]
pub enum AzResultSvgXmlNodeSvgParseError {
    Ok(AzSvgXmlNode),
    Err(AzSvgParseError),
}

/// Re-export of rust-allocated (stack based) `ResultSvgSvgParseError` struct
#[repr(C, u8)]
pub enum AzResultSvgSvgParseError {
    Ok(AzSvg),
    Err(AzSvgParseError),
}

/// Re-export of rust-allocated (stack based) `Css` struct
#[repr(C)]
#[pyclass(name = "Css")]
pub struct AzCss {
    #[pyo3(get, set)]
    pub stylesheets: AzStylesheetVec,
}

/// `AzAppLogLevelEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "AppLogLevel")]
pub struct AzAppLogLevelEnumWrapper {
    pub inner: AzAppLogLevel,
}

/// `AzLayoutSolverEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutSolver")]
pub struct AzLayoutSolverEnumWrapper {
    pub inner: AzLayoutSolver,
}

/// `AzVsyncEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "Vsync")]
pub struct AzVsyncEnumWrapper {
    pub inner: AzVsync,
}

/// `AzSrgbEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "Srgb")]
pub struct AzSrgbEnumWrapper {
    pub inner: AzSrgb,
}

/// `AzHwAccelerationEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "HwAcceleration")]
pub struct AzHwAccelerationEnumWrapper {
    pub inner: AzHwAcceleration,
}

/// `AzXWindowTypeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "XWindowType")]
pub struct AzXWindowTypeEnumWrapper {
    pub inner: AzXWindowType,
}

/// `AzVirtualKeyCodeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "VirtualKeyCode")]
pub struct AzVirtualKeyCodeEnumWrapper {
    pub inner: AzVirtualKeyCode,
}

/// `AzMouseCursorTypeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "MouseCursorType")]
pub struct AzMouseCursorTypeEnumWrapper {
    pub inner: AzMouseCursorType,
}

/// `AzRendererTypeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "RendererType")]
pub struct AzRendererTypeEnumWrapper {
    pub inner: AzRendererType,
}

/// `AzFullScreenModeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "FullScreenMode")]
pub struct AzFullScreenModeEnumWrapper {
    pub inner: AzFullScreenMode,
}

/// `AzWindowThemeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "WindowTheme")]
pub struct AzWindowThemeEnumWrapper {
    pub inner: AzWindowTheme,
}

/// `AzUpdateEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "Update")]
pub struct AzUpdateEnumWrapper {
    pub inner: AzUpdate,
}

/// `AzAnimationRepeatEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "AnimationRepeat")]
pub struct AzAnimationRepeatEnumWrapper {
    pub inner: AzAnimationRepeat,
}

/// `AzAnimationRepeatCountEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "AnimationRepeatCount")]
pub struct AzAnimationRepeatCountEnumWrapper {
    pub inner: AzAnimationRepeatCount,
}

/// `AzOnEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "On")]
pub struct AzOnEnumWrapper {
    pub inner: AzOn,
}

/// `AzHoverEventFilterEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "HoverEventFilter")]
pub struct AzHoverEventFilterEnumWrapper {
    pub inner: AzHoverEventFilter,
}

/// `AzFocusEventFilterEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "FocusEventFilter")]
pub struct AzFocusEventFilterEnumWrapper {
    pub inner: AzFocusEventFilter,
}

/// `AzWindowEventFilterEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "WindowEventFilter")]
pub struct AzWindowEventFilterEnumWrapper {
    pub inner: AzWindowEventFilter,
}

/// `AzComponentEventFilterEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ComponentEventFilter")]
pub struct AzComponentEventFilterEnumWrapper {
    pub inner: AzComponentEventFilter,
}

/// `AzApplicationEventFilterEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ApplicationEventFilter")]
pub struct AzApplicationEventFilterEnumWrapper {
    pub inner: AzApplicationEventFilter,
}

/// `AzTabIndexEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "TabIndex")]
pub struct AzTabIndexEnumWrapper {
    pub inner: AzTabIndex,
}

/// `AzNodeTypeKeyEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "NodeTypeKey")]
pub struct AzNodeTypeKeyEnumWrapper {
    pub inner: AzNodeTypeKey,
}

/// `AzCssPropertyTypeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssPropertyType")]
pub struct AzCssPropertyTypeEnumWrapper {
    pub inner: AzCssPropertyType,
}

/// `AzSizeMetricEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SizeMetric")]
pub struct AzSizeMetricEnumWrapper {
    pub inner: AzSizeMetric,
}

/// `AzBoxShadowClipModeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "BoxShadowClipMode")]
pub struct AzBoxShadowClipModeEnumWrapper {
    pub inner: AzBoxShadowClipMode,
}

/// `AzLayoutAlignContentEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutAlignContent")]
pub struct AzLayoutAlignContentEnumWrapper {
    pub inner: AzLayoutAlignContent,
}

/// `AzLayoutAlignItemsEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutAlignItems")]
pub struct AzLayoutAlignItemsEnumWrapper {
    pub inner: AzLayoutAlignItems,
}

/// `AzLayoutBoxSizingEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutBoxSizing")]
pub struct AzLayoutBoxSizingEnumWrapper {
    pub inner: AzLayoutBoxSizing,
}

/// `AzLayoutFlexDirectionEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutFlexDirection")]
pub struct AzLayoutFlexDirectionEnumWrapper {
    pub inner: AzLayoutFlexDirection,
}

/// `AzLayoutDisplayEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutDisplay")]
pub struct AzLayoutDisplayEnumWrapper {
    pub inner: AzLayoutDisplay,
}

/// `AzLayoutFloatEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutFloat")]
pub struct AzLayoutFloatEnumWrapper {
    pub inner: AzLayoutFloat,
}

/// `AzLayoutJustifyContentEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutJustifyContent")]
pub struct AzLayoutJustifyContentEnumWrapper {
    pub inner: AzLayoutJustifyContent,
}

/// `AzLayoutPositionEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutPosition")]
pub struct AzLayoutPositionEnumWrapper {
    pub inner: AzLayoutPosition,
}

/// `AzLayoutFlexWrapEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutFlexWrap")]
pub struct AzLayoutFlexWrapEnumWrapper {
    pub inner: AzLayoutFlexWrap,
}

/// `AzLayoutOverflowEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutOverflow")]
pub struct AzLayoutOverflowEnumWrapper {
    pub inner: AzLayoutOverflow,
}

/// `AzAngleMetricEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "AngleMetric")]
pub struct AzAngleMetricEnumWrapper {
    pub inner: AzAngleMetric,
}

/// `AzDirectionCornerEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "DirectionCorner")]
pub struct AzDirectionCornerEnumWrapper {
    pub inner: AzDirectionCorner,
}

/// `AzExtendModeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ExtendMode")]
pub struct AzExtendModeEnumWrapper {
    pub inner: AzExtendMode,
}

/// `AzShapeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "Shape")]
pub struct AzShapeEnumWrapper {
    pub inner: AzShape,
}

/// `AzRadialGradientSizeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "RadialGradientSize")]
pub struct AzRadialGradientSizeEnumWrapper {
    pub inner: AzRadialGradientSize,
}

/// `AzStyleBackgroundRepeatEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundRepeat")]
pub struct AzStyleBackgroundRepeatEnumWrapper {
    pub inner: AzStyleBackgroundRepeat,
}

/// `AzBorderStyleEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "BorderStyle")]
pub struct AzBorderStyleEnumWrapper {
    pub inner: AzBorderStyle,
}

/// `AzStyleCursorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleCursor")]
pub struct AzStyleCursorEnumWrapper {
    pub inner: AzStyleCursor,
}

/// `AzStyleBackfaceVisibilityEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackfaceVisibility")]
pub struct AzStyleBackfaceVisibilityEnumWrapper {
    pub inner: AzStyleBackfaceVisibility,
}

/// `AzStyleTextAlignEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleTextAlign")]
pub struct AzStyleTextAlignEnumWrapper {
    pub inner: AzStyleTextAlign,
}

/// `AzVertexAttributeTypeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "VertexAttributeType")]
pub struct AzVertexAttributeTypeEnumWrapper {
    pub inner: AzVertexAttributeType,
}

/// `AzIndexBufferFormatEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "IndexBufferFormat")]
pub struct AzIndexBufferFormatEnumWrapper {
    pub inner: AzIndexBufferFormat,
}

/// `AzGlTypeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "GlType")]
pub struct AzGlTypeEnumWrapper {
    pub inner: AzGlType,
}

/// `AzRawImageFormatEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "RawImageFormat")]
pub struct AzRawImageFormatEnumWrapper {
    pub inner: AzRawImageFormat,
}

/// `AzEncodeImageErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "EncodeImageError")]
pub struct AzEncodeImageErrorEnumWrapper {
    pub inner: AzEncodeImageError,
}

/// `AzDecodeImageErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "DecodeImageError")]
pub struct AzDecodeImageErrorEnumWrapper {
    pub inner: AzDecodeImageError,
}

/// `AzShapeRenderingEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ShapeRendering")]
pub struct AzShapeRenderingEnumWrapper {
    pub inner: AzShapeRendering,
}

/// `AzTextRenderingEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "TextRendering")]
pub struct AzTextRenderingEnumWrapper {
    pub inner: AzTextRendering,
}

/// `AzImageRenderingEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ImageRendering")]
pub struct AzImageRenderingEnumWrapper {
    pub inner: AzImageRendering,
}

/// `AzFontDatabaseEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "FontDatabase")]
pub struct AzFontDatabaseEnumWrapper {
    pub inner: AzFontDatabase,
}

/// `AzIndentEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "Indent")]
pub struct AzIndentEnumWrapper {
    pub inner: AzIndent,
}

/// `AzSvgFitToEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgFitTo")]
pub struct AzSvgFitToEnumWrapper {
    pub inner: AzSvgFitTo,
}

/// `AzSvgFillRuleEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgFillRule")]
pub struct AzSvgFillRuleEnumWrapper {
    pub inner: AzSvgFillRule,
}

/// `AzSvgLineJoinEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgLineJoin")]
pub struct AzSvgLineJoinEnumWrapper {
    pub inner: AzSvgLineJoin,
}

/// `AzSvgLineCapEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgLineCap")]
pub struct AzSvgLineCapEnumWrapper {
    pub inner: AzSvgLineCap,
}

/// `AzMsgBoxIconEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "MsgBoxIcon")]
pub struct AzMsgBoxIconEnumWrapper {
    pub inner: AzMsgBoxIcon,
}

/// `AzMsgBoxYesNoEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "MsgBoxYesNo")]
pub struct AzMsgBoxYesNoEnumWrapper {
    pub inner: AzMsgBoxYesNo,
}

/// `AzMsgBoxOkCancelEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "MsgBoxOkCancel")]
pub struct AzMsgBoxOkCancelEnumWrapper {
    pub inner: AzMsgBoxOkCancel,
}

/// `AzTerminateTimerEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "TerminateTimer")]
pub struct AzTerminateTimerEnumWrapper {
    pub inner: AzTerminateTimer,
}

/// `AzStyleFontFamilyVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleFontFamilyVecDestructor")]
pub struct AzStyleFontFamilyVecDestructorEnumWrapper {
    pub inner: AzStyleFontFamilyVecDestructor,
}

/// `AzTesselatedSvgNodeVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "TesselatedSvgNodeVecDestructor")]
pub struct AzTesselatedSvgNodeVecDestructorEnumWrapper {
    pub inner: AzTesselatedSvgNodeVecDestructor,
}

/// `AzXmlNodeVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "XmlNodeVecDestructor")]
pub struct AzXmlNodeVecDestructorEnumWrapper {
    pub inner: AzXmlNodeVecDestructor,
}

/// `AzFmtArgVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "FmtArgVecDestructor")]
pub struct AzFmtArgVecDestructorEnumWrapper {
    pub inner: AzFmtArgVecDestructor,
}

/// `AzInlineLineVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "InlineLineVecDestructor")]
pub struct AzInlineLineVecDestructorEnumWrapper {
    pub inner: AzInlineLineVecDestructor,
}

/// `AzInlineWordVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "InlineWordVecDestructor")]
pub struct AzInlineWordVecDestructorEnumWrapper {
    pub inner: AzInlineWordVecDestructor,
}

/// `AzInlineGlyphVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "InlineGlyphVecDestructor")]
pub struct AzInlineGlyphVecDestructorEnumWrapper {
    pub inner: AzInlineGlyphVecDestructor,
}

/// `AzInlineTextHitVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "InlineTextHitVecDestructor")]
pub struct AzInlineTextHitVecDestructorEnumWrapper {
    pub inner: AzInlineTextHitVecDestructor,
}

/// `AzMonitorVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "MonitorVecDestructor")]
pub struct AzMonitorVecDestructorEnumWrapper {
    pub inner: AzMonitorVecDestructor,
}

/// `AzVideoModeVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "VideoModeVecDestructor")]
pub struct AzVideoModeVecDestructorEnumWrapper {
    pub inner: AzVideoModeVecDestructor,
}

/// `AzDomVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "DomVecDestructor")]
pub struct AzDomVecDestructorEnumWrapper {
    pub inner: AzDomVecDestructor,
}

/// `AzIdOrClassVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "IdOrClassVecDestructor")]
pub struct AzIdOrClassVecDestructorEnumWrapper {
    pub inner: AzIdOrClassVecDestructor,
}

/// `AzNodeDataInlineCssPropertyVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "NodeDataInlineCssPropertyVecDestructor")]
pub struct AzNodeDataInlineCssPropertyVecDestructorEnumWrapper {
    pub inner: AzNodeDataInlineCssPropertyVecDestructor,
}

/// `AzStyleBackgroundContentVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundContentVecDestructor")]
pub struct AzStyleBackgroundContentVecDestructorEnumWrapper {
    pub inner: AzStyleBackgroundContentVecDestructor,
}

/// `AzStyleBackgroundPositionVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundPositionVecDestructor")]
pub struct AzStyleBackgroundPositionVecDestructorEnumWrapper {
    pub inner: AzStyleBackgroundPositionVecDestructor,
}

/// `AzStyleBackgroundRepeatVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundRepeatVecDestructor")]
pub struct AzStyleBackgroundRepeatVecDestructorEnumWrapper {
    pub inner: AzStyleBackgroundRepeatVecDestructor,
}

/// `AzStyleBackgroundSizeVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundSizeVecDestructor")]
pub struct AzStyleBackgroundSizeVecDestructorEnumWrapper {
    pub inner: AzStyleBackgroundSizeVecDestructor,
}

/// `AzStyleTransformVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleTransformVecDestructor")]
pub struct AzStyleTransformVecDestructorEnumWrapper {
    pub inner: AzStyleTransformVecDestructor,
}

/// `AzCssPropertyVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssPropertyVecDestructor")]
pub struct AzCssPropertyVecDestructorEnumWrapper {
    pub inner: AzCssPropertyVecDestructor,
}

/// `AzSvgMultiPolygonVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgMultiPolygonVecDestructor")]
pub struct AzSvgMultiPolygonVecDestructorEnumWrapper {
    pub inner: AzSvgMultiPolygonVecDestructor,
}

/// `AzSvgPathVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgPathVecDestructor")]
pub struct AzSvgPathVecDestructorEnumWrapper {
    pub inner: AzSvgPathVecDestructor,
}

/// `AzVertexAttributeVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "VertexAttributeVecDestructor")]
pub struct AzVertexAttributeVecDestructorEnumWrapper {
    pub inner: AzVertexAttributeVecDestructor,
}

/// `AzSvgPathElementVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgPathElementVecDestructor")]
pub struct AzSvgPathElementVecDestructorEnumWrapper {
    pub inner: AzSvgPathElementVecDestructor,
}

/// `AzSvgVertexVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgVertexVecDestructor")]
pub struct AzSvgVertexVecDestructorEnumWrapper {
    pub inner: AzSvgVertexVecDestructor,
}

/// `AzU32VecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "U32VecDestructor")]
pub struct AzU32VecDestructorEnumWrapper {
    pub inner: AzU32VecDestructor,
}

/// `AzXWindowTypeVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "XWindowTypeVecDestructor")]
pub struct AzXWindowTypeVecDestructorEnumWrapper {
    pub inner: AzXWindowTypeVecDestructor,
}

/// `AzVirtualKeyCodeVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "VirtualKeyCodeVecDestructor")]
pub struct AzVirtualKeyCodeVecDestructorEnumWrapper {
    pub inner: AzVirtualKeyCodeVecDestructor,
}

/// `AzCascadeInfoVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CascadeInfoVecDestructor")]
pub struct AzCascadeInfoVecDestructorEnumWrapper {
    pub inner: AzCascadeInfoVecDestructor,
}

/// `AzScanCodeVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ScanCodeVecDestructor")]
pub struct AzScanCodeVecDestructorEnumWrapper {
    pub inner: AzScanCodeVecDestructor,
}

/// `AzCssDeclarationVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssDeclarationVecDestructor")]
pub struct AzCssDeclarationVecDestructorEnumWrapper {
    pub inner: AzCssDeclarationVecDestructor,
}

/// `AzCssPathSelectorVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssPathSelectorVecDestructor")]
pub struct AzCssPathSelectorVecDestructorEnumWrapper {
    pub inner: AzCssPathSelectorVecDestructor,
}

/// `AzStylesheetVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StylesheetVecDestructor")]
pub struct AzStylesheetVecDestructorEnumWrapper {
    pub inner: AzStylesheetVecDestructor,
}

/// `AzCssRuleBlockVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssRuleBlockVecDestructor")]
pub struct AzCssRuleBlockVecDestructorEnumWrapper {
    pub inner: AzCssRuleBlockVecDestructor,
}

/// `AzF32VecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "F32VecDestructor")]
pub struct AzF32VecDestructorEnumWrapper {
    pub inner: AzF32VecDestructor,
}

/// `AzU16VecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "U16VecDestructor")]
pub struct AzU16VecDestructorEnumWrapper {
    pub inner: AzU16VecDestructor,
}

/// `AzU8VecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "U8VecDestructor")]
pub struct AzU8VecDestructorEnumWrapper {
    pub inner: AzU8VecDestructor,
}

/// `AzCallbackDataVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CallbackDataVecDestructor")]
pub struct AzCallbackDataVecDestructorEnumWrapper {
    pub inner: AzCallbackDataVecDestructor,
}

/// `AzDebugMessageVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "DebugMessageVecDestructor")]
pub struct AzDebugMessageVecDestructorEnumWrapper {
    pub inner: AzDebugMessageVecDestructor,
}

/// `AzGLuintVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "GLuintVecDestructor")]
pub struct AzGLuintVecDestructorEnumWrapper {
    pub inner: AzGLuintVecDestructor,
}

/// `AzGLintVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "GLintVecDestructor")]
pub struct AzGLintVecDestructorEnumWrapper {
    pub inner: AzGLintVecDestructor,
}

/// `AzStringVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StringVecDestructor")]
pub struct AzStringVecDestructorEnumWrapper {
    pub inner: AzStringVecDestructor,
}

/// `AzStringPairVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StringPairVecDestructor")]
pub struct AzStringPairVecDestructorEnumWrapper {
    pub inner: AzStringPairVecDestructor,
}

/// `AzNormalizedLinearColorStopVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "NormalizedLinearColorStopVecDestructor")]
pub struct AzNormalizedLinearColorStopVecDestructorEnumWrapper {
    pub inner: AzNormalizedLinearColorStopVecDestructor,
}

/// `AzNormalizedRadialColorStopVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "NormalizedRadialColorStopVecDestructor")]
pub struct AzNormalizedRadialColorStopVecDestructorEnumWrapper {
    pub inner: AzNormalizedRadialColorStopVecDestructor,
}

/// `AzNodeIdVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "NodeIdVecDestructor")]
pub struct AzNodeIdVecDestructorEnumWrapper {
    pub inner: AzNodeIdVecDestructor,
}

/// `AzNodeVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "NodeVecDestructor")]
pub struct AzNodeVecDestructorEnumWrapper {
    pub inner: AzNodeVecDestructor,
}

/// `AzStyledNodeVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyledNodeVecDestructor")]
pub struct AzStyledNodeVecDestructorEnumWrapper {
    pub inner: AzStyledNodeVecDestructor,
}

/// `AzTagIdsToNodeIdsMappingVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "TagIdsToNodeIdsMappingVecDestructor")]
pub struct AzTagIdsToNodeIdsMappingVecDestructorEnumWrapper {
    pub inner: AzTagIdsToNodeIdsMappingVecDestructor,
}

/// `AzParentWithNodeDepthVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ParentWithNodeDepthVecDestructor")]
pub struct AzParentWithNodeDepthVecDestructorEnumWrapper {
    pub inner: AzParentWithNodeDepthVecDestructor,
}

/// `AzNodeDataVecDestructorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "NodeDataVecDestructor")]
pub struct AzNodeDataVecDestructorEnumWrapper {
    pub inner: AzNodeDataVecDestructor,
}

/// `AzOptionI16EnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionI16")]
pub struct AzOptionI16EnumWrapper {
    pub inner: AzOptionI16,
}

/// `AzOptionU16EnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionU16")]
pub struct AzOptionU16EnumWrapper {
    pub inner: AzOptionU16,
}

/// `AzOptionU32EnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionU32")]
pub struct AzOptionU32EnumWrapper {
    pub inner: AzOptionU32,
}

/// `AzOptionHwndHandleEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionHwndHandle")]
pub struct AzOptionHwndHandleEnumWrapper {
    pub inner: AzOptionHwndHandle,
}

/// `AzOptionX11VisualEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionX11Visual")]
pub struct AzOptionX11VisualEnumWrapper {
    pub inner: AzOptionX11Visual,
}

/// `AzOptionI32EnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionI32")]
pub struct AzOptionI32EnumWrapper {
    pub inner: AzOptionI32,
}

/// `AzOptionF32EnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionF32")]
pub struct AzOptionF32EnumWrapper {
    pub inner: AzOptionF32,
}

/// `AzOptionCharEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionChar")]
pub struct AzOptionCharEnumWrapper {
    pub inner: AzOptionChar,
}

/// `AzOptionUsizeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionUsize")]
pub struct AzOptionUsizeEnumWrapper {
    pub inner: AzOptionUsize,
}

/// `AzRawWindowHandleEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "RawWindowHandle")]
pub struct AzRawWindowHandleEnumWrapper {
    pub inner: AzRawWindowHandle,
}

/// `AzAcceleratorKeyEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "AcceleratorKey")]
pub struct AzAcceleratorKeyEnumWrapper {
    pub inner: AzAcceleratorKey,
}

/// `AzCursorPositionEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CursorPosition")]
pub struct AzCursorPositionEnumWrapper {
    pub inner: AzCursorPosition,
}

/// `AzWindowPositionEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "WindowPosition")]
pub struct AzWindowPositionEnumWrapper {
    pub inner: AzWindowPosition,
}

/// `AzImePositionEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ImePosition")]
pub struct AzImePositionEnumWrapper {
    pub inner: AzImePosition,
}

/// `AzPositionInfoEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "PositionInfo")]
pub struct AzPositionInfoEnumWrapper {
    pub inner: AzPositionInfo,
}

/// `AzNotEventFilterEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "NotEventFilter")]
pub struct AzNotEventFilterEnumWrapper {
    pub inner: AzNotEventFilter,
}

/// `AzCssNthChildSelectorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssNthChildSelector")]
pub struct AzCssNthChildSelectorEnumWrapper {
    pub inner: AzCssNthChildSelector,
}

/// `AzDirectionEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "Direction")]
pub struct AzDirectionEnumWrapper {
    pub inner: AzDirection,
}

/// `AzBackgroundPositionHorizontalEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "BackgroundPositionHorizontal")]
pub struct AzBackgroundPositionHorizontalEnumWrapper {
    pub inner: AzBackgroundPositionHorizontal,
}

/// `AzBackgroundPositionVerticalEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "BackgroundPositionVertical")]
pub struct AzBackgroundPositionVerticalEnumWrapper {
    pub inner: AzBackgroundPositionVertical,
}

/// `AzStyleBackgroundSizeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundSize")]
pub struct AzStyleBackgroundSizeEnumWrapper {
    pub inner: AzStyleBackgroundSize,
}

/// `AzStyleBoxShadowValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBoxShadowValue")]
pub struct AzStyleBoxShadowValueEnumWrapper {
    pub inner: AzStyleBoxShadowValue,
}

/// `AzLayoutAlignContentValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutAlignContentValue")]
pub struct AzLayoutAlignContentValueEnumWrapper {
    pub inner: AzLayoutAlignContentValue,
}

/// `AzLayoutAlignItemsValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutAlignItemsValue")]
pub struct AzLayoutAlignItemsValueEnumWrapper {
    pub inner: AzLayoutAlignItemsValue,
}

/// `AzLayoutBottomValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutBottomValue")]
pub struct AzLayoutBottomValueEnumWrapper {
    pub inner: AzLayoutBottomValue,
}

/// `AzLayoutBoxSizingValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutBoxSizingValue")]
pub struct AzLayoutBoxSizingValueEnumWrapper {
    pub inner: AzLayoutBoxSizingValue,
}

/// `AzLayoutFlexDirectionValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutFlexDirectionValue")]
pub struct AzLayoutFlexDirectionValueEnumWrapper {
    pub inner: AzLayoutFlexDirectionValue,
}

/// `AzLayoutDisplayValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutDisplayValue")]
pub struct AzLayoutDisplayValueEnumWrapper {
    pub inner: AzLayoutDisplayValue,
}

/// `AzLayoutFlexGrowValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutFlexGrowValue")]
pub struct AzLayoutFlexGrowValueEnumWrapper {
    pub inner: AzLayoutFlexGrowValue,
}

/// `AzLayoutFlexShrinkValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutFlexShrinkValue")]
pub struct AzLayoutFlexShrinkValueEnumWrapper {
    pub inner: AzLayoutFlexShrinkValue,
}

/// `AzLayoutFloatValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutFloatValue")]
pub struct AzLayoutFloatValueEnumWrapper {
    pub inner: AzLayoutFloatValue,
}

/// `AzLayoutHeightValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutHeightValue")]
pub struct AzLayoutHeightValueEnumWrapper {
    pub inner: AzLayoutHeightValue,
}

/// `AzLayoutJustifyContentValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutJustifyContentValue")]
pub struct AzLayoutJustifyContentValueEnumWrapper {
    pub inner: AzLayoutJustifyContentValue,
}

/// `AzLayoutLeftValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutLeftValue")]
pub struct AzLayoutLeftValueEnumWrapper {
    pub inner: AzLayoutLeftValue,
}

/// `AzLayoutMarginBottomValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutMarginBottomValue")]
pub struct AzLayoutMarginBottomValueEnumWrapper {
    pub inner: AzLayoutMarginBottomValue,
}

/// `AzLayoutMarginLeftValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutMarginLeftValue")]
pub struct AzLayoutMarginLeftValueEnumWrapper {
    pub inner: AzLayoutMarginLeftValue,
}

/// `AzLayoutMarginRightValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutMarginRightValue")]
pub struct AzLayoutMarginRightValueEnumWrapper {
    pub inner: AzLayoutMarginRightValue,
}

/// `AzLayoutMarginTopValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutMarginTopValue")]
pub struct AzLayoutMarginTopValueEnumWrapper {
    pub inner: AzLayoutMarginTopValue,
}

/// `AzLayoutMaxHeightValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutMaxHeightValue")]
pub struct AzLayoutMaxHeightValueEnumWrapper {
    pub inner: AzLayoutMaxHeightValue,
}

/// `AzLayoutMaxWidthValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutMaxWidthValue")]
pub struct AzLayoutMaxWidthValueEnumWrapper {
    pub inner: AzLayoutMaxWidthValue,
}

/// `AzLayoutMinHeightValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutMinHeightValue")]
pub struct AzLayoutMinHeightValueEnumWrapper {
    pub inner: AzLayoutMinHeightValue,
}

/// `AzLayoutMinWidthValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutMinWidthValue")]
pub struct AzLayoutMinWidthValueEnumWrapper {
    pub inner: AzLayoutMinWidthValue,
}

/// `AzLayoutPaddingBottomValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutPaddingBottomValue")]
pub struct AzLayoutPaddingBottomValueEnumWrapper {
    pub inner: AzLayoutPaddingBottomValue,
}

/// `AzLayoutPaddingLeftValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutPaddingLeftValue")]
pub struct AzLayoutPaddingLeftValueEnumWrapper {
    pub inner: AzLayoutPaddingLeftValue,
}

/// `AzLayoutPaddingRightValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutPaddingRightValue")]
pub struct AzLayoutPaddingRightValueEnumWrapper {
    pub inner: AzLayoutPaddingRightValue,
}

/// `AzLayoutPaddingTopValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutPaddingTopValue")]
pub struct AzLayoutPaddingTopValueEnumWrapper {
    pub inner: AzLayoutPaddingTopValue,
}

/// `AzLayoutPositionValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutPositionValue")]
pub struct AzLayoutPositionValueEnumWrapper {
    pub inner: AzLayoutPositionValue,
}

/// `AzLayoutRightValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutRightValue")]
pub struct AzLayoutRightValueEnumWrapper {
    pub inner: AzLayoutRightValue,
}

/// `AzLayoutTopValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutTopValue")]
pub struct AzLayoutTopValueEnumWrapper {
    pub inner: AzLayoutTopValue,
}

/// `AzLayoutWidthValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutWidthValue")]
pub struct AzLayoutWidthValueEnumWrapper {
    pub inner: AzLayoutWidthValue,
}

/// `AzLayoutFlexWrapValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutFlexWrapValue")]
pub struct AzLayoutFlexWrapValueEnumWrapper {
    pub inner: AzLayoutFlexWrapValue,
}

/// `AzLayoutOverflowValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutOverflowValue")]
pub struct AzLayoutOverflowValueEnumWrapper {
    pub inner: AzLayoutOverflowValue,
}

/// `AzStyleBorderBottomColorValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderBottomColorValue")]
pub struct AzStyleBorderBottomColorValueEnumWrapper {
    pub inner: AzStyleBorderBottomColorValue,
}

/// `AzStyleBorderBottomLeftRadiusValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderBottomLeftRadiusValue")]
pub struct AzStyleBorderBottomLeftRadiusValueEnumWrapper {
    pub inner: AzStyleBorderBottomLeftRadiusValue,
}

/// `AzStyleBorderBottomRightRadiusValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderBottomRightRadiusValue")]
pub struct AzStyleBorderBottomRightRadiusValueEnumWrapper {
    pub inner: AzStyleBorderBottomRightRadiusValue,
}

/// `AzStyleBorderBottomStyleValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderBottomStyleValue")]
pub struct AzStyleBorderBottomStyleValueEnumWrapper {
    pub inner: AzStyleBorderBottomStyleValue,
}

/// `AzLayoutBorderBottomWidthValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutBorderBottomWidthValue")]
pub struct AzLayoutBorderBottomWidthValueEnumWrapper {
    pub inner: AzLayoutBorderBottomWidthValue,
}

/// `AzStyleBorderLeftColorValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderLeftColorValue")]
pub struct AzStyleBorderLeftColorValueEnumWrapper {
    pub inner: AzStyleBorderLeftColorValue,
}

/// `AzStyleBorderLeftStyleValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderLeftStyleValue")]
pub struct AzStyleBorderLeftStyleValueEnumWrapper {
    pub inner: AzStyleBorderLeftStyleValue,
}

/// `AzLayoutBorderLeftWidthValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutBorderLeftWidthValue")]
pub struct AzLayoutBorderLeftWidthValueEnumWrapper {
    pub inner: AzLayoutBorderLeftWidthValue,
}

/// `AzStyleBorderRightColorValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderRightColorValue")]
pub struct AzStyleBorderRightColorValueEnumWrapper {
    pub inner: AzStyleBorderRightColorValue,
}

/// `AzStyleBorderRightStyleValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderRightStyleValue")]
pub struct AzStyleBorderRightStyleValueEnumWrapper {
    pub inner: AzStyleBorderRightStyleValue,
}

/// `AzLayoutBorderRightWidthValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutBorderRightWidthValue")]
pub struct AzLayoutBorderRightWidthValueEnumWrapper {
    pub inner: AzLayoutBorderRightWidthValue,
}

/// `AzStyleBorderTopColorValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderTopColorValue")]
pub struct AzStyleBorderTopColorValueEnumWrapper {
    pub inner: AzStyleBorderTopColorValue,
}

/// `AzStyleBorderTopLeftRadiusValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderTopLeftRadiusValue")]
pub struct AzStyleBorderTopLeftRadiusValueEnumWrapper {
    pub inner: AzStyleBorderTopLeftRadiusValue,
}

/// `AzStyleBorderTopRightRadiusValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderTopRightRadiusValue")]
pub struct AzStyleBorderTopRightRadiusValueEnumWrapper {
    pub inner: AzStyleBorderTopRightRadiusValue,
}

/// `AzStyleBorderTopStyleValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBorderTopStyleValue")]
pub struct AzStyleBorderTopStyleValueEnumWrapper {
    pub inner: AzStyleBorderTopStyleValue,
}

/// `AzLayoutBorderTopWidthValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutBorderTopWidthValue")]
pub struct AzLayoutBorderTopWidthValueEnumWrapper {
    pub inner: AzLayoutBorderTopWidthValue,
}

/// `AzStyleCursorValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleCursorValue")]
pub struct AzStyleCursorValueEnumWrapper {
    pub inner: AzStyleCursorValue,
}

/// `AzStyleFontSizeValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleFontSizeValue")]
pub struct AzStyleFontSizeValueEnumWrapper {
    pub inner: AzStyleFontSizeValue,
}

/// `AzStyleLetterSpacingValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleLetterSpacingValue")]
pub struct AzStyleLetterSpacingValueEnumWrapper {
    pub inner: AzStyleLetterSpacingValue,
}

/// `AzStyleLineHeightValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleLineHeightValue")]
pub struct AzStyleLineHeightValueEnumWrapper {
    pub inner: AzStyleLineHeightValue,
}

/// `AzStyleTabWidthValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleTabWidthValue")]
pub struct AzStyleTabWidthValueEnumWrapper {
    pub inner: AzStyleTabWidthValue,
}

/// `AzStyleTextAlignValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleTextAlignValue")]
pub struct AzStyleTextAlignValueEnumWrapper {
    pub inner: AzStyleTextAlignValue,
}

/// `AzStyleTextColorValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleTextColorValue")]
pub struct AzStyleTextColorValueEnumWrapper {
    pub inner: AzStyleTextColorValue,
}

/// `AzStyleWordSpacingValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleWordSpacingValue")]
pub struct AzStyleWordSpacingValueEnumWrapper {
    pub inner: AzStyleWordSpacingValue,
}

/// `AzStyleOpacityValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleOpacityValue")]
pub struct AzStyleOpacityValueEnumWrapper {
    pub inner: AzStyleOpacityValue,
}

/// `AzStyleTransformOriginValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleTransformOriginValue")]
pub struct AzStyleTransformOriginValueEnumWrapper {
    pub inner: AzStyleTransformOriginValue,
}

/// `AzStylePerspectiveOriginValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StylePerspectiveOriginValue")]
pub struct AzStylePerspectiveOriginValueEnumWrapper {
    pub inner: AzStylePerspectiveOriginValue,
}

/// `AzStyleBackfaceVisibilityValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackfaceVisibilityValue")]
pub struct AzStyleBackfaceVisibilityValueEnumWrapper {
    pub inner: AzStyleBackfaceVisibilityValue,
}

/// `AzDurationEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "Duration")]
pub struct AzDurationEnumWrapper {
    pub inner: AzDuration,
}

/// `AzThreadSendMsgEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ThreadSendMsg")]
pub struct AzThreadSendMsgEnumWrapper {
    pub inner: AzThreadSendMsg,
}

/// `AzOptionPositionInfoEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionPositionInfo")]
pub struct AzOptionPositionInfoEnumWrapper {
    pub inner: AzOptionPositionInfo,
}

/// `AzOptionTimerIdEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionTimerId")]
pub struct AzOptionTimerIdEnumWrapper {
    pub inner: AzOptionTimerId,
}

/// `AzOptionThreadIdEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionThreadId")]
pub struct AzOptionThreadIdEnumWrapper {
    pub inner: AzOptionThreadId,
}

/// `AzOptionImageRefEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionImageRef")]
pub struct AzOptionImageRefEnumWrapper {
    pub inner: AzOptionImageRef,
}

/// `AzOptionFontRefEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionFontRef")]
pub struct AzOptionFontRefEnumWrapper {
    pub inner: AzOptionFontRef,
}

/// `AzOptionSystemClipboardEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionSystemClipboard")]
pub struct AzOptionSystemClipboardEnumWrapper {
    pub inner: AzOptionSystemClipboard,
}

/// `AzOptionFileEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionFile")]
pub struct AzOptionFileEnumWrapper {
    pub inner: AzOptionFile,
}

/// `AzOptionGlEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionGl")]
pub struct AzOptionGlEnumWrapper {
    pub inner: AzOptionGl,
}

/// `AzOptionPercentageValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionPercentageValue")]
pub struct AzOptionPercentageValueEnumWrapper {
    pub inner: AzOptionPercentageValue,
}

/// `AzOptionAngleValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionAngleValue")]
pub struct AzOptionAngleValueEnumWrapper {
    pub inner: AzOptionAngleValue,
}

/// `AzOptionRendererOptionsEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionRendererOptions")]
pub struct AzOptionRendererOptionsEnumWrapper {
    pub inner: AzOptionRendererOptions,
}

/// `AzOptionCallbackEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionCallback")]
pub struct AzOptionCallbackEnumWrapper {
    pub inner: AzOptionCallback,
}

/// `AzOptionThreadSendMsgEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionThreadSendMsg")]
pub struct AzOptionThreadSendMsgEnumWrapper {
    pub inner: AzOptionThreadSendMsg,
}

/// `AzOptionLayoutRectEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionLayoutRect")]
pub struct AzOptionLayoutRectEnumWrapper {
    pub inner: AzOptionLayoutRect,
}

/// `AzOptionRefAnyEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionRefAny")]
pub struct AzOptionRefAnyEnumWrapper {
    pub inner: AzOptionRefAny,
}

/// `AzOptionLayoutPointEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionLayoutPoint")]
pub struct AzOptionLayoutPointEnumWrapper {
    pub inner: AzOptionLayoutPoint,
}

/// `AzOptionLayoutSizeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionLayoutSize")]
pub struct AzOptionLayoutSizeEnumWrapper {
    pub inner: AzOptionLayoutSize,
}

/// `AzOptionWindowThemeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionWindowTheme")]
pub struct AzOptionWindowThemeEnumWrapper {
    pub inner: AzOptionWindowTheme,
}

/// `AzOptionNodeIdEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionNodeId")]
pub struct AzOptionNodeIdEnumWrapper {
    pub inner: AzOptionNodeId,
}

/// `AzOptionDomNodeIdEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionDomNodeId")]
pub struct AzOptionDomNodeIdEnumWrapper {
    pub inner: AzOptionDomNodeId,
}

/// `AzOptionColorUEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionColorU")]
pub struct AzOptionColorUEnumWrapper {
    pub inner: AzOptionColorU,
}

/// `AzOptionSvgDashPatternEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionSvgDashPattern")]
pub struct AzOptionSvgDashPatternEnumWrapper {
    pub inner: AzOptionSvgDashPattern,
}

/// `AzOptionLogicalPositionEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionLogicalPosition")]
pub struct AzOptionLogicalPositionEnumWrapper {
    pub inner: AzOptionLogicalPosition,
}

/// `AzOptionPhysicalPositionI32EnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionPhysicalPositionI32")]
pub struct AzOptionPhysicalPositionI32EnumWrapper {
    pub inner: AzOptionPhysicalPositionI32,
}

/// `AzOptionMouseCursorTypeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionMouseCursorType")]
pub struct AzOptionMouseCursorTypeEnumWrapper {
    pub inner: AzOptionMouseCursorType,
}

/// `AzOptionLogicalSizeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionLogicalSize")]
pub struct AzOptionLogicalSizeEnumWrapper {
    pub inner: AzOptionLogicalSize,
}

/// `AzOptionVirtualKeyCodeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionVirtualKeyCode")]
pub struct AzOptionVirtualKeyCodeEnumWrapper {
    pub inner: AzOptionVirtualKeyCode,
}

/// `AzOptionImageMaskEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionImageMask")]
pub struct AzOptionImageMaskEnumWrapper {
    pub inner: AzOptionImageMask,
}

/// `AzOptionTabIndexEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionTabIndex")]
pub struct AzOptionTabIndexEnumWrapper {
    pub inner: AzOptionTabIndex,
}

/// `AzOptionTagIdEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionTagId")]
pub struct AzOptionTagIdEnumWrapper {
    pub inner: AzOptionTagId,
}

/// `AzOptionDurationEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionDuration")]
pub struct AzOptionDurationEnumWrapper {
    pub inner: AzOptionDuration,
}

/// `AzOptionU8VecEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionU8Vec")]
pub struct AzOptionU8VecEnumWrapper {
    pub inner: AzOptionU8Vec,
}

/// `AzOptionU8VecRefEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionU8VecRef")]
pub struct AzOptionU8VecRefEnumWrapper {
    pub inner: AzOptionU8VecRef,
}

/// `AzResultU8VecEncodeImageErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ResultU8VecEncodeImageError")]
pub struct AzResultU8VecEncodeImageErrorEnumWrapper {
    pub inner: AzResultU8VecEncodeImageError,
}

/// `AzWindowIconEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "WindowIcon")]
pub struct AzWindowIconEnumWrapper {
    pub inner: AzWindowIcon,
}

/// `AzAnimationEasingEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "AnimationEasing")]
pub struct AzAnimationEasingEnumWrapper {
    pub inner: AzAnimationEasing,
}

/// `AzEventFilterEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "EventFilter")]
pub struct AzEventFilterEnumWrapper {
    pub inner: AzEventFilter,
}

/// `AzCssPathPseudoSelectorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssPathPseudoSelector")]
pub struct AzCssPathPseudoSelectorEnumWrapper {
    pub inner: AzCssPathPseudoSelector,
}

/// `AzAnimationInterpolationFunctionEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "AnimationInterpolationFunction")]
pub struct AzAnimationInterpolationFunctionEnumWrapper {
    pub inner: AzAnimationInterpolationFunction,
}

/// `AzStyleTransformEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleTransform")]
pub struct AzStyleTransformEnumWrapper {
    pub inner: AzStyleTransform,
}

/// `AzStyleBackgroundPositionVecValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundPositionVecValue")]
pub struct AzStyleBackgroundPositionVecValueEnumWrapper {
    pub inner: AzStyleBackgroundPositionVecValue,
}

/// `AzStyleBackgroundRepeatVecValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundRepeatVecValue")]
pub struct AzStyleBackgroundRepeatVecValueEnumWrapper {
    pub inner: AzStyleBackgroundRepeatVecValue,
}

/// `AzStyleBackgroundSizeVecValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundSizeVecValue")]
pub struct AzStyleBackgroundSizeVecValueEnumWrapper {
    pub inner: AzStyleBackgroundSizeVecValue,
}

/// `AzRawImageDataEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "RawImageData")]
pub struct AzRawImageDataEnumWrapper {
    pub inner: AzRawImageData,
}

/// `AzSvgPathElementEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgPathElement")]
pub struct AzSvgPathElementEnumWrapper {
    pub inner: AzSvgPathElement,
}

/// `AzInstantEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "Instant")]
pub struct AzInstantEnumWrapper {
    pub inner: AzInstant,
}

/// `AzThreadReceiveMsgEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ThreadReceiveMsg")]
pub struct AzThreadReceiveMsgEnumWrapper {
    pub inner: AzThreadReceiveMsg,
}

/// `AzOptionMouseStateEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionMouseState")]
pub struct AzOptionMouseStateEnumWrapper {
    pub inner: AzOptionMouseState,
}

/// `AzOptionKeyboardStateEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionKeyboardState")]
pub struct AzOptionKeyboardStateEnumWrapper {
    pub inner: AzOptionKeyboardState,
}

/// `AzOptionStringVecEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionStringVec")]
pub struct AzOptionStringVecEnumWrapper {
    pub inner: AzOptionStringVec,
}

/// `AzOptionThreadReceiveMsgEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionThreadReceiveMsg")]
pub struct AzOptionThreadReceiveMsgEnumWrapper {
    pub inner: AzOptionThreadReceiveMsg,
}

/// `AzOptionTaskBarIconEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionTaskBarIcon")]
pub struct AzOptionTaskBarIconEnumWrapper {
    pub inner: AzOptionTaskBarIcon,
}

/// `AzOptionWindowIconEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionWindowIcon")]
pub struct AzOptionWindowIconEnumWrapper {
    pub inner: AzOptionWindowIcon,
}

/// `AzOptionStringEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionString")]
pub struct AzOptionStringEnumWrapper {
    pub inner: AzOptionString,
}

/// `AzOptionTextureEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionTexture")]
pub struct AzOptionTextureEnumWrapper {
    pub inner: AzOptionTexture,
}

/// `AzOptionInstantEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionInstant")]
pub struct AzOptionInstantEnumWrapper {
    pub inner: AzOptionInstant,
}

/// `AzLayoutCallbackEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "LayoutCallback")]
pub struct AzLayoutCallbackEnumWrapper {
    pub inner: AzLayoutCallback,
}

/// `AzInlineWordEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "InlineWord")]
pub struct AzInlineWordEnumWrapper {
    pub inner: AzInlineWord,
}

/// `AzNodeTypeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "NodeType")]
pub struct AzNodeTypeEnumWrapper {
    pub inner: AzNodeType,
}

/// `AzIdOrClassEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "IdOrClass")]
pub struct AzIdOrClassEnumWrapper {
    pub inner: AzIdOrClass,
}

/// `AzCssPathSelectorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssPathSelector")]
pub struct AzCssPathSelectorEnumWrapper {
    pub inner: AzCssPathSelector,
}

/// `AzStyleBackgroundContentEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundContent")]
pub struct AzStyleBackgroundContentEnumWrapper {
    pub inner: AzStyleBackgroundContent,
}

/// `AzStyleFontFamilyEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleFontFamily")]
pub struct AzStyleFontFamilyEnumWrapper {
    pub inner: AzStyleFontFamily,
}

/// `AzScrollbarStyleValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ScrollbarStyleValue")]
pub struct AzScrollbarStyleValueEnumWrapper {
    pub inner: AzScrollbarStyleValue,
}

/// `AzStyleTransformVecValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleTransformVecValue")]
pub struct AzStyleTransformVecValueEnumWrapper {
    pub inner: AzStyleTransformVecValue,
}

/// `AzSvgStyleEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgStyle")]
pub struct AzSvgStyleEnumWrapper {
    pub inner: AzSvgStyle,
}

/// `AzFmtValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "FmtValue")]
pub struct AzFmtValueEnumWrapper {
    pub inner: AzFmtValue,
}

/// `AzOptionFileTypeListEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionFileTypeList")]
pub struct AzOptionFileTypeListEnumWrapper {
    pub inner: AzOptionFileTypeList,
}

/// `AzOptionRawImageEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionRawImage")]
pub struct AzOptionRawImageEnumWrapper {
    pub inner: AzOptionRawImage,
}

/// `AzOptionWaylandThemeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionWaylandTheme")]
pub struct AzOptionWaylandThemeEnumWrapper {
    pub inner: AzOptionWaylandTheme,
}

/// `AzResultRawImageDecodeImageErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ResultRawImageDecodeImageError")]
pub struct AzResultRawImageDecodeImageErrorEnumWrapper {
    pub inner: AzResultRawImageDecodeImageError,
}

/// `AzXmlStreamErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "XmlStreamError")]
pub struct AzXmlStreamErrorEnumWrapper {
    pub inner: AzXmlStreamError,
}

/// `AzStyleBackgroundContentVecValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleBackgroundContentVecValue")]
pub struct AzStyleBackgroundContentVecValueEnumWrapper {
    pub inner: AzStyleBackgroundContentVecValue,
}

/// `AzStyleFontFamilyVecValueEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "StyleFontFamilyVecValue")]
pub struct AzStyleFontFamilyVecValueEnumWrapper {
    pub inner: AzStyleFontFamilyVecValue,
}

/// `AzCssPropertyEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssProperty")]
pub struct AzCssPropertyEnumWrapper {
    pub inner: AzCssProperty,
}

/// `AzCssPropertySourceEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssPropertySource")]
pub struct AzCssPropertySourceEnumWrapper {
    pub inner: AzCssPropertySource,
}

/// `AzOptionCssPropertyEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionCssProperty")]
pub struct AzOptionCssPropertyEnumWrapper {
    pub inner: AzOptionCssProperty,
}

/// `AzNodeDataInlineCssPropertyEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "NodeDataInlineCssProperty")]
pub struct AzNodeDataInlineCssPropertyEnumWrapper {
    pub inner: AzNodeDataInlineCssProperty,
}

/// `AzSvgNodeEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgNode")]
pub struct AzSvgNodeEnumWrapper {
    pub inner: AzSvgNode,
}

/// `AzOptionWindowStateEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionWindowState")]
pub struct AzOptionWindowStateEnumWrapper {
    pub inner: AzOptionWindowState,
}

/// `AzOptionInlineTextEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionInlineText")]
pub struct AzOptionInlineTextEnumWrapper {
    pub inner: AzOptionInlineText,
}

/// `AzXmlParseErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "XmlParseError")]
pub struct AzXmlParseErrorEnumWrapper {
    pub inner: AzXmlParseError,
}

/// `AzFocusTargetEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "FocusTarget")]
pub struct AzFocusTargetEnumWrapper {
    pub inner: AzFocusTarget,
}

/// `AzCssDeclarationEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "CssDeclaration")]
pub struct AzCssDeclarationEnumWrapper {
    pub inner: AzCssDeclaration,
}

/// `AzXmlErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "XmlError")]
pub struct AzXmlErrorEnumWrapper {
    pub inner: AzXmlError,
}

/// `AzOptionDomEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "OptionDom")]
pub struct AzOptionDomEnumWrapper {
    pub inner: AzOptionDom,
}

/// `AzResultXmlXmlErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ResultXmlXmlError")]
pub struct AzResultXmlXmlErrorEnumWrapper {
    pub inner: AzResultXmlXmlError,
}

/// `AzSvgParseErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "SvgParseError")]
pub struct AzSvgParseErrorEnumWrapper {
    pub inner: AzSvgParseError,
}

/// `AzResultSvgXmlNodeSvgParseErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ResultSvgXmlNodeSvgParseError")]
pub struct AzResultSvgXmlNodeSvgParseErrorEnumWrapper {
    pub inner: AzResultSvgXmlNodeSvgParseError,
}

/// `AzResultSvgSvgParseErrorEnumWrapper` struct
#[repr(transparent)]
#[pyclass(name = "ResultSvgSvgParseError")]
pub struct AzResultSvgSvgParseErrorEnumWrapper {
    pub inner: AzResultSvgSvgParseError,
}


// Necessary because the Python interpreter may send structs across different threads
unsafe impl Send for AzApp { }
unsafe impl Send for AzIOSHandle { }
unsafe impl Send for AzMacOSHandle { }
unsafe impl Send for AzXlibHandle { }
unsafe impl Send for AzXcbHandle { }
unsafe impl Send for AzWaylandHandle { }
unsafe impl Send for AzWindowsHandle { }
unsafe impl Send for AzAndroidHandle { }
unsafe impl Send for AzRefCount { }
unsafe impl Send for AzCssPropertyCache { }
unsafe impl Send for AzGlVoidPtrConst { }
unsafe impl Send for AzGlVoidPtrMut { }
unsafe impl Send for AzU8VecRef { }
unsafe impl Send for AzU8VecRefMut { }
unsafe impl Send for AzF32VecRef { }
unsafe impl Send for AzI32VecRef { }
unsafe impl Send for AzGLuintVecRef { }
unsafe impl Send for AzGLenumVecRef { }
unsafe impl Send for AzGLintVecRefMut { }
unsafe impl Send for AzGLint64VecRefMut { }
unsafe impl Send for AzGLbooleanVecRefMut { }
unsafe impl Send for AzGLfloatVecRefMut { }
unsafe impl Send for AzRefstr { }
unsafe impl Send for AzGLsyncPtr { }
unsafe impl Send for AzImageRef { }
unsafe impl Send for AzFontRef { }
unsafe impl Send for AzSvg { }
unsafe impl Send for AzSvgXmlNode { }
unsafe impl Send for AzFile { }
unsafe impl Send for AzSystemClipboard { }
unsafe impl Send for AzThread { }
unsafe impl Send for AzThreadSender { }
unsafe impl Send for AzThreadReceiver { }
unsafe impl Send for AzOptionHwndHandle { }
unsafe impl Send for AzOptionX11Visual { }
unsafe impl Send for AzIFrameCallbackInfo { }
unsafe impl Send for AzRefAny { }
unsafe impl Send for AzStyleBoxShadow { }
unsafe impl Send for AzStyleBackgroundSize { }
unsafe impl Send for AzGl { }
unsafe impl Send for AzRefstrVecRef { }
unsafe impl Send for AzFontMetrics { }
unsafe impl Send for AzInstantPtr { }
unsafe impl Send for AzXmlNodeVec { }
unsafe impl Send for AzInlineGlyphVec { }
unsafe impl Send for AzInlineTextHitVec { }
unsafe impl Send for AzVideoModeVec { }
unsafe impl Send for AzDomVec { }
unsafe impl Send for AzStyleBackgroundPositionVec { }
unsafe impl Send for AzStyleBackgroundRepeatVec { }
unsafe impl Send for AzStyleBackgroundSizeVec { }
unsafe impl Send for AzSvgVertexVec { }
unsafe impl Send for AzU32Vec { }
unsafe impl Send for AzXWindowTypeVec { }
unsafe impl Send for AzVirtualKeyCodeVec { }
unsafe impl Send for AzCascadeInfoVec { }
unsafe impl Send for AzScanCodeVec { }
unsafe impl Send for AzU16Vec { }
unsafe impl Send for AzF32Vec { }
unsafe impl Send for AzU8Vec { }
unsafe impl Send for AzGLuintVec { }
unsafe impl Send for AzGLintVec { }
unsafe impl Send for AzNormalizedLinearColorStopVec { }
unsafe impl Send for AzNormalizedRadialColorStopVec { }
unsafe impl Send for AzNodeIdVec { }
unsafe impl Send for AzNodeVec { }
unsafe impl Send for AzParentWithNodeDepthVec { }
unsafe impl Send for AzRenderImageCallbackInfo { }
unsafe impl Send for AzLayoutCallbackInfo { }
unsafe impl Send for AzTesselatedSvgNodeVecRef { }
unsafe impl Send for AzTesselatedSvgNodeVec { }
unsafe impl Send for AzStyleTransformVec { }
unsafe impl Send for AzSvgPathElementVec { }
unsafe impl Send for AzStringVec { }
unsafe impl Send for AzStyledNodeVec { }
unsafe impl Send for AzTagIdsToNodeIdsMappingVec { }
unsafe impl Send for AzWaylandTheme { }
unsafe impl Send for AzStyleFontFamilyVec { }
unsafe impl Send for AzFmtArgVec { }
unsafe impl Send for AzInlineWordVec { }
unsafe impl Send for AzMonitorVec { }
unsafe impl Send for AzIdOrClassVec { }
unsafe impl Send for AzStyleBackgroundContentVec { }
unsafe impl Send for AzSvgPathVec { }
unsafe impl Send for AzVertexAttributeVec { }
unsafe impl Send for AzCssPathSelectorVec { }
unsafe impl Send for AzCallbackDataVec { }
unsafe impl Send for AzDebugMessageVec { }
unsafe impl Send for AzStringPairVec { }
unsafe impl Send for AzInlineLineVec { }
unsafe impl Send for AzCssPropertyVec { }
unsafe impl Send for AzSvgMultiPolygonVec { }
unsafe impl Send for AzCallbackInfo { }
unsafe impl Send for AzTimerCallbackInfo { }
unsafe impl Send for AzNodeDataInlineCssPropertyVec { }
unsafe impl Send for AzCssDeclarationVec { }
unsafe impl Send for AzNodeDataVec { }
unsafe impl Send for AzCssRuleBlockVec { }
unsafe impl Send for AzStylesheetVec { }


// Python objects must implement Clone at minimum
impl Clone for AzApp { fn clone(&self) -> Self { let r: &azul_impl::app::AzAppPtr = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAppLogLevelEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::AppLogLevel = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutSolverEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::LayoutSolverVersion = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVsyncEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::Vsync = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSrgbEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::Srgb = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzHwAccelerationEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::HwAcceleration = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPoint { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutPoint = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutSize { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutSize = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIOSHandle { fn clone(&self) -> Self { let r: &azul_impl::window::IOSHandle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMacOSHandle { fn clone(&self) -> Self { let r: &azul_impl::window::MacOSHandle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXlibHandle { fn clone(&self) -> Self { let r: &azul_impl::window::XlibHandle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXcbHandle { fn clone(&self) -> Self { let r: &azul_impl::window::XcbHandle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWaylandHandle { fn clone(&self) -> Self { let r: &azul_impl::window::WaylandHandle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWindowsHandle { fn clone(&self) -> Self { let r: &azul_impl::window::WindowsHandle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWebHandle { fn clone(&self) -> Self { let r: &azul_impl::window::WebHandle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAndroidHandle { fn clone(&self) -> Self { let r: &azul_impl::window::AndroidHandle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXWindowTypeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::XWindowType = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzPhysicalPositionI32 { fn clone(&self) -> Self { let r: &azul_impl::window::PhysicalPosition<i32> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzPhysicalSizeU32 { fn clone(&self) -> Self { let r: &azul_impl::window::PhysicalSize<u32> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLogicalPosition { fn clone(&self) -> Self { let r: &azul_impl::window::LogicalPosition = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLogicalSize { fn clone(&self) -> Self { let r: &azul_impl::window::LogicalSize = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIconKey { fn clone(&self) -> Self { let r: &azul_impl::window::IconKey = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVirtualKeyCodeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::VirtualKeyCode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWindowFlags { fn clone(&self) -> Self { let r: &azul_impl::window::WindowFlags = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDebugState { fn clone(&self) -> Self { let r: &azul_impl::window::DebugState = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMouseCursorTypeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::MouseCursorType = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRendererTypeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::RendererType = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMacWindowOptions { fn clone(&self) -> Self { let r: &azul_impl::window::MacWindowOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWasmWindowOptions { fn clone(&self) -> Self { let r: &azul_impl::window::WasmWindowOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFullScreenModeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::FullScreenMode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWindowThemeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::WindowTheme = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTouchState { fn clone(&self) -> Self { let r: &azul_impl::window::TouchState = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMarshaledLayoutCallbackInner { fn clone(&self) -> Self { let r: &azul_impl::callbacks::MarshaledLayoutCallbackInner = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutCallbackInner { fn clone(&self) -> Self { let r: &azul_impl::callbacks::LayoutCallbackInner = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCallback { fn clone(&self) -> Self { let r: &azul_impl::callbacks::Callback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzUpdateEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::UpdateScreen = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeId { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::AzNodeId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDomId { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::DomId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzPositionInfoInner { fn clone(&self) -> Self { let r: &azul_impl::ui_solver::PositionInfoInner = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAnimationRepeatEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::AnimationRepeat = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAnimationRepeatCountEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::AnimationRepeatCount = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIFrameCallback { fn clone(&self) -> Self { let r: &azul_impl::callbacks::IFrameCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRenderImageCallback { fn clone(&self) -> Self { let r: &azul_impl::callbacks::RenderImageCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTimerCallback { fn clone(&self) -> Self { let r: &azul_impl::callbacks::TimerCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWriteBackCallback { fn clone(&self) -> Self { let r: &azul_impl::callbacks::WriteBackCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadCallback { fn clone(&self) -> Self { let r: &azul_impl::callbacks::ThreadCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRefCount { fn clone(&self) -> Self { let r: &azul_impl::callbacks::RefCount = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOnEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::On = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzHoverEventFilterEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::HoverEventFilter = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFocusEventFilterEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::FocusEventFilter = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWindowEventFilterEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::WindowEventFilter = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzComponentEventFilterEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::ComponentEventFilter = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzApplicationEventFilterEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::ApplicationEventFilter = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTabIndexEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::TabIndex = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeTypeKeyEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::NodeTypeTag = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssNthChildPattern { fn clone(&self) -> Self { let r: &azul_impl::css::CssNthChildPattern = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPropertyTypeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyType = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzColorU { fn clone(&self) -> Self { let r: &azul_impl::css::ColorU = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSizeMetricEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::SizeMetric = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFloatValue { fn clone(&self) -> Self { let r: &azul_impl::css::FloatValue = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzBoxShadowClipModeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::BoxShadowClipMode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutAlignContentEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutAlignContent = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutAlignItemsEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutAlignItems = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBoxSizingEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutBoxSizing = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutFlexDirectionEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutFlexDirection = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutDisplayEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutDisplay = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutFloatEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutFloat = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutJustifyContentEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutJustifyContent = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPositionEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutPosition = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutFlexWrapEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutFlexWrap = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutOverflowEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutOverflow = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAngleMetricEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::AngleMetric = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDirectionCornerEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::DirectionCorner = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzExtendModeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::ExtendMode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzShapeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::Shape = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRadialGradientSizeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::RadialGradientSize = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundRepeatEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundRepeat = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzBorderStyleEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::BorderStyle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleCursorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleCursor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackfaceVisibilityEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackfaceVisibility = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTextAlignEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTextAlignmentHorz = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNode { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::AzNode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCascadeInfo { fn clone(&self) -> Self { let r: &azul_impl::style::CascadeInfo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyledNodeState { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::StyledNodeState = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTagId { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::AzTagId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPropertyCache { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::CssPropertyCachePtr = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGlVoidPtrConst { fn clone(&self) -> Self { let r: &azul_impl::gl::GlVoidPtrConst = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGlVoidPtrMut { fn clone(&self) -> Self { let r: &azul_impl::gl::GlVoidPtrMut = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGlShaderPrecisionFormatReturn { fn clone(&self) -> Self { let r: &azul_impl::gl::GlShaderPrecisionFormatReturn = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVertexAttributeTypeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::VertexAttributeType = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIndexBufferFormatEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::IndexBufferFormat = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGlTypeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::AzGlType = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzU8VecRef { fn clone(&self) -> Self { let r: &azul_impl::gl::U8VecRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzU8VecRefMut { fn clone(&self) -> Self { let r: &azul_impl::gl::U8VecRefMut = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzF32VecRef { fn clone(&self) -> Self { let r: &azul_impl::gl::F32VecRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzI32VecRef { fn clone(&self) -> Self { let r: &azul_impl::gl::I32VecRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLuintVecRef { fn clone(&self) -> Self { let r: &azul_impl::gl::GLuintVecRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLenumVecRef { fn clone(&self) -> Self { let r: &azul_impl::gl::GLenumVecRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLintVecRefMut { fn clone(&self) -> Self { let r: &azul_impl::gl::GLintVecRefMut = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLint64VecRefMut { fn clone(&self) -> Self { let r: &azul_impl::gl::GLint64VecRefMut = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLbooleanVecRefMut { fn clone(&self) -> Self { let r: &azul_impl::gl::GLbooleanVecRefMut = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLfloatVecRefMut { fn clone(&self) -> Self { let r: &azul_impl::gl::GLfloatVecRefMut = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRefstr { fn clone(&self) -> Self { let r: &azul_impl::gl::Refstr = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLsyncPtr { fn clone(&self) -> Self { let r: &azul_impl::gl::GLsyncPtr = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTextureFlags { fn clone(&self) -> Self { let r: &azul_impl::gl::TextureFlags = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzImageRef { fn clone(&self) -> Self { let r: &azul_impl::resources::ImageRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRawImageFormatEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::RawImageFormat = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzEncodeImageErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::encode::EncodeImageError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDecodeImageErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::decode::DecodeImageError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFontRef { fn clone(&self) -> Self { let r: &azul_impl::css::FontRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvg { fn clone(&self) -> Self { let r: &azul_impl::svg::Svg = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgXmlNode { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgXmlNode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgCircle { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgCircle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgPoint { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgPoint = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgRect { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgRect = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgVertex { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgVertex = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzShapeRenderingEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::ShapeRendering = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTextRenderingEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::TextRendering = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzImageRenderingEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::ImageRendering = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFontDatabaseEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::FontDatabase = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIndentEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::Indent = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgFitToEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgFitTo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgFillRuleEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgFillRule = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgTransform { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgTransform = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgLineJoinEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgLineJoin = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgLineCapEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgLineCap = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgDashPattern { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgDashPattern = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFile { fn clone(&self) -> Self { let r: &azul_impl::file::File = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMsgBox { fn clone(&self) -> Self { let r: &azul_impl::dialogs::MsgBox = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMsgBoxIconEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dialogs::MsgBoxIcon = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMsgBoxYesNoEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dialogs::YesNo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMsgBoxOkCancelEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dialogs::OkCancel = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFileDialog { fn clone(&self) -> Self { let r: &azul_impl::dialogs::FileDialog = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzColorPickerDialog { fn clone(&self) -> Self { let r: &azul_impl::dialogs::ColorPickerDialog = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSystemClipboard { fn clone(&self) -> Self { let r: &azul_impl::window::Clipboard = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInstantPtrCloneFn { fn clone(&self) -> Self { let r: &azul_impl::task::InstantPtrCloneCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInstantPtrDestructorFn { fn clone(&self) -> Self { let r: &azul_impl::task::InstantPtrDestructorCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSystemTick { fn clone(&self) -> Self { let r: &azul_impl::task::SystemTick = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSystemTimeDiff { fn clone(&self) -> Self { let r: &azul_impl::task::SystemTimeDiff = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSystemTickDiff { fn clone(&self) -> Self { let r: &azul_impl::task::SystemTickDiff = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTimerId { fn clone(&self) -> Self { let r: &azul_impl::task::TimerId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTerminateTimerEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::TerminateTimer = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadId { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThread { fn clone(&self) -> Self { let r: &azul_impl::task::Thread = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadSender { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadSender = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadReceiver { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadReceiver = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCreateThreadFn { fn clone(&self) -> Self { let r: &azul_impl::task::CreateThreadCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGetSystemTimeFn { fn clone(&self) -> Self { let r: &azul_impl::task::GetSystemTimeCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCheckThreadFinishedFn { fn clone(&self) -> Self { let r: &azul_impl::task::CheckThreadFinishedCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLibrarySendThreadMsgFn { fn clone(&self) -> Self { let r: &azul_impl::task::LibrarySendThreadMsgCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLibraryReceiveThreadMsgFn { fn clone(&self) -> Self { let r: &azul_impl::task::LibraryReceiveThreadMsgCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadRecvFn { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadRecvCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadSendFn { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadSendCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadDestructorFn { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadDestructorCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadReceiverDestructorFn { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadReceiverDestructorCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadSenderDestructorFn { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadSenderDestructorCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleFontFamilyVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleFontFamilyVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTesselatedSvgNodeVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::TesselatedSvgNodeVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXmlNodeVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::xml::XmlNodeVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFmtArgVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::str::FmtArgVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineLineVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::InlineLineVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineWordVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::InlineWordVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineGlyphVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::InlineGlyphVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineTextHitVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::InlineTextHitVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMonitorVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::MonitorVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVideoModeVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::VideoModeVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDomVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::DomVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIdOrClassVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::IdOrClassVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeDataInlineCssPropertyVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::NodeDataInlineCssPropertyVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundContentVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundContentVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundPositionVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundPositionVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundRepeatVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundRepeatVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundSizeVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundSizeVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPropertyVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgMultiPolygonVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgMultiPolygonVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgPathVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgPathVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVertexAttributeVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::VertexAttributeVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgPathElementVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgPathElementVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgVertexVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgVertexVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzU32VecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::U32VecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXWindowTypeVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::XWindowTypeVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVirtualKeyCodeVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::VirtualKeyCodeVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCascadeInfoVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::style::CascadeInfoVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzScanCodeVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::ScanCodeVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssDeclarationVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssDeclarationVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPathSelectorVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPathSelectorVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStylesheetVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StylesheetVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssRuleBlockVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssRuleBlockVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzF32VecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::F32VecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzU16VecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::U16VecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzU8VecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::U8VecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCallbackDataVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::CallbackDataVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDebugMessageVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::AzDebugMessageVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLuintVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::GLuintVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLintVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::GLintVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStringVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StringVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStringPairVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::StringPairVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNormalizedLinearColorStopVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::NormalizedLinearColorStopVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNormalizedRadialColorStopVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::NormalizedRadialColorStopVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeIdVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::NodeIdVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::AzNodeVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyledNodeVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::StyledNodeVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTagIdsToNodeIdsMappingVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::TagIdToNodeIdMappingVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzParentWithNodeDepthVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::ParentWithNodeDepthVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeDataVecDestructorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::NodeDataVecDestructor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionI16EnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionI16 = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionU16EnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionU16 = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionU32EnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionU32 = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionHwndHandleEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionHwndHandle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionX11VisualEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionX11Visual = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionI32EnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionI32 = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionF32EnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionF32 = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionCharEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionChar = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionUsizeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::OptionUsize = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgParseErrorPosition { fn clone(&self) -> Self { let r: &azul_impl::xml::XmlTextPos = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSystemCallbacks { fn clone(&self) -> Self { let r: &azul_impl::task::ExternalSystemCallbacks = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRendererOptions { fn clone(&self) -> Self { let r: &azul_impl::window::RendererOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutRect { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutRect = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRawWindowHandleEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::RawWindowHandle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLogicalRect { fn clone(&self) -> Self { let r: &azul_impl::window::LogicalRect = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAcceleratorKeyEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::AcceleratorKey = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCursorPositionEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::CursorPosition = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWindowPositionEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::WindowPosition = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzImePositionEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::ImePosition = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVideoMode { fn clone(&self) -> Self { let r: &azul_impl::window::VideoMode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDomNodeId { fn clone(&self) -> Self { let r: &azul_impl::callbacks::DomNodeId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzPositionInfoEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::ui_solver::PositionInfo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzHidpiAdjustedBounds { fn clone(&self) -> Self { let r: &azul_impl::callbacks::HidpiAdjustedBounds = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineGlyph { fn clone(&self) -> Self { let r: &azul_core::callbacks::InlineGlyph = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineTextHit { fn clone(&self) -> Self { let r: &azul_core::callbacks::InlineTextHit = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIFrameCallbackInfo { fn clone(&self) -> Self { let r: &azul_impl::callbacks::IFrameCallbackInfo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTimerCallbackReturn { fn clone(&self) -> Self { let r: &azul_impl::callbacks::TimerCallbackReturn = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRefAny { fn clone(&self) -> Self { let r: &azul_impl::callbacks::RefAny = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIFrameNode { fn clone(&self) -> Self { let r: &azul_impl::dom::IFrameNode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNotEventFilterEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::NotEventFilter = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssNthChildSelectorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssNthChildSelector = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzPixelValue { fn clone(&self) -> Self { let r: &azul_impl::css::PixelValue = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzPixelValueNoPercent { fn clone(&self) -> Self { let r: &azul_impl::css::PixelValueNoPercent = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBoxShadow { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBoxShadow = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBottom { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutBottom = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutFlexGrow { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutFlexGrow = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutFlexShrink { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutFlexShrink = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutHeight { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutHeight = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutLeft { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutLeft = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMarginBottom { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutMarginBottom = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMarginLeft { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutMarginLeft = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMarginRight { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutMarginRight = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMarginTop { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutMarginTop = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMaxHeight { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutMaxHeight = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMaxWidth { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutMaxWidth = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMinHeight { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutMinHeight = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMinWidth { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutMinWidth = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPaddingBottom { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutPaddingBottom = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPaddingLeft { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutPaddingLeft = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPaddingRight { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutPaddingRight = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPaddingTop { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutPaddingTop = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutRight { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutRight = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutTop { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutTop = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutWidth { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutWidth = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzPercentageValue { fn clone(&self) -> Self { let r: &azul_impl::css::PercentageValue = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAngleValue { fn clone(&self) -> Self { let r: &azul_impl::css::AngleValue = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNormalizedLinearColorStop { fn clone(&self) -> Self { let r: &azul_impl::css::NormalizedLinearColorStop = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNormalizedRadialColorStop { fn clone(&self) -> Self { let r: &azul_impl::css::NormalizedRadialColorStop = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDirectionCorners { fn clone(&self) -> Self { let r: &azul_impl::css::DirectionCorners = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDirectionEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::Direction = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzBackgroundPositionHorizontalEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::BackgroundPositionHorizontal = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzBackgroundPositionVerticalEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::BackgroundPositionVertical = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundPosition { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundPosition = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundSizeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundSize = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderBottomColor { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderBottomColor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderBottomLeftRadius { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderBottomLeftRadius = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderBottomRightRadius { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderBottomRightRadius = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderBottomStyle { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderBottomStyle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBorderBottomWidth { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutBorderBottomWidth = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderLeftColor { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderLeftColor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderLeftStyle { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderLeftStyle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBorderLeftWidth { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutBorderLeftWidth = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderRightColor { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderRightColor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderRightStyle { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderRightStyle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBorderRightWidth { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutBorderRightWidth = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderTopColor { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderTopColor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderTopLeftRadius { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderTopLeftRadius = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderTopRightRadius { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderTopRightRadius = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderTopStyle { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBorderTopStyle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBorderTopWidth { fn clone(&self) -> Self { let r: &azul_impl::css::LayoutBorderTopWidth = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleFontSize { fn clone(&self) -> Self { let r: &azul_impl::css::StyleFontSize = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleLetterSpacing { fn clone(&self) -> Self { let r: &azul_impl::css::StyleLetterSpacing = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleLineHeight { fn clone(&self) -> Self { let r: &azul_impl::css::StyleLineHeight = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTabWidth { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTabWidth = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleOpacity { fn clone(&self) -> Self { let r: &azul_impl::css::StyleOpacity = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformOrigin { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformOrigin = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStylePerspectiveOrigin { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformOrigin = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformMatrix2D { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformMatrix2D = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformMatrix3D { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformMatrix3D = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformTranslate2D { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformTranslate2D = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformTranslate3D { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformTranslate3D = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformRotate3D { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformRotate3D = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformScale2D { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformScale2D = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformScale3D { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformScale3D = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformSkew2D { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformSkew2D = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTextColor { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTextColor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleWordSpacing { fn clone(&self) -> Self { let r: &azul_impl::css::StyleWordSpacing = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBoxShadowValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBoxShadow> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutAlignContentValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutAlignContent> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutAlignItemsValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutAlignItems> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBottomValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutBottom> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBoxSizingValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutBoxSizing> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutFlexDirectionValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutFlexDirection> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutDisplayValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutDisplay> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutFlexGrowValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutFlexGrow> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutFlexShrinkValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutFlexShrink> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutFloatValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutFloat> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutHeightValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutHeight> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutJustifyContentValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutJustifyContent> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutLeftValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutLeft> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMarginBottomValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutMarginBottom> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMarginLeftValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutMarginLeft> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMarginRightValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutMarginRight> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMarginTopValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutMarginTop> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMaxHeightValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutMaxHeight> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMaxWidthValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutMaxWidth> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMinHeightValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutMinHeight> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutMinWidthValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutMinWidth> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPaddingBottomValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutPaddingBottom> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPaddingLeftValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutPaddingLeft> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPaddingRightValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutPaddingRight> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPaddingTopValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutPaddingTop> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutPositionValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutPosition> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutRightValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutRight> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutTopValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutTop> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutWidthValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutWidth> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutFlexWrapValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutFlexWrap> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutOverflowValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutOverflow> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderBottomColorValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderBottomColor> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderBottomLeftRadiusValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderBottomLeftRadius> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderBottomRightRadiusValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderBottomRightRadius> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderBottomStyleValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderBottomStyle> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBorderBottomWidthValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutBorderBottomWidth> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderLeftColorValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderLeftColor> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderLeftStyleValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderLeftStyle> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBorderLeftWidthValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutBorderLeftWidth> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderRightColorValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderRightColor> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderRightStyleValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderRightStyle> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBorderRightWidthValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutBorderRightWidth> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderTopColorValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderTopColor> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderTopLeftRadiusValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderTopLeftRadius> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderTopRightRadiusValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderTopRightRadius> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBorderTopStyleValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBorderTopStyle> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutBorderTopWidthValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzLayoutBorderTopWidth> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleCursorValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleCursor> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleFontSizeValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleFontSize> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleLetterSpacingValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleLetterSpacing> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleLineHeightValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleLineHeight> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTabWidthValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleTabWidth> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTextAlignValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleTextAlign> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTextColorValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleTextColor> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleWordSpacingValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleWordSpacing> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleOpacityValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleOpacity> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformOriginValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleTransformOrigin> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStylePerspectiveOriginValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStylePerspectiveOrigin> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackfaceVisibilityValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBackfaceVisibility> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzParentWithNodeDepth { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::ParentWithNodeDepth = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGl { fn clone(&self) -> Self { let r: &azul_impl::gl::GlContextPtr = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRefstrVecRef { fn clone(&self) -> Self { let r: &azul_impl::gl::RefstrVecRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzImageMask { fn clone(&self) -> Self { let r: &azul_impl::resources::ImageMask = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFontMetrics { fn clone(&self) -> Self { let r: &azul_impl::css::FontMetrics = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgLine { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgLine = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgQuadraticCurve { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgQuadraticCurve = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgCubicCurve { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgCubicCurve = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgStringFormatOptions { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgXmlOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgFillStyle { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgFillStyle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInstantPtr { fn clone(&self) -> Self { let r: &azul_impl::task::AzInstantPtr = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDurationEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::Duration = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadSendMsgEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadSendMsg = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadWriteBackMsg { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadWriteBackMsg = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXmlNodeVec { fn clone(&self) -> Self { let r: &azul_impl::xml::XmlNodeVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineGlyphVec { fn clone(&self) -> Self { let r: &azul_impl::callbacks::InlineGlyphVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineTextHitVec { fn clone(&self) -> Self { let r: &azul_impl::callbacks::InlineTextHitVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVideoModeVec { fn clone(&self) -> Self { let r: &azul_impl::window::VideoModeVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDomVec { fn clone(&self) -> Self { let r: &azul_impl::dom::DomVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundPositionVec { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundPositionVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundRepeatVec { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundRepeatVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundSizeVec { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundSizeVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgVertexVec { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgVertexVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzU32Vec { fn clone(&self) -> Self { let r: &azul_impl::css::U32Vec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXWindowTypeVec { fn clone(&self) -> Self { let r: &azul_impl::window::XWindowTypeVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVirtualKeyCodeVec { fn clone(&self) -> Self { let r: &azul_impl::window::VirtualKeyCodeVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCascadeInfoVec { fn clone(&self) -> Self { let r: &azul_impl::style::CascadeInfoVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzScanCodeVec { fn clone(&self) -> Self { let r: &azul_impl::window::ScanCodeVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzU16Vec { fn clone(&self) -> Self { let r: &azul_impl::css::U16Vec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzF32Vec { fn clone(&self) -> Self { let r: &azul_impl::css::F32Vec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzU8Vec { fn clone(&self) -> Self { let r: &azul_impl::css::U8Vec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLuintVec { fn clone(&self) -> Self { let r: &azul_impl::gl::GLuintVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGLintVec { fn clone(&self) -> Self { let r: &azul_impl::gl::GLintVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNormalizedLinearColorStopVec { fn clone(&self) -> Self { let r: &azul_impl::css::NormalizedLinearColorStopVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNormalizedRadialColorStopVec { fn clone(&self) -> Self { let r: &azul_impl::css::NormalizedRadialColorStopVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeIdVec { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::NodeIdVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeVec { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::AzNodeVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzParentWithNodeDepthVec { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::ParentWithNodeDepthVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionPositionInfoEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::ui_solver::OptionPositionInfo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionTimerIdEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::OptionTimerId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionThreadIdEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::OptionThreadId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionImageRefEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::OptionImageRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionFontRefEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionFontRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionSystemClipboardEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionClipboard = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionFileEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::file::OptionFile = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionGlEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::OptionGlContextPtr = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionPercentageValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionPercentageValue = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionAngleValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionAngleValue = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionRendererOptionsEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionRendererOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionCallbackEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::OptionCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionThreadSendMsgEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::OptionThreadSendMsg = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionLayoutRectEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionLayoutRect = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionRefAnyEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::OptionRefAny = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionLayoutPointEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionLayoutPoint = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionLayoutSizeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionLayoutSize = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionWindowThemeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionWindowTheme = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionNodeIdEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::OptionNodeId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionDomNodeIdEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::OptionDomNodeId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionColorUEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionColorU = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionSvgDashPatternEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::OptionSvgDashPattern = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionLogicalPositionEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionLogicalPosition = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionPhysicalPositionI32EnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionPhysicalPositionI32 = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionMouseCursorTypeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionMouseCursorType = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionLogicalSizeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionLogicalSize = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionVirtualKeyCodeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionVirtualKeyCode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionImageMaskEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::OptionImageMask = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionTabIndexEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::OptionTabIndex = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionTagIdEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::OptionTagId = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionDurationEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::OptionDuration = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionU8VecEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionU8Vec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionU8VecRefEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::OptionU8VecRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzResultU8VecEncodeImageErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::encode::ResultU8VecEncodeImageError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNonXmlCharError { fn clone(&self) -> Self { let r: &azul_impl::xml::NonXmlCharError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInvalidCharError { fn clone(&self) -> Self { let r: &azul_impl::xml::InvalidCharError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInvalidCharMultipleError { fn clone(&self) -> Self { let r: &azul_impl::xml::InvalidCharMultipleError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInvalidQuoteError { fn clone(&self) -> Self { let r: &azul_impl::xml::InvalidQuoteError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInvalidSpaceError { fn clone(&self) -> Self { let r: &azul_impl::xml::InvalidSpaceError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAppConfig { fn clone(&self) -> Self { let r: &azul_impl::resources::AppConfig = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSmallWindowIconBytes { fn clone(&self) -> Self { let r: &azul_impl::window::SmallWindowIconBytes = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLargeWindowIconBytes { fn clone(&self) -> Self { let r: &azul_impl::window::LargeWindowIconBytes = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWindowIconEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::WindowIcon = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTaskBarIcon { fn clone(&self) -> Self { let r: &azul_impl::window::TaskBarIcon = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWindowSize { fn clone(&self) -> Self { let r: &azul_impl::window::WindowSize = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzKeyboardState { fn clone(&self) -> Self { let r: &azul_impl::window::KeyboardState = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMouseState { fn clone(&self) -> Self { let r: &azul_impl::window::MouseState = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMarshaledLayoutCallback { fn clone(&self) -> Self { let r: &azul_impl::callbacks::MarshaledLayoutCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineTextContents { fn clone(&self) -> Self { let r: &azul_core::callbacks::InlineTextContents = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAnimationEasingEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::AnimationInterpolationFunction = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRenderImageCallbackInfo { fn clone(&self) -> Self { let r: &azul_impl::callbacks::RenderImageCallbackInfo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutCallbackInfo { fn clone(&self) -> Self { let r: &azul_impl::callbacks::LayoutCallbackInfo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzEventFilterEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::EventFilter = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPathPseudoSelectorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPathPseudoSelector = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAnimationInterpolationFunctionEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::AnimationInterpolationFunction = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInterpolateContext { fn clone(&self) -> Self { let r: &azul_impl::css::InterpolateResolver = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLinearGradient { fn clone(&self) -> Self { let r: &azul_impl::css::LinearGradient = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRadialGradient { fn clone(&self) -> Self { let r: &azul_impl::css::RadialGradient = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzConicGradient { fn clone(&self) -> Self { let r: &azul_impl::css::ConicGradient = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransform = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundPositionVecValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBackgroundPositionVec> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundRepeatVecValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBackgroundRepeatVec> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundSizeVecValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBackgroundSizeVec> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyledNode { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::StyledNode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTagIdToNodeIdMapping { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::TagIdToNodeIdMapping = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTexture { fn clone(&self) -> Self { let r: &azul_impl::gl::Texture = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGetProgramBinaryReturn { fn clone(&self) -> Self { let r: &azul_impl::gl::GetProgramBinaryReturn = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRawImageDataEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::RawImageData = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFontSource { fn clone(&self) -> Self { let r: &azul_impl::resources::LoadedFontSource = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgPathElementEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgPathElement = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTesselatedSvgNode { fn clone(&self) -> Self { let r: &azul_impl::svg::TesselatedSvgNode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTesselatedSvgNodeVecRef { fn clone(&self) -> Self { let r: &azul_impl::svg::TesselatedSvgNodeVecRef = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgRenderOptions { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgRenderOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgStrokeStyle { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgStrokeStyle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXml { fn clone(&self) -> Self { let r: &azul_impl::xml::Xml = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInstantEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::Instant = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzThreadReceiveMsgEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::ThreadReceiveMsg = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzString { fn clone(&self) -> Self { let r: &azul_impl::css::AzString = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTesselatedSvgNodeVec { fn clone(&self) -> Self { let r: &azul_impl::svg::TesselatedSvgNodeVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformVec { fn clone(&self) -> Self { let r: &azul_impl::css::StyleTransformVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgPathElementVec { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgPathElementVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStringVec { fn clone(&self) -> Self { let r: &azul_impl::css::StringVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyledNodeVec { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::StyledNodeVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTagIdsToNodeIdsMappingVec { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::TagIdsToNodeIdsMappingVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionMouseStateEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionMouseState = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionKeyboardStateEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionKeyboardState = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionStringVecEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionStringVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionThreadReceiveMsgEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::OptionThreadReceiveMsg = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionTaskBarIconEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionTaskBarIcon = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionWindowIconEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionWindowIcon = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionStringEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionAzString = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionTextureEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::gl::OptionTexture = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionInstantEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::task::OptionInstant = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDuplicatedNamespaceError { fn clone(&self) -> Self { let r: &azul_impl::xml::DuplicatedNamespaceError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzUnknownNamespaceError { fn clone(&self) -> Self { let r: &azul_impl::xml::UnknownNamespaceError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzUnexpectedCloseTagError { fn clone(&self) -> Self { let r: &azul_impl::xml::UnexpectedCloseTagError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzUnknownEntityReferenceError { fn clone(&self) -> Self { let r: &azul_impl::xml::UnknownEntityReferenceError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDuplicatedAttributeError { fn clone(&self) -> Self { let r: &azul_impl::xml::DuplicatedAttributeError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInvalidStringError { fn clone(&self) -> Self { let r: &azul_impl::xml::InvalidStringError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWindowsWindowOptions { fn clone(&self) -> Self { let r: &azul_impl::window::WindowsWindowOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWaylandTheme { fn clone(&self) -> Self { let r: &azul_impl::window::WaylandTheme = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStringPair { fn clone(&self) -> Self { let r: &azul_impl::window::AzStringPair = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMonitor { fn clone(&self) -> Self { let r: &azul_impl::window::Monitor = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLayoutCallbackEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::LayoutCallback = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineWordEnumWrapper { fn clone(&self) -> Self { let r: &azul_core::callbacks::InlineWord = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCallbackData { fn clone(&self) -> Self { let r: &azul_impl::dom::CallbackData = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeTypeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::NodeType = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIdOrClassEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::IdOrClass = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPathSelectorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPathSelector = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundContentEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundContent = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzScrollbarInfo { fn clone(&self) -> Self { let r: &azul_impl::css::ScrollbarInfo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzScrollbarStyle { fn clone(&self) -> Self { let r: &azul_impl::css::ScrollbarStyle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleFontFamilyEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::StyleFontFamily = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzScrollbarStyleValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzScrollbarStyle> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleTransformVecValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleTransformVec> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVertexAttribute { fn clone(&self) -> Self { let r: &azul_impl::gl::VertexAttribute = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDebugMessage { fn clone(&self) -> Self { let r: &azul_impl::gl::AzDebugMessage = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGetActiveAttribReturn { fn clone(&self) -> Self { let r: &azul_impl::gl::GetActiveAttribReturn = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzGetActiveUniformReturn { fn clone(&self) -> Self { let r: &azul_impl::gl::GetActiveUniformReturn = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzRawImage { fn clone(&self) -> Self { let r: &azul_impl::resources::RawImage = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgPath { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgPath = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgParseOptions { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgParseOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgStyleEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgStyle = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFileTypeList { fn clone(&self) -> Self { let r: &azul_impl::dialogs::FileTypeList = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTimer { fn clone(&self) -> Self { let r: &azul_impl::task::Timer = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFmtValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::str::FmtValue = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFmtArg { fn clone(&self) -> Self { let r: &azul_impl::str::FmtArg = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleFontFamilyVec { fn clone(&self) -> Self { let r: &azul_impl::css::StyleFontFamilyVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFmtArgVec { fn clone(&self) -> Self { let r: &azul_impl::str::FmtArgVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineWordVec { fn clone(&self) -> Self { let r: &azul_impl::callbacks::InlineWordVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzMonitorVec { fn clone(&self) -> Self { let r: &azul_impl::window::MonitorVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIdOrClassVec { fn clone(&self) -> Self { let r: &azul_impl::dom::IdOrClassVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundContentVec { fn clone(&self) -> Self { let r: &azul_impl::css::StyleBackgroundContentVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgPathVec { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgPathVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVertexAttributeVec { fn clone(&self) -> Self { let r: &azul_impl::gl::VertexAttributeVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPathSelectorVec { fn clone(&self) -> Self { let r: &azul_impl::css::CssPathSelectorVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCallbackDataVec { fn clone(&self) -> Self { let r: &azul_impl::dom::CallbackDataVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDebugMessageVec { fn clone(&self) -> Self { let r: &azul_impl::gl::AzDebugMessageVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStringPairVec { fn clone(&self) -> Self { let r: &azul_impl::window::StringPairVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionFileTypeListEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dialogs::OptionFileTypeList = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionRawImageEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::OptionRawImage = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionWaylandThemeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionWaylandTheme = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzResultRawImageDecodeImageErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::resources::decode::ResultRawImageDecodeImageError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXmlStreamErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::xml::XmlStreamError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzLinuxWindowOptions { fn clone(&self) -> Self { let r: &azul_impl::window::LinuxWindowOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineLine { fn clone(&self) -> Self { let r: &azul_impl::callbacks::InlineLine = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPath { fn clone(&self) -> Self { let r: &azul_impl::css::CssPath = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleBackgroundContentVecValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleBackgroundContentVec> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyleFontFamilyVecValueEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyValue::<AzStyleFontFamilyVec> = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPropertyEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssProperty = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPropertySourceEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::CssPropertySource = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVertexLayout { fn clone(&self) -> Self { let r: &azul_impl::gl::VertexLayout = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVertexArrayObject { fn clone(&self) -> Self { let r: &azul_impl::gl::VertexArrayObject = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzVertexBuffer { fn clone(&self) -> Self { let r: &azul_impl::gl::VertexBuffer = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgMultiPolygon { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgMultiPolygon = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXmlNode { fn clone(&self) -> Self { let r: &azul_impl::xml::XmlNode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineLineVec { fn clone(&self) -> Self { let r: &azul_impl::callbacks::InlineLineVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssPropertyVec { fn clone(&self) -> Self { let r: &azul_impl::css::CssPropertyVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgMultiPolygonVec { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgMultiPolygonVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionCssPropertyEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::OptionCssProperty = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXmlTextError { fn clone(&self) -> Self { let r: &azul_impl::xml::XmlTextError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzPlatformSpecificOptions { fn clone(&self) -> Self { let r: &azul_impl::window::PlatformSpecificOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWindowState { fn clone(&self) -> Self { let r: &azul_impl::window::WindowState = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCallbackInfo { fn clone(&self) -> Self { let r: &azul_impl::callbacks::CallbackInfo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzInlineText { fn clone(&self) -> Self { let r: &azul_impl::callbacks::InlineText = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFocusTargetPath { fn clone(&self) -> Self { let r: &azul_impl::callbacks::FocusTargetPath = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzAnimation { fn clone(&self) -> Self { let r: &azul_impl::callbacks::Animation = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzTimerCallbackInfo { fn clone(&self) -> Self { let r: &azul_impl::callbacks::TimerCallbackInfo = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeDataInlineCssPropertyEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::NodeDataInlineCssProperty = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDynamicCssProperty { fn clone(&self) -> Self { let r: &azul_impl::css::DynamicCssProperty = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgNodeEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgNode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgStyledNode { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgStyledNode = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeDataInlineCssPropertyVec { fn clone(&self) -> Self { let r: &azul_impl::dom::NodeDataInlineCssPropertyVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionWindowStateEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::window::OptionWindowState = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionInlineTextEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::OptionInlineText = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXmlParseErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::xml::XmlParseError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzWindowCreateOptions { fn clone(&self) -> Self { let r: &azul_impl::window::WindowCreateOptions = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzFocusTargetEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::callbacks::FocusTarget = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeData { fn clone(&self) -> Self { let r: &azul_impl::dom::NodeData = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssDeclarationEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::css::CssDeclaration = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssDeclarationVec { fn clone(&self) -> Self { let r: &azul_impl::css::CssDeclarationVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzNodeDataVec { fn clone(&self) -> Self { let r: &azul_impl::dom::NodeDataVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzXmlErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::xml::XmlError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzDom { fn clone(&self) -> Self { let r: &azul_impl::dom::Dom = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssRuleBlock { fn clone(&self) -> Self { let r: &azul_impl::css::CssRuleBlock = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStyledDom { fn clone(&self) -> Self { let r: &azul_impl::styled_dom::StyledDom = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCssRuleBlockVec { fn clone(&self) -> Self { let r: &azul_impl::css::CssRuleBlockVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzOptionDomEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::dom::OptionDom = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzResultXmlXmlErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::xml::ResultXmlXmlError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzSvgParseErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::SvgParseError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzIFrameCallbackReturn { fn clone(&self) -> Self { let r: &azul_impl::callbacks::IFrameCallbackReturn = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStylesheet { fn clone(&self) -> Self { let r: &azul_impl::css::Stylesheet = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzStylesheetVec { fn clone(&self) -> Self { let r: &azul_impl::css::StylesheetVec = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzResultSvgXmlNodeSvgParseErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::ResultSvgXmlNodeSvgParseError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzResultSvgSvgParseErrorEnumWrapper { fn clone(&self) -> Self { let r: &azul_impl::svg::ResultSvgSvgParseError = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }
impl Clone for AzCss { fn clone(&self) -> Self { let r: &azul_impl::css::Css = unsafe { mem::transmute(self) }; unsafe { mem::transmute(r.clone()) } } }

// Implement Drop for all objects with drop constructors
impl Drop for AzApp { fn drop(&mut self) { crate::AzApp_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzRefCount { fn drop(&mut self) { crate::AzRefCount_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzCssPropertyCache { fn drop(&mut self) { crate::AzCssPropertyCache_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzGLsyncPtr { fn drop(&mut self) { crate::AzGLsyncPtr_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzImageRef { fn drop(&mut self) { crate::AzImageRef_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzFontRef { fn drop(&mut self) { crate::AzFontRef_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzSvg { fn drop(&mut self) { crate::AzSvg_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzSvgXmlNode { fn drop(&mut self) { crate::AzSvgXmlNode_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzFile { fn drop(&mut self) { crate::AzFile_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzSystemClipboard { fn drop(&mut self) { crate::AzSystemClipboard_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzThread { fn drop(&mut self) { crate::AzThread_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzThreadSender { fn drop(&mut self) { crate::AzThreadSender_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzThreadReceiver { fn drop(&mut self) { crate::AzThreadReceiver_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzRefAny { fn drop(&mut self) { crate::AzRefAny_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzGl { fn drop(&mut self) { crate::AzGl_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzInstantPtr { fn drop(&mut self) { crate::AzInstantPtr_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzXmlNodeVec { fn drop(&mut self) { crate::AzXmlNodeVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzInlineGlyphVec { fn drop(&mut self) { crate::AzInlineGlyphVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzInlineTextHitVec { fn drop(&mut self) { crate::AzInlineTextHitVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzVideoModeVec { fn drop(&mut self) { crate::AzVideoModeVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzDomVec { fn drop(&mut self) { crate::AzDomVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzStyleBackgroundPositionVec { fn drop(&mut self) { crate::AzStyleBackgroundPositionVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzStyleBackgroundRepeatVec { fn drop(&mut self) { crate::AzStyleBackgroundRepeatVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzStyleBackgroundSizeVec { fn drop(&mut self) { crate::AzStyleBackgroundSizeVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzSvgVertexVec { fn drop(&mut self) { crate::AzSvgVertexVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzU32Vec { fn drop(&mut self) { crate::AzU32Vec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzXWindowTypeVec { fn drop(&mut self) { crate::AzXWindowTypeVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzVirtualKeyCodeVec { fn drop(&mut self) { crate::AzVirtualKeyCodeVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzCascadeInfoVec { fn drop(&mut self) { crate::AzCascadeInfoVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzScanCodeVec { fn drop(&mut self) { crate::AzScanCodeVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzU16Vec { fn drop(&mut self) { crate::AzU16Vec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzF32Vec { fn drop(&mut self) { crate::AzF32Vec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzU8Vec { fn drop(&mut self) { crate::AzU8Vec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzGLuintVec { fn drop(&mut self) { crate::AzGLuintVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzGLintVec { fn drop(&mut self) { crate::AzGLintVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzNormalizedLinearColorStopVec { fn drop(&mut self) { crate::AzNormalizedLinearColorStopVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzNormalizedRadialColorStopVec { fn drop(&mut self) { crate::AzNormalizedRadialColorStopVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzNodeIdVec { fn drop(&mut self) { crate::AzNodeIdVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzNodeVec { fn drop(&mut self) { crate::AzNodeVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzParentWithNodeDepthVec { fn drop(&mut self) { crate::AzParentWithNodeDepthVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzTexture { fn drop(&mut self) { crate::AzTexture_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzTesselatedSvgNodeVec { fn drop(&mut self) { crate::AzTesselatedSvgNodeVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzStyleTransformVec { fn drop(&mut self) { crate::AzStyleTransformVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzSvgPathElementVec { fn drop(&mut self) { crate::AzSvgPathElementVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzStringVec { fn drop(&mut self) { crate::AzStringVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzStyledNodeVec { fn drop(&mut self) { crate::AzStyledNodeVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzTagIdsToNodeIdsMappingVec { fn drop(&mut self) { crate::AzTagIdsToNodeIdsMappingVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzStyleFontFamilyVec { fn drop(&mut self) { crate::AzStyleFontFamilyVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzFmtArgVec { fn drop(&mut self) { crate::AzFmtArgVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzInlineWordVec { fn drop(&mut self) { crate::AzInlineWordVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzMonitorVec { fn drop(&mut self) { crate::AzMonitorVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzIdOrClassVec { fn drop(&mut self) { crate::AzIdOrClassVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzStyleBackgroundContentVec { fn drop(&mut self) { crate::AzStyleBackgroundContentVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzSvgPathVec { fn drop(&mut self) { crate::AzSvgPathVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzVertexAttributeVec { fn drop(&mut self) { crate::AzVertexAttributeVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzCssPathSelectorVec { fn drop(&mut self) { crate::AzCssPathSelectorVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzCallbackDataVec { fn drop(&mut self) { crate::AzCallbackDataVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzDebugMessageVec { fn drop(&mut self) { crate::AzDebugMessageVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzStringPairVec { fn drop(&mut self) { crate::AzStringPairVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzInlineLineVec { fn drop(&mut self) { crate::AzInlineLineVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzCssPropertyVec { fn drop(&mut self) { crate::AzCssPropertyVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzSvgMultiPolygonVec { fn drop(&mut self) { crate::AzSvgMultiPolygonVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzNodeDataInlineCssPropertyVec { fn drop(&mut self) { crate::AzNodeDataInlineCssPropertyVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzCssDeclarationVec { fn drop(&mut self) { crate::AzCssDeclarationVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzNodeDataVec { fn drop(&mut self) { crate::AzNodeDataVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzCssRuleBlockVec { fn drop(&mut self) { crate::AzCssRuleBlockVec_delete(unsafe { mem::transmute(self) }); } }
impl Drop for AzStylesheetVec { fn drop(&mut self) { crate::AzStylesheetVec_delete(unsafe { mem::transmute(self) }); } }


#[pymethods]
impl AzApp {
    fn add_window(&mut self, window: AzWindowCreateOptions) -> () {
        unsafe { mem::transmute(crate::AzApp_addWindow(
            mem::transmute(self),
            mem::transmute(window),
        )) }
    }
    fn add_image(&mut self, id: String, image: AzImageRef) -> () {
        let id = pystring_to_azstring(&id);
        unsafe { mem::transmute(crate::AzApp_addImage(
            mem::transmute(self),
            mem::transmute(id),
            mem::transmute(image),
        )) }
    }
    fn get_monitors(&self) -> AzMonitorVec {
        unsafe { mem::transmute(crate::AzApp_getMonitors(
            mem::transmute(self),
        )) }
    }
    fn run(&self, window: AzWindowCreateOptions) -> () {
        unsafe { mem::transmute(crate::AzApp_run(
            mem::transmute(self),
            mem::transmute(window),
        )) }
    }
}

#[pymethods]
impl AzAppConfig {
    #[staticmethod]
    fn new(layout_solver: AzLayoutSolverEnumWrapper) -> AzAppConfig {
        unsafe { mem::transmute(crate::AzAppConfig_new(
            mem::transmute(layout_solver),
        )) }
    }
}

#[pymethods]
impl AzAppLogLevelEnumWrapper {
    #[classattr]
    fn Off() -> AzAppLogLevelEnumWrapper { AzAppLogLevelEnumWrapper { inner: AzAppLogLevel::Off } }
    #[classattr]
    fn Error() -> AzAppLogLevelEnumWrapper { AzAppLogLevelEnumWrapper { inner: AzAppLogLevel::Error } }
    #[classattr]
    fn Warn() -> AzAppLogLevelEnumWrapper { AzAppLogLevelEnumWrapper { inner: AzAppLogLevel::Warn } }
    #[classattr]
    fn Info() -> AzAppLogLevelEnumWrapper { AzAppLogLevelEnumWrapper { inner: AzAppLogLevel::Info } }
    #[classattr]
    fn Debug() -> AzAppLogLevelEnumWrapper { AzAppLogLevelEnumWrapper { inner: AzAppLogLevel::Debug } }
    #[classattr]
    fn Trace() -> AzAppLogLevelEnumWrapper { AzAppLogLevelEnumWrapper { inner: AzAppLogLevel::Trace } }
}

#[pymethods]
impl AzLayoutSolverEnumWrapper {
    #[classattr]
    fn Default() -> AzLayoutSolverEnumWrapper { AzLayoutSolverEnumWrapper { inner: AzLayoutSolver::Default } }
}

#[pymethods]
impl AzSystemCallbacks {
    #[staticmethod]
    fn library_internal() -> AzSystemCallbacks {
        unsafe { mem::transmute(crate::AzSystemCallbacks_libraryInternal()) }
    }
}

#[pymethods]
impl AzWindowCreateOptions {
}

#[pymethods]
impl AzVsyncEnumWrapper {
    #[classattr]
    fn Enabled() -> AzVsyncEnumWrapper { AzVsyncEnumWrapper { inner: AzVsync::Enabled } }
    #[classattr]
    fn Disabled() -> AzVsyncEnumWrapper { AzVsyncEnumWrapper { inner: AzVsync::Disabled } }
}

#[pymethods]
impl AzSrgbEnumWrapper {
    #[classattr]
    fn Enabled() -> AzSrgbEnumWrapper { AzSrgbEnumWrapper { inner: AzSrgb::Enabled } }
    #[classattr]
    fn Disabled() -> AzSrgbEnumWrapper { AzSrgbEnumWrapper { inner: AzSrgb::Disabled } }
}

#[pymethods]
impl AzHwAccelerationEnumWrapper {
    #[classattr]
    fn Enabled() -> AzHwAccelerationEnumWrapper { AzHwAccelerationEnumWrapper { inner: AzHwAcceleration::Enabled } }
    #[classattr]
    fn Disabled() -> AzHwAccelerationEnumWrapper { AzHwAccelerationEnumWrapper { inner: AzHwAcceleration::Disabled } }
}

#[pymethods]
impl AzRawWindowHandleEnumWrapper {
    #[staticmethod]
    fn IOS(v: AzIOSHandle) -> AzRawWindowHandleEnumWrapper { AzRawWindowHandleEnumWrapper { inner: AzRawWindowHandle::IOS(v) } }
    #[staticmethod]
    fn MacOS(v: AzMacOSHandle) -> AzRawWindowHandleEnumWrapper { AzRawWindowHandleEnumWrapper { inner: AzRawWindowHandle::MacOS(v) } }
    #[staticmethod]
    fn Xlib(v: AzXlibHandle) -> AzRawWindowHandleEnumWrapper { AzRawWindowHandleEnumWrapper { inner: AzRawWindowHandle::Xlib(v) } }
    #[staticmethod]
    fn Xcb(v: AzXcbHandle) -> AzRawWindowHandleEnumWrapper { AzRawWindowHandleEnumWrapper { inner: AzRawWindowHandle::Xcb(v) } }
    #[staticmethod]
    fn Wayland(v: AzWaylandHandle) -> AzRawWindowHandleEnumWrapper { AzRawWindowHandleEnumWrapper { inner: AzRawWindowHandle::Wayland(v) } }
    #[staticmethod]
    fn Windows(v: AzWindowsHandle) -> AzRawWindowHandleEnumWrapper { AzRawWindowHandleEnumWrapper { inner: AzRawWindowHandle::Windows(v) } }
    #[staticmethod]
    fn Web(v: AzWebHandle) -> AzRawWindowHandleEnumWrapper { AzRawWindowHandleEnumWrapper { inner: AzRawWindowHandle::Web(v) } }
    #[staticmethod]
    fn Android(v: AzAndroidHandle) -> AzRawWindowHandleEnumWrapper { AzRawWindowHandleEnumWrapper { inner: AzRawWindowHandle::Android(v) } }
    #[classattr]
    fn Unsupported() -> AzRawWindowHandleEnumWrapper { AzRawWindowHandleEnumWrapper { inner: AzRawWindowHandle::Unsupported } }
}

#[pymethods]
impl AzXWindowTypeEnumWrapper {
    #[classattr]
    fn Desktop() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Desktop } }
    #[classattr]
    fn Dock() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Dock } }
    #[classattr]
    fn Toolbar() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Toolbar } }
    #[classattr]
    fn Menu() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Menu } }
    #[classattr]
    fn Utility() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Utility } }
    #[classattr]
    fn Splash() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Splash } }
    #[classattr]
    fn Dialog() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Dialog } }
    #[classattr]
    fn DropdownMenu() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::DropdownMenu } }
    #[classattr]
    fn PopupMenu() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::PopupMenu } }
    #[classattr]
    fn Tooltip() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Tooltip } }
    #[classattr]
    fn Notification() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Notification } }
    #[classattr]
    fn Combo() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Combo } }
    #[classattr]
    fn Dnd() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Dnd } }
    #[classattr]
    fn Normal() -> AzXWindowTypeEnumWrapper { AzXWindowTypeEnumWrapper { inner: AzXWindowType::Normal } }
}

#[pymethods]
impl AzWindowIconEnumWrapper {
    #[staticmethod]
    fn Small(v: AzSmallWindowIconBytes) -> AzWindowIconEnumWrapper { AzWindowIconEnumWrapper { inner: AzWindowIcon::Small(v) } }
    #[staticmethod]
    fn Large(v: AzLargeWindowIconBytes) -> AzWindowIconEnumWrapper { AzWindowIconEnumWrapper { inner: AzWindowIcon::Large(v) } }
}

#[pymethods]
impl AzVirtualKeyCodeEnumWrapper {
    #[classattr]
    fn Key1() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Key1 } }
    #[classattr]
    fn Key2() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Key2 } }
    #[classattr]
    fn Key3() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Key3 } }
    #[classattr]
    fn Key4() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Key4 } }
    #[classattr]
    fn Key5() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Key5 } }
    #[classattr]
    fn Key6() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Key6 } }
    #[classattr]
    fn Key7() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Key7 } }
    #[classattr]
    fn Key8() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Key8 } }
    #[classattr]
    fn Key9() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Key9 } }
    #[classattr]
    fn Key0() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Key0 } }
    #[classattr]
    fn A() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::A } }
    #[classattr]
    fn B() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::B } }
    #[classattr]
    fn C() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::C } }
    #[classattr]
    fn D() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::D } }
    #[classattr]
    fn E() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::E } }
    #[classattr]
    fn F() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F } }
    #[classattr]
    fn G() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::G } }
    #[classattr]
    fn H() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::H } }
    #[classattr]
    fn I() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::I } }
    #[classattr]
    fn J() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::J } }
    #[classattr]
    fn K() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::K } }
    #[classattr]
    fn L() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::L } }
    #[classattr]
    fn M() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::M } }
    #[classattr]
    fn N() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::N } }
    #[classattr]
    fn O() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::O } }
    #[classattr]
    fn P() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::P } }
    #[classattr]
    fn Q() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Q } }
    #[classattr]
    fn R() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::R } }
    #[classattr]
    fn S() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::S } }
    #[classattr]
    fn T() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::T } }
    #[classattr]
    fn U() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::U } }
    #[classattr]
    fn V() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::V } }
    #[classattr]
    fn W() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::W } }
    #[classattr]
    fn X() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::X } }
    #[classattr]
    fn Y() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Y } }
    #[classattr]
    fn Z() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Z } }
    #[classattr]
    fn Escape() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Escape } }
    #[classattr]
    fn F1() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F1 } }
    #[classattr]
    fn F2() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F2 } }
    #[classattr]
    fn F3() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F3 } }
    #[classattr]
    fn F4() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F4 } }
    #[classattr]
    fn F5() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F5 } }
    #[classattr]
    fn F6() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F6 } }
    #[classattr]
    fn F7() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F7 } }
    #[classattr]
    fn F8() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F8 } }
    #[classattr]
    fn F9() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F9 } }
    #[classattr]
    fn F10() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F10 } }
    #[classattr]
    fn F11() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F11 } }
    #[classattr]
    fn F12() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F12 } }
    #[classattr]
    fn F13() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F13 } }
    #[classattr]
    fn F14() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F14 } }
    #[classattr]
    fn F15() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F15 } }
    #[classattr]
    fn F16() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F16 } }
    #[classattr]
    fn F17() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F17 } }
    #[classattr]
    fn F18() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F18 } }
    #[classattr]
    fn F19() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F19 } }
    #[classattr]
    fn F20() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F20 } }
    #[classattr]
    fn F21() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F21 } }
    #[classattr]
    fn F22() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F22 } }
    #[classattr]
    fn F23() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F23 } }
    #[classattr]
    fn F24() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::F24 } }
    #[classattr]
    fn Snapshot() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Snapshot } }
    #[classattr]
    fn Scroll() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Scroll } }
    #[classattr]
    fn Pause() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Pause } }
    #[classattr]
    fn Insert() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Insert } }
    #[classattr]
    fn Home() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Home } }
    #[classattr]
    fn Delete() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Delete } }
    #[classattr]
    fn End() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::End } }
    #[classattr]
    fn PageDown() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::PageDown } }
    #[classattr]
    fn PageUp() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::PageUp } }
    #[classattr]
    fn Left() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Left } }
    #[classattr]
    fn Up() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Up } }
    #[classattr]
    fn Right() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Right } }
    #[classattr]
    fn Down() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Down } }
    #[classattr]
    fn Back() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Back } }
    #[classattr]
    fn Return() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Return } }
    #[classattr]
    fn Space() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Space } }
    #[classattr]
    fn Compose() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Compose } }
    #[classattr]
    fn Caret() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Caret } }
    #[classattr]
    fn Numlock() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numlock } }
    #[classattr]
    fn Numpad0() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numpad0 } }
    #[classattr]
    fn Numpad1() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numpad1 } }
    #[classattr]
    fn Numpad2() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numpad2 } }
    #[classattr]
    fn Numpad3() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numpad3 } }
    #[classattr]
    fn Numpad4() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numpad4 } }
    #[classattr]
    fn Numpad5() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numpad5 } }
    #[classattr]
    fn Numpad6() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numpad6 } }
    #[classattr]
    fn Numpad7() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numpad7 } }
    #[classattr]
    fn Numpad8() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numpad8 } }
    #[classattr]
    fn Numpad9() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Numpad9 } }
    #[classattr]
    fn NumpadAdd() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NumpadAdd } }
    #[classattr]
    fn NumpadDivide() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NumpadDivide } }
    #[classattr]
    fn NumpadDecimal() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NumpadDecimal } }
    #[classattr]
    fn NumpadComma() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NumpadComma } }
    #[classattr]
    fn NumpadEnter() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NumpadEnter } }
    #[classattr]
    fn NumpadEquals() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NumpadEquals } }
    #[classattr]
    fn NumpadMultiply() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NumpadMultiply } }
    #[classattr]
    fn NumpadSubtract() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NumpadSubtract } }
    #[classattr]
    fn AbntC1() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::AbntC1 } }
    #[classattr]
    fn AbntC2() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::AbntC2 } }
    #[classattr]
    fn Apostrophe() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Apostrophe } }
    #[classattr]
    fn Apps() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Apps } }
    #[classattr]
    fn Asterisk() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Asterisk } }
    #[classattr]
    fn At() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::At } }
    #[classattr]
    fn Ax() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Ax } }
    #[classattr]
    fn Backslash() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Backslash } }
    #[classattr]
    fn Calculator() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Calculator } }
    #[classattr]
    fn Capital() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Capital } }
    #[classattr]
    fn Colon() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Colon } }
    #[classattr]
    fn Comma() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Comma } }
    #[classattr]
    fn Convert() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Convert } }
    #[classattr]
    fn Equals() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Equals } }
    #[classattr]
    fn Grave() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Grave } }
    #[classattr]
    fn Kana() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Kana } }
    #[classattr]
    fn Kanji() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Kanji } }
    #[classattr]
    fn LAlt() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::LAlt } }
    #[classattr]
    fn LBracket() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::LBracket } }
    #[classattr]
    fn LControl() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::LControl } }
    #[classattr]
    fn LShift() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::LShift } }
    #[classattr]
    fn LWin() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::LWin } }
    #[classattr]
    fn Mail() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Mail } }
    #[classattr]
    fn MediaSelect() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::MediaSelect } }
    #[classattr]
    fn MediaStop() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::MediaStop } }
    #[classattr]
    fn Minus() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Minus } }
    #[classattr]
    fn Mute() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Mute } }
    #[classattr]
    fn MyComputer() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::MyComputer } }
    #[classattr]
    fn NavigateForward() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NavigateForward } }
    #[classattr]
    fn NavigateBackward() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NavigateBackward } }
    #[classattr]
    fn NextTrack() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NextTrack } }
    #[classattr]
    fn NoConvert() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::NoConvert } }
    #[classattr]
    fn OEM102() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::OEM102 } }
    #[classattr]
    fn Period() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Period } }
    #[classattr]
    fn PlayPause() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::PlayPause } }
    #[classattr]
    fn Plus() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Plus } }
    #[classattr]
    fn Power() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Power } }
    #[classattr]
    fn PrevTrack() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::PrevTrack } }
    #[classattr]
    fn RAlt() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::RAlt } }
    #[classattr]
    fn RBracket() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::RBracket } }
    #[classattr]
    fn RControl() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::RControl } }
    #[classattr]
    fn RShift() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::RShift } }
    #[classattr]
    fn RWin() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::RWin } }
    #[classattr]
    fn Semicolon() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Semicolon } }
    #[classattr]
    fn Slash() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Slash } }
    #[classattr]
    fn Sleep() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Sleep } }
    #[classattr]
    fn Stop() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Stop } }
    #[classattr]
    fn Sysrq() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Sysrq } }
    #[classattr]
    fn Tab() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Tab } }
    #[classattr]
    fn Underline() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Underline } }
    #[classattr]
    fn Unlabeled() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Unlabeled } }
    #[classattr]
    fn VolumeDown() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::VolumeDown } }
    #[classattr]
    fn VolumeUp() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::VolumeUp } }
    #[classattr]
    fn Wake() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Wake } }
    #[classattr]
    fn WebBack() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::WebBack } }
    #[classattr]
    fn WebFavorites() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::WebFavorites } }
    #[classattr]
    fn WebForward() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::WebForward } }
    #[classattr]
    fn WebHome() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::WebHome } }
    #[classattr]
    fn WebRefresh() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::WebRefresh } }
    #[classattr]
    fn WebSearch() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::WebSearch } }
    #[classattr]
    fn WebStop() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::WebStop } }
    #[classattr]
    fn Yen() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Yen } }
    #[classattr]
    fn Copy() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Copy } }
    #[classattr]
    fn Paste() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Paste } }
    #[classattr]
    fn Cut() -> AzVirtualKeyCodeEnumWrapper { AzVirtualKeyCodeEnumWrapper { inner: AzVirtualKeyCode::Cut } }
}

#[pymethods]
impl AzAcceleratorKeyEnumWrapper {
    #[classattr]
    fn Ctrl() -> AzAcceleratorKeyEnumWrapper { AzAcceleratorKeyEnumWrapper { inner: AzAcceleratorKey::Ctrl } }
    #[classattr]
    fn Alt() -> AzAcceleratorKeyEnumWrapper { AzAcceleratorKeyEnumWrapper { inner: AzAcceleratorKey::Alt } }
    #[classattr]
    fn Shift() -> AzAcceleratorKeyEnumWrapper { AzAcceleratorKeyEnumWrapper { inner: AzAcceleratorKey::Shift } }
    #[staticmethod]
    fn Key(v: AzVirtualKeyCodeEnumWrapper) -> AzAcceleratorKeyEnumWrapper { AzAcceleratorKeyEnumWrapper { inner: AzAcceleratorKey::Key(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzMouseCursorTypeEnumWrapper {
    #[classattr]
    fn Default() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Default } }
    #[classattr]
    fn Crosshair() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Crosshair } }
    #[classattr]
    fn Hand() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Hand } }
    #[classattr]
    fn Arrow() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Arrow } }
    #[classattr]
    fn Move() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Move } }
    #[classattr]
    fn Text() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Text } }
    #[classattr]
    fn Wait() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Wait } }
    #[classattr]
    fn Help() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Help } }
    #[classattr]
    fn Progress() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Progress } }
    #[classattr]
    fn NotAllowed() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::NotAllowed } }
    #[classattr]
    fn ContextMenu() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::ContextMenu } }
    #[classattr]
    fn Cell() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Cell } }
    #[classattr]
    fn VerticalText() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::VerticalText } }
    #[classattr]
    fn Alias() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Alias } }
    #[classattr]
    fn Copy() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Copy } }
    #[classattr]
    fn NoDrop() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::NoDrop } }
    #[classattr]
    fn Grab() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Grab } }
    #[classattr]
    fn Grabbing() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::Grabbing } }
    #[classattr]
    fn AllScroll() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::AllScroll } }
    #[classattr]
    fn ZoomIn() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::ZoomIn } }
    #[classattr]
    fn ZoomOut() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::ZoomOut } }
    #[classattr]
    fn EResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::EResize } }
    #[classattr]
    fn NResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::NResize } }
    #[classattr]
    fn NeResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::NeResize } }
    #[classattr]
    fn NwResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::NwResize } }
    #[classattr]
    fn SResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::SResize } }
    #[classattr]
    fn SeResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::SeResize } }
    #[classattr]
    fn SwResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::SwResize } }
    #[classattr]
    fn WResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::WResize } }
    #[classattr]
    fn EwResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::EwResize } }
    #[classattr]
    fn NsResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::NsResize } }
    #[classattr]
    fn NeswResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::NeswResize } }
    #[classattr]
    fn NwseResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::NwseResize } }
    #[classattr]
    fn ColResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::ColResize } }
    #[classattr]
    fn RowResize() -> AzMouseCursorTypeEnumWrapper { AzMouseCursorTypeEnumWrapper { inner: AzMouseCursorType::RowResize } }
}

#[pymethods]
impl AzCursorPositionEnumWrapper {
    #[classattr]
    fn OutOfWindow() -> AzCursorPositionEnumWrapper { AzCursorPositionEnumWrapper { inner: AzCursorPosition::OutOfWindow } }
    #[classattr]
    fn Uninitialized() -> AzCursorPositionEnumWrapper { AzCursorPositionEnumWrapper { inner: AzCursorPosition::Uninitialized } }
    #[staticmethod]
    fn InWindow(v: AzLogicalPosition) -> AzCursorPositionEnumWrapper { AzCursorPositionEnumWrapper { inner: AzCursorPosition::InWindow(v) } }
}

#[pymethods]
impl AzRendererTypeEnumWrapper {
    #[classattr]
    fn Hardware() -> AzRendererTypeEnumWrapper { AzRendererTypeEnumWrapper { inner: AzRendererType::Hardware } }
    #[classattr]
    fn Software() -> AzRendererTypeEnumWrapper { AzRendererTypeEnumWrapper { inner: AzRendererType::Software } }
}

#[pymethods]
impl AzFullScreenModeEnumWrapper {
    #[classattr]
    fn SlowFullScreen() -> AzFullScreenModeEnumWrapper { AzFullScreenModeEnumWrapper { inner: AzFullScreenMode::SlowFullScreen } }
    #[classattr]
    fn FastFullScreen() -> AzFullScreenModeEnumWrapper { AzFullScreenModeEnumWrapper { inner: AzFullScreenMode::FastFullScreen } }
    #[classattr]
    fn SlowWindowed() -> AzFullScreenModeEnumWrapper { AzFullScreenModeEnumWrapper { inner: AzFullScreenMode::SlowWindowed } }
    #[classattr]
    fn FastWindowed() -> AzFullScreenModeEnumWrapper { AzFullScreenModeEnumWrapper { inner: AzFullScreenMode::FastWindowed } }
}

#[pymethods]
impl AzWindowThemeEnumWrapper {
    #[classattr]
    fn DarkMode() -> AzWindowThemeEnumWrapper { AzWindowThemeEnumWrapper { inner: AzWindowTheme::DarkMode } }
    #[classattr]
    fn LightMode() -> AzWindowThemeEnumWrapper { AzWindowThemeEnumWrapper { inner: AzWindowTheme::LightMode } }
}

#[pymethods]
impl AzWindowPositionEnumWrapper {
    #[classattr]
    fn Uninitialized() -> AzWindowPositionEnumWrapper { AzWindowPositionEnumWrapper { inner: AzWindowPosition::Uninitialized } }
    #[staticmethod]
    fn Initialized(v: AzPhysicalPositionI32) -> AzWindowPositionEnumWrapper { AzWindowPositionEnumWrapper { inner: AzWindowPosition::Initialized(v) } }
}

#[pymethods]
impl AzImePositionEnumWrapper {
    #[classattr]
    fn Uninitialized() -> AzImePositionEnumWrapper { AzImePositionEnumWrapper { inner: AzImePosition::Uninitialized } }
    #[staticmethod]
    fn Initialized(v: AzLogicalPosition) -> AzImePositionEnumWrapper { AzImePositionEnumWrapper { inner: AzImePosition::Initialized(v) } }
}

#[pymethods]
impl AzWindowState {
    #[staticmethod]
    fn default() -> AzWindowState {
        unsafe { mem::transmute(crate::AzWindowState_default()) }
    }
}

#[pymethods]
impl AzLayoutCallbackEnumWrapper {
    #[staticmethod]
    fn Raw(v: AzLayoutCallbackInner) -> AzLayoutCallbackEnumWrapper { AzLayoutCallbackEnumWrapper { inner: AzLayoutCallback::Raw(v) } }
    #[staticmethod]
    fn Marshaled(v: AzMarshaledLayoutCallback) -> AzLayoutCallbackEnumWrapper { AzLayoutCallbackEnumWrapper { inner: AzLayoutCallback::Marshaled(v) } }
}

#[pymethods]
impl AzCallbackInfo {
    fn get_hit_node(&self) -> AzDomNodeId {
        unsafe { mem::transmute(crate::AzCallbackInfo_getHitNode(
            mem::transmute(self),
        )) }
    }
    fn get_system_time_fn(&self) -> AzGetSystemTimeFn {
        unsafe { mem::transmute(crate::AzCallbackInfo_getSystemTimeFn(
            mem::transmute(self),
        )) }
    }
    fn get_cursor_relative_to_viewport(&self) -> Option<AzLogicalPosition> {
        let m: AzOptionLogicalPosition = unsafe { mem::transmute(crate::AzCallbackInfo_getCursorRelativeToViewport(
            mem::transmute(self),
        )) };
        match m {
            AzOptionLogicalPosition::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionLogicalPosition::None => None,
        }

    }
    fn get_cursor_relative_to_node(&self) -> Option<AzLogicalPosition> {
        let m: AzOptionLogicalPosition = unsafe { mem::transmute(crate::AzCallbackInfo_getCursorRelativeToNode(
            mem::transmute(self),
        )) };
        match m {
            AzOptionLogicalPosition::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionLogicalPosition::None => None,
        }

    }
    fn get_current_window_state(&self) -> AzWindowState {
        unsafe { mem::transmute(crate::AzCallbackInfo_getCurrentWindowState(
            mem::transmute(self),
        )) }
    }
    fn get_current_keyboard_state(&self) -> AzKeyboardState {
        unsafe { mem::transmute(crate::AzCallbackInfo_getCurrentKeyboardState(
            mem::transmute(self),
        )) }
    }
    fn get_current_mouse_state(&self) -> AzMouseState {
        unsafe { mem::transmute(crate::AzCallbackInfo_getCurrentMouseState(
            mem::transmute(self),
        )) }
    }
    fn get_previous_window_state(&self) -> Option<AzWindowState> {
        let m: AzOptionWindowState = unsafe { mem::transmute(crate::AzCallbackInfo_getPreviousWindowState(
            mem::transmute(self),
        )) };
        match m {
            AzOptionWindowState::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionWindowState::None => None,
        }

    }
    fn get_previous_keyboard_state(&self) -> Option<AzKeyboardState> {
        let m: AzOptionKeyboardState = unsafe { mem::transmute(crate::AzCallbackInfo_getPreviousKeyboardState(
            mem::transmute(self),
        )) };
        match m {
            AzOptionKeyboardState::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionKeyboardState::None => None,
        }

    }
    fn get_previous_mouse_state(&self) -> Option<AzMouseState> {
        let m: AzOptionMouseState = unsafe { mem::transmute(crate::AzCallbackInfo_getPreviousMouseState(
            mem::transmute(self),
        )) };
        match m {
            AzOptionMouseState::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionMouseState::None => None,
        }

    }
    fn get_current_window_handle(&self) -> AzRawWindowHandleEnumWrapper {
        unsafe { mem::transmute(crate::AzCallbackInfo_getCurrentWindowHandle(
            mem::transmute(self),
        )) }
    }
    fn get_gl_context(&self) -> Option<AzGl> {
        let m: AzOptionGl = unsafe { mem::transmute(crate::AzCallbackInfo_getGlContext(
            mem::transmute(self),
        )) };
        match m {
            AzOptionGl::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionGl::None => None,
        }

    }
    fn get_scroll_position(&self, node_id: AzDomNodeId) -> Option<AzLogicalPosition> {
        let m: AzOptionLogicalPosition = unsafe { mem::transmute(crate::AzCallbackInfo_getScrollPosition(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionLogicalPosition::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionLogicalPosition::None => None,
        }

    }
    fn get_dataset(&mut self, node_id: AzDomNodeId) -> Option<AzRefAny> {
        let m: AzOptionRefAny = unsafe { mem::transmute(crate::AzCallbackInfo_getDataset(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionRefAny::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionRefAny::None => None,
        }

    }
    fn get_string_contents(&self, node_id: AzDomNodeId) -> Option<String> {
        let m: AzOptionString = unsafe { mem::transmute(crate::AzCallbackInfo_getStringContents(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionString::Some(s) => Some({ let s: AzString = unsafe { mem::transmute(s) }; s.into() }),
            AzOptionString::None => None,
        }

    }
    fn get_inline_text(&self, node_id: AzDomNodeId) -> Option<AzInlineText> {
        let m: AzOptionInlineText = unsafe { mem::transmute(crate::AzCallbackInfo_getInlineText(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionInlineText::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionInlineText::None => None,
        }

    }
    fn get_index_in_parent(&mut self, node_id: AzDomNodeId) -> usize {
        unsafe { mem::transmute(crate::AzCallbackInfo_getIndexInParent(
            mem::transmute(self),
            mem::transmute(node_id),
        )) }
    }
    fn get_parent(&mut self, node_id: AzDomNodeId) -> Option<AzDomNodeId> {
        let m: AzOptionDomNodeId = unsafe { mem::transmute(crate::AzCallbackInfo_getParent(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionDomNodeId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionDomNodeId::None => None,
        }

    }
    fn get_previous_sibling(&mut self, node_id: AzDomNodeId) -> Option<AzDomNodeId> {
        let m: AzOptionDomNodeId = unsafe { mem::transmute(crate::AzCallbackInfo_getPreviousSibling(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionDomNodeId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionDomNodeId::None => None,
        }

    }
    fn get_next_sibling(&mut self, node_id: AzDomNodeId) -> Option<AzDomNodeId> {
        let m: AzOptionDomNodeId = unsafe { mem::transmute(crate::AzCallbackInfo_getNextSibling(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionDomNodeId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionDomNodeId::None => None,
        }

    }
    fn get_first_child(&mut self, node_id: AzDomNodeId) -> Option<AzDomNodeId> {
        let m: AzOptionDomNodeId = unsafe { mem::transmute(crate::AzCallbackInfo_getFirstChild(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionDomNodeId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionDomNodeId::None => None,
        }

    }
    fn get_last_child(&mut self, node_id: AzDomNodeId) -> Option<AzDomNodeId> {
        let m: AzOptionDomNodeId = unsafe { mem::transmute(crate::AzCallbackInfo_getLastChild(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionDomNodeId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionDomNodeId::None => None,
        }

    }
    fn get_node_position(&mut self, node_id: AzDomNodeId) -> Option<AzPositionInfoEnumWrapper> {
        let m: AzOptionPositionInfo = unsafe { mem::transmute(crate::AzCallbackInfo_getNodePosition(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionPositionInfo::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionPositionInfo::None => None,
        }

    }
    fn get_node_size(&mut self, node_id: AzDomNodeId) -> Option<AzLogicalSize> {
        let m: AzOptionLogicalSize = unsafe { mem::transmute(crate::AzCallbackInfo_getNodeSize(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionLogicalSize::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionLogicalSize::None => None,
        }

    }
    fn get_computed_css_property(&mut self, node_id: AzDomNodeId, property_type: AzCssPropertyTypeEnumWrapper) -> Option<AzCssPropertyEnumWrapper> {
        let m: AzOptionCssProperty = unsafe { mem::transmute(crate::AzCallbackInfo_getComputedCssProperty(
            mem::transmute(self),
            mem::transmute(node_id),
            mem::transmute(property_type),
        )) };
        match m {
            AzOptionCssProperty::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionCssProperty::None => None,
        }

    }
    fn set_window_state(&mut self, new_state: AzWindowState) -> () {
        unsafe { mem::transmute(crate::AzCallbackInfo_setWindowState(
            mem::transmute(self),
            mem::transmute(new_state),
        )) }
    }
    fn set_focus(&mut self, target: AzFocusTargetEnumWrapper) -> () {
        unsafe { mem::transmute(crate::AzCallbackInfo_setFocus(
            mem::transmute(self),
            mem::transmute(target),
        )) }
    }
    fn set_css_property(&mut self, node_id: AzDomNodeId, new_property: AzCssPropertyEnumWrapper) -> () {
        unsafe { mem::transmute(crate::AzCallbackInfo_setCssProperty(
            mem::transmute(self),
            mem::transmute(node_id),
            mem::transmute(new_property),
        )) }
    }
    fn set_scroll_position(&mut self, node_id: AzDomNodeId, scroll_position: AzLogicalPosition) -> () {
        unsafe { mem::transmute(crate::AzCallbackInfo_setScrollPosition(
            mem::transmute(self),
            mem::transmute(node_id),
            mem::transmute(scroll_position),
        )) }
    }
    fn set_string_contents(&mut self, node_id: AzDomNodeId, string: String) -> () {
        let string = pystring_to_azstring(&string);
        unsafe { mem::transmute(crate::AzCallbackInfo_setStringContents(
            mem::transmute(self),
            mem::transmute(node_id),
            mem::transmute(string),
        )) }
    }
    fn add_image(&mut self, id: String, image: AzImageRef) -> () {
        let id = pystring_to_azstring(&id);
        unsafe { mem::transmute(crate::AzCallbackInfo_addImage(
            mem::transmute(self),
            mem::transmute(id),
            mem::transmute(image),
        )) }
    }
    fn has_image(&self, id: String) -> bool {
        let id = pystring_to_azstring(&id);
        unsafe { mem::transmute(crate::AzCallbackInfo_hasImage(
            mem::transmute(self),
            mem::transmute(id),
        )) }
    }
    fn get_image(&self, id: String) -> Option<AzImageRef> {
        let id = pystring_to_azstring(&id);
        let m: AzOptionImageRef = unsafe { mem::transmute(crate::AzCallbackInfo_getImage(
            mem::transmute(self),
            mem::transmute(id),
        )) };
        match m {
            AzOptionImageRef::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionImageRef::None => None,
        }

    }
    fn update_image(&mut self, node_id: AzDomNodeId, new_image: AzImageRef) -> () {
        unsafe { mem::transmute(crate::AzCallbackInfo_updateImage(
            mem::transmute(self),
            mem::transmute(node_id),
            mem::transmute(new_image),
        )) }
    }
    fn delete_image(&mut self, id: String) -> () {
        let id = pystring_to_azstring(&id);
        unsafe { mem::transmute(crate::AzCallbackInfo_deleteImage(
            mem::transmute(self),
            mem::transmute(id),
        )) }
    }
    fn update_image_mask(&mut self, node_id: AzDomNodeId, new_mask: AzImageMask) -> () {
        unsafe { mem::transmute(crate::AzCallbackInfo_updateImageMask(
            mem::transmute(self),
            mem::transmute(node_id),
            mem::transmute(new_mask),
        )) }
    }
    fn stop_propagation(&mut self) -> () {
        unsafe { mem::transmute(crate::AzCallbackInfo_stopPropagation(
            mem::transmute(self),
        )) }
    }
    fn create_window(&mut self, new_window: AzWindowCreateOptions) -> () {
        unsafe { mem::transmute(crate::AzCallbackInfo_createWindow(
            mem::transmute(self),
            mem::transmute(new_window),
        )) }
    }
    fn start_timer(&mut self, timer: AzTimer) -> Option<AzTimerId> {
        let m: AzOptionTimerId = unsafe { mem::transmute(crate::AzCallbackInfo_startTimer(
            mem::transmute(self),
            mem::transmute(timer),
        )) };
        match m {
            AzOptionTimerId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionTimerId::None => None,
        }

    }
    fn start_animation(&mut self, node: AzDomNodeId, animation: AzAnimation) -> Option<AzTimerId> {
        let m: AzOptionTimerId = unsafe { mem::transmute(crate::AzCallbackInfo_startAnimation(
            mem::transmute(self),
            mem::transmute(node),
            mem::transmute(animation),
        )) };
        match m {
            AzOptionTimerId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionTimerId::None => None,
        }

    }
    fn stop_timer(&mut self, timer_id: AzTimerId) -> bool {
        unsafe { mem::transmute(crate::AzCallbackInfo_stopTimer(
            mem::transmute(self),
            mem::transmute(timer_id),
        )) }
    }
    fn start_thread(&mut self, thread_initialize_data: AzRefAny, writeback_data: AzRefAny, callback: AzThreadCallback) -> Option<AzThreadId> {
        let m: AzOptionThreadId = unsafe { mem::transmute(crate::AzCallbackInfo_startThread(
            mem::transmute(self),
            mem::transmute(thread_initialize_data),
            mem::transmute(writeback_data),
            mem::transmute(callback),
        )) };
        match m {
            AzOptionThreadId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionThreadId::None => None,
        }

    }
    fn send_thread_msg(&mut self, thread_id: AzThreadId, msg: AzThreadSendMsgEnumWrapper) -> bool {
        unsafe { mem::transmute(crate::AzCallbackInfo_sendThreadMsg(
            mem::transmute(self),
            mem::transmute(thread_id),
            mem::transmute(msg),
        )) }
    }
    fn stop_thread(&mut self, thread_id: AzThreadId) -> bool {
        unsafe { mem::transmute(crate::AzCallbackInfo_stopThread(
            mem::transmute(self),
            mem::transmute(thread_id),
        )) }
    }
}

#[pymethods]
impl AzUpdateEnumWrapper {
    #[classattr]
    fn DoNothing() -> AzUpdateEnumWrapper { AzUpdateEnumWrapper { inner: AzUpdate::DoNothing } }
    #[classattr]
    fn RefreshDom() -> AzUpdateEnumWrapper { AzUpdateEnumWrapper { inner: AzUpdate::RefreshDom } }
    #[classattr]
    fn RefreshDomAllWindows() -> AzUpdateEnumWrapper { AzUpdateEnumWrapper { inner: AzUpdate::RefreshDomAllWindows } }
}

#[pymethods]
impl AzPositionInfoEnumWrapper {
    #[staticmethod]
    fn Static(v: AzPositionInfoInner) -> AzPositionInfoEnumWrapper { AzPositionInfoEnumWrapper { inner: AzPositionInfo::Static(v) } }
    #[staticmethod]
    fn Fixed(v: AzPositionInfoInner) -> AzPositionInfoEnumWrapper { AzPositionInfoEnumWrapper { inner: AzPositionInfo::Fixed(v) } }
    #[staticmethod]
    fn Absolute(v: AzPositionInfoInner) -> AzPositionInfoEnumWrapper { AzPositionInfoEnumWrapper { inner: AzPositionInfo::Absolute(v) } }
    #[staticmethod]
    fn Relative(v: AzPositionInfoInner) -> AzPositionInfoEnumWrapper { AzPositionInfoEnumWrapper { inner: AzPositionInfo::Relative(v) } }
}

#[pymethods]
impl AzHidpiAdjustedBounds {
    fn get_logical_size(&self) -> AzLogicalSize {
        unsafe { mem::transmute(crate::AzHidpiAdjustedBounds_getLogicalSize(
            mem::transmute(self),
        )) }
    }
    fn get_physical_size(&self) -> AzPhysicalSizeU32 {
        unsafe { mem::transmute(crate::AzHidpiAdjustedBounds_getPhysicalSize(
            mem::transmute(self),
        )) }
    }
    fn get_hidpi_factor(&self) -> f32 {
        unsafe { mem::transmute(crate::AzHidpiAdjustedBounds_getHidpiFactor(
            mem::transmute(self),
        )) }
    }
}

#[pymethods]
impl AzInlineText {
    fn hit_test(&self, position: AzLogicalPosition) -> AzInlineTextHitVec {
        unsafe { mem::transmute(crate::AzInlineText_hitTest(
            mem::transmute(self),
            mem::transmute(position),
        )) }
    }
}

#[pymethods]
impl AzInlineWordEnumWrapper {
    #[classattr]
    fn Tab() -> AzInlineWordEnumWrapper { AzInlineWordEnumWrapper { inner: AzInlineWord::Tab } }
    #[classattr]
    fn Return() -> AzInlineWordEnumWrapper { AzInlineWordEnumWrapper { inner: AzInlineWord::Return } }
    #[classattr]
    fn Space() -> AzInlineWordEnumWrapper { AzInlineWordEnumWrapper { inner: AzInlineWord::Space } }
    #[staticmethod]
    fn Word(v: AzInlineTextContents) -> AzInlineWordEnumWrapper { AzInlineWordEnumWrapper { inner: AzInlineWord::Word(v) } }
}

#[pymethods]
impl AzFocusTargetEnumWrapper {
    #[staticmethod]
    fn Id(v: AzDomNodeId) -> AzFocusTargetEnumWrapper { AzFocusTargetEnumWrapper { inner: AzFocusTarget::Id(v) } }
    #[staticmethod]
    fn Path(v: AzFocusTargetPath) -> AzFocusTargetEnumWrapper { AzFocusTargetEnumWrapper { inner: AzFocusTarget::Path(v) } }
    #[classattr]
    fn Previous() -> AzFocusTargetEnumWrapper { AzFocusTargetEnumWrapper { inner: AzFocusTarget::Previous } }
    #[classattr]
    fn Next() -> AzFocusTargetEnumWrapper { AzFocusTargetEnumWrapper { inner: AzFocusTarget::Next } }
    #[classattr]
    fn First() -> AzFocusTargetEnumWrapper { AzFocusTargetEnumWrapper { inner: AzFocusTarget::First } }
    #[classattr]
    fn Last() -> AzFocusTargetEnumWrapper { AzFocusTargetEnumWrapper { inner: AzFocusTarget::Last } }
    #[classattr]
    fn NoFocus() -> AzFocusTargetEnumWrapper { AzFocusTargetEnumWrapper { inner: AzFocusTarget::NoFocus } }
}

#[pymethods]
impl AzAnimationRepeatEnumWrapper {
    #[classattr]
    fn NoRepeat() -> AzAnimationRepeatEnumWrapper { AzAnimationRepeatEnumWrapper { inner: AzAnimationRepeat::NoRepeat } }
    #[classattr]
    fn Loop() -> AzAnimationRepeatEnumWrapper { AzAnimationRepeatEnumWrapper { inner: AzAnimationRepeat::Loop } }
    #[classattr]
    fn PingPong() -> AzAnimationRepeatEnumWrapper { AzAnimationRepeatEnumWrapper { inner: AzAnimationRepeat::PingPong } }
}

#[pymethods]
impl AzAnimationRepeatCountEnumWrapper {
    #[staticmethod]
    fn Times(v: usize) -> AzAnimationRepeatCountEnumWrapper { AzAnimationRepeatCountEnumWrapper { inner: AzAnimationRepeatCount::Times(v) } }
    #[classattr]
    fn Infinite() -> AzAnimationRepeatCountEnumWrapper { AzAnimationRepeatCountEnumWrapper { inner: AzAnimationRepeatCount::Infinite } }
}

#[pymethods]
impl AzAnimationEasingEnumWrapper {
    #[classattr]
    fn Ease() -> AzAnimationEasingEnumWrapper { AzAnimationEasingEnumWrapper { inner: AzAnimationEasing::Ease } }
    #[classattr]
    fn Linear() -> AzAnimationEasingEnumWrapper { AzAnimationEasingEnumWrapper { inner: AzAnimationEasing::Linear } }
    #[classattr]
    fn EaseIn() -> AzAnimationEasingEnumWrapper { AzAnimationEasingEnumWrapper { inner: AzAnimationEasing::EaseIn } }
    #[classattr]
    fn EaseOut() -> AzAnimationEasingEnumWrapper { AzAnimationEasingEnumWrapper { inner: AzAnimationEasing::EaseOut } }
    #[classattr]
    fn EaseInOut() -> AzAnimationEasingEnumWrapper { AzAnimationEasingEnumWrapper { inner: AzAnimationEasing::EaseInOut } }
    #[staticmethod]
    fn CubicBezier(v: AzSvgCubicCurve) -> AzAnimationEasingEnumWrapper { AzAnimationEasingEnumWrapper { inner: AzAnimationEasing::CubicBezier(v) } }
}

#[pymethods]
impl AzRenderImageCallbackInfo {
    fn get_gl_context(&self) -> Option<AzGl> {
        let m: AzOptionGl = unsafe { mem::transmute(crate::AzRenderImageCallbackInfo_getGlContext(
            mem::transmute(self),
        )) };
        match m {
            AzOptionGl::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionGl::None => None,
        }

    }
    fn get_bounds(&self) -> AzHidpiAdjustedBounds {
        unsafe { mem::transmute(crate::AzRenderImageCallbackInfo_getBounds(
            mem::transmute(self),
        )) }
    }
    fn get_callback_node_id(&self) -> AzDomNodeId {
        unsafe { mem::transmute(crate::AzRenderImageCallbackInfo_getCallbackNodeId(
            mem::transmute(self),
        )) }
    }
    fn get_inline_text(&self, node_id: AzDomNodeId) -> Option<AzInlineText> {
        let m: AzOptionInlineText = unsafe { mem::transmute(crate::AzRenderImageCallbackInfo_getInlineText(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionInlineText::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionInlineText::None => None,
        }

    }
    fn get_index_in_parent(&mut self, node_id: AzDomNodeId) -> usize {
        unsafe { mem::transmute(crate::AzRenderImageCallbackInfo_getIndexInParent(
            mem::transmute(self),
            mem::transmute(node_id),
        )) }
    }
    fn get_parent(&mut self, node_id: AzDomNodeId) -> Option<AzDomNodeId> {
        let m: AzOptionDomNodeId = unsafe { mem::transmute(crate::AzRenderImageCallbackInfo_getParent(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionDomNodeId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionDomNodeId::None => None,
        }

    }
    fn get_previous_sibling(&mut self, node_id: AzDomNodeId) -> Option<AzDomNodeId> {
        let m: AzOptionDomNodeId = unsafe { mem::transmute(crate::AzRenderImageCallbackInfo_getPreviousSibling(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionDomNodeId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionDomNodeId::None => None,
        }

    }
    fn get_next_sibling(&mut self, node_id: AzDomNodeId) -> Option<AzDomNodeId> {
        let m: AzOptionDomNodeId = unsafe { mem::transmute(crate::AzRenderImageCallbackInfo_getNextSibling(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionDomNodeId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionDomNodeId::None => None,
        }

    }
    fn get_first_child(&mut self, node_id: AzDomNodeId) -> Option<AzDomNodeId> {
        let m: AzOptionDomNodeId = unsafe { mem::transmute(crate::AzRenderImageCallbackInfo_getFirstChild(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionDomNodeId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionDomNodeId::None => None,
        }

    }
    fn get_last_child(&mut self, node_id: AzDomNodeId) -> Option<AzDomNodeId> {
        let m: AzOptionDomNodeId = unsafe { mem::transmute(crate::AzRenderImageCallbackInfo_getLastChild(
            mem::transmute(self),
            mem::transmute(node_id),
        )) };
        match m {
            AzOptionDomNodeId::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionDomNodeId::None => None,
        }

    }
}

#[pymethods]
impl AzRefCount {
    fn can_be_shared(&self) -> bool {
        unsafe { mem::transmute(crate::AzRefCount_canBeShared(
            mem::transmute(self),
        )) }
    }
    fn can_be_shared_mut(&self) -> bool {
        unsafe { mem::transmute(crate::AzRefCount_canBeSharedMut(
            mem::transmute(self),
        )) }
    }
    fn increase_ref(&mut self) -> () {
        unsafe { mem::transmute(crate::AzRefCount_increaseRef(
            mem::transmute(self),
        )) }
    }
    fn decrease_ref(&mut self) -> () {
        unsafe { mem::transmute(crate::AzRefCount_decreaseRef(
            mem::transmute(self),
        )) }
    }
    fn increase_refmut(&mut self) -> () {
        unsafe { mem::transmute(crate::AzRefCount_increaseRefmut(
            mem::transmute(self),
        )) }
    }
    fn decrease_refmut(&mut self) -> () {
        unsafe { mem::transmute(crate::AzRefCount_decreaseRefmut(
            mem::transmute(self),
        )) }
    }
}

#[pymethods]
impl AzRefAny {
    fn get_type_id(&self) -> u64 {
        unsafe { mem::transmute(crate::AzRefAny_getTypeId(
            mem::transmute(self),
        )) }
    }
    fn get_type_name(&self) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzRefAny_getTypeName(
            mem::transmute(self),
        )) })
    }
}

#[pymethods]
impl AzLayoutCallbackInfo {
    fn get_gl_context(&self) -> Option<AzGl> {
        let m: AzOptionGl = unsafe { mem::transmute(crate::AzLayoutCallbackInfo_getGlContext(
            mem::transmute(self),
        )) };
        match m {
            AzOptionGl::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionGl::None => None,
        }

    }
    fn get_system_fonts(&self) -> AzStringPairVec {
        unsafe { mem::transmute(crate::AzLayoutCallbackInfo_getSystemFonts(
            mem::transmute(self),
        )) }
    }
    fn get_image(&self, id: String) -> Option<AzImageRef> {
        let id = pystring_to_azstring(&id);
        let m: AzOptionImageRef = unsafe { mem::transmute(crate::AzLayoutCallbackInfo_getImage(
            mem::transmute(self),
            mem::transmute(id),
        )) };
        match m {
            AzOptionImageRef::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionImageRef::None => None,
        }

    }
}

#[pymethods]
impl AzDom {
    fn node_count(&self) -> usize {
        unsafe { mem::transmute(crate::AzDom_nodeCount(
            mem::transmute(self),
        )) }
    }
    fn style(&mut self, css: AzCss) -> AzStyledDom {
        unsafe { mem::transmute(crate::AzDom_style(
            mem::transmute(self),
            mem::transmute(css),
        )) }
    }
}

#[pymethods]
impl AzNodeTypeEnumWrapper {
    #[classattr]
    fn Body() -> AzNodeTypeEnumWrapper { AzNodeTypeEnumWrapper { inner: AzNodeType::Body } }
    #[classattr]
    fn Div() -> AzNodeTypeEnumWrapper { AzNodeTypeEnumWrapper { inner: AzNodeType::Div } }
    #[classattr]
    fn Br() -> AzNodeTypeEnumWrapper { AzNodeTypeEnumWrapper { inner: AzNodeType::Br } }
    #[staticmethod]
    fn Text(v: AzString) -> AzNodeTypeEnumWrapper { AzNodeTypeEnumWrapper { inner: AzNodeType::Text(v) } }
    #[staticmethod]
    fn Image(v: AzImageRef) -> AzNodeTypeEnumWrapper { AzNodeTypeEnumWrapper { inner: AzNodeType::Image(v) } }
    #[staticmethod]
    fn IFrame(v: AzIFrameNode) -> AzNodeTypeEnumWrapper { AzNodeTypeEnumWrapper { inner: AzNodeType::IFrame(v) } }
}

#[pymethods]
impl AzOnEnumWrapper {
    #[classattr]
    fn MouseOver() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::MouseOver } }
    #[classattr]
    fn MouseDown() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::MouseDown } }
    #[classattr]
    fn LeftMouseDown() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::LeftMouseDown } }
    #[classattr]
    fn MiddleMouseDown() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::MiddleMouseDown } }
    #[classattr]
    fn RightMouseDown() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::RightMouseDown } }
    #[classattr]
    fn MouseUp() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::MouseUp } }
    #[classattr]
    fn LeftMouseUp() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::LeftMouseUp } }
    #[classattr]
    fn MiddleMouseUp() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::MiddleMouseUp } }
    #[classattr]
    fn RightMouseUp() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::RightMouseUp } }
    #[classattr]
    fn MouseEnter() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::MouseEnter } }
    #[classattr]
    fn MouseLeave() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::MouseLeave } }
    #[classattr]
    fn Scroll() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::Scroll } }
    #[classattr]
    fn TextInput() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::TextInput } }
    #[classattr]
    fn VirtualKeyDown() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::VirtualKeyDown } }
    #[classattr]
    fn VirtualKeyUp() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::VirtualKeyUp } }
    #[classattr]
    fn HoveredFile() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::HoveredFile } }
    #[classattr]
    fn DroppedFile() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::DroppedFile } }
    #[classattr]
    fn HoveredFileCancelled() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::HoveredFileCancelled } }
    #[classattr]
    fn FocusReceived() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::FocusReceived } }
    #[classattr]
    fn FocusLost() -> AzOnEnumWrapper { AzOnEnumWrapper { inner: AzOn::FocusLost } }
}

#[pymethods]
impl AzEventFilterEnumWrapper {
    #[staticmethod]
    fn Hover(v: AzHoverEventFilterEnumWrapper) -> AzEventFilterEnumWrapper { AzEventFilterEnumWrapper { inner: AzEventFilter::Hover(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Not(v: AzNotEventFilterEnumWrapper) -> AzEventFilterEnumWrapper { AzEventFilterEnumWrapper { inner: AzEventFilter::Not(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Focus(v: AzFocusEventFilterEnumWrapper) -> AzEventFilterEnumWrapper { AzEventFilterEnumWrapper { inner: AzEventFilter::Focus(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Window(v: AzWindowEventFilterEnumWrapper) -> AzEventFilterEnumWrapper { AzEventFilterEnumWrapper { inner: AzEventFilter::Window(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Component(v: AzComponentEventFilterEnumWrapper) -> AzEventFilterEnumWrapper { AzEventFilterEnumWrapper { inner: AzEventFilter::Component(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Application(v: AzApplicationEventFilterEnumWrapper) -> AzEventFilterEnumWrapper { AzEventFilterEnumWrapper { inner: AzEventFilter::Application(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzHoverEventFilterEnumWrapper {
    #[classattr]
    fn MouseOver() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::MouseOver } }
    #[classattr]
    fn MouseDown() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::MouseDown } }
    #[classattr]
    fn LeftMouseDown() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::LeftMouseDown } }
    #[classattr]
    fn RightMouseDown() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::RightMouseDown } }
    #[classattr]
    fn MiddleMouseDown() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::MiddleMouseDown } }
    #[classattr]
    fn MouseUp() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::MouseUp } }
    #[classattr]
    fn LeftMouseUp() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::LeftMouseUp } }
    #[classattr]
    fn RightMouseUp() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::RightMouseUp } }
    #[classattr]
    fn MiddleMouseUp() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::MiddleMouseUp } }
    #[classattr]
    fn MouseEnter() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::MouseEnter } }
    #[classattr]
    fn MouseLeave() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::MouseLeave } }
    #[classattr]
    fn Scroll() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::Scroll } }
    #[classattr]
    fn ScrollStart() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::ScrollStart } }
    #[classattr]
    fn ScrollEnd() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::ScrollEnd } }
    #[classattr]
    fn TextInput() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::TextInput } }
    #[classattr]
    fn VirtualKeyDown() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::VirtualKeyDown } }
    #[classattr]
    fn VirtualKeyUp() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::VirtualKeyUp } }
    #[classattr]
    fn HoveredFile() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::HoveredFile } }
    #[classattr]
    fn DroppedFile() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::DroppedFile } }
    #[classattr]
    fn HoveredFileCancelled() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::HoveredFileCancelled } }
    #[classattr]
    fn TouchStart() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::TouchStart } }
    #[classattr]
    fn TouchMove() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::TouchMove } }
    #[classattr]
    fn TouchEnd() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::TouchEnd } }
    #[classattr]
    fn TouchCancel() -> AzHoverEventFilterEnumWrapper { AzHoverEventFilterEnumWrapper { inner: AzHoverEventFilter::TouchCancel } }
}

#[pymethods]
impl AzFocusEventFilterEnumWrapper {
    #[classattr]
    fn MouseOver() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::MouseOver } }
    #[classattr]
    fn MouseDown() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::MouseDown } }
    #[classattr]
    fn LeftMouseDown() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::LeftMouseDown } }
    #[classattr]
    fn RightMouseDown() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::RightMouseDown } }
    #[classattr]
    fn MiddleMouseDown() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::MiddleMouseDown } }
    #[classattr]
    fn MouseUp() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::MouseUp } }
    #[classattr]
    fn LeftMouseUp() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::LeftMouseUp } }
    #[classattr]
    fn RightMouseUp() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::RightMouseUp } }
    #[classattr]
    fn MiddleMouseUp() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::MiddleMouseUp } }
    #[classattr]
    fn MouseEnter() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::MouseEnter } }
    #[classattr]
    fn MouseLeave() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::MouseLeave } }
    #[classattr]
    fn Scroll() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::Scroll } }
    #[classattr]
    fn ScrollStart() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::ScrollStart } }
    #[classattr]
    fn ScrollEnd() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::ScrollEnd } }
    #[classattr]
    fn TextInput() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::TextInput } }
    #[classattr]
    fn VirtualKeyDown() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::VirtualKeyDown } }
    #[classattr]
    fn VirtualKeyUp() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::VirtualKeyUp } }
    #[classattr]
    fn FocusReceived() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::FocusReceived } }
    #[classattr]
    fn FocusLost() -> AzFocusEventFilterEnumWrapper { AzFocusEventFilterEnumWrapper { inner: AzFocusEventFilter::FocusLost } }
}

#[pymethods]
impl AzNotEventFilterEnumWrapper {
    #[staticmethod]
    fn Hover(v: AzHoverEventFilterEnumWrapper) -> AzNotEventFilterEnumWrapper { AzNotEventFilterEnumWrapper { inner: AzNotEventFilter::Hover(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Focus(v: AzFocusEventFilterEnumWrapper) -> AzNotEventFilterEnumWrapper { AzNotEventFilterEnumWrapper { inner: AzNotEventFilter::Focus(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzWindowEventFilterEnumWrapper {
    #[classattr]
    fn MouseOver() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::MouseOver } }
    #[classattr]
    fn MouseDown() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::MouseDown } }
    #[classattr]
    fn LeftMouseDown() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::LeftMouseDown } }
    #[classattr]
    fn RightMouseDown() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::RightMouseDown } }
    #[classattr]
    fn MiddleMouseDown() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::MiddleMouseDown } }
    #[classattr]
    fn MouseUp() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::MouseUp } }
    #[classattr]
    fn LeftMouseUp() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::LeftMouseUp } }
    #[classattr]
    fn RightMouseUp() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::RightMouseUp } }
    #[classattr]
    fn MiddleMouseUp() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::MiddleMouseUp } }
    #[classattr]
    fn MouseEnter() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::MouseEnter } }
    #[classattr]
    fn MouseLeave() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::MouseLeave } }
    #[classattr]
    fn Scroll() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::Scroll } }
    #[classattr]
    fn ScrollStart() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::ScrollStart } }
    #[classattr]
    fn ScrollEnd() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::ScrollEnd } }
    #[classattr]
    fn TextInput() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::TextInput } }
    #[classattr]
    fn VirtualKeyDown() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::VirtualKeyDown } }
    #[classattr]
    fn VirtualKeyUp() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::VirtualKeyUp } }
    #[classattr]
    fn HoveredFile() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::HoveredFile } }
    #[classattr]
    fn DroppedFile() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::DroppedFile } }
    #[classattr]
    fn HoveredFileCancelled() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::HoveredFileCancelled } }
    #[classattr]
    fn Resized() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::Resized } }
    #[classattr]
    fn Moved() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::Moved } }
    #[classattr]
    fn TouchStart() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::TouchStart } }
    #[classattr]
    fn TouchMove() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::TouchMove } }
    #[classattr]
    fn TouchEnd() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::TouchEnd } }
    #[classattr]
    fn TouchCancel() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::TouchCancel } }
    #[classattr]
    fn FocusReceived() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::FocusReceived } }
    #[classattr]
    fn FocusLost() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::FocusLost } }
    #[classattr]
    fn CloseRequested() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::CloseRequested } }
    #[classattr]
    fn ThemeChanged() -> AzWindowEventFilterEnumWrapper { AzWindowEventFilterEnumWrapper { inner: AzWindowEventFilter::ThemeChanged } }
}

#[pymethods]
impl AzComponentEventFilterEnumWrapper {
    #[classattr]
    fn AfterMount() -> AzComponentEventFilterEnumWrapper { AzComponentEventFilterEnumWrapper { inner: AzComponentEventFilter::AfterMount } }
    #[classattr]
    fn BeforeUnmount() -> AzComponentEventFilterEnumWrapper { AzComponentEventFilterEnumWrapper { inner: AzComponentEventFilter::BeforeUnmount } }
    #[classattr]
    fn NodeResized() -> AzComponentEventFilterEnumWrapper { AzComponentEventFilterEnumWrapper { inner: AzComponentEventFilter::NodeResized } }
}

#[pymethods]
impl AzApplicationEventFilterEnumWrapper {
    #[classattr]
    fn DeviceConnected() -> AzApplicationEventFilterEnumWrapper { AzApplicationEventFilterEnumWrapper { inner: AzApplicationEventFilter::DeviceConnected } }
    #[classattr]
    fn DeviceDisconnected() -> AzApplicationEventFilterEnumWrapper { AzApplicationEventFilterEnumWrapper { inner: AzApplicationEventFilter::DeviceDisconnected } }
}

#[pymethods]
impl AzTabIndexEnumWrapper {
    #[classattr]
    fn Auto() -> AzTabIndexEnumWrapper { AzTabIndexEnumWrapper { inner: AzTabIndex::Auto } }
    #[staticmethod]
    fn OverrideInParent(v: u32) -> AzTabIndexEnumWrapper { AzTabIndexEnumWrapper { inner: AzTabIndex::OverrideInParent(v) } }
    #[classattr]
    fn NoKeyboardFocus() -> AzTabIndexEnumWrapper { AzTabIndexEnumWrapper { inner: AzTabIndex::NoKeyboardFocus } }
}

#[pymethods]
impl AzIdOrClassEnumWrapper {
    #[staticmethod]
    fn Id(v: AzString) -> AzIdOrClassEnumWrapper { AzIdOrClassEnumWrapper { inner: AzIdOrClass::Id(v) } }
    #[staticmethod]
    fn Class(v: AzString) -> AzIdOrClassEnumWrapper { AzIdOrClassEnumWrapper { inner: AzIdOrClass::Class(v) } }
}

#[pymethods]
impl AzNodeDataInlineCssPropertyEnumWrapper {
    #[staticmethod]
    fn Normal(v: AzCssPropertyEnumWrapper) -> AzNodeDataInlineCssPropertyEnumWrapper { AzNodeDataInlineCssPropertyEnumWrapper { inner: AzNodeDataInlineCssProperty::Normal(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Active(v: AzCssPropertyEnumWrapper) -> AzNodeDataInlineCssPropertyEnumWrapper { AzNodeDataInlineCssPropertyEnumWrapper { inner: AzNodeDataInlineCssProperty::Active(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Focus(v: AzCssPropertyEnumWrapper) -> AzNodeDataInlineCssPropertyEnumWrapper { AzNodeDataInlineCssPropertyEnumWrapper { inner: AzNodeDataInlineCssProperty::Focus(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Hover(v: AzCssPropertyEnumWrapper) -> AzNodeDataInlineCssPropertyEnumWrapper { AzNodeDataInlineCssPropertyEnumWrapper { inner: AzNodeDataInlineCssProperty::Hover(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzCssDeclarationEnumWrapper {
    #[staticmethod]
    fn Static(v: AzCssPropertyEnumWrapper) -> AzCssDeclarationEnumWrapper { AzCssDeclarationEnumWrapper { inner: AzCssDeclaration::Static(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Dynamic(v: AzDynamicCssProperty) -> AzCssDeclarationEnumWrapper { AzCssDeclarationEnumWrapper { inner: AzCssDeclaration::Dynamic(v) } }
}

#[pymethods]
impl AzCssPathSelectorEnumWrapper {
    #[classattr]
    fn Global() -> AzCssPathSelectorEnumWrapper { AzCssPathSelectorEnumWrapper { inner: AzCssPathSelector::Global } }
    #[staticmethod]
    fn Type(v: AzNodeTypeKeyEnumWrapper) -> AzCssPathSelectorEnumWrapper { AzCssPathSelectorEnumWrapper { inner: AzCssPathSelector::Type(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Class(v: AzString) -> AzCssPathSelectorEnumWrapper { AzCssPathSelectorEnumWrapper { inner: AzCssPathSelector::Class(v) } }
    #[staticmethod]
    fn Id(v: AzString) -> AzCssPathSelectorEnumWrapper { AzCssPathSelectorEnumWrapper { inner: AzCssPathSelector::Id(v) } }
    #[staticmethod]
    fn PseudoSelector(v: AzCssPathPseudoSelectorEnumWrapper) -> AzCssPathSelectorEnumWrapper { AzCssPathSelectorEnumWrapper { inner: AzCssPathSelector::PseudoSelector(unsafe { mem::transmute(v) }) } }
    #[classattr]
    fn DirectChildren() -> AzCssPathSelectorEnumWrapper { AzCssPathSelectorEnumWrapper { inner: AzCssPathSelector::DirectChildren } }
    #[classattr]
    fn Children() -> AzCssPathSelectorEnumWrapper { AzCssPathSelectorEnumWrapper { inner: AzCssPathSelector::Children } }
}

#[pymethods]
impl AzNodeTypeKeyEnumWrapper {
    #[classattr]
    fn Body() -> AzNodeTypeKeyEnumWrapper { AzNodeTypeKeyEnumWrapper { inner: AzNodeTypeKey::Body } }
    #[classattr]
    fn Div() -> AzNodeTypeKeyEnumWrapper { AzNodeTypeKeyEnumWrapper { inner: AzNodeTypeKey::Div } }
    #[classattr]
    fn Br() -> AzNodeTypeKeyEnumWrapper { AzNodeTypeKeyEnumWrapper { inner: AzNodeTypeKey::Br } }
    #[classattr]
    fn P() -> AzNodeTypeKeyEnumWrapper { AzNodeTypeKeyEnumWrapper { inner: AzNodeTypeKey::P } }
    #[classattr]
    fn Img() -> AzNodeTypeKeyEnumWrapper { AzNodeTypeKeyEnumWrapper { inner: AzNodeTypeKey::Img } }
    #[classattr]
    fn IFrame() -> AzNodeTypeKeyEnumWrapper { AzNodeTypeKeyEnumWrapper { inner: AzNodeTypeKey::IFrame } }
}

#[pymethods]
impl AzCssPathPseudoSelectorEnumWrapper {
    #[classattr]
    fn First() -> AzCssPathPseudoSelectorEnumWrapper { AzCssPathPseudoSelectorEnumWrapper { inner: AzCssPathPseudoSelector::First } }
    #[classattr]
    fn Last() -> AzCssPathPseudoSelectorEnumWrapper { AzCssPathPseudoSelectorEnumWrapper { inner: AzCssPathPseudoSelector::Last } }
    #[staticmethod]
    fn NthChild(v: AzCssNthChildSelectorEnumWrapper) -> AzCssPathPseudoSelectorEnumWrapper { AzCssPathPseudoSelectorEnumWrapper { inner: AzCssPathPseudoSelector::NthChild(unsafe { mem::transmute(v) }) } }
    #[classattr]
    fn Hover() -> AzCssPathPseudoSelectorEnumWrapper { AzCssPathPseudoSelectorEnumWrapper { inner: AzCssPathPseudoSelector::Hover } }
    #[classattr]
    fn Active() -> AzCssPathPseudoSelectorEnumWrapper { AzCssPathPseudoSelectorEnumWrapper { inner: AzCssPathPseudoSelector::Active } }
    #[classattr]
    fn Focus() -> AzCssPathPseudoSelectorEnumWrapper { AzCssPathPseudoSelectorEnumWrapper { inner: AzCssPathPseudoSelector::Focus } }
}

#[pymethods]
impl AzCssNthChildSelectorEnumWrapper {
    #[staticmethod]
    fn Number(v: u32) -> AzCssNthChildSelectorEnumWrapper { AzCssNthChildSelectorEnumWrapper { inner: AzCssNthChildSelector::Number(v) } }
    #[classattr]
    fn Even() -> AzCssNthChildSelectorEnumWrapper { AzCssNthChildSelectorEnumWrapper { inner: AzCssNthChildSelector::Even } }
    #[classattr]
    fn Odd() -> AzCssNthChildSelectorEnumWrapper { AzCssNthChildSelectorEnumWrapper { inner: AzCssNthChildSelector::Odd } }
    #[staticmethod]
    fn Pattern(v: AzCssNthChildPattern) -> AzCssNthChildSelectorEnumWrapper { AzCssNthChildSelectorEnumWrapper { inner: AzCssNthChildSelector::Pattern(v) } }
}

#[pymethods]
impl AzCss {
    #[staticmethod]
    fn empty() -> AzCss {
        unsafe { mem::transmute(crate::AzCss_empty()) }
    }
    #[staticmethod]
    fn from_string(s: String) -> AzCss {
        let s = pystring_to_azstring(&s);
        unsafe { mem::transmute(crate::AzCss_fromString(
            mem::transmute(s),
        )) }
    }
}

#[pymethods]
impl AzCssPropertyTypeEnumWrapper {
    #[classattr]
    fn TextColor() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::TextColor } }
    #[classattr]
    fn FontSize() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::FontSize } }
    #[classattr]
    fn FontFamily() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::FontFamily } }
    #[classattr]
    fn TextAlign() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::TextAlign } }
    #[classattr]
    fn LetterSpacing() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::LetterSpacing } }
    #[classattr]
    fn LineHeight() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::LineHeight } }
    #[classattr]
    fn WordSpacing() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::WordSpacing } }
    #[classattr]
    fn TabWidth() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::TabWidth } }
    #[classattr]
    fn Cursor() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Cursor } }
    #[classattr]
    fn Display() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Display } }
    #[classattr]
    fn Float() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Float } }
    #[classattr]
    fn BoxSizing() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BoxSizing } }
    #[classattr]
    fn Width() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Width } }
    #[classattr]
    fn Height() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Height } }
    #[classattr]
    fn MinWidth() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::MinWidth } }
    #[classattr]
    fn MinHeight() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::MinHeight } }
    #[classattr]
    fn MaxWidth() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::MaxWidth } }
    #[classattr]
    fn MaxHeight() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::MaxHeight } }
    #[classattr]
    fn Position() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Position } }
    #[classattr]
    fn Top() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Top } }
    #[classattr]
    fn Right() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Right } }
    #[classattr]
    fn Left() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Left } }
    #[classattr]
    fn Bottom() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Bottom } }
    #[classattr]
    fn FlexWrap() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::FlexWrap } }
    #[classattr]
    fn FlexDirection() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::FlexDirection } }
    #[classattr]
    fn FlexGrow() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::FlexGrow } }
    #[classattr]
    fn FlexShrink() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::FlexShrink } }
    #[classattr]
    fn JustifyContent() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::JustifyContent } }
    #[classattr]
    fn AlignItems() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::AlignItems } }
    #[classattr]
    fn AlignContent() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::AlignContent } }
    #[classattr]
    fn OverflowX() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::OverflowX } }
    #[classattr]
    fn OverflowY() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::OverflowY } }
    #[classattr]
    fn PaddingTop() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::PaddingTop } }
    #[classattr]
    fn PaddingLeft() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::PaddingLeft } }
    #[classattr]
    fn PaddingRight() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::PaddingRight } }
    #[classattr]
    fn PaddingBottom() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::PaddingBottom } }
    #[classattr]
    fn MarginTop() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::MarginTop } }
    #[classattr]
    fn MarginLeft() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::MarginLeft } }
    #[classattr]
    fn MarginRight() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::MarginRight } }
    #[classattr]
    fn MarginBottom() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::MarginBottom } }
    #[classattr]
    fn Background() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Background } }
    #[classattr]
    fn BackgroundImage() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BackgroundImage } }
    #[classattr]
    fn BackgroundColor() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BackgroundColor } }
    #[classattr]
    fn BackgroundPosition() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BackgroundPosition } }
    #[classattr]
    fn BackgroundSize() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BackgroundSize } }
    #[classattr]
    fn BackgroundRepeat() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BackgroundRepeat } }
    #[classattr]
    fn BorderTopLeftRadius() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderTopLeftRadius } }
    #[classattr]
    fn BorderTopRightRadius() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderTopRightRadius } }
    #[classattr]
    fn BorderBottomLeftRadius() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderBottomLeftRadius } }
    #[classattr]
    fn BorderBottomRightRadius() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderBottomRightRadius } }
    #[classattr]
    fn BorderTopColor() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderTopColor } }
    #[classattr]
    fn BorderRightColor() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderRightColor } }
    #[classattr]
    fn BorderLeftColor() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderLeftColor } }
    #[classattr]
    fn BorderBottomColor() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderBottomColor } }
    #[classattr]
    fn BorderTopStyle() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderTopStyle } }
    #[classattr]
    fn BorderRightStyle() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderRightStyle } }
    #[classattr]
    fn BorderLeftStyle() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderLeftStyle } }
    #[classattr]
    fn BorderBottomStyle() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderBottomStyle } }
    #[classattr]
    fn BorderTopWidth() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderTopWidth } }
    #[classattr]
    fn BorderRightWidth() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderRightWidth } }
    #[classattr]
    fn BorderLeftWidth() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderLeftWidth } }
    #[classattr]
    fn BorderBottomWidth() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BorderBottomWidth } }
    #[classattr]
    fn BoxShadowLeft() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BoxShadowLeft } }
    #[classattr]
    fn BoxShadowRight() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BoxShadowRight } }
    #[classattr]
    fn BoxShadowTop() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BoxShadowTop } }
    #[classattr]
    fn BoxShadowBottom() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BoxShadowBottom } }
    #[classattr]
    fn ScrollbarStyle() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::ScrollbarStyle } }
    #[classattr]
    fn Opacity() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Opacity } }
    #[classattr]
    fn Transform() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::Transform } }
    #[classattr]
    fn PerspectiveOrigin() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::PerspectiveOrigin } }
    #[classattr]
    fn TransformOrigin() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::TransformOrigin } }
    #[classattr]
    fn BackfaceVisibility() -> AzCssPropertyTypeEnumWrapper { AzCssPropertyTypeEnumWrapper { inner: AzCssPropertyType::BackfaceVisibility } }
}

#[pymethods]
impl AzAnimationInterpolationFunctionEnumWrapper {
    #[classattr]
    fn Ease() -> AzAnimationInterpolationFunctionEnumWrapper { AzAnimationInterpolationFunctionEnumWrapper { inner: AzAnimationInterpolationFunction::Ease } }
    #[classattr]
    fn Linear() -> AzAnimationInterpolationFunctionEnumWrapper { AzAnimationInterpolationFunctionEnumWrapper { inner: AzAnimationInterpolationFunction::Linear } }
    #[classattr]
    fn EaseIn() -> AzAnimationInterpolationFunctionEnumWrapper { AzAnimationInterpolationFunctionEnumWrapper { inner: AzAnimationInterpolationFunction::EaseIn } }
    #[classattr]
    fn EaseOut() -> AzAnimationInterpolationFunctionEnumWrapper { AzAnimationInterpolationFunctionEnumWrapper { inner: AzAnimationInterpolationFunction::EaseOut } }
    #[classattr]
    fn EaseInOut() -> AzAnimationInterpolationFunctionEnumWrapper { AzAnimationInterpolationFunctionEnumWrapper { inner: AzAnimationInterpolationFunction::EaseInOut } }
    #[staticmethod]
    fn CubicBezier(v: AzSvgCubicCurve) -> AzAnimationInterpolationFunctionEnumWrapper { AzAnimationInterpolationFunctionEnumWrapper { inner: AzAnimationInterpolationFunction::CubicBezier(v) } }
}

#[pymethods]
impl AzColorU {
    #[staticmethod]
    fn from_str(string: String) -> AzColorU {
        let string = pystring_to_azstring(&string);
        unsafe { mem::transmute(crate::AzColorU_fromStr(
            mem::transmute(string),
        )) }
    }
    fn to_hash(&self) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzColorU_toHash(
            mem::transmute(self),
        )) })
    }
}

#[pymethods]
impl AzSizeMetricEnumWrapper {
    #[classattr]
    fn Px() -> AzSizeMetricEnumWrapper { AzSizeMetricEnumWrapper { inner: AzSizeMetric::Px } }
    #[classattr]
    fn Pt() -> AzSizeMetricEnumWrapper { AzSizeMetricEnumWrapper { inner: AzSizeMetric::Pt } }
    #[classattr]
    fn Em() -> AzSizeMetricEnumWrapper { AzSizeMetricEnumWrapper { inner: AzSizeMetric::Em } }
    #[classattr]
    fn Percent() -> AzSizeMetricEnumWrapper { AzSizeMetricEnumWrapper { inner: AzSizeMetric::Percent } }
}

#[pymethods]
impl AzBoxShadowClipModeEnumWrapper {
    #[classattr]
    fn Outset() -> AzBoxShadowClipModeEnumWrapper { AzBoxShadowClipModeEnumWrapper { inner: AzBoxShadowClipMode::Outset } }
    #[classattr]
    fn Inset() -> AzBoxShadowClipModeEnumWrapper { AzBoxShadowClipModeEnumWrapper { inner: AzBoxShadowClipMode::Inset } }
}

#[pymethods]
impl AzLayoutAlignContentEnumWrapper {
    #[classattr]
    fn Stretch() -> AzLayoutAlignContentEnumWrapper { AzLayoutAlignContentEnumWrapper { inner: AzLayoutAlignContent::Stretch } }
    #[classattr]
    fn Center() -> AzLayoutAlignContentEnumWrapper { AzLayoutAlignContentEnumWrapper { inner: AzLayoutAlignContent::Center } }
    #[classattr]
    fn Start() -> AzLayoutAlignContentEnumWrapper { AzLayoutAlignContentEnumWrapper { inner: AzLayoutAlignContent::Start } }
    #[classattr]
    fn End() -> AzLayoutAlignContentEnumWrapper { AzLayoutAlignContentEnumWrapper { inner: AzLayoutAlignContent::End } }
    #[classattr]
    fn SpaceBetween() -> AzLayoutAlignContentEnumWrapper { AzLayoutAlignContentEnumWrapper { inner: AzLayoutAlignContent::SpaceBetween } }
    #[classattr]
    fn SpaceAround() -> AzLayoutAlignContentEnumWrapper { AzLayoutAlignContentEnumWrapper { inner: AzLayoutAlignContent::SpaceAround } }
}

#[pymethods]
impl AzLayoutAlignItemsEnumWrapper {
    #[classattr]
    fn Stretch() -> AzLayoutAlignItemsEnumWrapper { AzLayoutAlignItemsEnumWrapper { inner: AzLayoutAlignItems::Stretch } }
    #[classattr]
    fn Center() -> AzLayoutAlignItemsEnumWrapper { AzLayoutAlignItemsEnumWrapper { inner: AzLayoutAlignItems::Center } }
    #[classattr]
    fn FlexStart() -> AzLayoutAlignItemsEnumWrapper { AzLayoutAlignItemsEnumWrapper { inner: AzLayoutAlignItems::FlexStart } }
    #[classattr]
    fn FlexEnd() -> AzLayoutAlignItemsEnumWrapper { AzLayoutAlignItemsEnumWrapper { inner: AzLayoutAlignItems::FlexEnd } }
}

#[pymethods]
impl AzLayoutBoxSizingEnumWrapper {
    #[classattr]
    fn ContentBox() -> AzLayoutBoxSizingEnumWrapper { AzLayoutBoxSizingEnumWrapper { inner: AzLayoutBoxSizing::ContentBox } }
    #[classattr]
    fn BorderBox() -> AzLayoutBoxSizingEnumWrapper { AzLayoutBoxSizingEnumWrapper { inner: AzLayoutBoxSizing::BorderBox } }
}

#[pymethods]
impl AzLayoutFlexDirectionEnumWrapper {
    #[classattr]
    fn Row() -> AzLayoutFlexDirectionEnumWrapper { AzLayoutFlexDirectionEnumWrapper { inner: AzLayoutFlexDirection::Row } }
    #[classattr]
    fn RowReverse() -> AzLayoutFlexDirectionEnumWrapper { AzLayoutFlexDirectionEnumWrapper { inner: AzLayoutFlexDirection::RowReverse } }
    #[classattr]
    fn Column() -> AzLayoutFlexDirectionEnumWrapper { AzLayoutFlexDirectionEnumWrapper { inner: AzLayoutFlexDirection::Column } }
    #[classattr]
    fn ColumnReverse() -> AzLayoutFlexDirectionEnumWrapper { AzLayoutFlexDirectionEnumWrapper { inner: AzLayoutFlexDirection::ColumnReverse } }
}

#[pymethods]
impl AzLayoutDisplayEnumWrapper {
    #[classattr]
    fn None() -> AzLayoutDisplayEnumWrapper { AzLayoutDisplayEnumWrapper { inner: AzLayoutDisplay::None } }
    #[classattr]
    fn Flex() -> AzLayoutDisplayEnumWrapper { AzLayoutDisplayEnumWrapper { inner: AzLayoutDisplay::Flex } }
    #[classattr]
    fn Block() -> AzLayoutDisplayEnumWrapper { AzLayoutDisplayEnumWrapper { inner: AzLayoutDisplay::Block } }
    #[classattr]
    fn InlineBlock() -> AzLayoutDisplayEnumWrapper { AzLayoutDisplayEnumWrapper { inner: AzLayoutDisplay::InlineBlock } }
}

#[pymethods]
impl AzLayoutFloatEnumWrapper {
    #[classattr]
    fn Left() -> AzLayoutFloatEnumWrapper { AzLayoutFloatEnumWrapper { inner: AzLayoutFloat::Left } }
    #[classattr]
    fn Right() -> AzLayoutFloatEnumWrapper { AzLayoutFloatEnumWrapper { inner: AzLayoutFloat::Right } }
}

#[pymethods]
impl AzLayoutJustifyContentEnumWrapper {
    #[classattr]
    fn Start() -> AzLayoutJustifyContentEnumWrapper { AzLayoutJustifyContentEnumWrapper { inner: AzLayoutJustifyContent::Start } }
    #[classattr]
    fn End() -> AzLayoutJustifyContentEnumWrapper { AzLayoutJustifyContentEnumWrapper { inner: AzLayoutJustifyContent::End } }
    #[classattr]
    fn Center() -> AzLayoutJustifyContentEnumWrapper { AzLayoutJustifyContentEnumWrapper { inner: AzLayoutJustifyContent::Center } }
    #[classattr]
    fn SpaceBetween() -> AzLayoutJustifyContentEnumWrapper { AzLayoutJustifyContentEnumWrapper { inner: AzLayoutJustifyContent::SpaceBetween } }
    #[classattr]
    fn SpaceAround() -> AzLayoutJustifyContentEnumWrapper { AzLayoutJustifyContentEnumWrapper { inner: AzLayoutJustifyContent::SpaceAround } }
    #[classattr]
    fn SpaceEvenly() -> AzLayoutJustifyContentEnumWrapper { AzLayoutJustifyContentEnumWrapper { inner: AzLayoutJustifyContent::SpaceEvenly } }
}

#[pymethods]
impl AzLayoutPositionEnumWrapper {
    #[classattr]
    fn Static() -> AzLayoutPositionEnumWrapper { AzLayoutPositionEnumWrapper { inner: AzLayoutPosition::Static } }
    #[classattr]
    fn Relative() -> AzLayoutPositionEnumWrapper { AzLayoutPositionEnumWrapper { inner: AzLayoutPosition::Relative } }
    #[classattr]
    fn Absolute() -> AzLayoutPositionEnumWrapper { AzLayoutPositionEnumWrapper { inner: AzLayoutPosition::Absolute } }
    #[classattr]
    fn Fixed() -> AzLayoutPositionEnumWrapper { AzLayoutPositionEnumWrapper { inner: AzLayoutPosition::Fixed } }
}

#[pymethods]
impl AzLayoutFlexWrapEnumWrapper {
    #[classattr]
    fn Wrap() -> AzLayoutFlexWrapEnumWrapper { AzLayoutFlexWrapEnumWrapper { inner: AzLayoutFlexWrap::Wrap } }
    #[classattr]
    fn NoWrap() -> AzLayoutFlexWrapEnumWrapper { AzLayoutFlexWrapEnumWrapper { inner: AzLayoutFlexWrap::NoWrap } }
}

#[pymethods]
impl AzLayoutOverflowEnumWrapper {
    #[classattr]
    fn Scroll() -> AzLayoutOverflowEnumWrapper { AzLayoutOverflowEnumWrapper { inner: AzLayoutOverflow::Scroll } }
    #[classattr]
    fn Auto() -> AzLayoutOverflowEnumWrapper { AzLayoutOverflowEnumWrapper { inner: AzLayoutOverflow::Auto } }
    #[classattr]
    fn Hidden() -> AzLayoutOverflowEnumWrapper { AzLayoutOverflowEnumWrapper { inner: AzLayoutOverflow::Hidden } }
    #[classattr]
    fn Visible() -> AzLayoutOverflowEnumWrapper { AzLayoutOverflowEnumWrapper { inner: AzLayoutOverflow::Visible } }
}

#[pymethods]
impl AzAngleMetricEnumWrapper {
    #[classattr]
    fn Degree() -> AzAngleMetricEnumWrapper { AzAngleMetricEnumWrapper { inner: AzAngleMetric::Degree } }
    #[classattr]
    fn Radians() -> AzAngleMetricEnumWrapper { AzAngleMetricEnumWrapper { inner: AzAngleMetric::Radians } }
    #[classattr]
    fn Grad() -> AzAngleMetricEnumWrapper { AzAngleMetricEnumWrapper { inner: AzAngleMetric::Grad } }
    #[classattr]
    fn Turn() -> AzAngleMetricEnumWrapper { AzAngleMetricEnumWrapper { inner: AzAngleMetric::Turn } }
    #[classattr]
    fn Percent() -> AzAngleMetricEnumWrapper { AzAngleMetricEnumWrapper { inner: AzAngleMetric::Percent } }
}

#[pymethods]
impl AzDirectionCornerEnumWrapper {
    #[classattr]
    fn Right() -> AzDirectionCornerEnumWrapper { AzDirectionCornerEnumWrapper { inner: AzDirectionCorner::Right } }
    #[classattr]
    fn Left() -> AzDirectionCornerEnumWrapper { AzDirectionCornerEnumWrapper { inner: AzDirectionCorner::Left } }
    #[classattr]
    fn Top() -> AzDirectionCornerEnumWrapper { AzDirectionCornerEnumWrapper { inner: AzDirectionCorner::Top } }
    #[classattr]
    fn Bottom() -> AzDirectionCornerEnumWrapper { AzDirectionCornerEnumWrapper { inner: AzDirectionCorner::Bottom } }
    #[classattr]
    fn TopRight() -> AzDirectionCornerEnumWrapper { AzDirectionCornerEnumWrapper { inner: AzDirectionCorner::TopRight } }
    #[classattr]
    fn TopLeft() -> AzDirectionCornerEnumWrapper { AzDirectionCornerEnumWrapper { inner: AzDirectionCorner::TopLeft } }
    #[classattr]
    fn BottomRight() -> AzDirectionCornerEnumWrapper { AzDirectionCornerEnumWrapper { inner: AzDirectionCorner::BottomRight } }
    #[classattr]
    fn BottomLeft() -> AzDirectionCornerEnumWrapper { AzDirectionCornerEnumWrapper { inner: AzDirectionCorner::BottomLeft } }
}

#[pymethods]
impl AzDirectionEnumWrapper {
    #[staticmethod]
    fn Angle(v: AzAngleValue) -> AzDirectionEnumWrapper { AzDirectionEnumWrapper { inner: AzDirection::Angle(v) } }
    #[staticmethod]
    fn FromTo(v: AzDirectionCorners) -> AzDirectionEnumWrapper { AzDirectionEnumWrapper { inner: AzDirection::FromTo(v) } }
}

#[pymethods]
impl AzExtendModeEnumWrapper {
    #[classattr]
    fn Clamp() -> AzExtendModeEnumWrapper { AzExtendModeEnumWrapper { inner: AzExtendMode::Clamp } }
    #[classattr]
    fn Repeat() -> AzExtendModeEnumWrapper { AzExtendModeEnumWrapper { inner: AzExtendMode::Repeat } }
}

#[pymethods]
impl AzShapeEnumWrapper {
    #[classattr]
    fn Ellipse() -> AzShapeEnumWrapper { AzShapeEnumWrapper { inner: AzShape::Ellipse } }
    #[classattr]
    fn Circle() -> AzShapeEnumWrapper { AzShapeEnumWrapper { inner: AzShape::Circle } }
}

#[pymethods]
impl AzRadialGradientSizeEnumWrapper {
    #[classattr]
    fn ClosestSide() -> AzRadialGradientSizeEnumWrapper { AzRadialGradientSizeEnumWrapper { inner: AzRadialGradientSize::ClosestSide } }
    #[classattr]
    fn ClosestCorner() -> AzRadialGradientSizeEnumWrapper { AzRadialGradientSizeEnumWrapper { inner: AzRadialGradientSize::ClosestCorner } }
    #[classattr]
    fn FarthestSide() -> AzRadialGradientSizeEnumWrapper { AzRadialGradientSizeEnumWrapper { inner: AzRadialGradientSize::FarthestSide } }
    #[classattr]
    fn FarthestCorner() -> AzRadialGradientSizeEnumWrapper { AzRadialGradientSizeEnumWrapper { inner: AzRadialGradientSize::FarthestCorner } }
}

#[pymethods]
impl AzStyleBackgroundContentEnumWrapper {
    #[staticmethod]
    fn LinearGradient(v: AzLinearGradient) -> AzStyleBackgroundContentEnumWrapper { AzStyleBackgroundContentEnumWrapper { inner: AzStyleBackgroundContent::LinearGradient(v) } }
    #[staticmethod]
    fn RadialGradient(v: AzRadialGradient) -> AzStyleBackgroundContentEnumWrapper { AzStyleBackgroundContentEnumWrapper { inner: AzStyleBackgroundContent::RadialGradient(v) } }
    #[staticmethod]
    fn ConicGradient(v: AzConicGradient) -> AzStyleBackgroundContentEnumWrapper { AzStyleBackgroundContentEnumWrapper { inner: AzStyleBackgroundContent::ConicGradient(v) } }
    #[staticmethod]
    fn Image(v: AzString) -> AzStyleBackgroundContentEnumWrapper { AzStyleBackgroundContentEnumWrapper { inner: AzStyleBackgroundContent::Image(v) } }
    #[staticmethod]
    fn Color(v: AzColorU) -> AzStyleBackgroundContentEnumWrapper { AzStyleBackgroundContentEnumWrapper { inner: AzStyleBackgroundContent::Color(v) } }
}

#[pymethods]
impl AzBackgroundPositionHorizontalEnumWrapper {
    #[classattr]
    fn Left() -> AzBackgroundPositionHorizontalEnumWrapper { AzBackgroundPositionHorizontalEnumWrapper { inner: AzBackgroundPositionHorizontal::Left } }
    #[classattr]
    fn Center() -> AzBackgroundPositionHorizontalEnumWrapper { AzBackgroundPositionHorizontalEnumWrapper { inner: AzBackgroundPositionHorizontal::Center } }
    #[classattr]
    fn Right() -> AzBackgroundPositionHorizontalEnumWrapper { AzBackgroundPositionHorizontalEnumWrapper { inner: AzBackgroundPositionHorizontal::Right } }
    #[staticmethod]
    fn Exact(v: AzPixelValue) -> AzBackgroundPositionHorizontalEnumWrapper { AzBackgroundPositionHorizontalEnumWrapper { inner: AzBackgroundPositionHorizontal::Exact(v) } }
}

#[pymethods]
impl AzBackgroundPositionVerticalEnumWrapper {
    #[classattr]
    fn Top() -> AzBackgroundPositionVerticalEnumWrapper { AzBackgroundPositionVerticalEnumWrapper { inner: AzBackgroundPositionVertical::Top } }
    #[classattr]
    fn Center() -> AzBackgroundPositionVerticalEnumWrapper { AzBackgroundPositionVerticalEnumWrapper { inner: AzBackgroundPositionVertical::Center } }
    #[classattr]
    fn Bottom() -> AzBackgroundPositionVerticalEnumWrapper { AzBackgroundPositionVerticalEnumWrapper { inner: AzBackgroundPositionVertical::Bottom } }
    #[staticmethod]
    fn Exact(v: AzPixelValue) -> AzBackgroundPositionVerticalEnumWrapper { AzBackgroundPositionVerticalEnumWrapper { inner: AzBackgroundPositionVertical::Exact(v) } }
}

#[pymethods]
impl AzStyleBackgroundRepeatEnumWrapper {
    #[classattr]
    fn NoRepeat() -> AzStyleBackgroundRepeatEnumWrapper { AzStyleBackgroundRepeatEnumWrapper { inner: AzStyleBackgroundRepeat::NoRepeat } }
    #[classattr]
    fn Repeat() -> AzStyleBackgroundRepeatEnumWrapper { AzStyleBackgroundRepeatEnumWrapper { inner: AzStyleBackgroundRepeat::Repeat } }
    #[classattr]
    fn RepeatX() -> AzStyleBackgroundRepeatEnumWrapper { AzStyleBackgroundRepeatEnumWrapper { inner: AzStyleBackgroundRepeat::RepeatX } }
    #[classattr]
    fn RepeatY() -> AzStyleBackgroundRepeatEnumWrapper { AzStyleBackgroundRepeatEnumWrapper { inner: AzStyleBackgroundRepeat::RepeatY } }
}

#[pymethods]
impl AzStyleBackgroundSizeEnumWrapper {
    #[classattr]
    fn Contain() -> AzStyleBackgroundSizeEnumWrapper { AzStyleBackgroundSizeEnumWrapper { inner: AzStyleBackgroundSize::Contain } }
    #[classattr]
    fn Cover() -> AzStyleBackgroundSizeEnumWrapper { AzStyleBackgroundSizeEnumWrapper { inner: AzStyleBackgroundSize::Cover } }
}

#[pymethods]
impl AzBorderStyleEnumWrapper {
    #[classattr]
    fn None() -> AzBorderStyleEnumWrapper { AzBorderStyleEnumWrapper { inner: AzBorderStyle::None } }
    #[classattr]
    fn Solid() -> AzBorderStyleEnumWrapper { AzBorderStyleEnumWrapper { inner: AzBorderStyle::Solid } }
    #[classattr]
    fn Double() -> AzBorderStyleEnumWrapper { AzBorderStyleEnumWrapper { inner: AzBorderStyle::Double } }
    #[classattr]
    fn Dotted() -> AzBorderStyleEnumWrapper { AzBorderStyleEnumWrapper { inner: AzBorderStyle::Dotted } }
    #[classattr]
    fn Dashed() -> AzBorderStyleEnumWrapper { AzBorderStyleEnumWrapper { inner: AzBorderStyle::Dashed } }
    #[classattr]
    fn Hidden() -> AzBorderStyleEnumWrapper { AzBorderStyleEnumWrapper { inner: AzBorderStyle::Hidden } }
    #[classattr]
    fn Groove() -> AzBorderStyleEnumWrapper { AzBorderStyleEnumWrapper { inner: AzBorderStyle::Groove } }
    #[classattr]
    fn Ridge() -> AzBorderStyleEnumWrapper { AzBorderStyleEnumWrapper { inner: AzBorderStyle::Ridge } }
    #[classattr]
    fn Inset() -> AzBorderStyleEnumWrapper { AzBorderStyleEnumWrapper { inner: AzBorderStyle::Inset } }
    #[classattr]
    fn Outset() -> AzBorderStyleEnumWrapper { AzBorderStyleEnumWrapper { inner: AzBorderStyle::Outset } }
}

#[pymethods]
impl AzStyleCursorEnumWrapper {
    #[classattr]
    fn Alias() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Alias } }
    #[classattr]
    fn AllScroll() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::AllScroll } }
    #[classattr]
    fn Cell() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Cell } }
    #[classattr]
    fn ColResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::ColResize } }
    #[classattr]
    fn ContextMenu() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::ContextMenu } }
    #[classattr]
    fn Copy() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Copy } }
    #[classattr]
    fn Crosshair() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Crosshair } }
    #[classattr]
    fn Default() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Default } }
    #[classattr]
    fn EResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::EResize } }
    #[classattr]
    fn EwResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::EwResize } }
    #[classattr]
    fn Grab() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Grab } }
    #[classattr]
    fn Grabbing() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Grabbing } }
    #[classattr]
    fn Help() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Help } }
    #[classattr]
    fn Move() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Move } }
    #[classattr]
    fn NResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::NResize } }
    #[classattr]
    fn NsResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::NsResize } }
    #[classattr]
    fn NeswResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::NeswResize } }
    #[classattr]
    fn NwseResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::NwseResize } }
    #[classattr]
    fn Pointer() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Pointer } }
    #[classattr]
    fn Progress() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Progress } }
    #[classattr]
    fn RowResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::RowResize } }
    #[classattr]
    fn SResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::SResize } }
    #[classattr]
    fn SeResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::SeResize } }
    #[classattr]
    fn Text() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Text } }
    #[classattr]
    fn Unset() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Unset } }
    #[classattr]
    fn VerticalText() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::VerticalText } }
    #[classattr]
    fn WResize() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::WResize } }
    #[classattr]
    fn Wait() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::Wait } }
    #[classattr]
    fn ZoomIn() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::ZoomIn } }
    #[classattr]
    fn ZoomOut() -> AzStyleCursorEnumWrapper { AzStyleCursorEnumWrapper { inner: AzStyleCursor::ZoomOut } }
}

#[pymethods]
impl AzStyleFontFamilyEnumWrapper {
    #[staticmethod]
    fn System(v: AzString) -> AzStyleFontFamilyEnumWrapper { AzStyleFontFamilyEnumWrapper { inner: AzStyleFontFamily::System(v) } }
    #[staticmethod]
    fn File(v: AzString) -> AzStyleFontFamilyEnumWrapper { AzStyleFontFamilyEnumWrapper { inner: AzStyleFontFamily::File(v) } }
    #[staticmethod]
    fn Ref(v: AzFontRef) -> AzStyleFontFamilyEnumWrapper { AzStyleFontFamilyEnumWrapper { inner: AzStyleFontFamily::Ref(v) } }
}

#[pymethods]
impl AzStyleBackfaceVisibilityEnumWrapper {
    #[classattr]
    fn Hidden() -> AzStyleBackfaceVisibilityEnumWrapper { AzStyleBackfaceVisibilityEnumWrapper { inner: AzStyleBackfaceVisibility::Hidden } }
    #[classattr]
    fn Visible() -> AzStyleBackfaceVisibilityEnumWrapper { AzStyleBackfaceVisibilityEnumWrapper { inner: AzStyleBackfaceVisibility::Visible } }
}

#[pymethods]
impl AzStyleTransformEnumWrapper {
    #[staticmethod]
    fn Matrix(v: AzStyleTransformMatrix2D) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::Matrix(v) } }
    #[staticmethod]
    fn Matrix3D(v: AzStyleTransformMatrix3D) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::Matrix3D(v) } }
    #[staticmethod]
    fn Translate(v: AzStyleTransformTranslate2D) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::Translate(v) } }
    #[staticmethod]
    fn Translate3D(v: AzStyleTransformTranslate3D) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::Translate3D(v) } }
    #[staticmethod]
    fn TranslateX(v: AzPixelValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::TranslateX(v) } }
    #[staticmethod]
    fn TranslateY(v: AzPixelValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::TranslateY(v) } }
    #[staticmethod]
    fn TranslateZ(v: AzPixelValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::TranslateZ(v) } }
    #[staticmethod]
    fn Rotate(v: AzAngleValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::Rotate(v) } }
    #[staticmethod]
    fn Rotate3D(v: AzStyleTransformRotate3D) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::Rotate3D(v) } }
    #[staticmethod]
    fn RotateX(v: AzAngleValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::RotateX(v) } }
    #[staticmethod]
    fn RotateY(v: AzAngleValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::RotateY(v) } }
    #[staticmethod]
    fn RotateZ(v: AzAngleValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::RotateZ(v) } }
    #[staticmethod]
    fn Scale(v: AzStyleTransformScale2D) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::Scale(v) } }
    #[staticmethod]
    fn Scale3D(v: AzStyleTransformScale3D) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::Scale3D(v) } }
    #[staticmethod]
    fn ScaleX(v: AzPercentageValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::ScaleX(v) } }
    #[staticmethod]
    fn ScaleY(v: AzPercentageValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::ScaleY(v) } }
    #[staticmethod]
    fn ScaleZ(v: AzPercentageValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::ScaleZ(v) } }
    #[staticmethod]
    fn Skew(v: AzStyleTransformSkew2D) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::Skew(v) } }
    #[staticmethod]
    fn SkewX(v: AzPercentageValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::SkewX(v) } }
    #[staticmethod]
    fn SkewY(v: AzPercentageValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::SkewY(v) } }
    #[staticmethod]
    fn Perspective(v: AzPixelValue) -> AzStyleTransformEnumWrapper { AzStyleTransformEnumWrapper { inner: AzStyleTransform::Perspective(v) } }
}

#[pymethods]
impl AzStyleTextAlignEnumWrapper {
    #[classattr]
    fn Left() -> AzStyleTextAlignEnumWrapper { AzStyleTextAlignEnumWrapper { inner: AzStyleTextAlign::Left } }
    #[classattr]
    fn Center() -> AzStyleTextAlignEnumWrapper { AzStyleTextAlignEnumWrapper { inner: AzStyleTextAlign::Center } }
    #[classattr]
    fn Right() -> AzStyleTextAlignEnumWrapper { AzStyleTextAlignEnumWrapper { inner: AzStyleTextAlign::Right } }
}

#[pymethods]
impl AzStyleBoxShadowValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBoxShadowValueEnumWrapper { AzStyleBoxShadowValueEnumWrapper { inner: AzStyleBoxShadowValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBoxShadowValueEnumWrapper { AzStyleBoxShadowValueEnumWrapper { inner: AzStyleBoxShadowValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBoxShadowValueEnumWrapper { AzStyleBoxShadowValueEnumWrapper { inner: AzStyleBoxShadowValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBoxShadowValueEnumWrapper { AzStyleBoxShadowValueEnumWrapper { inner: AzStyleBoxShadowValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBoxShadow) -> AzStyleBoxShadowValueEnumWrapper { AzStyleBoxShadowValueEnumWrapper { inner: AzStyleBoxShadowValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutAlignContentValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutAlignContentValueEnumWrapper { AzLayoutAlignContentValueEnumWrapper { inner: AzLayoutAlignContentValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutAlignContentValueEnumWrapper { AzLayoutAlignContentValueEnumWrapper { inner: AzLayoutAlignContentValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutAlignContentValueEnumWrapper { AzLayoutAlignContentValueEnumWrapper { inner: AzLayoutAlignContentValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutAlignContentValueEnumWrapper { AzLayoutAlignContentValueEnumWrapper { inner: AzLayoutAlignContentValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutAlignContentEnumWrapper) -> AzLayoutAlignContentValueEnumWrapper { AzLayoutAlignContentValueEnumWrapper { inner: AzLayoutAlignContentValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzLayoutAlignItemsValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutAlignItemsValueEnumWrapper { AzLayoutAlignItemsValueEnumWrapper { inner: AzLayoutAlignItemsValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutAlignItemsValueEnumWrapper { AzLayoutAlignItemsValueEnumWrapper { inner: AzLayoutAlignItemsValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutAlignItemsValueEnumWrapper { AzLayoutAlignItemsValueEnumWrapper { inner: AzLayoutAlignItemsValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutAlignItemsValueEnumWrapper { AzLayoutAlignItemsValueEnumWrapper { inner: AzLayoutAlignItemsValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutAlignItemsEnumWrapper) -> AzLayoutAlignItemsValueEnumWrapper { AzLayoutAlignItemsValueEnumWrapper { inner: AzLayoutAlignItemsValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzLayoutBottomValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutBottomValueEnumWrapper { AzLayoutBottomValueEnumWrapper { inner: AzLayoutBottomValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutBottomValueEnumWrapper { AzLayoutBottomValueEnumWrapper { inner: AzLayoutBottomValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutBottomValueEnumWrapper { AzLayoutBottomValueEnumWrapper { inner: AzLayoutBottomValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutBottomValueEnumWrapper { AzLayoutBottomValueEnumWrapper { inner: AzLayoutBottomValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutBottom) -> AzLayoutBottomValueEnumWrapper { AzLayoutBottomValueEnumWrapper { inner: AzLayoutBottomValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutBoxSizingValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutBoxSizingValueEnumWrapper { AzLayoutBoxSizingValueEnumWrapper { inner: AzLayoutBoxSizingValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutBoxSizingValueEnumWrapper { AzLayoutBoxSizingValueEnumWrapper { inner: AzLayoutBoxSizingValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutBoxSizingValueEnumWrapper { AzLayoutBoxSizingValueEnumWrapper { inner: AzLayoutBoxSizingValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutBoxSizingValueEnumWrapper { AzLayoutBoxSizingValueEnumWrapper { inner: AzLayoutBoxSizingValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutBoxSizingEnumWrapper) -> AzLayoutBoxSizingValueEnumWrapper { AzLayoutBoxSizingValueEnumWrapper { inner: AzLayoutBoxSizingValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzLayoutFlexDirectionValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutFlexDirectionValueEnumWrapper { AzLayoutFlexDirectionValueEnumWrapper { inner: AzLayoutFlexDirectionValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutFlexDirectionValueEnumWrapper { AzLayoutFlexDirectionValueEnumWrapper { inner: AzLayoutFlexDirectionValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutFlexDirectionValueEnumWrapper { AzLayoutFlexDirectionValueEnumWrapper { inner: AzLayoutFlexDirectionValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutFlexDirectionValueEnumWrapper { AzLayoutFlexDirectionValueEnumWrapper { inner: AzLayoutFlexDirectionValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutFlexDirectionEnumWrapper) -> AzLayoutFlexDirectionValueEnumWrapper { AzLayoutFlexDirectionValueEnumWrapper { inner: AzLayoutFlexDirectionValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzLayoutDisplayValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutDisplayValueEnumWrapper { AzLayoutDisplayValueEnumWrapper { inner: AzLayoutDisplayValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutDisplayValueEnumWrapper { AzLayoutDisplayValueEnumWrapper { inner: AzLayoutDisplayValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutDisplayValueEnumWrapper { AzLayoutDisplayValueEnumWrapper { inner: AzLayoutDisplayValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutDisplayValueEnumWrapper { AzLayoutDisplayValueEnumWrapper { inner: AzLayoutDisplayValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutDisplayEnumWrapper) -> AzLayoutDisplayValueEnumWrapper { AzLayoutDisplayValueEnumWrapper { inner: AzLayoutDisplayValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzLayoutFlexGrowValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutFlexGrowValueEnumWrapper { AzLayoutFlexGrowValueEnumWrapper { inner: AzLayoutFlexGrowValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutFlexGrowValueEnumWrapper { AzLayoutFlexGrowValueEnumWrapper { inner: AzLayoutFlexGrowValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutFlexGrowValueEnumWrapper { AzLayoutFlexGrowValueEnumWrapper { inner: AzLayoutFlexGrowValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutFlexGrowValueEnumWrapper { AzLayoutFlexGrowValueEnumWrapper { inner: AzLayoutFlexGrowValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutFlexGrow) -> AzLayoutFlexGrowValueEnumWrapper { AzLayoutFlexGrowValueEnumWrapper { inner: AzLayoutFlexGrowValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutFlexShrinkValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutFlexShrinkValueEnumWrapper { AzLayoutFlexShrinkValueEnumWrapper { inner: AzLayoutFlexShrinkValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutFlexShrinkValueEnumWrapper { AzLayoutFlexShrinkValueEnumWrapper { inner: AzLayoutFlexShrinkValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutFlexShrinkValueEnumWrapper { AzLayoutFlexShrinkValueEnumWrapper { inner: AzLayoutFlexShrinkValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutFlexShrinkValueEnumWrapper { AzLayoutFlexShrinkValueEnumWrapper { inner: AzLayoutFlexShrinkValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutFlexShrink) -> AzLayoutFlexShrinkValueEnumWrapper { AzLayoutFlexShrinkValueEnumWrapper { inner: AzLayoutFlexShrinkValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutFloatValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutFloatValueEnumWrapper { AzLayoutFloatValueEnumWrapper { inner: AzLayoutFloatValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutFloatValueEnumWrapper { AzLayoutFloatValueEnumWrapper { inner: AzLayoutFloatValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutFloatValueEnumWrapper { AzLayoutFloatValueEnumWrapper { inner: AzLayoutFloatValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutFloatValueEnumWrapper { AzLayoutFloatValueEnumWrapper { inner: AzLayoutFloatValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutFloatEnumWrapper) -> AzLayoutFloatValueEnumWrapper { AzLayoutFloatValueEnumWrapper { inner: AzLayoutFloatValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzLayoutHeightValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutHeightValueEnumWrapper { AzLayoutHeightValueEnumWrapper { inner: AzLayoutHeightValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutHeightValueEnumWrapper { AzLayoutHeightValueEnumWrapper { inner: AzLayoutHeightValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutHeightValueEnumWrapper { AzLayoutHeightValueEnumWrapper { inner: AzLayoutHeightValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutHeightValueEnumWrapper { AzLayoutHeightValueEnumWrapper { inner: AzLayoutHeightValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutHeight) -> AzLayoutHeightValueEnumWrapper { AzLayoutHeightValueEnumWrapper { inner: AzLayoutHeightValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutJustifyContentValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutJustifyContentValueEnumWrapper { AzLayoutJustifyContentValueEnumWrapper { inner: AzLayoutJustifyContentValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutJustifyContentValueEnumWrapper { AzLayoutJustifyContentValueEnumWrapper { inner: AzLayoutJustifyContentValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutJustifyContentValueEnumWrapper { AzLayoutJustifyContentValueEnumWrapper { inner: AzLayoutJustifyContentValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutJustifyContentValueEnumWrapper { AzLayoutJustifyContentValueEnumWrapper { inner: AzLayoutJustifyContentValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutJustifyContentEnumWrapper) -> AzLayoutJustifyContentValueEnumWrapper { AzLayoutJustifyContentValueEnumWrapper { inner: AzLayoutJustifyContentValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzLayoutLeftValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutLeftValueEnumWrapper { AzLayoutLeftValueEnumWrapper { inner: AzLayoutLeftValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutLeftValueEnumWrapper { AzLayoutLeftValueEnumWrapper { inner: AzLayoutLeftValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutLeftValueEnumWrapper { AzLayoutLeftValueEnumWrapper { inner: AzLayoutLeftValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutLeftValueEnumWrapper { AzLayoutLeftValueEnumWrapper { inner: AzLayoutLeftValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutLeft) -> AzLayoutLeftValueEnumWrapper { AzLayoutLeftValueEnumWrapper { inner: AzLayoutLeftValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutMarginBottomValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutMarginBottomValueEnumWrapper { AzLayoutMarginBottomValueEnumWrapper { inner: AzLayoutMarginBottomValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutMarginBottomValueEnumWrapper { AzLayoutMarginBottomValueEnumWrapper { inner: AzLayoutMarginBottomValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutMarginBottomValueEnumWrapper { AzLayoutMarginBottomValueEnumWrapper { inner: AzLayoutMarginBottomValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutMarginBottomValueEnumWrapper { AzLayoutMarginBottomValueEnumWrapper { inner: AzLayoutMarginBottomValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutMarginBottom) -> AzLayoutMarginBottomValueEnumWrapper { AzLayoutMarginBottomValueEnumWrapper { inner: AzLayoutMarginBottomValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutMarginLeftValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutMarginLeftValueEnumWrapper { AzLayoutMarginLeftValueEnumWrapper { inner: AzLayoutMarginLeftValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutMarginLeftValueEnumWrapper { AzLayoutMarginLeftValueEnumWrapper { inner: AzLayoutMarginLeftValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutMarginLeftValueEnumWrapper { AzLayoutMarginLeftValueEnumWrapper { inner: AzLayoutMarginLeftValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutMarginLeftValueEnumWrapper { AzLayoutMarginLeftValueEnumWrapper { inner: AzLayoutMarginLeftValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutMarginLeft) -> AzLayoutMarginLeftValueEnumWrapper { AzLayoutMarginLeftValueEnumWrapper { inner: AzLayoutMarginLeftValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutMarginRightValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutMarginRightValueEnumWrapper { AzLayoutMarginRightValueEnumWrapper { inner: AzLayoutMarginRightValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutMarginRightValueEnumWrapper { AzLayoutMarginRightValueEnumWrapper { inner: AzLayoutMarginRightValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutMarginRightValueEnumWrapper { AzLayoutMarginRightValueEnumWrapper { inner: AzLayoutMarginRightValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutMarginRightValueEnumWrapper { AzLayoutMarginRightValueEnumWrapper { inner: AzLayoutMarginRightValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutMarginRight) -> AzLayoutMarginRightValueEnumWrapper { AzLayoutMarginRightValueEnumWrapper { inner: AzLayoutMarginRightValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutMarginTopValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutMarginTopValueEnumWrapper { AzLayoutMarginTopValueEnumWrapper { inner: AzLayoutMarginTopValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutMarginTopValueEnumWrapper { AzLayoutMarginTopValueEnumWrapper { inner: AzLayoutMarginTopValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutMarginTopValueEnumWrapper { AzLayoutMarginTopValueEnumWrapper { inner: AzLayoutMarginTopValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutMarginTopValueEnumWrapper { AzLayoutMarginTopValueEnumWrapper { inner: AzLayoutMarginTopValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutMarginTop) -> AzLayoutMarginTopValueEnumWrapper { AzLayoutMarginTopValueEnumWrapper { inner: AzLayoutMarginTopValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutMaxHeightValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutMaxHeightValueEnumWrapper { AzLayoutMaxHeightValueEnumWrapper { inner: AzLayoutMaxHeightValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutMaxHeightValueEnumWrapper { AzLayoutMaxHeightValueEnumWrapper { inner: AzLayoutMaxHeightValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutMaxHeightValueEnumWrapper { AzLayoutMaxHeightValueEnumWrapper { inner: AzLayoutMaxHeightValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutMaxHeightValueEnumWrapper { AzLayoutMaxHeightValueEnumWrapper { inner: AzLayoutMaxHeightValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutMaxHeight) -> AzLayoutMaxHeightValueEnumWrapper { AzLayoutMaxHeightValueEnumWrapper { inner: AzLayoutMaxHeightValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutMaxWidthValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutMaxWidthValueEnumWrapper { AzLayoutMaxWidthValueEnumWrapper { inner: AzLayoutMaxWidthValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutMaxWidthValueEnumWrapper { AzLayoutMaxWidthValueEnumWrapper { inner: AzLayoutMaxWidthValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutMaxWidthValueEnumWrapper { AzLayoutMaxWidthValueEnumWrapper { inner: AzLayoutMaxWidthValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutMaxWidthValueEnumWrapper { AzLayoutMaxWidthValueEnumWrapper { inner: AzLayoutMaxWidthValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutMaxWidth) -> AzLayoutMaxWidthValueEnumWrapper { AzLayoutMaxWidthValueEnumWrapper { inner: AzLayoutMaxWidthValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutMinHeightValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutMinHeightValueEnumWrapper { AzLayoutMinHeightValueEnumWrapper { inner: AzLayoutMinHeightValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutMinHeightValueEnumWrapper { AzLayoutMinHeightValueEnumWrapper { inner: AzLayoutMinHeightValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutMinHeightValueEnumWrapper { AzLayoutMinHeightValueEnumWrapper { inner: AzLayoutMinHeightValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutMinHeightValueEnumWrapper { AzLayoutMinHeightValueEnumWrapper { inner: AzLayoutMinHeightValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutMinHeight) -> AzLayoutMinHeightValueEnumWrapper { AzLayoutMinHeightValueEnumWrapper { inner: AzLayoutMinHeightValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutMinWidthValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutMinWidthValueEnumWrapper { AzLayoutMinWidthValueEnumWrapper { inner: AzLayoutMinWidthValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutMinWidthValueEnumWrapper { AzLayoutMinWidthValueEnumWrapper { inner: AzLayoutMinWidthValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutMinWidthValueEnumWrapper { AzLayoutMinWidthValueEnumWrapper { inner: AzLayoutMinWidthValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutMinWidthValueEnumWrapper { AzLayoutMinWidthValueEnumWrapper { inner: AzLayoutMinWidthValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutMinWidth) -> AzLayoutMinWidthValueEnumWrapper { AzLayoutMinWidthValueEnumWrapper { inner: AzLayoutMinWidthValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutPaddingBottomValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutPaddingBottomValueEnumWrapper { AzLayoutPaddingBottomValueEnumWrapper { inner: AzLayoutPaddingBottomValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutPaddingBottomValueEnumWrapper { AzLayoutPaddingBottomValueEnumWrapper { inner: AzLayoutPaddingBottomValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutPaddingBottomValueEnumWrapper { AzLayoutPaddingBottomValueEnumWrapper { inner: AzLayoutPaddingBottomValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutPaddingBottomValueEnumWrapper { AzLayoutPaddingBottomValueEnumWrapper { inner: AzLayoutPaddingBottomValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutPaddingBottom) -> AzLayoutPaddingBottomValueEnumWrapper { AzLayoutPaddingBottomValueEnumWrapper { inner: AzLayoutPaddingBottomValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutPaddingLeftValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutPaddingLeftValueEnumWrapper { AzLayoutPaddingLeftValueEnumWrapper { inner: AzLayoutPaddingLeftValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutPaddingLeftValueEnumWrapper { AzLayoutPaddingLeftValueEnumWrapper { inner: AzLayoutPaddingLeftValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutPaddingLeftValueEnumWrapper { AzLayoutPaddingLeftValueEnumWrapper { inner: AzLayoutPaddingLeftValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutPaddingLeftValueEnumWrapper { AzLayoutPaddingLeftValueEnumWrapper { inner: AzLayoutPaddingLeftValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutPaddingLeft) -> AzLayoutPaddingLeftValueEnumWrapper { AzLayoutPaddingLeftValueEnumWrapper { inner: AzLayoutPaddingLeftValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutPaddingRightValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutPaddingRightValueEnumWrapper { AzLayoutPaddingRightValueEnumWrapper { inner: AzLayoutPaddingRightValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutPaddingRightValueEnumWrapper { AzLayoutPaddingRightValueEnumWrapper { inner: AzLayoutPaddingRightValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutPaddingRightValueEnumWrapper { AzLayoutPaddingRightValueEnumWrapper { inner: AzLayoutPaddingRightValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutPaddingRightValueEnumWrapper { AzLayoutPaddingRightValueEnumWrapper { inner: AzLayoutPaddingRightValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutPaddingRight) -> AzLayoutPaddingRightValueEnumWrapper { AzLayoutPaddingRightValueEnumWrapper { inner: AzLayoutPaddingRightValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutPaddingTopValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutPaddingTopValueEnumWrapper { AzLayoutPaddingTopValueEnumWrapper { inner: AzLayoutPaddingTopValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutPaddingTopValueEnumWrapper { AzLayoutPaddingTopValueEnumWrapper { inner: AzLayoutPaddingTopValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutPaddingTopValueEnumWrapper { AzLayoutPaddingTopValueEnumWrapper { inner: AzLayoutPaddingTopValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutPaddingTopValueEnumWrapper { AzLayoutPaddingTopValueEnumWrapper { inner: AzLayoutPaddingTopValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutPaddingTop) -> AzLayoutPaddingTopValueEnumWrapper { AzLayoutPaddingTopValueEnumWrapper { inner: AzLayoutPaddingTopValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutPositionValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutPositionValueEnumWrapper { AzLayoutPositionValueEnumWrapper { inner: AzLayoutPositionValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutPositionValueEnumWrapper { AzLayoutPositionValueEnumWrapper { inner: AzLayoutPositionValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutPositionValueEnumWrapper { AzLayoutPositionValueEnumWrapper { inner: AzLayoutPositionValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutPositionValueEnumWrapper { AzLayoutPositionValueEnumWrapper { inner: AzLayoutPositionValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutPositionEnumWrapper) -> AzLayoutPositionValueEnumWrapper { AzLayoutPositionValueEnumWrapper { inner: AzLayoutPositionValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzLayoutRightValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutRightValueEnumWrapper { AzLayoutRightValueEnumWrapper { inner: AzLayoutRightValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutRightValueEnumWrapper { AzLayoutRightValueEnumWrapper { inner: AzLayoutRightValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutRightValueEnumWrapper { AzLayoutRightValueEnumWrapper { inner: AzLayoutRightValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutRightValueEnumWrapper { AzLayoutRightValueEnumWrapper { inner: AzLayoutRightValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutRight) -> AzLayoutRightValueEnumWrapper { AzLayoutRightValueEnumWrapper { inner: AzLayoutRightValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutTopValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutTopValueEnumWrapper { AzLayoutTopValueEnumWrapper { inner: AzLayoutTopValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutTopValueEnumWrapper { AzLayoutTopValueEnumWrapper { inner: AzLayoutTopValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutTopValueEnumWrapper { AzLayoutTopValueEnumWrapper { inner: AzLayoutTopValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutTopValueEnumWrapper { AzLayoutTopValueEnumWrapper { inner: AzLayoutTopValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutTop) -> AzLayoutTopValueEnumWrapper { AzLayoutTopValueEnumWrapper { inner: AzLayoutTopValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutWidthValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutWidthValueEnumWrapper { AzLayoutWidthValueEnumWrapper { inner: AzLayoutWidthValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutWidthValueEnumWrapper { AzLayoutWidthValueEnumWrapper { inner: AzLayoutWidthValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutWidthValueEnumWrapper { AzLayoutWidthValueEnumWrapper { inner: AzLayoutWidthValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutWidthValueEnumWrapper { AzLayoutWidthValueEnumWrapper { inner: AzLayoutWidthValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutWidth) -> AzLayoutWidthValueEnumWrapper { AzLayoutWidthValueEnumWrapper { inner: AzLayoutWidthValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutFlexWrapValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutFlexWrapValueEnumWrapper { AzLayoutFlexWrapValueEnumWrapper { inner: AzLayoutFlexWrapValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutFlexWrapValueEnumWrapper { AzLayoutFlexWrapValueEnumWrapper { inner: AzLayoutFlexWrapValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutFlexWrapValueEnumWrapper { AzLayoutFlexWrapValueEnumWrapper { inner: AzLayoutFlexWrapValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutFlexWrapValueEnumWrapper { AzLayoutFlexWrapValueEnumWrapper { inner: AzLayoutFlexWrapValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutFlexWrapEnumWrapper) -> AzLayoutFlexWrapValueEnumWrapper { AzLayoutFlexWrapValueEnumWrapper { inner: AzLayoutFlexWrapValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzLayoutOverflowValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutOverflowValueEnumWrapper { AzLayoutOverflowValueEnumWrapper { inner: AzLayoutOverflowValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutOverflowValueEnumWrapper { AzLayoutOverflowValueEnumWrapper { inner: AzLayoutOverflowValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutOverflowValueEnumWrapper { AzLayoutOverflowValueEnumWrapper { inner: AzLayoutOverflowValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutOverflowValueEnumWrapper { AzLayoutOverflowValueEnumWrapper { inner: AzLayoutOverflowValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutOverflowEnumWrapper) -> AzLayoutOverflowValueEnumWrapper { AzLayoutOverflowValueEnumWrapper { inner: AzLayoutOverflowValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzScrollbarStyleValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzScrollbarStyleValueEnumWrapper { AzScrollbarStyleValueEnumWrapper { inner: AzScrollbarStyleValue::Auto } }
    #[classattr]
    fn None() -> AzScrollbarStyleValueEnumWrapper { AzScrollbarStyleValueEnumWrapper { inner: AzScrollbarStyleValue::None } }
    #[classattr]
    fn Inherit() -> AzScrollbarStyleValueEnumWrapper { AzScrollbarStyleValueEnumWrapper { inner: AzScrollbarStyleValue::Inherit } }
    #[classattr]
    fn Initial() -> AzScrollbarStyleValueEnumWrapper { AzScrollbarStyleValueEnumWrapper { inner: AzScrollbarStyleValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzScrollbarStyle) -> AzScrollbarStyleValueEnumWrapper { AzScrollbarStyleValueEnumWrapper { inner: AzScrollbarStyleValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBackgroundContentVecValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBackgroundContentVecValueEnumWrapper { AzStyleBackgroundContentVecValueEnumWrapper { inner: AzStyleBackgroundContentVecValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBackgroundContentVecValueEnumWrapper { AzStyleBackgroundContentVecValueEnumWrapper { inner: AzStyleBackgroundContentVecValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBackgroundContentVecValueEnumWrapper { AzStyleBackgroundContentVecValueEnumWrapper { inner: AzStyleBackgroundContentVecValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBackgroundContentVecValueEnumWrapper { AzStyleBackgroundContentVecValueEnumWrapper { inner: AzStyleBackgroundContentVecValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBackgroundContentVec) -> AzStyleBackgroundContentVecValueEnumWrapper { AzStyleBackgroundContentVecValueEnumWrapper { inner: AzStyleBackgroundContentVecValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBackgroundPositionVecValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBackgroundPositionVecValueEnumWrapper { AzStyleBackgroundPositionVecValueEnumWrapper { inner: AzStyleBackgroundPositionVecValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBackgroundPositionVecValueEnumWrapper { AzStyleBackgroundPositionVecValueEnumWrapper { inner: AzStyleBackgroundPositionVecValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBackgroundPositionVecValueEnumWrapper { AzStyleBackgroundPositionVecValueEnumWrapper { inner: AzStyleBackgroundPositionVecValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBackgroundPositionVecValueEnumWrapper { AzStyleBackgroundPositionVecValueEnumWrapper { inner: AzStyleBackgroundPositionVecValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBackgroundPositionVec) -> AzStyleBackgroundPositionVecValueEnumWrapper { AzStyleBackgroundPositionVecValueEnumWrapper { inner: AzStyleBackgroundPositionVecValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBackgroundRepeatVecValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBackgroundRepeatVecValueEnumWrapper { AzStyleBackgroundRepeatVecValueEnumWrapper { inner: AzStyleBackgroundRepeatVecValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBackgroundRepeatVecValueEnumWrapper { AzStyleBackgroundRepeatVecValueEnumWrapper { inner: AzStyleBackgroundRepeatVecValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBackgroundRepeatVecValueEnumWrapper { AzStyleBackgroundRepeatVecValueEnumWrapper { inner: AzStyleBackgroundRepeatVecValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBackgroundRepeatVecValueEnumWrapper { AzStyleBackgroundRepeatVecValueEnumWrapper { inner: AzStyleBackgroundRepeatVecValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBackgroundRepeatVec) -> AzStyleBackgroundRepeatVecValueEnumWrapper { AzStyleBackgroundRepeatVecValueEnumWrapper { inner: AzStyleBackgroundRepeatVecValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBackgroundSizeVecValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBackgroundSizeVecValueEnumWrapper { AzStyleBackgroundSizeVecValueEnumWrapper { inner: AzStyleBackgroundSizeVecValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBackgroundSizeVecValueEnumWrapper { AzStyleBackgroundSizeVecValueEnumWrapper { inner: AzStyleBackgroundSizeVecValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBackgroundSizeVecValueEnumWrapper { AzStyleBackgroundSizeVecValueEnumWrapper { inner: AzStyleBackgroundSizeVecValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBackgroundSizeVecValueEnumWrapper { AzStyleBackgroundSizeVecValueEnumWrapper { inner: AzStyleBackgroundSizeVecValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBackgroundSizeVec) -> AzStyleBackgroundSizeVecValueEnumWrapper { AzStyleBackgroundSizeVecValueEnumWrapper { inner: AzStyleBackgroundSizeVecValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderBottomColorValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderBottomColorValueEnumWrapper { AzStyleBorderBottomColorValueEnumWrapper { inner: AzStyleBorderBottomColorValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderBottomColorValueEnumWrapper { AzStyleBorderBottomColorValueEnumWrapper { inner: AzStyleBorderBottomColorValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderBottomColorValueEnumWrapper { AzStyleBorderBottomColorValueEnumWrapper { inner: AzStyleBorderBottomColorValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderBottomColorValueEnumWrapper { AzStyleBorderBottomColorValueEnumWrapper { inner: AzStyleBorderBottomColorValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderBottomColor) -> AzStyleBorderBottomColorValueEnumWrapper { AzStyleBorderBottomColorValueEnumWrapper { inner: AzStyleBorderBottomColorValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderBottomLeftRadiusValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderBottomLeftRadiusValueEnumWrapper { AzStyleBorderBottomLeftRadiusValueEnumWrapper { inner: AzStyleBorderBottomLeftRadiusValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderBottomLeftRadiusValueEnumWrapper { AzStyleBorderBottomLeftRadiusValueEnumWrapper { inner: AzStyleBorderBottomLeftRadiusValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderBottomLeftRadiusValueEnumWrapper { AzStyleBorderBottomLeftRadiusValueEnumWrapper { inner: AzStyleBorderBottomLeftRadiusValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderBottomLeftRadiusValueEnumWrapper { AzStyleBorderBottomLeftRadiusValueEnumWrapper { inner: AzStyleBorderBottomLeftRadiusValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderBottomLeftRadius) -> AzStyleBorderBottomLeftRadiusValueEnumWrapper { AzStyleBorderBottomLeftRadiusValueEnumWrapper { inner: AzStyleBorderBottomLeftRadiusValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderBottomRightRadiusValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderBottomRightRadiusValueEnumWrapper { AzStyleBorderBottomRightRadiusValueEnumWrapper { inner: AzStyleBorderBottomRightRadiusValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderBottomRightRadiusValueEnumWrapper { AzStyleBorderBottomRightRadiusValueEnumWrapper { inner: AzStyleBorderBottomRightRadiusValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderBottomRightRadiusValueEnumWrapper { AzStyleBorderBottomRightRadiusValueEnumWrapper { inner: AzStyleBorderBottomRightRadiusValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderBottomRightRadiusValueEnumWrapper { AzStyleBorderBottomRightRadiusValueEnumWrapper { inner: AzStyleBorderBottomRightRadiusValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderBottomRightRadius) -> AzStyleBorderBottomRightRadiusValueEnumWrapper { AzStyleBorderBottomRightRadiusValueEnumWrapper { inner: AzStyleBorderBottomRightRadiusValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderBottomStyleValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderBottomStyleValueEnumWrapper { AzStyleBorderBottomStyleValueEnumWrapper { inner: AzStyleBorderBottomStyleValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderBottomStyleValueEnumWrapper { AzStyleBorderBottomStyleValueEnumWrapper { inner: AzStyleBorderBottomStyleValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderBottomStyleValueEnumWrapper { AzStyleBorderBottomStyleValueEnumWrapper { inner: AzStyleBorderBottomStyleValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderBottomStyleValueEnumWrapper { AzStyleBorderBottomStyleValueEnumWrapper { inner: AzStyleBorderBottomStyleValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderBottomStyle) -> AzStyleBorderBottomStyleValueEnumWrapper { AzStyleBorderBottomStyleValueEnumWrapper { inner: AzStyleBorderBottomStyleValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutBorderBottomWidthValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutBorderBottomWidthValueEnumWrapper { AzLayoutBorderBottomWidthValueEnumWrapper { inner: AzLayoutBorderBottomWidthValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutBorderBottomWidthValueEnumWrapper { AzLayoutBorderBottomWidthValueEnumWrapper { inner: AzLayoutBorderBottomWidthValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutBorderBottomWidthValueEnumWrapper { AzLayoutBorderBottomWidthValueEnumWrapper { inner: AzLayoutBorderBottomWidthValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutBorderBottomWidthValueEnumWrapper { AzLayoutBorderBottomWidthValueEnumWrapper { inner: AzLayoutBorderBottomWidthValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutBorderBottomWidth) -> AzLayoutBorderBottomWidthValueEnumWrapper { AzLayoutBorderBottomWidthValueEnumWrapper { inner: AzLayoutBorderBottomWidthValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderLeftColorValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderLeftColorValueEnumWrapper { AzStyleBorderLeftColorValueEnumWrapper { inner: AzStyleBorderLeftColorValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderLeftColorValueEnumWrapper { AzStyleBorderLeftColorValueEnumWrapper { inner: AzStyleBorderLeftColorValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderLeftColorValueEnumWrapper { AzStyleBorderLeftColorValueEnumWrapper { inner: AzStyleBorderLeftColorValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderLeftColorValueEnumWrapper { AzStyleBorderLeftColorValueEnumWrapper { inner: AzStyleBorderLeftColorValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderLeftColor) -> AzStyleBorderLeftColorValueEnumWrapper { AzStyleBorderLeftColorValueEnumWrapper { inner: AzStyleBorderLeftColorValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderLeftStyleValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderLeftStyleValueEnumWrapper { AzStyleBorderLeftStyleValueEnumWrapper { inner: AzStyleBorderLeftStyleValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderLeftStyleValueEnumWrapper { AzStyleBorderLeftStyleValueEnumWrapper { inner: AzStyleBorderLeftStyleValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderLeftStyleValueEnumWrapper { AzStyleBorderLeftStyleValueEnumWrapper { inner: AzStyleBorderLeftStyleValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderLeftStyleValueEnumWrapper { AzStyleBorderLeftStyleValueEnumWrapper { inner: AzStyleBorderLeftStyleValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderLeftStyle) -> AzStyleBorderLeftStyleValueEnumWrapper { AzStyleBorderLeftStyleValueEnumWrapper { inner: AzStyleBorderLeftStyleValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutBorderLeftWidthValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutBorderLeftWidthValueEnumWrapper { AzLayoutBorderLeftWidthValueEnumWrapper { inner: AzLayoutBorderLeftWidthValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutBorderLeftWidthValueEnumWrapper { AzLayoutBorderLeftWidthValueEnumWrapper { inner: AzLayoutBorderLeftWidthValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutBorderLeftWidthValueEnumWrapper { AzLayoutBorderLeftWidthValueEnumWrapper { inner: AzLayoutBorderLeftWidthValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutBorderLeftWidthValueEnumWrapper { AzLayoutBorderLeftWidthValueEnumWrapper { inner: AzLayoutBorderLeftWidthValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutBorderLeftWidth) -> AzLayoutBorderLeftWidthValueEnumWrapper { AzLayoutBorderLeftWidthValueEnumWrapper { inner: AzLayoutBorderLeftWidthValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderRightColorValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderRightColorValueEnumWrapper { AzStyleBorderRightColorValueEnumWrapper { inner: AzStyleBorderRightColorValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderRightColorValueEnumWrapper { AzStyleBorderRightColorValueEnumWrapper { inner: AzStyleBorderRightColorValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderRightColorValueEnumWrapper { AzStyleBorderRightColorValueEnumWrapper { inner: AzStyleBorderRightColorValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderRightColorValueEnumWrapper { AzStyleBorderRightColorValueEnumWrapper { inner: AzStyleBorderRightColorValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderRightColor) -> AzStyleBorderRightColorValueEnumWrapper { AzStyleBorderRightColorValueEnumWrapper { inner: AzStyleBorderRightColorValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderRightStyleValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderRightStyleValueEnumWrapper { AzStyleBorderRightStyleValueEnumWrapper { inner: AzStyleBorderRightStyleValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderRightStyleValueEnumWrapper { AzStyleBorderRightStyleValueEnumWrapper { inner: AzStyleBorderRightStyleValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderRightStyleValueEnumWrapper { AzStyleBorderRightStyleValueEnumWrapper { inner: AzStyleBorderRightStyleValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderRightStyleValueEnumWrapper { AzStyleBorderRightStyleValueEnumWrapper { inner: AzStyleBorderRightStyleValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderRightStyle) -> AzStyleBorderRightStyleValueEnumWrapper { AzStyleBorderRightStyleValueEnumWrapper { inner: AzStyleBorderRightStyleValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutBorderRightWidthValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutBorderRightWidthValueEnumWrapper { AzLayoutBorderRightWidthValueEnumWrapper { inner: AzLayoutBorderRightWidthValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutBorderRightWidthValueEnumWrapper { AzLayoutBorderRightWidthValueEnumWrapper { inner: AzLayoutBorderRightWidthValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutBorderRightWidthValueEnumWrapper { AzLayoutBorderRightWidthValueEnumWrapper { inner: AzLayoutBorderRightWidthValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutBorderRightWidthValueEnumWrapper { AzLayoutBorderRightWidthValueEnumWrapper { inner: AzLayoutBorderRightWidthValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutBorderRightWidth) -> AzLayoutBorderRightWidthValueEnumWrapper { AzLayoutBorderRightWidthValueEnumWrapper { inner: AzLayoutBorderRightWidthValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderTopColorValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderTopColorValueEnumWrapper { AzStyleBorderTopColorValueEnumWrapper { inner: AzStyleBorderTopColorValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderTopColorValueEnumWrapper { AzStyleBorderTopColorValueEnumWrapper { inner: AzStyleBorderTopColorValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderTopColorValueEnumWrapper { AzStyleBorderTopColorValueEnumWrapper { inner: AzStyleBorderTopColorValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderTopColorValueEnumWrapper { AzStyleBorderTopColorValueEnumWrapper { inner: AzStyleBorderTopColorValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderTopColor) -> AzStyleBorderTopColorValueEnumWrapper { AzStyleBorderTopColorValueEnumWrapper { inner: AzStyleBorderTopColorValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderTopLeftRadiusValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderTopLeftRadiusValueEnumWrapper { AzStyleBorderTopLeftRadiusValueEnumWrapper { inner: AzStyleBorderTopLeftRadiusValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderTopLeftRadiusValueEnumWrapper { AzStyleBorderTopLeftRadiusValueEnumWrapper { inner: AzStyleBorderTopLeftRadiusValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderTopLeftRadiusValueEnumWrapper { AzStyleBorderTopLeftRadiusValueEnumWrapper { inner: AzStyleBorderTopLeftRadiusValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderTopLeftRadiusValueEnumWrapper { AzStyleBorderTopLeftRadiusValueEnumWrapper { inner: AzStyleBorderTopLeftRadiusValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderTopLeftRadius) -> AzStyleBorderTopLeftRadiusValueEnumWrapper { AzStyleBorderTopLeftRadiusValueEnumWrapper { inner: AzStyleBorderTopLeftRadiusValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderTopRightRadiusValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderTopRightRadiusValueEnumWrapper { AzStyleBorderTopRightRadiusValueEnumWrapper { inner: AzStyleBorderTopRightRadiusValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderTopRightRadiusValueEnumWrapper { AzStyleBorderTopRightRadiusValueEnumWrapper { inner: AzStyleBorderTopRightRadiusValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderTopRightRadiusValueEnumWrapper { AzStyleBorderTopRightRadiusValueEnumWrapper { inner: AzStyleBorderTopRightRadiusValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderTopRightRadiusValueEnumWrapper { AzStyleBorderTopRightRadiusValueEnumWrapper { inner: AzStyleBorderTopRightRadiusValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderTopRightRadius) -> AzStyleBorderTopRightRadiusValueEnumWrapper { AzStyleBorderTopRightRadiusValueEnumWrapper { inner: AzStyleBorderTopRightRadiusValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBorderTopStyleValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBorderTopStyleValueEnumWrapper { AzStyleBorderTopStyleValueEnumWrapper { inner: AzStyleBorderTopStyleValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBorderTopStyleValueEnumWrapper { AzStyleBorderTopStyleValueEnumWrapper { inner: AzStyleBorderTopStyleValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBorderTopStyleValueEnumWrapper { AzStyleBorderTopStyleValueEnumWrapper { inner: AzStyleBorderTopStyleValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBorderTopStyleValueEnumWrapper { AzStyleBorderTopStyleValueEnumWrapper { inner: AzStyleBorderTopStyleValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBorderTopStyle) -> AzStyleBorderTopStyleValueEnumWrapper { AzStyleBorderTopStyleValueEnumWrapper { inner: AzStyleBorderTopStyleValue::Exact(v) } }
}

#[pymethods]
impl AzLayoutBorderTopWidthValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzLayoutBorderTopWidthValueEnumWrapper { AzLayoutBorderTopWidthValueEnumWrapper { inner: AzLayoutBorderTopWidthValue::Auto } }
    #[classattr]
    fn None() -> AzLayoutBorderTopWidthValueEnumWrapper { AzLayoutBorderTopWidthValueEnumWrapper { inner: AzLayoutBorderTopWidthValue::None } }
    #[classattr]
    fn Inherit() -> AzLayoutBorderTopWidthValueEnumWrapper { AzLayoutBorderTopWidthValueEnumWrapper { inner: AzLayoutBorderTopWidthValue::Inherit } }
    #[classattr]
    fn Initial() -> AzLayoutBorderTopWidthValueEnumWrapper { AzLayoutBorderTopWidthValueEnumWrapper { inner: AzLayoutBorderTopWidthValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzLayoutBorderTopWidth) -> AzLayoutBorderTopWidthValueEnumWrapper { AzLayoutBorderTopWidthValueEnumWrapper { inner: AzLayoutBorderTopWidthValue::Exact(v) } }
}

#[pymethods]
impl AzStyleCursorValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleCursorValueEnumWrapper { AzStyleCursorValueEnumWrapper { inner: AzStyleCursorValue::Auto } }
    #[classattr]
    fn None() -> AzStyleCursorValueEnumWrapper { AzStyleCursorValueEnumWrapper { inner: AzStyleCursorValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleCursorValueEnumWrapper { AzStyleCursorValueEnumWrapper { inner: AzStyleCursorValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleCursorValueEnumWrapper { AzStyleCursorValueEnumWrapper { inner: AzStyleCursorValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleCursorEnumWrapper) -> AzStyleCursorValueEnumWrapper { AzStyleCursorValueEnumWrapper { inner: AzStyleCursorValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzStyleFontFamilyVecValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleFontFamilyVecValueEnumWrapper { AzStyleFontFamilyVecValueEnumWrapper { inner: AzStyleFontFamilyVecValue::Auto } }
    #[classattr]
    fn None() -> AzStyleFontFamilyVecValueEnumWrapper { AzStyleFontFamilyVecValueEnumWrapper { inner: AzStyleFontFamilyVecValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleFontFamilyVecValueEnumWrapper { AzStyleFontFamilyVecValueEnumWrapper { inner: AzStyleFontFamilyVecValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleFontFamilyVecValueEnumWrapper { AzStyleFontFamilyVecValueEnumWrapper { inner: AzStyleFontFamilyVecValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleFontFamilyVec) -> AzStyleFontFamilyVecValueEnumWrapper { AzStyleFontFamilyVecValueEnumWrapper { inner: AzStyleFontFamilyVecValue::Exact(v) } }
}

#[pymethods]
impl AzStyleFontSizeValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleFontSizeValueEnumWrapper { AzStyleFontSizeValueEnumWrapper { inner: AzStyleFontSizeValue::Auto } }
    #[classattr]
    fn None() -> AzStyleFontSizeValueEnumWrapper { AzStyleFontSizeValueEnumWrapper { inner: AzStyleFontSizeValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleFontSizeValueEnumWrapper { AzStyleFontSizeValueEnumWrapper { inner: AzStyleFontSizeValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleFontSizeValueEnumWrapper { AzStyleFontSizeValueEnumWrapper { inner: AzStyleFontSizeValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleFontSize) -> AzStyleFontSizeValueEnumWrapper { AzStyleFontSizeValueEnumWrapper { inner: AzStyleFontSizeValue::Exact(v) } }
}

#[pymethods]
impl AzStyleLetterSpacingValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleLetterSpacingValueEnumWrapper { AzStyleLetterSpacingValueEnumWrapper { inner: AzStyleLetterSpacingValue::Auto } }
    #[classattr]
    fn None() -> AzStyleLetterSpacingValueEnumWrapper { AzStyleLetterSpacingValueEnumWrapper { inner: AzStyleLetterSpacingValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleLetterSpacingValueEnumWrapper { AzStyleLetterSpacingValueEnumWrapper { inner: AzStyleLetterSpacingValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleLetterSpacingValueEnumWrapper { AzStyleLetterSpacingValueEnumWrapper { inner: AzStyleLetterSpacingValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleLetterSpacing) -> AzStyleLetterSpacingValueEnumWrapper { AzStyleLetterSpacingValueEnumWrapper { inner: AzStyleLetterSpacingValue::Exact(v) } }
}

#[pymethods]
impl AzStyleLineHeightValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleLineHeightValueEnumWrapper { AzStyleLineHeightValueEnumWrapper { inner: AzStyleLineHeightValue::Auto } }
    #[classattr]
    fn None() -> AzStyleLineHeightValueEnumWrapper { AzStyleLineHeightValueEnumWrapper { inner: AzStyleLineHeightValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleLineHeightValueEnumWrapper { AzStyleLineHeightValueEnumWrapper { inner: AzStyleLineHeightValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleLineHeightValueEnumWrapper { AzStyleLineHeightValueEnumWrapper { inner: AzStyleLineHeightValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleLineHeight) -> AzStyleLineHeightValueEnumWrapper { AzStyleLineHeightValueEnumWrapper { inner: AzStyleLineHeightValue::Exact(v) } }
}

#[pymethods]
impl AzStyleTabWidthValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleTabWidthValueEnumWrapper { AzStyleTabWidthValueEnumWrapper { inner: AzStyleTabWidthValue::Auto } }
    #[classattr]
    fn None() -> AzStyleTabWidthValueEnumWrapper { AzStyleTabWidthValueEnumWrapper { inner: AzStyleTabWidthValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleTabWidthValueEnumWrapper { AzStyleTabWidthValueEnumWrapper { inner: AzStyleTabWidthValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleTabWidthValueEnumWrapper { AzStyleTabWidthValueEnumWrapper { inner: AzStyleTabWidthValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleTabWidth) -> AzStyleTabWidthValueEnumWrapper { AzStyleTabWidthValueEnumWrapper { inner: AzStyleTabWidthValue::Exact(v) } }
}

#[pymethods]
impl AzStyleTextAlignValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleTextAlignValueEnumWrapper { AzStyleTextAlignValueEnumWrapper { inner: AzStyleTextAlignValue::Auto } }
    #[classattr]
    fn None() -> AzStyleTextAlignValueEnumWrapper { AzStyleTextAlignValueEnumWrapper { inner: AzStyleTextAlignValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleTextAlignValueEnumWrapper { AzStyleTextAlignValueEnumWrapper { inner: AzStyleTextAlignValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleTextAlignValueEnumWrapper { AzStyleTextAlignValueEnumWrapper { inner: AzStyleTextAlignValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleTextAlignEnumWrapper) -> AzStyleTextAlignValueEnumWrapper { AzStyleTextAlignValueEnumWrapper { inner: AzStyleTextAlignValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzStyleTextColorValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleTextColorValueEnumWrapper { AzStyleTextColorValueEnumWrapper { inner: AzStyleTextColorValue::Auto } }
    #[classattr]
    fn None() -> AzStyleTextColorValueEnumWrapper { AzStyleTextColorValueEnumWrapper { inner: AzStyleTextColorValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleTextColorValueEnumWrapper { AzStyleTextColorValueEnumWrapper { inner: AzStyleTextColorValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleTextColorValueEnumWrapper { AzStyleTextColorValueEnumWrapper { inner: AzStyleTextColorValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleTextColor) -> AzStyleTextColorValueEnumWrapper { AzStyleTextColorValueEnumWrapper { inner: AzStyleTextColorValue::Exact(v) } }
}

#[pymethods]
impl AzStyleWordSpacingValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleWordSpacingValueEnumWrapper { AzStyleWordSpacingValueEnumWrapper { inner: AzStyleWordSpacingValue::Auto } }
    #[classattr]
    fn None() -> AzStyleWordSpacingValueEnumWrapper { AzStyleWordSpacingValueEnumWrapper { inner: AzStyleWordSpacingValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleWordSpacingValueEnumWrapper { AzStyleWordSpacingValueEnumWrapper { inner: AzStyleWordSpacingValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleWordSpacingValueEnumWrapper { AzStyleWordSpacingValueEnumWrapper { inner: AzStyleWordSpacingValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleWordSpacing) -> AzStyleWordSpacingValueEnumWrapper { AzStyleWordSpacingValueEnumWrapper { inner: AzStyleWordSpacingValue::Exact(v) } }
}

#[pymethods]
impl AzStyleOpacityValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleOpacityValueEnumWrapper { AzStyleOpacityValueEnumWrapper { inner: AzStyleOpacityValue::Auto } }
    #[classattr]
    fn None() -> AzStyleOpacityValueEnumWrapper { AzStyleOpacityValueEnumWrapper { inner: AzStyleOpacityValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleOpacityValueEnumWrapper { AzStyleOpacityValueEnumWrapper { inner: AzStyleOpacityValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleOpacityValueEnumWrapper { AzStyleOpacityValueEnumWrapper { inner: AzStyleOpacityValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleOpacity) -> AzStyleOpacityValueEnumWrapper { AzStyleOpacityValueEnumWrapper { inner: AzStyleOpacityValue::Exact(v) } }
}

#[pymethods]
impl AzStyleTransformVecValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleTransformVecValueEnumWrapper { AzStyleTransformVecValueEnumWrapper { inner: AzStyleTransformVecValue::Auto } }
    #[classattr]
    fn None() -> AzStyleTransformVecValueEnumWrapper { AzStyleTransformVecValueEnumWrapper { inner: AzStyleTransformVecValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleTransformVecValueEnumWrapper { AzStyleTransformVecValueEnumWrapper { inner: AzStyleTransformVecValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleTransformVecValueEnumWrapper { AzStyleTransformVecValueEnumWrapper { inner: AzStyleTransformVecValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleTransformVec) -> AzStyleTransformVecValueEnumWrapper { AzStyleTransformVecValueEnumWrapper { inner: AzStyleTransformVecValue::Exact(v) } }
}

#[pymethods]
impl AzStyleTransformOriginValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleTransformOriginValueEnumWrapper { AzStyleTransformOriginValueEnumWrapper { inner: AzStyleTransformOriginValue::Auto } }
    #[classattr]
    fn None() -> AzStyleTransformOriginValueEnumWrapper { AzStyleTransformOriginValueEnumWrapper { inner: AzStyleTransformOriginValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleTransformOriginValueEnumWrapper { AzStyleTransformOriginValueEnumWrapper { inner: AzStyleTransformOriginValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleTransformOriginValueEnumWrapper { AzStyleTransformOriginValueEnumWrapper { inner: AzStyleTransformOriginValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleTransformOrigin) -> AzStyleTransformOriginValueEnumWrapper { AzStyleTransformOriginValueEnumWrapper { inner: AzStyleTransformOriginValue::Exact(v) } }
}

#[pymethods]
impl AzStylePerspectiveOriginValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStylePerspectiveOriginValueEnumWrapper { AzStylePerspectiveOriginValueEnumWrapper { inner: AzStylePerspectiveOriginValue::Auto } }
    #[classattr]
    fn None() -> AzStylePerspectiveOriginValueEnumWrapper { AzStylePerspectiveOriginValueEnumWrapper { inner: AzStylePerspectiveOriginValue::None } }
    #[classattr]
    fn Inherit() -> AzStylePerspectiveOriginValueEnumWrapper { AzStylePerspectiveOriginValueEnumWrapper { inner: AzStylePerspectiveOriginValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStylePerspectiveOriginValueEnumWrapper { AzStylePerspectiveOriginValueEnumWrapper { inner: AzStylePerspectiveOriginValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStylePerspectiveOrigin) -> AzStylePerspectiveOriginValueEnumWrapper { AzStylePerspectiveOriginValueEnumWrapper { inner: AzStylePerspectiveOriginValue::Exact(v) } }
}

#[pymethods]
impl AzStyleBackfaceVisibilityValueEnumWrapper {
    #[classattr]
    fn Auto() -> AzStyleBackfaceVisibilityValueEnumWrapper { AzStyleBackfaceVisibilityValueEnumWrapper { inner: AzStyleBackfaceVisibilityValue::Auto } }
    #[classattr]
    fn None() -> AzStyleBackfaceVisibilityValueEnumWrapper { AzStyleBackfaceVisibilityValueEnumWrapper { inner: AzStyleBackfaceVisibilityValue::None } }
    #[classattr]
    fn Inherit() -> AzStyleBackfaceVisibilityValueEnumWrapper { AzStyleBackfaceVisibilityValueEnumWrapper { inner: AzStyleBackfaceVisibilityValue::Inherit } }
    #[classattr]
    fn Initial() -> AzStyleBackfaceVisibilityValueEnumWrapper { AzStyleBackfaceVisibilityValueEnumWrapper { inner: AzStyleBackfaceVisibilityValue::Initial } }
    #[staticmethod]
    fn Exact(v: AzStyleBackfaceVisibilityEnumWrapper) -> AzStyleBackfaceVisibilityValueEnumWrapper { AzStyleBackfaceVisibilityValueEnumWrapper { inner: AzStyleBackfaceVisibilityValue::Exact(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzCssPropertyEnumWrapper {
    #[staticmethod]
    fn TextColor(v: AzStyleTextColorValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::TextColor(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn FontSize(v: AzStyleFontSizeValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::FontSize(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn FontFamily(v: AzStyleFontFamilyVecValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::FontFamily(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn TextAlign(v: AzStyleTextAlignValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::TextAlign(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn LetterSpacing(v: AzStyleLetterSpacingValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::LetterSpacing(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn LineHeight(v: AzStyleLineHeightValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::LineHeight(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn WordSpacing(v: AzStyleWordSpacingValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::WordSpacing(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn TabWidth(v: AzStyleTabWidthValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::TabWidth(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Cursor(v: AzStyleCursorValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Cursor(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Display(v: AzLayoutDisplayValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Display(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Float(v: AzLayoutFloatValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Float(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BoxSizing(v: AzLayoutBoxSizingValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BoxSizing(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Width(v: AzLayoutWidthValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Width(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Height(v: AzLayoutHeightValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Height(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn MinWidth(v: AzLayoutMinWidthValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::MinWidth(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn MinHeight(v: AzLayoutMinHeightValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::MinHeight(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn MaxWidth(v: AzLayoutMaxWidthValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::MaxWidth(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn MaxHeight(v: AzLayoutMaxHeightValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::MaxHeight(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Position(v: AzLayoutPositionValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Position(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Top(v: AzLayoutTopValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Top(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Right(v: AzLayoutRightValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Right(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Left(v: AzLayoutLeftValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Left(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Bottom(v: AzLayoutBottomValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Bottom(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn FlexWrap(v: AzLayoutFlexWrapValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::FlexWrap(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn FlexDirection(v: AzLayoutFlexDirectionValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::FlexDirection(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn FlexGrow(v: AzLayoutFlexGrowValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::FlexGrow(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn FlexShrink(v: AzLayoutFlexShrinkValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::FlexShrink(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn JustifyContent(v: AzLayoutJustifyContentValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::JustifyContent(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn AlignItems(v: AzLayoutAlignItemsValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::AlignItems(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn AlignContent(v: AzLayoutAlignContentValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::AlignContent(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BackgroundContent(v: AzStyleBackgroundContentVecValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BackgroundContent(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BackgroundPosition(v: AzStyleBackgroundPositionVecValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BackgroundPosition(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BackgroundSize(v: AzStyleBackgroundSizeVecValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BackgroundSize(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BackgroundRepeat(v: AzStyleBackgroundRepeatVecValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BackgroundRepeat(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn OverflowX(v: AzLayoutOverflowValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::OverflowX(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn OverflowY(v: AzLayoutOverflowValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::OverflowY(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn PaddingTop(v: AzLayoutPaddingTopValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::PaddingTop(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn PaddingLeft(v: AzLayoutPaddingLeftValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::PaddingLeft(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn PaddingRight(v: AzLayoutPaddingRightValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::PaddingRight(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn PaddingBottom(v: AzLayoutPaddingBottomValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::PaddingBottom(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn MarginTop(v: AzLayoutMarginTopValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::MarginTop(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn MarginLeft(v: AzLayoutMarginLeftValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::MarginLeft(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn MarginRight(v: AzLayoutMarginRightValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::MarginRight(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn MarginBottom(v: AzLayoutMarginBottomValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::MarginBottom(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderTopLeftRadius(v: AzStyleBorderTopLeftRadiusValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderTopLeftRadius(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderTopRightRadius(v: AzStyleBorderTopRightRadiusValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderTopRightRadius(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderBottomLeftRadius(v: AzStyleBorderBottomLeftRadiusValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderBottomLeftRadius(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderBottomRightRadius(v: AzStyleBorderBottomRightRadiusValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderBottomRightRadius(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderTopColor(v: AzStyleBorderTopColorValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderTopColor(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderRightColor(v: AzStyleBorderRightColorValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderRightColor(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderLeftColor(v: AzStyleBorderLeftColorValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderLeftColor(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderBottomColor(v: AzStyleBorderBottomColorValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderBottomColor(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderTopStyle(v: AzStyleBorderTopStyleValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderTopStyle(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderRightStyle(v: AzStyleBorderRightStyleValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderRightStyle(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderLeftStyle(v: AzStyleBorderLeftStyleValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderLeftStyle(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderBottomStyle(v: AzStyleBorderBottomStyleValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderBottomStyle(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderTopWidth(v: AzLayoutBorderTopWidthValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderTopWidth(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderRightWidth(v: AzLayoutBorderRightWidthValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderRightWidth(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderLeftWidth(v: AzLayoutBorderLeftWidthValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderLeftWidth(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BorderBottomWidth(v: AzLayoutBorderBottomWidthValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BorderBottomWidth(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BoxShadowLeft(v: AzStyleBoxShadowValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BoxShadowLeft(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BoxShadowRight(v: AzStyleBoxShadowValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BoxShadowRight(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BoxShadowTop(v: AzStyleBoxShadowValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BoxShadowTop(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BoxShadowBottom(v: AzStyleBoxShadowValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BoxShadowBottom(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn ScrollbarStyle(v: AzScrollbarStyleValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::ScrollbarStyle(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Opacity(v: AzStyleOpacityValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Opacity(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn Transform(v: AzStyleTransformVecValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::Transform(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn TransformOrigin(v: AzStyleTransformOriginValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::TransformOrigin(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn PerspectiveOrigin(v: AzStylePerspectiveOriginValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::PerspectiveOrigin(unsafe { mem::transmute(v) }) } }
    #[staticmethod]
    fn BackfaceVisibility(v: AzStyleBackfaceVisibilityValueEnumWrapper) -> AzCssPropertyEnumWrapper { AzCssPropertyEnumWrapper { inner: AzCssProperty::BackfaceVisibility(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzCssPropertySourceEnumWrapper {
    #[staticmethod]
    fn Css(v: AzCssPath) -> AzCssPropertySourceEnumWrapper { AzCssPropertySourceEnumWrapper { inner: AzCssPropertySource::Css(v) } }
    #[classattr]
    fn Inline() -> AzCssPropertySourceEnumWrapper { AzCssPropertySourceEnumWrapper { inner: AzCssPropertySource::Inline } }
}

#[pymethods]
impl AzStyledDom {
    #[staticmethod]
    fn new(dom: AzDom, css: AzCss) -> AzStyledDom {
        unsafe { mem::transmute(crate::AzStyledDom_new(
            mem::transmute(dom),
            mem::transmute(css),
        )) }
    }
    #[staticmethod]
    fn default() -> AzStyledDom {
        unsafe { mem::transmute(crate::AzStyledDom_default()) }
    }
    #[staticmethod]
    fn from_xml(xml_string: String) -> AzStyledDom {
        let xml_string = pystring_to_azstring(&xml_string);
        unsafe { mem::transmute(crate::AzStyledDom_fromXml(
            mem::transmute(xml_string),
        )) }
    }
    #[staticmethod]
    fn from_file(xml_file_path: String) -> AzStyledDom {
        let xml_file_path = pystring_to_azstring(&xml_file_path);
        unsafe { mem::transmute(crate::AzStyledDom_fromFile(
            mem::transmute(xml_file_path),
        )) }
    }
    fn append_child(&mut self, dom: AzStyledDom) -> () {
        unsafe { mem::transmute(crate::AzStyledDom_appendChild(
            mem::transmute(self),
            mem::transmute(dom),
        )) }
    }
    fn restyle(&mut self, css: AzCss) -> () {
        unsafe { mem::transmute(crate::AzStyledDom_restyle(
            mem::transmute(self),
            mem::transmute(css),
        )) }
    }
    fn node_count(&self) -> usize {
        unsafe { mem::transmute(crate::AzStyledDom_nodeCount(
            mem::transmute(self),
        )) }
    }
    fn get_html_string(&self) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzStyledDom_getHtmlString(
            mem::transmute(self),
        )) })
    }
}

#[pymethods]
impl AzTexture {
    #[staticmethod]
    fn allocate_clip_mask(gl: AzGl, size: AzLayoutSize) -> AzTexture {
        unsafe { mem::transmute(crate::AzTexture_allocateClipMask(
            mem::transmute(gl),
            mem::transmute(size),
        )) }
    }
    fn draw_clip_mask(&mut self, node: &AzTesselatedSvgNode) -> bool {
        unsafe { mem::transmute(crate::AzTexture_drawClipMask(
            mem::transmute(self),
            mem::transmute(node),
        )) }
    }
    fn apply_fxaa(&mut self) -> bool {
        unsafe { mem::transmute(crate::AzTexture_applyFxaa(
            mem::transmute(self),
        )) }
    }
}

#[pymethods]
impl AzGl {
    fn get_type(&self) -> AzGlTypeEnumWrapper {
        unsafe { mem::transmute(crate::AzGl_getType(
            mem::transmute(self),
        )) }
    }
    fn buffer_data_untyped(&self, target: u32, size: isize, data: AzGlVoidPtrConst, usage: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_bufferDataUntyped(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(size),
            mem::transmute(data),
            mem::transmute(usage),
        )) }
    }
    fn buffer_sub_data_untyped(&self, target: u32, offset: isize, size: isize, data: AzGlVoidPtrConst) -> () {
        unsafe { mem::transmute(crate::AzGl_bufferSubDataUntyped(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(offset),
            mem::transmute(size),
            mem::transmute(data),
        )) }
    }
    fn map_buffer(&self, target: u32, access: u32) -> AzGlVoidPtrMut {
        unsafe { mem::transmute(crate::AzGl_mapBuffer(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(access),
        )) }
    }
    fn map_buffer_range(&self, target: u32, offset: isize, length: isize, access: u32) -> AzGlVoidPtrMut {
        unsafe { mem::transmute(crate::AzGl_mapBufferRange(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(offset),
            mem::transmute(length),
            mem::transmute(access),
        )) }
    }
    fn unmap_buffer(&self, target: u32) -> u8 {
        unsafe { mem::transmute(crate::AzGl_unmapBuffer(
            mem::transmute(self),
            mem::transmute(target),
        )) }
    }
    fn tex_buffer(&self, target: u32, internal_format: u32, buffer: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_texBuffer(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(internal_format),
            mem::transmute(buffer),
        )) }
    }
    fn shader_source(&self, shader: u32, strings: AzStringVec) -> () {
        unsafe { mem::transmute(crate::AzGl_shaderSource(
            mem::transmute(self),
            mem::transmute(shader),
            mem::transmute(strings),
        )) }
    }
    fn read_buffer(&self, mode: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_readBuffer(
            mem::transmute(self),
            mem::transmute(mode),
        )) }
    }
    fn read_pixels_into_buffer(&self, x: i32, y: i32, width: i32, height: i32, format: u32, pixel_type: u32, mut dst_buffer: Vec<u8>) -> () {
        let dst_buffer = pybytesrefmut_to_vecu8refmut(&mut dst_buffer);
        unsafe { mem::transmute(crate::AzGl_readPixelsIntoBuffer(
            mem::transmute(self),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(format),
            mem::transmute(pixel_type),
            mem::transmute(dst_buffer),
        )) }
    }
    fn read_pixels(&self, x: i32, y: i32, width: i32, height: i32, format: u32, pixel_type: u32) -> Vec<u8> {
        az_vecu8_to_py_vecu8(unsafe { mem::transmute(crate::AzGl_readPixels(
            mem::transmute(self),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(format),
            mem::transmute(pixel_type),
        )) })
    }
    fn read_pixels_into_pbo(&self, x: i32, y: i32, width: i32, height: i32, format: u32, pixel_type: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_readPixelsIntoPbo(
            mem::transmute(self),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(format),
            mem::transmute(pixel_type),
        )) }
    }
    fn sample_coverage(&self, value: f32, invert: bool) -> () {
        unsafe { mem::transmute(crate::AzGl_sampleCoverage(
            mem::transmute(self),
            mem::transmute(value),
            mem::transmute(invert),
        )) }
    }
    fn polygon_offset(&self, factor: f32, units: f32) -> () {
        unsafe { mem::transmute(crate::AzGl_polygonOffset(
            mem::transmute(self),
            mem::transmute(factor),
            mem::transmute(units),
        )) }
    }
    fn pixel_store_i(&self, name: u32, param: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_pixelStoreI(
            mem::transmute(self),
            mem::transmute(name),
            mem::transmute(param),
        )) }
    }
    fn gen_buffers(&self, n: i32) -> AzGLuintVec {
        unsafe { mem::transmute(crate::AzGl_genBuffers(
            mem::transmute(self),
            mem::transmute(n),
        )) }
    }
    fn gen_renderbuffers(&self, n: i32) -> AzGLuintVec {
        unsafe { mem::transmute(crate::AzGl_genRenderbuffers(
            mem::transmute(self),
            mem::transmute(n),
        )) }
    }
    fn gen_framebuffers(&self, n: i32) -> AzGLuintVec {
        unsafe { mem::transmute(crate::AzGl_genFramebuffers(
            mem::transmute(self),
            mem::transmute(n),
        )) }
    }
    fn gen_textures(&self, n: i32) -> AzGLuintVec {
        unsafe { mem::transmute(crate::AzGl_genTextures(
            mem::transmute(self),
            mem::transmute(n),
        )) }
    }
    fn gen_vertex_arrays(&self, n: i32) -> AzGLuintVec {
        unsafe { mem::transmute(crate::AzGl_genVertexArrays(
            mem::transmute(self),
            mem::transmute(n),
        )) }
    }
    fn gen_queries(&self, n: i32) -> AzGLuintVec {
        unsafe { mem::transmute(crate::AzGl_genQueries(
            mem::transmute(self),
            mem::transmute(n),
        )) }
    }
    fn begin_query(&self, target: u32, id: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_beginQuery(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(id),
        )) }
    }
    fn end_query(&self, target: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_endQuery(
            mem::transmute(self),
            mem::transmute(target),
        )) }
    }
    fn query_counter(&self, id: u32, target: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_queryCounter(
            mem::transmute(self),
            mem::transmute(id),
            mem::transmute(target),
        )) }
    }
    fn get_query_object_iv(&self, id: u32, pname: u32) -> i32 {
        unsafe { mem::transmute(crate::AzGl_getQueryObjectIv(
            mem::transmute(self),
            mem::transmute(id),
            mem::transmute(pname),
        )) }
    }
    fn get_query_object_uiv(&self, id: u32, pname: u32) -> u32 {
        unsafe { mem::transmute(crate::AzGl_getQueryObjectUiv(
            mem::transmute(self),
            mem::transmute(id),
            mem::transmute(pname),
        )) }
    }
    fn get_query_object_i64v(&self, id: u32, pname: u32) -> i64 {
        unsafe { mem::transmute(crate::AzGl_getQueryObjectI64V(
            mem::transmute(self),
            mem::transmute(id),
            mem::transmute(pname),
        )) }
    }
    fn get_query_object_ui64v(&self, id: u32, pname: u32) -> u64 {
        unsafe { mem::transmute(crate::AzGl_getQueryObjectUi64V(
            mem::transmute(self),
            mem::transmute(id),
            mem::transmute(pname),
        )) }
    }
    fn delete_queries(&self, queries: Vec<u32>) -> () {
        let queries = pylist_u32_to_rust(&queries);
        unsafe { mem::transmute(crate::AzGl_deleteQueries(
            mem::transmute(self),
            mem::transmute(queries),
        )) }
    }
    fn delete_vertex_arrays(&self, vertex_arrays: Vec<u32>) -> () {
        let vertex_arrays = pylist_u32_to_rust(&vertex_arrays);
        unsafe { mem::transmute(crate::AzGl_deleteVertexArrays(
            mem::transmute(self),
            mem::transmute(vertex_arrays),
        )) }
    }
    fn delete_buffers(&self, buffers: Vec<u32>) -> () {
        let buffers = pylist_u32_to_rust(&buffers);
        unsafe { mem::transmute(crate::AzGl_deleteBuffers(
            mem::transmute(self),
            mem::transmute(buffers),
        )) }
    }
    fn delete_renderbuffers(&self, renderbuffers: Vec<u32>) -> () {
        let renderbuffers = pylist_u32_to_rust(&renderbuffers);
        unsafe { mem::transmute(crate::AzGl_deleteRenderbuffers(
            mem::transmute(self),
            mem::transmute(renderbuffers),
        )) }
    }
    fn delete_framebuffers(&self, framebuffers: Vec<u32>) -> () {
        let framebuffers = pylist_u32_to_rust(&framebuffers);
        unsafe { mem::transmute(crate::AzGl_deleteFramebuffers(
            mem::transmute(self),
            mem::transmute(framebuffers),
        )) }
    }
    fn delete_textures(&self, textures: Vec<u32>) -> () {
        let textures = pylist_u32_to_rust(&textures);
        unsafe { mem::transmute(crate::AzGl_deleteTextures(
            mem::transmute(self),
            mem::transmute(textures),
        )) }
    }
    fn framebuffer_renderbuffer(&self, target: u32, attachment: u32, renderbuffertarget: u32, renderbuffer: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_framebufferRenderbuffer(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(attachment),
            mem::transmute(renderbuffertarget),
            mem::transmute(renderbuffer),
        )) }
    }
    fn renderbuffer_storage(&self, target: u32, internalformat: u32, width: i32, height: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_renderbufferStorage(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(internalformat),
            mem::transmute(width),
            mem::transmute(height),
        )) }
    }
    fn depth_func(&self, func: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_depthFunc(
            mem::transmute(self),
            mem::transmute(func),
        )) }
    }
    fn active_texture(&self, texture: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_activeTexture(
            mem::transmute(self),
            mem::transmute(texture),
        )) }
    }
    fn attach_shader(&self, program: u32, shader: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_attachShader(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(shader),
        )) }
    }
    fn bind_attrib_location(&self, program: u32, index: u32, name: &str) -> () {
        let name = pystring_to_refstr(&name);
        unsafe { mem::transmute(crate::AzGl_bindAttribLocation(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(index),
            mem::transmute(name),
        )) }
    }
    fn get_uniform_iv(&self, program: u32, location: i32, mut result: Vec<i32>) -> () {
        let result = pylist_i32_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getUniformIv(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(location),
            mem::transmute(result),
        )) }
    }
    fn get_uniform_fv(&self, program: u32, location: i32, mut result: Vec<f32>) -> () {
        let result = pylist_glfoat_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getUniformFv(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(location),
            mem::transmute(result),
        )) }
    }
    fn get_uniform_block_index(&self, program: u32, name: &str) -> u32 {
        let name = pystring_to_refstr(&name);
        unsafe { mem::transmute(crate::AzGl_getUniformBlockIndex(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(name),
        )) }
    }
    fn get_uniform_indices(&self, program: u32, names: Vec<&str>) -> AzGLuintVec {
        let names = vec_string_to_vec_refstr(&names);
        let names = pylist_str_to_rust(&names);
        unsafe { mem::transmute(crate::AzGl_getUniformIndices(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(names),
        )) }
    }
    fn bind_buffer_base(&self, target: u32, index: u32, buffer: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_bindBufferBase(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(index),
            mem::transmute(buffer),
        )) }
    }
    fn bind_buffer_range(&self, target: u32, index: u32, buffer: u32, offset: isize, size: isize) -> () {
        unsafe { mem::transmute(crate::AzGl_bindBufferRange(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(index),
            mem::transmute(buffer),
            mem::transmute(offset),
            mem::transmute(size),
        )) }
    }
    fn uniform_block_binding(&self, program: u32, uniform_block_index: u32, uniform_block_binding: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniformBlockBinding(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(uniform_block_index),
            mem::transmute(uniform_block_binding),
        )) }
    }
    fn bind_buffer(&self, target: u32, buffer: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_bindBuffer(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(buffer),
        )) }
    }
    fn bind_vertex_array(&self, vao: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_bindVertexArray(
            mem::transmute(self),
            mem::transmute(vao),
        )) }
    }
    fn bind_renderbuffer(&self, target: u32, renderbuffer: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_bindRenderbuffer(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(renderbuffer),
        )) }
    }
    fn bind_framebuffer(&self, target: u32, framebuffer: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_bindFramebuffer(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(framebuffer),
        )) }
    }
    fn bind_texture(&self, target: u32, texture: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_bindTexture(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(texture),
        )) }
    }
    fn draw_buffers(&self, bufs: AzGLenumVecRef) -> () {
        unsafe { mem::transmute(crate::AzGl_drawBuffers(
            mem::transmute(self),
            mem::transmute(bufs),
        )) }
    }
    fn tex_image_2d(&self, target: u32, level: i32, internal_format: i32, width: i32, height: i32, border: i32, format: u32, ty: u32, opt_data: AzOptionU8VecRefEnumWrapper) -> () {
        unsafe { mem::transmute(crate::AzGl_texImage2D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(internal_format),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(border),
            mem::transmute(format),
            mem::transmute(ty),
            mem::transmute(opt_data),
        )) }
    }
    fn compressed_tex_image_2d(&self, target: u32, level: i32, internal_format: u32, width: i32, height: i32, border: i32, data: Vec<u8>) -> () {
        let data = pybytesref_to_vecu8_ref(&data);
        unsafe { mem::transmute(crate::AzGl_compressedTexImage2D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(internal_format),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(border),
            mem::transmute(data),
        )) }
    }
    fn compressed_tex_sub_image_2d(&self, target: u32, level: i32, xoffset: i32, yoffset: i32, width: i32, height: i32, format: u32, data: Vec<u8>) -> () {
        let data = pybytesref_to_vecu8_ref(&data);
        unsafe { mem::transmute(crate::AzGl_compressedTexSubImage2D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(xoffset),
            mem::transmute(yoffset),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(format),
            mem::transmute(data),
        )) }
    }
    fn tex_image_3d(&self, target: u32, level: i32, internal_format: i32, width: i32, height: i32, depth: i32, border: i32, format: u32, ty: u32, opt_data: AzOptionU8VecRefEnumWrapper) -> () {
        unsafe { mem::transmute(crate::AzGl_texImage3D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(internal_format),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(depth),
            mem::transmute(border),
            mem::transmute(format),
            mem::transmute(ty),
            mem::transmute(opt_data),
        )) }
    }
    fn copy_tex_image_2d(&self, target: u32, level: i32, internal_format: u32, x: i32, y: i32, width: i32, height: i32, border: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_copyTexImage2D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(internal_format),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(border),
        )) }
    }
    fn copy_tex_sub_image_2d(&self, target: u32, level: i32, xoffset: i32, yoffset: i32, x: i32, y: i32, width: i32, height: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_copyTexSubImage2D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(xoffset),
            mem::transmute(yoffset),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(width),
            mem::transmute(height),
        )) }
    }
    fn copy_tex_sub_image_3d(&self, target: u32, level: i32, xoffset: i32, yoffset: i32, zoffset: i32, x: i32, y: i32, width: i32, height: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_copyTexSubImage3D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(xoffset),
            mem::transmute(yoffset),
            mem::transmute(zoffset),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(width),
            mem::transmute(height),
        )) }
    }
    fn tex_sub_image_2d(&self, target: u32, level: i32, xoffset: i32, yoffset: i32, width: i32, height: i32, format: u32, ty: u32, data: Vec<u8>) -> () {
        let data = pybytesref_to_vecu8_ref(&data);
        unsafe { mem::transmute(crate::AzGl_texSubImage2D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(xoffset),
            mem::transmute(yoffset),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(format),
            mem::transmute(ty),
            mem::transmute(data),
        )) }
    }
    fn tex_sub_image_2d_pbo(&self, target: u32, level: i32, xoffset: i32, yoffset: i32, width: i32, height: i32, format: u32, ty: u32, offset: usize) -> () {
        unsafe { mem::transmute(crate::AzGl_texSubImage2DPbo(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(xoffset),
            mem::transmute(yoffset),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(format),
            mem::transmute(ty),
            mem::transmute(offset),
        )) }
    }
    fn tex_sub_image_3d(&self, target: u32, level: i32, xoffset: i32, yoffset: i32, zoffset: i32, width: i32, height: i32, depth: i32, format: u32, ty: u32, data: Vec<u8>) -> () {
        let data = pybytesref_to_vecu8_ref(&data);
        unsafe { mem::transmute(crate::AzGl_texSubImage3D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(xoffset),
            mem::transmute(yoffset),
            mem::transmute(zoffset),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(depth),
            mem::transmute(format),
            mem::transmute(ty),
            mem::transmute(data),
        )) }
    }
    fn tex_sub_image_3d_pbo(&self, target: u32, level: i32, xoffset: i32, yoffset: i32, zoffset: i32, width: i32, height: i32, depth: i32, format: u32, ty: u32, offset: usize) -> () {
        unsafe { mem::transmute(crate::AzGl_texSubImage3DPbo(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(xoffset),
            mem::transmute(yoffset),
            mem::transmute(zoffset),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(depth),
            mem::transmute(format),
            mem::transmute(ty),
            mem::transmute(offset),
        )) }
    }
    fn tex_storage_2d(&self, target: u32, levels: i32, internal_format: u32, width: i32, height: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_texStorage2D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(levels),
            mem::transmute(internal_format),
            mem::transmute(width),
            mem::transmute(height),
        )) }
    }
    fn tex_storage_3d(&self, target: u32, levels: i32, internal_format: u32, width: i32, height: i32, depth: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_texStorage3D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(levels),
            mem::transmute(internal_format),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(depth),
        )) }
    }
    fn get_tex_image_into_buffer(&self, target: u32, level: i32, format: u32, ty: u32, mut output: Vec<u8>) -> () {
        let output = pybytesrefmut_to_vecu8refmut(&mut output);
        unsafe { mem::transmute(crate::AzGl_getTexImageIntoBuffer(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(level),
            mem::transmute(format),
            mem::transmute(ty),
            mem::transmute(output),
        )) }
    }
    fn copy_image_sub_data(&self, src_name: u32, src_target: u32, src_level: i32, src_x: i32, src_y: i32, src_z: i32, dst_name: u32, dst_target: u32, dst_level: i32, dst_x: i32, dst_y: i32, dst_z: i32, src_width: i32, src_height: i32, src_depth: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_copyImageSubData(
            mem::transmute(self),
            mem::transmute(src_name),
            mem::transmute(src_target),
            mem::transmute(src_level),
            mem::transmute(src_x),
            mem::transmute(src_y),
            mem::transmute(src_z),
            mem::transmute(dst_name),
            mem::transmute(dst_target),
            mem::transmute(dst_level),
            mem::transmute(dst_x),
            mem::transmute(dst_y),
            mem::transmute(dst_z),
            mem::transmute(src_width),
            mem::transmute(src_height),
            mem::transmute(src_depth),
        )) }
    }
    fn invalidate_framebuffer(&self, target: u32, attachments: AzGLenumVecRef) -> () {
        unsafe { mem::transmute(crate::AzGl_invalidateFramebuffer(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(attachments),
        )) }
    }
    fn invalidate_sub_framebuffer(&self, target: u32, attachments: AzGLenumVecRef, xoffset: i32, yoffset: i32, width: i32, height: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_invalidateSubFramebuffer(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(attachments),
            mem::transmute(xoffset),
            mem::transmute(yoffset),
            mem::transmute(width),
            mem::transmute(height),
        )) }
    }
    fn get_integer_v(&self, name: u32, mut result: Vec<i32>) -> () {
        let result = pylist_i32_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getIntegerV(
            mem::transmute(self),
            mem::transmute(name),
            mem::transmute(result),
        )) }
    }
    fn get_integer_64v(&self, name: u32, mut result: Vec<i64>) -> () {
        let result = pylist_i64_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getInteger64V(
            mem::transmute(self),
            mem::transmute(name),
            mem::transmute(result),
        )) }
    }
    fn get_integer_iv(&self, name: u32, index: u32, mut result: Vec<i32>) -> () {
        let result = pylist_i32_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getIntegerIv(
            mem::transmute(self),
            mem::transmute(name),
            mem::transmute(index),
            mem::transmute(result),
        )) }
    }
    fn get_integer_64iv(&self, name: u32, index: u32, mut result: Vec<i64>) -> () {
        let result = pylist_i64_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getInteger64Iv(
            mem::transmute(self),
            mem::transmute(name),
            mem::transmute(index),
            mem::transmute(result),
        )) }
    }
    fn get_boolean_v(&self, name: u32, mut result: Vec<u8>) -> () {
        let result = pylist_bool_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getBooleanV(
            mem::transmute(self),
            mem::transmute(name),
            mem::transmute(result),
        )) }
    }
    fn get_float_v(&self, name: u32, mut result: Vec<f32>) -> () {
        let result = pylist_glfoat_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getFloatV(
            mem::transmute(self),
            mem::transmute(name),
            mem::transmute(result),
        )) }
    }
    fn get_framebuffer_attachment_parameter_iv(&self, target: u32, attachment: u32, pname: u32) -> i32 {
        unsafe { mem::transmute(crate::AzGl_getFramebufferAttachmentParameterIv(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(attachment),
            mem::transmute(pname),
        )) }
    }
    fn get_renderbuffer_parameter_iv(&self, target: u32, pname: u32) -> i32 {
        unsafe { mem::transmute(crate::AzGl_getRenderbufferParameterIv(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(pname),
        )) }
    }
    fn get_tex_parameter_iv(&self, target: u32, name: u32) -> i32 {
        unsafe { mem::transmute(crate::AzGl_getTexParameterIv(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(name),
        )) }
    }
    fn get_tex_parameter_fv(&self, target: u32, name: u32) -> f32 {
        unsafe { mem::transmute(crate::AzGl_getTexParameterFv(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(name),
        )) }
    }
    fn tex_parameter_i(&self, target: u32, pname: u32, param: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_texParameterI(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(pname),
            mem::transmute(param),
        )) }
    }
    fn tex_parameter_f(&self, target: u32, pname: u32, param: f32) -> () {
        unsafe { mem::transmute(crate::AzGl_texParameterF(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(pname),
            mem::transmute(param),
        )) }
    }
    fn framebuffer_texture_2d(&self, target: u32, attachment: u32, textarget: u32, texture: u32, level: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_framebufferTexture2D(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(attachment),
            mem::transmute(textarget),
            mem::transmute(texture),
            mem::transmute(level),
        )) }
    }
    fn framebuffer_texture_layer(&self, target: u32, attachment: u32, texture: u32, level: i32, layer: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_framebufferTextureLayer(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(attachment),
            mem::transmute(texture),
            mem::transmute(level),
            mem::transmute(layer),
        )) }
    }
    fn blit_framebuffer(&self, src_x0: i32, src_y0: i32, src_x1: i32, src_y1: i32, dst_x0: i32, dst_y0: i32, dst_x1: i32, dst_y1: i32, mask: u32, filter: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_blitFramebuffer(
            mem::transmute(self),
            mem::transmute(src_x0),
            mem::transmute(src_y0),
            mem::transmute(src_x1),
            mem::transmute(src_y1),
            mem::transmute(dst_x0),
            mem::transmute(dst_y0),
            mem::transmute(dst_x1),
            mem::transmute(dst_y1),
            mem::transmute(mask),
            mem::transmute(filter),
        )) }
    }
    fn vertex_attrib_4f(&self, index: u32, x: f32, y: f32, z: f32, w: f32) -> () {
        unsafe { mem::transmute(crate::AzGl_vertexAttrib4F(
            mem::transmute(self),
            mem::transmute(index),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(z),
            mem::transmute(w),
        )) }
    }
    fn vertex_attrib_pointer_f32(&self, index: u32, size: i32, normalized: bool, stride: i32, offset: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_vertexAttribPointerF32(
            mem::transmute(self),
            mem::transmute(index),
            mem::transmute(size),
            mem::transmute(normalized),
            mem::transmute(stride),
            mem::transmute(offset),
        )) }
    }
    fn vertex_attrib_pointer(&self, index: u32, size: i32, type_: u32, normalized: bool, stride: i32, offset: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_vertexAttribPointer(
            mem::transmute(self),
            mem::transmute(index),
            mem::transmute(size),
            mem::transmute(type_),
            mem::transmute(normalized),
            mem::transmute(stride),
            mem::transmute(offset),
        )) }
    }
    fn vertex_attrib_i_pointer(&self, index: u32, size: i32, type_: u32, stride: i32, offset: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_vertexAttribIPointer(
            mem::transmute(self),
            mem::transmute(index),
            mem::transmute(size),
            mem::transmute(type_),
            mem::transmute(stride),
            mem::transmute(offset),
        )) }
    }
    fn vertex_attrib_divisor(&self, index: u32, divisor: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_vertexAttribDivisor(
            mem::transmute(self),
            mem::transmute(index),
            mem::transmute(divisor),
        )) }
    }
    fn viewport(&self, x: i32, y: i32, width: i32, height: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_viewport(
            mem::transmute(self),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(width),
            mem::transmute(height),
        )) }
    }
    fn scissor(&self, x: i32, y: i32, width: i32, height: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_scissor(
            mem::transmute(self),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(width),
            mem::transmute(height),
        )) }
    }
    fn line_width(&self, width: f32) -> () {
        unsafe { mem::transmute(crate::AzGl_lineWidth(
            mem::transmute(self),
            mem::transmute(width),
        )) }
    }
    fn use_program(&self, program: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_useProgram(
            mem::transmute(self),
            mem::transmute(program),
        )) }
    }
    fn validate_program(&self, program: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_validateProgram(
            mem::transmute(self),
            mem::transmute(program),
        )) }
    }
    fn draw_arrays(&self, mode: u32, first: i32, count: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_drawArrays(
            mem::transmute(self),
            mem::transmute(mode),
            mem::transmute(first),
            mem::transmute(count),
        )) }
    }
    fn draw_arrays_instanced(&self, mode: u32, first: i32, count: i32, primcount: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_drawArraysInstanced(
            mem::transmute(self),
            mem::transmute(mode),
            mem::transmute(first),
            mem::transmute(count),
            mem::transmute(primcount),
        )) }
    }
    fn draw_elements(&self, mode: u32, count: i32, element_type: u32, indices_offset: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_drawElements(
            mem::transmute(self),
            mem::transmute(mode),
            mem::transmute(count),
            mem::transmute(element_type),
            mem::transmute(indices_offset),
        )) }
    }
    fn draw_elements_instanced(&self, mode: u32, count: i32, element_type: u32, indices_offset: u32, primcount: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_drawElementsInstanced(
            mem::transmute(self),
            mem::transmute(mode),
            mem::transmute(count),
            mem::transmute(element_type),
            mem::transmute(indices_offset),
            mem::transmute(primcount),
        )) }
    }
    fn blend_color(&self, r: f32, g: f32, b: f32, a: f32) -> () {
        unsafe { mem::transmute(crate::AzGl_blendColor(
            mem::transmute(self),
            mem::transmute(r),
            mem::transmute(g),
            mem::transmute(b),
            mem::transmute(a),
        )) }
    }
    fn blend_func(&self, sfactor: u32, dfactor: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_blendFunc(
            mem::transmute(self),
            mem::transmute(sfactor),
            mem::transmute(dfactor),
        )) }
    }
    fn blend_func_separate(&self, src_rgb: u32, dest_rgb: u32, src_alpha: u32, dest_alpha: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_blendFuncSeparate(
            mem::transmute(self),
            mem::transmute(src_rgb),
            mem::transmute(dest_rgb),
            mem::transmute(src_alpha),
            mem::transmute(dest_alpha),
        )) }
    }
    fn blend_equation(&self, mode: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_blendEquation(
            mem::transmute(self),
            mem::transmute(mode),
        )) }
    }
    fn blend_equation_separate(&self, mode_rgb: u32, mode_alpha: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_blendEquationSeparate(
            mem::transmute(self),
            mem::transmute(mode_rgb),
            mem::transmute(mode_alpha),
        )) }
    }
    fn color_mask(&self, r: bool, g: bool, b: bool, a: bool) -> () {
        unsafe { mem::transmute(crate::AzGl_colorMask(
            mem::transmute(self),
            mem::transmute(r),
            mem::transmute(g),
            mem::transmute(b),
            mem::transmute(a),
        )) }
    }
    fn cull_face(&self, mode: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_cullFace(
            mem::transmute(self),
            mem::transmute(mode),
        )) }
    }
    fn front_face(&self, mode: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_frontFace(
            mem::transmute(self),
            mem::transmute(mode),
        )) }
    }
    fn enable(&self, cap: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_enable(
            mem::transmute(self),
            mem::transmute(cap),
        )) }
    }
    fn disable(&self, cap: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_disable(
            mem::transmute(self),
            mem::transmute(cap),
        )) }
    }
    fn hint(&self, param_name: u32, param_val: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_hint(
            mem::transmute(self),
            mem::transmute(param_name),
            mem::transmute(param_val),
        )) }
    }
    fn is_enabled(&self, cap: u32) -> u8 {
        unsafe { mem::transmute(crate::AzGl_isEnabled(
            mem::transmute(self),
            mem::transmute(cap),
        )) }
    }
    fn is_shader(&self, shader: u32) -> u8 {
        unsafe { mem::transmute(crate::AzGl_isShader(
            mem::transmute(self),
            mem::transmute(shader),
        )) }
    }
    fn is_texture(&self, texture: u32) -> u8 {
        unsafe { mem::transmute(crate::AzGl_isTexture(
            mem::transmute(self),
            mem::transmute(texture),
        )) }
    }
    fn is_framebuffer(&self, framebuffer: u32) -> u8 {
        unsafe { mem::transmute(crate::AzGl_isFramebuffer(
            mem::transmute(self),
            mem::transmute(framebuffer),
        )) }
    }
    fn is_renderbuffer(&self, renderbuffer: u32) -> u8 {
        unsafe { mem::transmute(crate::AzGl_isRenderbuffer(
            mem::transmute(self),
            mem::transmute(renderbuffer),
        )) }
    }
    fn check_frame_buffer_status(&self, target: u32) -> u32 {
        unsafe { mem::transmute(crate::AzGl_checkFrameBufferStatus(
            mem::transmute(self),
            mem::transmute(target),
        )) }
    }
    fn enable_vertex_attrib_array(&self, index: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_enableVertexAttribArray(
            mem::transmute(self),
            mem::transmute(index),
        )) }
    }
    fn disable_vertex_attrib_array(&self, index: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_disableVertexAttribArray(
            mem::transmute(self),
            mem::transmute(index),
        )) }
    }
    fn uniform_1f(&self, location: i32, v0: f32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform1F(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(v0),
        )) }
    }
    fn uniform_1fv(&self, location: i32, values: Vec<f32>) -> () {
        let values = pylist_f32_to_rust(&values);
        unsafe { mem::transmute(crate::AzGl_uniform1Fv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(values),
        )) }
    }
    fn uniform_1i(&self, location: i32, v0: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform1I(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(v0),
        )) }
    }
    fn uniform_1iv(&self, location: i32, values: AzI32VecRef) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform1Iv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(values),
        )) }
    }
    fn uniform_1ui(&self, location: i32, v0: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform1Ui(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(v0),
        )) }
    }
    fn uniform_2f(&self, location: i32, v0: f32, v1: f32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform2F(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(v0),
            mem::transmute(v1),
        )) }
    }
    fn uniform_2fv(&self, location: i32, values: Vec<f32>) -> () {
        let values = pylist_f32_to_rust(&values);
        unsafe { mem::transmute(crate::AzGl_uniform2Fv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(values),
        )) }
    }
    fn uniform_2i(&self, location: i32, v0: i32, v1: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform2I(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(v0),
            mem::transmute(v1),
        )) }
    }
    fn uniform_2iv(&self, location: i32, values: AzI32VecRef) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform2Iv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(values),
        )) }
    }
    fn uniform_2ui(&self, location: i32, v0: u32, v1: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform2Ui(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(v0),
            mem::transmute(v1),
        )) }
    }
    fn uniform_3f(&self, location: i32, v0: f32, v1: f32, v2: f32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform3F(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(v0),
            mem::transmute(v1),
            mem::transmute(v2),
        )) }
    }
    fn uniform_3fv(&self, location: i32, values: Vec<f32>) -> () {
        let values = pylist_f32_to_rust(&values);
        unsafe { mem::transmute(crate::AzGl_uniform3Fv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(values),
        )) }
    }
    fn uniform_3i(&self, location: i32, v0: i32, v1: i32, v2: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform3I(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(v0),
            mem::transmute(v1),
            mem::transmute(v2),
        )) }
    }
    fn uniform_3iv(&self, location: i32, values: AzI32VecRef) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform3Iv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(values),
        )) }
    }
    fn uniform_3ui(&self, location: i32, v0: u32, v1: u32, v2: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform3Ui(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(v0),
            mem::transmute(v1),
            mem::transmute(v2),
        )) }
    }
    fn uniform_4f(&self, location: i32, x: f32, y: f32, z: f32, w: f32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform4F(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(z),
            mem::transmute(w),
        )) }
    }
    fn uniform_4i(&self, location: i32, x: i32, y: i32, z: i32, w: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform4I(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(z),
            mem::transmute(w),
        )) }
    }
    fn uniform_4iv(&self, location: i32, values: AzI32VecRef) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform4Iv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(values),
        )) }
    }
    fn uniform_4ui(&self, location: i32, x: u32, y: u32, z: u32, w: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_uniform4Ui(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(z),
            mem::transmute(w),
        )) }
    }
    fn uniform_4fv(&self, location: i32, values: Vec<f32>) -> () {
        let values = pylist_f32_to_rust(&values);
        unsafe { mem::transmute(crate::AzGl_uniform4Fv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(values),
        )) }
    }
    fn uniform_matrix_2fv(&self, location: i32, transpose: bool, value: Vec<f32>) -> () {
        let value = pylist_f32_to_rust(&value);
        unsafe { mem::transmute(crate::AzGl_uniformMatrix2Fv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(transpose),
            mem::transmute(value),
        )) }
    }
    fn uniform_matrix_3fv(&self, location: i32, transpose: bool, value: Vec<f32>) -> () {
        let value = pylist_f32_to_rust(&value);
        unsafe { mem::transmute(crate::AzGl_uniformMatrix3Fv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(transpose),
            mem::transmute(value),
        )) }
    }
    fn uniform_matrix_4fv(&self, location: i32, transpose: bool, value: Vec<f32>) -> () {
        let value = pylist_f32_to_rust(&value);
        unsafe { mem::transmute(crate::AzGl_uniformMatrix4Fv(
            mem::transmute(self),
            mem::transmute(location),
            mem::transmute(transpose),
            mem::transmute(value),
        )) }
    }
    fn depth_mask(&self, flag: bool) -> () {
        unsafe { mem::transmute(crate::AzGl_depthMask(
            mem::transmute(self),
            mem::transmute(flag),
        )) }
    }
    fn depth_range(&self, near: f64, far: f64) -> () {
        unsafe { mem::transmute(crate::AzGl_depthRange(
            mem::transmute(self),
            mem::transmute(near),
            mem::transmute(far),
        )) }
    }
    fn get_active_attrib(&self, program: u32, index: u32) -> AzGetActiveAttribReturn {
        unsafe { mem::transmute(crate::AzGl_getActiveAttrib(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(index),
        )) }
    }
    fn get_active_uniform(&self, program: u32, index: u32) -> AzGetActiveUniformReturn {
        unsafe { mem::transmute(crate::AzGl_getActiveUniform(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(index),
        )) }
    }
    fn get_active_uniforms_iv(&self, program: u32, indices: AzGLuintVec, pname: u32) -> AzGLintVec {
        unsafe { mem::transmute(crate::AzGl_getActiveUniformsIv(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(indices),
            mem::transmute(pname),
        )) }
    }
    fn get_active_uniform_block_i(&self, program: u32, index: u32, pname: u32) -> i32 {
        unsafe { mem::transmute(crate::AzGl_getActiveUniformBlockI(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(index),
            mem::transmute(pname),
        )) }
    }
    fn get_active_uniform_block_iv(&self, program: u32, index: u32, pname: u32) -> AzGLintVec {
        unsafe { mem::transmute(crate::AzGl_getActiveUniformBlockIv(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(index),
            mem::transmute(pname),
        )) }
    }
    fn get_active_uniform_block_name(&self, program: u32, index: u32) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzGl_getActiveUniformBlockName(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(index),
        )) })
    }
    fn get_attrib_location(&self, program: u32, name: &str) -> i32 {
        let name = pystring_to_refstr(&name);
        unsafe { mem::transmute(crate::AzGl_getAttribLocation(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(name),
        )) }
    }
    fn get_frag_data_location(&self, program: u32, name: &str) -> i32 {
        let name = pystring_to_refstr(&name);
        unsafe { mem::transmute(crate::AzGl_getFragDataLocation(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(name),
        )) }
    }
    fn get_uniform_location(&self, program: u32, name: &str) -> i32 {
        let name = pystring_to_refstr(&name);
        unsafe { mem::transmute(crate::AzGl_getUniformLocation(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(name),
        )) }
    }
    fn get_program_info_log(&self, program: u32) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzGl_getProgramInfoLog(
            mem::transmute(self),
            mem::transmute(program),
        )) })
    }
    fn get_program_iv(&self, program: u32, pname: u32, mut result: Vec<i32>) -> () {
        let result = pylist_i32_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getProgramIv(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(pname),
            mem::transmute(result),
        )) }
    }
    fn get_program_binary(&self, program: u32) -> AzGetProgramBinaryReturn {
        unsafe { mem::transmute(crate::AzGl_getProgramBinary(
            mem::transmute(self),
            mem::transmute(program),
        )) }
    }
    fn program_binary(&self, program: u32, format: u32, binary: Vec<u8>) -> () {
        let binary = pybytesref_to_vecu8_ref(&binary);
        unsafe { mem::transmute(crate::AzGl_programBinary(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(format),
            mem::transmute(binary),
        )) }
    }
    fn program_parameter_i(&self, program: u32, pname: u32, value: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_programParameterI(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(pname),
            mem::transmute(value),
        )) }
    }
    fn get_vertex_attrib_iv(&self, index: u32, pname: u32, mut result: Vec<i32>) -> () {
        let result = pylist_i32_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getVertexAttribIv(
            mem::transmute(self),
            mem::transmute(index),
            mem::transmute(pname),
            mem::transmute(result),
        )) }
    }
    fn get_vertex_attrib_fv(&self, index: u32, pname: u32, mut result: Vec<f32>) -> () {
        let result = pylist_glfoat_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getVertexAttribFv(
            mem::transmute(self),
            mem::transmute(index),
            mem::transmute(pname),
            mem::transmute(result),
        )) }
    }
    fn get_vertex_attrib_pointer_v(&self, index: u32, pname: u32) -> isize {
        unsafe { mem::transmute(crate::AzGl_getVertexAttribPointerV(
            mem::transmute(self),
            mem::transmute(index),
            mem::transmute(pname),
        )) }
    }
    fn get_buffer_parameter_iv(&self, target: u32, pname: u32) -> i32 {
        unsafe { mem::transmute(crate::AzGl_getBufferParameterIv(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(pname),
        )) }
    }
    fn get_shader_info_log(&self, shader: u32) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzGl_getShaderInfoLog(
            mem::transmute(self),
            mem::transmute(shader),
        )) })
    }
    fn get_string(&self, which: u32) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzGl_getString(
            mem::transmute(self),
            mem::transmute(which),
        )) })
    }
    fn get_string_i(&self, which: u32, index: u32) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzGl_getStringI(
            mem::transmute(self),
            mem::transmute(which),
            mem::transmute(index),
        )) })
    }
    fn get_shader_iv(&self, shader: u32, pname: u32, mut result: Vec<i32>) -> () {
        let result = pylist_i32_to_rust(&mut result);
        unsafe { mem::transmute(crate::AzGl_getShaderIv(
            mem::transmute(self),
            mem::transmute(shader),
            mem::transmute(pname),
            mem::transmute(result),
        )) }
    }
    fn get_shader_precision_format(&self, shader_type: u32, precision_type: u32) -> AzGlShaderPrecisionFormatReturn {
        unsafe { mem::transmute(crate::AzGl_getShaderPrecisionFormat(
            mem::transmute(self),
            mem::transmute(shader_type),
            mem::transmute(precision_type),
        )) }
    }
    fn compile_shader(&self, shader: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_compileShader(
            mem::transmute(self),
            mem::transmute(shader),
        )) }
    }
    fn create_program(&self) -> u32 {
        unsafe { mem::transmute(crate::AzGl_createProgram(
            mem::transmute(self),
        )) }
    }
    fn delete_program(&self, program: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_deleteProgram(
            mem::transmute(self),
            mem::transmute(program),
        )) }
    }
    fn create_shader(&self, shader_type: u32) -> u32 {
        unsafe { mem::transmute(crate::AzGl_createShader(
            mem::transmute(self),
            mem::transmute(shader_type),
        )) }
    }
    fn delete_shader(&self, shader: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_deleteShader(
            mem::transmute(self),
            mem::transmute(shader),
        )) }
    }
    fn detach_shader(&self, program: u32, shader: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_detachShader(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(shader),
        )) }
    }
    fn link_program(&self, program: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_linkProgram(
            mem::transmute(self),
            mem::transmute(program),
        )) }
    }
    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) -> () {
        unsafe { mem::transmute(crate::AzGl_clearColor(
            mem::transmute(self),
            mem::transmute(r),
            mem::transmute(g),
            mem::transmute(b),
            mem::transmute(a),
        )) }
    }
    fn clear(&self, buffer_mask: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_clear(
            mem::transmute(self),
            mem::transmute(buffer_mask),
        )) }
    }
    fn clear_depth(&self, depth: f64) -> () {
        unsafe { mem::transmute(crate::AzGl_clearDepth(
            mem::transmute(self),
            mem::transmute(depth),
        )) }
    }
    fn clear_stencil(&self, s: i32) -> () {
        unsafe { mem::transmute(crate::AzGl_clearStencil(
            mem::transmute(self),
            mem::transmute(s),
        )) }
    }
    fn flush(&self) -> () {
        unsafe { mem::transmute(crate::AzGl_flush(
            mem::transmute(self),
        )) }
    }
    fn finish(&self) -> () {
        unsafe { mem::transmute(crate::AzGl_finish(
            mem::transmute(self),
        )) }
    }
    fn get_error(&self) -> u32 {
        unsafe { mem::transmute(crate::AzGl_getError(
            mem::transmute(self),
        )) }
    }
    fn stencil_mask(&self, mask: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_stencilMask(
            mem::transmute(self),
            mem::transmute(mask),
        )) }
    }
    fn stencil_mask_separate(&self, face: u32, mask: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_stencilMaskSeparate(
            mem::transmute(self),
            mem::transmute(face),
            mem::transmute(mask),
        )) }
    }
    fn stencil_func(&self, func: u32, ref_: i32, mask: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_stencilFunc(
            mem::transmute(self),
            mem::transmute(func),
            mem::transmute(ref_),
            mem::transmute(mask),
        )) }
    }
    fn stencil_func_separate(&self, face: u32, func: u32, ref_: i32, mask: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_stencilFuncSeparate(
            mem::transmute(self),
            mem::transmute(face),
            mem::transmute(func),
            mem::transmute(ref_),
            mem::transmute(mask),
        )) }
    }
    fn stencil_op(&self, sfail: u32, dpfail: u32, dppass: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_stencilOp(
            mem::transmute(self),
            mem::transmute(sfail),
            mem::transmute(dpfail),
            mem::transmute(dppass),
        )) }
    }
    fn stencil_op_separate(&self, face: u32, sfail: u32, dpfail: u32, dppass: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_stencilOpSeparate(
            mem::transmute(self),
            mem::transmute(face),
            mem::transmute(sfail),
            mem::transmute(dpfail),
            mem::transmute(dppass),
        )) }
    }
    fn egl_image_target_texture2d_oes(&self, target: u32, image: AzGlVoidPtrConst) -> () {
        unsafe { mem::transmute(crate::AzGl_eglImageTargetTexture2DOes(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(image),
        )) }
    }
    fn generate_mipmap(&self, target: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_generateMipmap(
            mem::transmute(self),
            mem::transmute(target),
        )) }
    }
    fn insert_event_marker_ext(&self, message: &str) -> () {
        let message = pystring_to_refstr(&message);
        unsafe { mem::transmute(crate::AzGl_insertEventMarkerExt(
            mem::transmute(self),
            mem::transmute(message),
        )) }
    }
    fn push_group_marker_ext(&self, message: &str) -> () {
        let message = pystring_to_refstr(&message);
        unsafe { mem::transmute(crate::AzGl_pushGroupMarkerExt(
            mem::transmute(self),
            mem::transmute(message),
        )) }
    }
    fn pop_group_marker_ext(&self) -> () {
        unsafe { mem::transmute(crate::AzGl_popGroupMarkerExt(
            mem::transmute(self),
        )) }
    }
    fn debug_message_insert_khr(&self, source: u32, type_: u32, id: u32, severity: u32, message: &str) -> () {
        let message = pystring_to_refstr(&message);
        unsafe { mem::transmute(crate::AzGl_debugMessageInsertKhr(
            mem::transmute(self),
            mem::transmute(source),
            mem::transmute(type_),
            mem::transmute(id),
            mem::transmute(severity),
            mem::transmute(message),
        )) }
    }
    fn push_debug_group_khr(&self, source: u32, id: u32, message: &str) -> () {
        let message = pystring_to_refstr(&message);
        unsafe { mem::transmute(crate::AzGl_pushDebugGroupKhr(
            mem::transmute(self),
            mem::transmute(source),
            mem::transmute(id),
            mem::transmute(message),
        )) }
    }
    fn pop_debug_group_khr(&self) -> () {
        unsafe { mem::transmute(crate::AzGl_popDebugGroupKhr(
            mem::transmute(self),
        )) }
    }
    fn fence_sync(&self, condition: u32, flags: u32) -> AzGLsyncPtr {
        unsafe { mem::transmute(crate::AzGl_fenceSync(
            mem::transmute(self),
            mem::transmute(condition),
            mem::transmute(flags),
        )) }
    }
    fn client_wait_sync(&self, sync: AzGLsyncPtr, flags: u32, timeout: u64) -> u32 {
        unsafe { mem::transmute(crate::AzGl_clientWaitSync(
            mem::transmute(self),
            mem::transmute(sync),
            mem::transmute(flags),
            mem::transmute(timeout),
        )) }
    }
    fn wait_sync(&self, sync: AzGLsyncPtr, flags: u32, timeout: u64) -> () {
        unsafe { mem::transmute(crate::AzGl_waitSync(
            mem::transmute(self),
            mem::transmute(sync),
            mem::transmute(flags),
            mem::transmute(timeout),
        )) }
    }
    fn delete_sync(&self, sync: AzGLsyncPtr) -> () {
        unsafe { mem::transmute(crate::AzGl_deleteSync(
            mem::transmute(self),
            mem::transmute(sync),
        )) }
    }
    fn texture_range_apple(&self, target: u32, data: Vec<u8>) -> () {
        let data = pybytesref_to_vecu8_ref(&data);
        unsafe { mem::transmute(crate::AzGl_textureRangeApple(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(data),
        )) }
    }
    fn gen_fences_apple(&self, n: i32) -> AzGLuintVec {
        unsafe { mem::transmute(crate::AzGl_genFencesApple(
            mem::transmute(self),
            mem::transmute(n),
        )) }
    }
    fn delete_fences_apple(&self, fences: Vec<u32>) -> () {
        let fences = pylist_u32_to_rust(&fences);
        unsafe { mem::transmute(crate::AzGl_deleteFencesApple(
            mem::transmute(self),
            mem::transmute(fences),
        )) }
    }
    fn set_fence_apple(&self, fence: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_setFenceApple(
            mem::transmute(self),
            mem::transmute(fence),
        )) }
    }
    fn finish_fence_apple(&self, fence: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_finishFenceApple(
            mem::transmute(self),
            mem::transmute(fence),
        )) }
    }
    fn test_fence_apple(&self, fence: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_testFenceApple(
            mem::transmute(self),
            mem::transmute(fence),
        )) }
    }
    fn test_object_apple(&self, object: u32, name: u32) -> u8 {
        unsafe { mem::transmute(crate::AzGl_testObjectApple(
            mem::transmute(self),
            mem::transmute(object),
            mem::transmute(name),
        )) }
    }
    fn finish_object_apple(&self, object: u32, name: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_finishObjectApple(
            mem::transmute(self),
            mem::transmute(object),
            mem::transmute(name),
        )) }
    }
    fn get_frag_data_index(&self, program: u32, name: &str) -> i32 {
        let name = pystring_to_refstr(&name);
        unsafe { mem::transmute(crate::AzGl_getFragDataIndex(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(name),
        )) }
    }
    fn blend_barrier_khr(&self) -> () {
        unsafe { mem::transmute(crate::AzGl_blendBarrierKhr(
            mem::transmute(self),
        )) }
    }
    fn bind_frag_data_location_indexed(&self, program: u32, color_number: u32, index: u32, name: &str) -> () {
        let name = pystring_to_refstr(&name);
        unsafe { mem::transmute(crate::AzGl_bindFragDataLocationIndexed(
            mem::transmute(self),
            mem::transmute(program),
            mem::transmute(color_number),
            mem::transmute(index),
            mem::transmute(name),
        )) }
    }
    fn get_debug_messages(&self) -> AzDebugMessageVec {
        unsafe { mem::transmute(crate::AzGl_getDebugMessages(
            mem::transmute(self),
        )) }
    }
    fn provoking_vertex_angle(&self, mode: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_provokingVertexAngle(
            mem::transmute(self),
            mem::transmute(mode),
        )) }
    }
    fn gen_vertex_arrays_apple(&self, n: i32) -> AzGLuintVec {
        unsafe { mem::transmute(crate::AzGl_genVertexArraysApple(
            mem::transmute(self),
            mem::transmute(n),
        )) }
    }
    fn bind_vertex_array_apple(&self, vao: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_bindVertexArrayApple(
            mem::transmute(self),
            mem::transmute(vao),
        )) }
    }
    fn delete_vertex_arrays_apple(&self, vertex_arrays: Vec<u32>) -> () {
        let vertex_arrays = pylist_u32_to_rust(&vertex_arrays);
        unsafe { mem::transmute(crate::AzGl_deleteVertexArraysApple(
            mem::transmute(self),
            mem::transmute(vertex_arrays),
        )) }
    }
    fn copy_texture_chromium(&self, source_id: u32, source_level: i32, dest_target: u32, dest_id: u32, dest_level: i32, internal_format: i32, dest_type: u32, unpack_flip_y: u8, unpack_premultiply_alpha: u8, unpack_unmultiply_alpha: u8) -> () {
        unsafe { mem::transmute(crate::AzGl_copyTextureChromium(
            mem::transmute(self),
            mem::transmute(source_id),
            mem::transmute(source_level),
            mem::transmute(dest_target),
            mem::transmute(dest_id),
            mem::transmute(dest_level),
            mem::transmute(internal_format),
            mem::transmute(dest_type),
            mem::transmute(unpack_flip_y),
            mem::transmute(unpack_premultiply_alpha),
            mem::transmute(unpack_unmultiply_alpha),
        )) }
    }
    fn copy_sub_texture_chromium(&self, source_id: u32, source_level: i32, dest_target: u32, dest_id: u32, dest_level: i32, x_offset: i32, y_offset: i32, x: i32, y: i32, width: i32, height: i32, unpack_flip_y: u8, unpack_premultiply_alpha: u8, unpack_unmultiply_alpha: u8) -> () {
        unsafe { mem::transmute(crate::AzGl_copySubTextureChromium(
            mem::transmute(self),
            mem::transmute(source_id),
            mem::transmute(source_level),
            mem::transmute(dest_target),
            mem::transmute(dest_id),
            mem::transmute(dest_level),
            mem::transmute(x_offset),
            mem::transmute(y_offset),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(unpack_flip_y),
            mem::transmute(unpack_premultiply_alpha),
            mem::transmute(unpack_unmultiply_alpha),
        )) }
    }
    fn egl_image_target_renderbuffer_storage_oes(&self, target: u32, image: AzGlVoidPtrConst) -> () {
        unsafe { mem::transmute(crate::AzGl_eglImageTargetRenderbufferStorageOes(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(image),
        )) }
    }
    fn copy_texture_3d_angle(&self, source_id: u32, source_level: i32, dest_target: u32, dest_id: u32, dest_level: i32, internal_format: i32, dest_type: u32, unpack_flip_y: u8, unpack_premultiply_alpha: u8, unpack_unmultiply_alpha: u8) -> () {
        unsafe { mem::transmute(crate::AzGl_copyTexture3DAngle(
            mem::transmute(self),
            mem::transmute(source_id),
            mem::transmute(source_level),
            mem::transmute(dest_target),
            mem::transmute(dest_id),
            mem::transmute(dest_level),
            mem::transmute(internal_format),
            mem::transmute(dest_type),
            mem::transmute(unpack_flip_y),
            mem::transmute(unpack_premultiply_alpha),
            mem::transmute(unpack_unmultiply_alpha),
        )) }
    }
    fn copy_sub_texture_3d_angle(&self, source_id: u32, source_level: i32, dest_target: u32, dest_id: u32, dest_level: i32, x_offset: i32, y_offset: i32, z_offset: i32, x: i32, y: i32, z: i32, width: i32, height: i32, depth: i32, unpack_flip_y: u8, unpack_premultiply_alpha: u8, unpack_unmultiply_alpha: u8) -> () {
        unsafe { mem::transmute(crate::AzGl_copySubTexture3DAngle(
            mem::transmute(self),
            mem::transmute(source_id),
            mem::transmute(source_level),
            mem::transmute(dest_target),
            mem::transmute(dest_id),
            mem::transmute(dest_level),
            mem::transmute(x_offset),
            mem::transmute(y_offset),
            mem::transmute(z_offset),
            mem::transmute(x),
            mem::transmute(y),
            mem::transmute(z),
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(depth),
            mem::transmute(unpack_flip_y),
            mem::transmute(unpack_premultiply_alpha),
            mem::transmute(unpack_unmultiply_alpha),
        )) }
    }
    fn buffer_storage(&self, target: u32, size: isize, data: AzGlVoidPtrConst, flags: u32) -> () {
        unsafe { mem::transmute(crate::AzGl_bufferStorage(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(size),
            mem::transmute(data),
            mem::transmute(flags),
        )) }
    }
    fn flush_mapped_buffer_range(&self, target: u32, offset: isize, length: isize) -> () {
        unsafe { mem::transmute(crate::AzGl_flushMappedBufferRange(
            mem::transmute(self),
            mem::transmute(target),
            mem::transmute(offset),
            mem::transmute(length),
        )) }
    }
}

#[pymethods]
impl AzVertexAttributeTypeEnumWrapper {
    #[classattr]
    fn Float() -> AzVertexAttributeTypeEnumWrapper { AzVertexAttributeTypeEnumWrapper { inner: AzVertexAttributeType::Float } }
    #[classattr]
    fn Double() -> AzVertexAttributeTypeEnumWrapper { AzVertexAttributeTypeEnumWrapper { inner: AzVertexAttributeType::Double } }
    #[classattr]
    fn UnsignedByte() -> AzVertexAttributeTypeEnumWrapper { AzVertexAttributeTypeEnumWrapper { inner: AzVertexAttributeType::UnsignedByte } }
    #[classattr]
    fn UnsignedShort() -> AzVertexAttributeTypeEnumWrapper { AzVertexAttributeTypeEnumWrapper { inner: AzVertexAttributeType::UnsignedShort } }
    #[classattr]
    fn UnsignedInt() -> AzVertexAttributeTypeEnumWrapper { AzVertexAttributeTypeEnumWrapper { inner: AzVertexAttributeType::UnsignedInt } }
}

#[pymethods]
impl AzIndexBufferFormatEnumWrapper {
    #[classattr]
    fn Points() -> AzIndexBufferFormatEnumWrapper { AzIndexBufferFormatEnumWrapper { inner: AzIndexBufferFormat::Points } }
    #[classattr]
    fn Lines() -> AzIndexBufferFormatEnumWrapper { AzIndexBufferFormatEnumWrapper { inner: AzIndexBufferFormat::Lines } }
    #[classattr]
    fn LineStrip() -> AzIndexBufferFormatEnumWrapper { AzIndexBufferFormatEnumWrapper { inner: AzIndexBufferFormat::LineStrip } }
    #[classattr]
    fn Triangles() -> AzIndexBufferFormatEnumWrapper { AzIndexBufferFormatEnumWrapper { inner: AzIndexBufferFormat::Triangles } }
    #[classattr]
    fn TriangleStrip() -> AzIndexBufferFormatEnumWrapper { AzIndexBufferFormatEnumWrapper { inner: AzIndexBufferFormat::TriangleStrip } }
    #[classattr]
    fn TriangleFan() -> AzIndexBufferFormatEnumWrapper { AzIndexBufferFormatEnumWrapper { inner: AzIndexBufferFormat::TriangleFan } }
}

#[pymethods]
impl AzGlTypeEnumWrapper {
    #[classattr]
    fn Gl() -> AzGlTypeEnumWrapper { AzGlTypeEnumWrapper { inner: AzGlType::Gl } }
    #[classattr]
    fn Gles() -> AzGlTypeEnumWrapper { AzGlTypeEnumWrapper { inner: AzGlType::Gles } }
}

#[pymethods]
impl AzTextureFlags {
    #[staticmethod]
    fn default() -> AzTextureFlags {
        unsafe { mem::transmute(crate::AzTextureFlags_default()) }
    }
}

#[pymethods]
impl AzImageRef {
    #[staticmethod]
    fn invalid(width: usize, height: usize, format: AzRawImageFormatEnumWrapper) -> AzImageRef {
        unsafe { mem::transmute(crate::AzImageRef_invalid(
            mem::transmute(width),
            mem::transmute(height),
            mem::transmute(format),
        )) }
    }
    #[staticmethod]
    fn raw_image(data: AzRawImage) -> Option<AzImageRef> {
        let m: AzOptionImageRef = unsafe { mem::transmute(crate::AzImageRef_rawImage(
            mem::transmute(data),
        )) };
        match m {
            AzOptionImageRef::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionImageRef::None => None,
        }

    }
    #[staticmethod]
    fn gl_texture(texture: AzTexture) -> AzImageRef {
        unsafe { mem::transmute(crate::AzImageRef_glTexture(
            mem::transmute(texture),
        )) }
    }
    #[staticmethod]
    fn callback(callback: AzRenderImageCallback, data: AzRefAny) -> AzImageRef {
        unsafe { mem::transmute(crate::AzImageRef_callback(
            mem::transmute(callback),
            mem::transmute(data),
        )) }
    }
    fn clone_bytes(&self) -> AzImageRef {
        unsafe { mem::transmute(crate::AzImageRef_cloneBytes(
            mem::transmute(self),
        )) }
    }
    fn is_invalid(&self) -> bool {
        unsafe { mem::transmute(crate::AzImageRef_isInvalid(
            mem::transmute(self),
        )) }
    }
    fn is_gl_texture(&self) -> bool {
        unsafe { mem::transmute(crate::AzImageRef_isGlTexture(
            mem::transmute(self),
        )) }
    }
    fn is_raw_image(&self) -> bool {
        unsafe { mem::transmute(crate::AzImageRef_isRawImage(
            mem::transmute(self),
        )) }
    }
    fn is_callback(&self) -> bool {
        unsafe { mem::transmute(crate::AzImageRef_isCallback(
            mem::transmute(self),
        )) }
    }
}

#[pymethods]
impl AzRawImage {
    #[staticmethod]
    fn empty() -> AzRawImage {
        unsafe { mem::transmute(crate::AzRawImage_empty()) }
    }
    #[staticmethod]
    fn allocate_clip_mask(size: AzLayoutSize) -> AzRawImage {
        unsafe { mem::transmute(crate::AzRawImage_allocateClipMask(
            mem::transmute(size),
        )) }
    }
    #[staticmethod]
    fn decode_image_bytes_any(bytes: Vec<u8>) -> Result<AzRawImage, PyErr> {
        let bytes = pybytesref_to_vecu8_ref(&bytes);
        let m: AzResultRawImageDecodeImageError = unsafe { mem::transmute(crate::AzRawImage_decodeImageBytesAny(
            mem::transmute(bytes),
        )) };
        match m {
            AzResultRawImageDecodeImageError::Ok(o) => Ok(o.into()),
            AzResultRawImageDecodeImageError::Err(e) => Err(e.into()),
        }

    }
    fn draw_clip_mask(&mut self, node: &AzSvgNodeEnumWrapper, style: AzSvgStyleEnumWrapper) -> bool {
        unsafe { mem::transmute(crate::AzRawImage_drawClipMask(
            mem::transmute(self),
            mem::transmute(node),
            mem::transmute(style),
        )) }
    }
    fn encode_bmp(&self) -> Result<Vec<u8>, PyErr> {
        let m: AzResultU8VecEncodeImageError = unsafe { mem::transmute(crate::AzRawImage_encodeBmp(
            mem::transmute(self),
        )) };
        match m {
            AzResultU8VecEncodeImageError::Ok(o) => Ok(o.into()),
            AzResultU8VecEncodeImageError::Err(e) => Err(e.into()),
        }

    }
    fn encode_png(&self) -> Result<Vec<u8>, PyErr> {
        let m: AzResultU8VecEncodeImageError = unsafe { mem::transmute(crate::AzRawImage_encodePng(
            mem::transmute(self),
        )) };
        match m {
            AzResultU8VecEncodeImageError::Ok(o) => Ok(o.into()),
            AzResultU8VecEncodeImageError::Err(e) => Err(e.into()),
        }

    }
    fn encode_jpeg(&self) -> Result<Vec<u8>, PyErr> {
        let m: AzResultU8VecEncodeImageError = unsafe { mem::transmute(crate::AzRawImage_encodeJpeg(
            mem::transmute(self),
        )) };
        match m {
            AzResultU8VecEncodeImageError::Ok(o) => Ok(o.into()),
            AzResultU8VecEncodeImageError::Err(e) => Err(e.into()),
        }

    }
    fn encode_tga(&self) -> Result<Vec<u8>, PyErr> {
        let m: AzResultU8VecEncodeImageError = unsafe { mem::transmute(crate::AzRawImage_encodeTga(
            mem::transmute(self),
        )) };
        match m {
            AzResultU8VecEncodeImageError::Ok(o) => Ok(o.into()),
            AzResultU8VecEncodeImageError::Err(e) => Err(e.into()),
        }

    }
    fn encode_pnm(&self) -> Result<Vec<u8>, PyErr> {
        let m: AzResultU8VecEncodeImageError = unsafe { mem::transmute(crate::AzRawImage_encodePnm(
            mem::transmute(self),
        )) };
        match m {
            AzResultU8VecEncodeImageError::Ok(o) => Ok(o.into()),
            AzResultU8VecEncodeImageError::Err(e) => Err(e.into()),
        }

    }
    fn encode_gif(&self) -> Result<Vec<u8>, PyErr> {
        let m: AzResultU8VecEncodeImageError = unsafe { mem::transmute(crate::AzRawImage_encodeGif(
            mem::transmute(self),
        )) };
        match m {
            AzResultU8VecEncodeImageError::Ok(o) => Ok(o.into()),
            AzResultU8VecEncodeImageError::Err(e) => Err(e.into()),
        }

    }
    fn encode_tiff(&self) -> Result<Vec<u8>, PyErr> {
        let m: AzResultU8VecEncodeImageError = unsafe { mem::transmute(crate::AzRawImage_encodeTiff(
            mem::transmute(self),
        )) };
        match m {
            AzResultU8VecEncodeImageError::Ok(o) => Ok(o.into()),
            AzResultU8VecEncodeImageError::Err(e) => Err(e.into()),
        }

    }
}

#[pymethods]
impl AzRawImageFormatEnumWrapper {
    #[classattr]
    fn R8() -> AzRawImageFormatEnumWrapper { AzRawImageFormatEnumWrapper { inner: AzRawImageFormat::R8 } }
    #[classattr]
    fn R16() -> AzRawImageFormatEnumWrapper { AzRawImageFormatEnumWrapper { inner: AzRawImageFormat::R16 } }
    #[classattr]
    fn RG16() -> AzRawImageFormatEnumWrapper { AzRawImageFormatEnumWrapper { inner: AzRawImageFormat::RG16 } }
    #[classattr]
    fn BGRA8() -> AzRawImageFormatEnumWrapper { AzRawImageFormatEnumWrapper { inner: AzRawImageFormat::BGRA8 } }
    #[classattr]
    fn RGBAF32() -> AzRawImageFormatEnumWrapper { AzRawImageFormatEnumWrapper { inner: AzRawImageFormat::RGBAF32 } }
    #[classattr]
    fn RG8() -> AzRawImageFormatEnumWrapper { AzRawImageFormatEnumWrapper { inner: AzRawImageFormat::RG8 } }
    #[classattr]
    fn RGBAI32() -> AzRawImageFormatEnumWrapper { AzRawImageFormatEnumWrapper { inner: AzRawImageFormat::RGBAI32 } }
    #[classattr]
    fn RGBA8() -> AzRawImageFormatEnumWrapper { AzRawImageFormatEnumWrapper { inner: AzRawImageFormat::RGBA8 } }
}

#[pymethods]
impl AzEncodeImageErrorEnumWrapper {
    #[classattr]
    fn InsufficientMemory() -> AzEncodeImageErrorEnumWrapper { AzEncodeImageErrorEnumWrapper { inner: AzEncodeImageError::InsufficientMemory } }
    #[classattr]
    fn DimensionError() -> AzEncodeImageErrorEnumWrapper { AzEncodeImageErrorEnumWrapper { inner: AzEncodeImageError::DimensionError } }
    #[classattr]
    fn InvalidData() -> AzEncodeImageErrorEnumWrapper { AzEncodeImageErrorEnumWrapper { inner: AzEncodeImageError::InvalidData } }
    #[classattr]
    fn Unknown() -> AzEncodeImageErrorEnumWrapper { AzEncodeImageErrorEnumWrapper { inner: AzEncodeImageError::Unknown } }
}

#[pymethods]
impl AzDecodeImageErrorEnumWrapper {
    #[classattr]
    fn InsufficientMemory() -> AzDecodeImageErrorEnumWrapper { AzDecodeImageErrorEnumWrapper { inner: AzDecodeImageError::InsufficientMemory } }
    #[classattr]
    fn DimensionError() -> AzDecodeImageErrorEnumWrapper { AzDecodeImageErrorEnumWrapper { inner: AzDecodeImageError::DimensionError } }
    #[classattr]
    fn UnsupportedImageFormat() -> AzDecodeImageErrorEnumWrapper { AzDecodeImageErrorEnumWrapper { inner: AzDecodeImageError::UnsupportedImageFormat } }
    #[classattr]
    fn Unknown() -> AzDecodeImageErrorEnumWrapper { AzDecodeImageErrorEnumWrapper { inner: AzDecodeImageError::Unknown } }
}

#[pymethods]
impl AzRawImageDataEnumWrapper {
    #[staticmethod]
    fn U8(v: AzU8Vec) -> AzRawImageDataEnumWrapper { AzRawImageDataEnumWrapper { inner: AzRawImageData::U8(v) } }
    #[staticmethod]
    fn U16(v: AzU16Vec) -> AzRawImageDataEnumWrapper { AzRawImageDataEnumWrapper { inner: AzRawImageData::U16(v) } }
    #[staticmethod]
    fn F32(v: AzF32Vec) -> AzRawImageDataEnumWrapper { AzRawImageDataEnumWrapper { inner: AzRawImageData::F32(v) } }
}

#[pymethods]
impl AzFontRef {
    #[staticmethod]
    fn parse(source: AzFontSource) -> Option<AzFontRef> {
        let m: AzOptionFontRef = unsafe { mem::transmute(crate::AzFontRef_parse(
            mem::transmute(source),
        )) };
        match m {
            AzOptionFontRef::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionFontRef::None => None,
        }

    }
    fn get_font_metrics(&self) -> AzFontMetrics {
        unsafe { mem::transmute(crate::AzFontRef_getFontMetrics(
            mem::transmute(self),
        )) }
    }
}

#[pymethods]
impl AzSvg {
    #[staticmethod]
    fn from_string(svg_string: String, parse_options: AzSvgParseOptions) -> Result<AzSvg, PyErr> {
        let svg_string = pystring_to_azstring(&svg_string);
        let m: AzResultSvgSvgParseError = unsafe { mem::transmute(crate::AzSvg_fromString(
            mem::transmute(svg_string),
            mem::transmute(parse_options),
        )) };
        match m {
            AzResultSvgSvgParseError::Ok(o) => Ok(o.into()),
            AzResultSvgSvgParseError::Err(e) => Err(e.into()),
        }

    }
    #[staticmethod]
    fn from_bytes(svg_bytes: Vec<u8>, parse_options: AzSvgParseOptions) -> Result<AzSvg, PyErr> {
        let svg_bytes = pybytesref_to_vecu8_ref(&svg_bytes);
        let m: AzResultSvgSvgParseError = unsafe { mem::transmute(crate::AzSvg_fromBytes(
            mem::transmute(svg_bytes),
            mem::transmute(parse_options),
        )) };
        match m {
            AzResultSvgSvgParseError::Ok(o) => Ok(o.into()),
            AzResultSvgSvgParseError::Err(e) => Err(e.into()),
        }

    }
    fn get_root(&self) -> AzSvgXmlNode {
        unsafe { mem::transmute(crate::AzSvg_getRoot(
            mem::transmute(self),
        )) }
    }
    fn render(&self, options: AzSvgRenderOptions) -> Option<AzRawImage> {
        let m: AzOptionRawImage = unsafe { mem::transmute(crate::AzSvg_render(
            mem::transmute(self),
            mem::transmute(options),
        )) };
        match m {
            AzOptionRawImage::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionRawImage::None => None,
        }

    }
    fn to_string(&self, options: AzSvgStringFormatOptions) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzSvg_toString(
            mem::transmute(self),
            mem::transmute(options),
        )) })
    }
}

#[pymethods]
impl AzSvgXmlNode {
    #[staticmethod]
    fn parse_from(svg_bytes: Vec<u8>, parse_options: AzSvgParseOptions) -> Result<AzSvgXmlNode, PyErr> {
        let svg_bytes = pybytesref_to_vecu8_ref(&svg_bytes);
        let m: AzResultSvgXmlNodeSvgParseError = unsafe { mem::transmute(crate::AzSvgXmlNode_parseFrom(
            mem::transmute(svg_bytes),
            mem::transmute(parse_options),
        )) };
        match m {
            AzResultSvgXmlNodeSvgParseError::Ok(o) => Ok(o.into()),
            AzResultSvgXmlNodeSvgParseError::Err(e) => Err(e.into()),
        }

    }
    fn render(&self, options: AzSvgRenderOptions) -> Option<AzRawImage> {
        let m: AzOptionRawImage = unsafe { mem::transmute(crate::AzSvgXmlNode_render(
            mem::transmute(self),
            mem::transmute(options),
        )) };
        match m {
            AzOptionRawImage::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionRawImage::None => None,
        }

    }
    fn to_string(&self, options: AzSvgStringFormatOptions) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzSvgXmlNode_toString(
            mem::transmute(self),
            mem::transmute(options),
        )) })
    }
}

#[pymethods]
impl AzSvgMultiPolygon {
    fn tesselate_fill(&self, fill_style: AzSvgFillStyle) -> AzTesselatedSvgNode {
        unsafe { mem::transmute(crate::AzSvgMultiPolygon_tesselateFill(
            mem::transmute(self),
            mem::transmute(fill_style),
        )) }
    }
    fn tesselate_stroke(&self, stroke_style: AzSvgStrokeStyle) -> AzTesselatedSvgNode {
        unsafe { mem::transmute(crate::AzSvgMultiPolygon_tesselateStroke(
            mem::transmute(self),
            mem::transmute(stroke_style),
        )) }
    }
}

#[pymethods]
impl AzSvgNodeEnumWrapper {
    #[staticmethod]
    fn MultiPolygonCollection(v: AzSvgMultiPolygonVec) -> AzSvgNodeEnumWrapper { AzSvgNodeEnumWrapper { inner: AzSvgNode::MultiPolygonCollection(v) } }
    #[staticmethod]
    fn MultiPolygon(v: AzSvgMultiPolygon) -> AzSvgNodeEnumWrapper { AzSvgNodeEnumWrapper { inner: AzSvgNode::MultiPolygon(v) } }
    #[staticmethod]
    fn Path(v: AzSvgPath) -> AzSvgNodeEnumWrapper { AzSvgNodeEnumWrapper { inner: AzSvgNode::Path(v) } }
    #[staticmethod]
    fn Circle(v: AzSvgCircle) -> AzSvgNodeEnumWrapper { AzSvgNodeEnumWrapper { inner: AzSvgNode::Circle(v) } }
    #[staticmethod]
    fn Rect(v: AzSvgRect) -> AzSvgNodeEnumWrapper { AzSvgNodeEnumWrapper { inner: AzSvgNode::Rect(v) } }
}

#[pymethods]
impl AzSvgStyledNode {
    fn tesselate(&self) -> AzTesselatedSvgNode {
        unsafe { mem::transmute(crate::AzSvgStyledNode_tesselate(
            mem::transmute(self),
        )) }
    }
}

#[pymethods]
impl AzSvgCircle {
    fn tesselate_fill(&self, fill_style: AzSvgFillStyle) -> AzTesselatedSvgNode {
        unsafe { mem::transmute(crate::AzSvgCircle_tesselateFill(
            mem::transmute(self),
            mem::transmute(fill_style),
        )) }
    }
    fn tesselate_stroke(&self, stroke_style: AzSvgStrokeStyle) -> AzTesselatedSvgNode {
        unsafe { mem::transmute(crate::AzSvgCircle_tesselateStroke(
            mem::transmute(self),
            mem::transmute(stroke_style),
        )) }
    }
}

#[pymethods]
impl AzSvgPath {
    fn tesselate_fill(&self, fill_style: AzSvgFillStyle) -> AzTesselatedSvgNode {
        unsafe { mem::transmute(crate::AzSvgPath_tesselateFill(
            mem::transmute(self),
            mem::transmute(fill_style),
        )) }
    }
    fn tesselate_stroke(&self, stroke_style: AzSvgStrokeStyle) -> AzTesselatedSvgNode {
        unsafe { mem::transmute(crate::AzSvgPath_tesselateStroke(
            mem::transmute(self),
            mem::transmute(stroke_style),
        )) }
    }
}

#[pymethods]
impl AzSvgPathElementEnumWrapper {
    #[staticmethod]
    fn Line(v: AzSvgLine) -> AzSvgPathElementEnumWrapper { AzSvgPathElementEnumWrapper { inner: AzSvgPathElement::Line(v) } }
    #[staticmethod]
    fn QuadraticCurve(v: AzSvgQuadraticCurve) -> AzSvgPathElementEnumWrapper { AzSvgPathElementEnumWrapper { inner: AzSvgPathElement::QuadraticCurve(v) } }
    #[staticmethod]
    fn CubicCurve(v: AzSvgCubicCurve) -> AzSvgPathElementEnumWrapper { AzSvgPathElementEnumWrapper { inner: AzSvgPathElement::CubicCurve(v) } }
}

#[pymethods]
impl AzSvgRect {
    fn tesselate_fill(&self, fill_style: AzSvgFillStyle) -> AzTesselatedSvgNode {
        unsafe { mem::transmute(crate::AzSvgRect_tesselateFill(
            mem::transmute(self),
            mem::transmute(fill_style),
        )) }
    }
    fn tesselate_stroke(&self, stroke_style: AzSvgStrokeStyle) -> AzTesselatedSvgNode {
        unsafe { mem::transmute(crate::AzSvgRect_tesselateStroke(
            mem::transmute(self),
            mem::transmute(stroke_style),
        )) }
    }
}

#[pymethods]
impl AzTesselatedSvgNode {
    #[staticmethod]
    fn empty() -> AzTesselatedSvgNode {
        unsafe { mem::transmute(crate::AzTesselatedSvgNode_empty()) }
    }
    #[staticmethod]
    fn from_nodes(nodes: Vec<AzTesselatedSvgNode>) -> AzTesselatedSvgNode {
        let nodes = pylist_tesselated_svg_node(&nodes);
        unsafe { mem::transmute(crate::AzTesselatedSvgNode_fromNodes(
            mem::transmute(nodes),
        )) }
    }
}

#[pymethods]
impl AzSvgParseOptions {
    #[staticmethod]
    fn default() -> AzSvgParseOptions {
        unsafe { mem::transmute(crate::AzSvgParseOptions_default()) }
    }
}

#[pymethods]
impl AzShapeRenderingEnumWrapper {
    #[classattr]
    fn OptimizeSpeed() -> AzShapeRenderingEnumWrapper { AzShapeRenderingEnumWrapper { inner: AzShapeRendering::OptimizeSpeed } }
    #[classattr]
    fn CrispEdges() -> AzShapeRenderingEnumWrapper { AzShapeRenderingEnumWrapper { inner: AzShapeRendering::CrispEdges } }
    #[classattr]
    fn GeometricPrecision() -> AzShapeRenderingEnumWrapper { AzShapeRenderingEnumWrapper { inner: AzShapeRendering::GeometricPrecision } }
}

#[pymethods]
impl AzTextRenderingEnumWrapper {
    #[classattr]
    fn OptimizeSpeed() -> AzTextRenderingEnumWrapper { AzTextRenderingEnumWrapper { inner: AzTextRendering::OptimizeSpeed } }
    #[classattr]
    fn OptimizeLegibility() -> AzTextRenderingEnumWrapper { AzTextRenderingEnumWrapper { inner: AzTextRendering::OptimizeLegibility } }
    #[classattr]
    fn GeometricPrecision() -> AzTextRenderingEnumWrapper { AzTextRenderingEnumWrapper { inner: AzTextRendering::GeometricPrecision } }
}

#[pymethods]
impl AzImageRenderingEnumWrapper {
    #[classattr]
    fn OptimizeQuality() -> AzImageRenderingEnumWrapper { AzImageRenderingEnumWrapper { inner: AzImageRendering::OptimizeQuality } }
    #[classattr]
    fn OptimizeSpeed() -> AzImageRenderingEnumWrapper { AzImageRenderingEnumWrapper { inner: AzImageRendering::OptimizeSpeed } }
}

#[pymethods]
impl AzFontDatabaseEnumWrapper {
    #[classattr]
    fn Empty() -> AzFontDatabaseEnumWrapper { AzFontDatabaseEnumWrapper { inner: AzFontDatabase::Empty } }
    #[classattr]
    fn System() -> AzFontDatabaseEnumWrapper { AzFontDatabaseEnumWrapper { inner: AzFontDatabase::System } }
}

#[pymethods]
impl AzSvgRenderOptions {
    #[staticmethod]
    fn default() -> AzSvgRenderOptions {
        unsafe { mem::transmute(crate::AzSvgRenderOptions_default()) }
    }
}

#[pymethods]
impl AzIndentEnumWrapper {
    #[classattr]
    fn None() -> AzIndentEnumWrapper { AzIndentEnumWrapper { inner: AzIndent::None } }
    #[staticmethod]
    fn Spaces(v: u8) -> AzIndentEnumWrapper { AzIndentEnumWrapper { inner: AzIndent::Spaces(v) } }
    #[classattr]
    fn Tabs() -> AzIndentEnumWrapper { AzIndentEnumWrapper { inner: AzIndent::Tabs } }
}

#[pymethods]
impl AzSvgFitToEnumWrapper {
    #[classattr]
    fn Original() -> AzSvgFitToEnumWrapper { AzSvgFitToEnumWrapper { inner: AzSvgFitTo::Original } }
    #[staticmethod]
    fn Width(v: u32) -> AzSvgFitToEnumWrapper { AzSvgFitToEnumWrapper { inner: AzSvgFitTo::Width(v) } }
    #[staticmethod]
    fn Height(v: u32) -> AzSvgFitToEnumWrapper { AzSvgFitToEnumWrapper { inner: AzSvgFitTo::Height(v) } }
    #[staticmethod]
    fn Zoom(v: f32) -> AzSvgFitToEnumWrapper { AzSvgFitToEnumWrapper { inner: AzSvgFitTo::Zoom(v) } }
}

#[pymethods]
impl AzSvgStyleEnumWrapper {
    #[staticmethod]
    fn Fill(v: AzSvgFillStyle) -> AzSvgStyleEnumWrapper { AzSvgStyleEnumWrapper { inner: AzSvgStyle::Fill(v) } }
    #[staticmethod]
    fn Stroke(v: AzSvgStrokeStyle) -> AzSvgStyleEnumWrapper { AzSvgStyleEnumWrapper { inner: AzSvgStyle::Stroke(v) } }
}

#[pymethods]
impl AzSvgFillRuleEnumWrapper {
    #[classattr]
    fn Winding() -> AzSvgFillRuleEnumWrapper { AzSvgFillRuleEnumWrapper { inner: AzSvgFillRule::Winding } }
    #[classattr]
    fn EvenOdd() -> AzSvgFillRuleEnumWrapper { AzSvgFillRuleEnumWrapper { inner: AzSvgFillRule::EvenOdd } }
}

#[pymethods]
impl AzSvgLineJoinEnumWrapper {
    #[classattr]
    fn Miter() -> AzSvgLineJoinEnumWrapper { AzSvgLineJoinEnumWrapper { inner: AzSvgLineJoin::Miter } }
    #[classattr]
    fn MiterClip() -> AzSvgLineJoinEnumWrapper { AzSvgLineJoinEnumWrapper { inner: AzSvgLineJoin::MiterClip } }
    #[classattr]
    fn Round() -> AzSvgLineJoinEnumWrapper { AzSvgLineJoinEnumWrapper { inner: AzSvgLineJoin::Round } }
    #[classattr]
    fn Bevel() -> AzSvgLineJoinEnumWrapper { AzSvgLineJoinEnumWrapper { inner: AzSvgLineJoin::Bevel } }
}

#[pymethods]
impl AzSvgLineCapEnumWrapper {
    #[classattr]
    fn Butt() -> AzSvgLineCapEnumWrapper { AzSvgLineCapEnumWrapper { inner: AzSvgLineCap::Butt } }
    #[classattr]
    fn Square() -> AzSvgLineCapEnumWrapper { AzSvgLineCapEnumWrapper { inner: AzSvgLineCap::Square } }
    #[classattr]
    fn Round() -> AzSvgLineCapEnumWrapper { AzSvgLineCapEnumWrapper { inner: AzSvgLineCap::Round } }
}

#[pymethods]
impl AzXml {
    #[staticmethod]
    fn from_str(xml_string: &str) -> Result<AzXml, PyErr> {
        let xml_string = pystring_to_refstr(&xml_string);
        let m: AzResultXmlXmlError = unsafe { mem::transmute(crate::AzXml_fromStr(
            mem::transmute(xml_string),
        )) };
        match m {
            AzResultXmlXmlError::Ok(o) => Ok(o.into()),
            AzResultXmlXmlError::Err(e) => Err(e.into()),
        }

    }
}

#[pymethods]
impl AzFile {
    #[staticmethod]
    fn open(path: String) -> Option<AzFile> {
        let path = pystring_to_azstring(&path);
        let m: AzOptionFile = unsafe { mem::transmute(crate::AzFile_open(
            mem::transmute(path),
        )) };
        match m {
            AzOptionFile::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionFile::None => None,
        }

    }
    #[staticmethod]
    fn create(path: String) -> Option<AzFile> {
        let path = pystring_to_azstring(&path);
        let m: AzOptionFile = unsafe { mem::transmute(crate::AzFile_create(
            mem::transmute(path),
        )) };
        match m {
            AzOptionFile::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionFile::None => None,
        }

    }
    fn read_to_string(&mut self) -> Option<String> {
        let m: AzOptionString = unsafe { mem::transmute(crate::AzFile_readToString(
            mem::transmute(self),
        )) };
        match m {
            AzOptionString::Some(s) => Some({ let s: AzString = unsafe { mem::transmute(s) }; s.into() }),
            AzOptionString::None => None,
        }

    }
    fn read_to_bytes(&mut self) -> Option<Vec<u8>> {
        let m: AzOptionU8Vec = unsafe { mem::transmute(crate::AzFile_readToBytes(
            mem::transmute(self),
        )) };
        match m {
            AzOptionU8Vec::Some(s) => Some({ let s: AzU8Vec = unsafe { mem::transmute(s) }; s.into() }),
            AzOptionU8Vec::None => None,
        }

    }
    fn write_string(&mut self, bytes: &str) -> bool {
        let bytes = pystring_to_refstr(&bytes);
        unsafe { mem::transmute(crate::AzFile_writeString(
            mem::transmute(self),
            mem::transmute(bytes),
        )) }
    }
    fn write_bytes(&mut self, bytes: Vec<u8>) -> bool {
        let bytes = pybytesref_to_vecu8_ref(&bytes);
        unsafe { mem::transmute(crate::AzFile_writeBytes(
            mem::transmute(self),
            mem::transmute(bytes),
        )) }
    }
    fn close(&mut self) -> () {
        unsafe { mem::transmute(crate::AzFile_close(
            mem::transmute(self),
        )) }
    }
}

#[pymethods]
impl AzMsgBox {
    #[staticmethod]
    fn ok(icon: AzMsgBoxIconEnumWrapper, title: String, message: String) -> bool {
        let title = pystring_to_azstring(&title);
        let message = pystring_to_azstring(&message);
        unsafe { mem::transmute(crate::AzMsgBox_ok(
            mem::transmute(icon),
            mem::transmute(title),
            mem::transmute(message),
        )) }
    }
    #[staticmethod]
    fn ok_cancel(icon: AzMsgBoxIconEnumWrapper, title: String, message: String, default_value: AzMsgBoxOkCancelEnumWrapper) -> AzMsgBoxOkCancelEnumWrapper {
        let title = pystring_to_azstring(&title);
        let message = pystring_to_azstring(&message);
        unsafe { mem::transmute(crate::AzMsgBox_okCancel(
            mem::transmute(icon),
            mem::transmute(title),
            mem::transmute(message),
            mem::transmute(default_value),
        )) }
    }
    #[staticmethod]
    fn yes_no(icon: AzMsgBoxIconEnumWrapper, title: String, message: String, default_value: AzMsgBoxYesNoEnumWrapper) -> AzMsgBoxYesNoEnumWrapper {
        let title = pystring_to_azstring(&title);
        let message = pystring_to_azstring(&message);
        unsafe { mem::transmute(crate::AzMsgBox_yesNo(
            mem::transmute(icon),
            mem::transmute(title),
            mem::transmute(message),
            mem::transmute(default_value),
        )) }
    }
}

#[pymethods]
impl AzMsgBoxIconEnumWrapper {
    #[classattr]
    fn Info() -> AzMsgBoxIconEnumWrapper { AzMsgBoxIconEnumWrapper { inner: AzMsgBoxIcon::Info } }
    #[classattr]
    fn Warning() -> AzMsgBoxIconEnumWrapper { AzMsgBoxIconEnumWrapper { inner: AzMsgBoxIcon::Warning } }
    #[classattr]
    fn Error() -> AzMsgBoxIconEnumWrapper { AzMsgBoxIconEnumWrapper { inner: AzMsgBoxIcon::Error } }
    #[classattr]
    fn Question() -> AzMsgBoxIconEnumWrapper { AzMsgBoxIconEnumWrapper { inner: AzMsgBoxIcon::Question } }
}

#[pymethods]
impl AzMsgBoxYesNoEnumWrapper {
    #[classattr]
    fn Yes() -> AzMsgBoxYesNoEnumWrapper { AzMsgBoxYesNoEnumWrapper { inner: AzMsgBoxYesNo::Yes } }
    #[classattr]
    fn No() -> AzMsgBoxYesNoEnumWrapper { AzMsgBoxYesNoEnumWrapper { inner: AzMsgBoxYesNo::No } }
}

#[pymethods]
impl AzMsgBoxOkCancelEnumWrapper {
    #[classattr]
    fn Ok() -> AzMsgBoxOkCancelEnumWrapper { AzMsgBoxOkCancelEnumWrapper { inner: AzMsgBoxOkCancel::Ok } }
    #[classattr]
    fn Cancel() -> AzMsgBoxOkCancelEnumWrapper { AzMsgBoxOkCancelEnumWrapper { inner: AzMsgBoxOkCancel::Cancel } }
}

#[pymethods]
impl AzFileDialog {
    #[staticmethod]
    fn select_file(title: String, default_path: AzOptionStringEnumWrapper, filter_list: AzOptionFileTypeListEnumWrapper) -> Option<String> {
        let title = pystring_to_azstring(&title);
        let m: AzOptionString = unsafe { mem::transmute(crate::AzFileDialog_selectFile(
            mem::transmute(title),
            mem::transmute(default_path),
            mem::transmute(filter_list),
        )) };
        match m {
            AzOptionString::Some(s) => Some({ let s: AzString = unsafe { mem::transmute(s) }; s.into() }),
            AzOptionString::None => None,
        }

    }
    #[staticmethod]
    fn select_multiple_files(title: String, default_path: AzOptionStringEnumWrapper, filter_list: AzOptionFileTypeListEnumWrapper) -> Option<AzStringVec> {
        let title = pystring_to_azstring(&title);
        let m: AzOptionStringVec = unsafe { mem::transmute(crate::AzFileDialog_selectMultipleFiles(
            mem::transmute(title),
            mem::transmute(default_path),
            mem::transmute(filter_list),
        )) };
        match m {
            AzOptionStringVec::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionStringVec::None => None,
        }

    }
    #[staticmethod]
    fn select_folder(title: String, default_path: AzOptionStringEnumWrapper) -> Option<String> {
        let title = pystring_to_azstring(&title);
        let m: AzOptionString = unsafe { mem::transmute(crate::AzFileDialog_selectFolder(
            mem::transmute(title),
            mem::transmute(default_path),
        )) };
        match m {
            AzOptionString::Some(s) => Some({ let s: AzString = unsafe { mem::transmute(s) }; s.into() }),
            AzOptionString::None => None,
        }

    }
    #[staticmethod]
    fn save_file(title: String, default_path: AzOptionStringEnumWrapper) -> Option<String> {
        let title = pystring_to_azstring(&title);
        let m: AzOptionString = unsafe { mem::transmute(crate::AzFileDialog_saveFile(
            mem::transmute(title),
            mem::transmute(default_path),
        )) };
        match m {
            AzOptionString::Some(s) => Some({ let s: AzString = unsafe { mem::transmute(s) }; s.into() }),
            AzOptionString::None => None,
        }

    }
}

#[pymethods]
impl AzColorPickerDialog {
    #[staticmethod]
    fn open(title: String, default_color: AzOptionColorUEnumWrapper) -> Option<AzColorU> {
        let title = pystring_to_azstring(&title);
        let m: AzOptionColorU = unsafe { mem::transmute(crate::AzColorPickerDialog_open(
            mem::transmute(title),
            mem::transmute(default_color),
        )) };
        match m {
            AzOptionColorU::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionColorU::None => None,
        }

    }
}

#[pymethods]
impl AzSystemClipboard {
    #[staticmethod]
    fn new() -> Option<AzSystemClipboard> {
        let m: AzOptionSystemClipboard = unsafe { mem::transmute(crate::AzSystemClipboard_new()) };
        match m {
            AzOptionSystemClipboard::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionSystemClipboard::None => None,
        }

    }
    fn get_string_contents(&self) -> Option<String> {
        let m: AzOptionString = unsafe { mem::transmute(crate::AzSystemClipboard_getStringContents(
            mem::transmute(self),
        )) };
        match m {
            AzOptionString::Some(s) => Some({ let s: AzString = unsafe { mem::transmute(s) }; s.into() }),
            AzOptionString::None => None,
        }

    }
    fn set_string_contents(&mut self, contents: String) -> bool {
        let contents = pystring_to_azstring(&contents);
        unsafe { mem::transmute(crate::AzSystemClipboard_setStringContents(
            mem::transmute(self),
            mem::transmute(contents),
        )) }
    }
}

#[pymethods]
impl AzInstantEnumWrapper {
    #[staticmethod]
    fn System(v: AzInstantPtr) -> AzInstantEnumWrapper { AzInstantEnumWrapper { inner: AzInstant::System(v) } }
    #[staticmethod]
    fn Tick(v: AzSystemTick) -> AzInstantEnumWrapper { AzInstantEnumWrapper { inner: AzInstant::Tick(v) } }
}

#[pymethods]
impl AzDurationEnumWrapper {
    #[staticmethod]
    fn System(v: AzSystemTimeDiff) -> AzDurationEnumWrapper { AzDurationEnumWrapper { inner: AzDuration::System(v) } }
    #[staticmethod]
    fn Tick(v: AzSystemTickDiff) -> AzDurationEnumWrapper { AzDurationEnumWrapper { inner: AzDuration::Tick(v) } }
}

#[pymethods]
impl AzTimer {
    fn with_delay(&self, delay: AzDurationEnumWrapper) -> AzTimer {
        unsafe { mem::transmute(crate::AzTimer_withDelay(
            mem::transmute(self),
            mem::transmute(delay),
        )) }
    }
    fn with_interval(&self, interval: AzDurationEnumWrapper) -> AzTimer {
        unsafe { mem::transmute(crate::AzTimer_withInterval(
            mem::transmute(self),
            mem::transmute(interval),
        )) }
    }
    fn with_timeout(&self, timeout: AzDurationEnumWrapper) -> AzTimer {
        unsafe { mem::transmute(crate::AzTimer_withTimeout(
            mem::transmute(self),
            mem::transmute(timeout),
        )) }
    }
}

#[pymethods]
impl AzTerminateTimerEnumWrapper {
    #[classattr]
    fn Terminate() -> AzTerminateTimerEnumWrapper { AzTerminateTimerEnumWrapper { inner: AzTerminateTimer::Terminate } }
    #[classattr]
    fn Continue() -> AzTerminateTimerEnumWrapper { AzTerminateTimerEnumWrapper { inner: AzTerminateTimer::Continue } }
}

#[pymethods]
impl AzThreadSender {
    fn send(&mut self, msg: AzThreadReceiveMsgEnumWrapper) -> bool {
        unsafe { mem::transmute(crate::AzThreadSender_send(
            mem::transmute(self),
            mem::transmute(msg),
        )) }
    }
}

#[pymethods]
impl AzThreadReceiver {
    fn receive(&mut self) -> Option<AzThreadSendMsgEnumWrapper> {
        let m: AzOptionThreadSendMsg = unsafe { mem::transmute(crate::AzThreadReceiver_receive(
            mem::transmute(self),
        )) };
        match m {
            AzOptionThreadSendMsg::Some(s) => Some(unsafe { mem::transmute(s) }),
            AzOptionThreadSendMsg::None => None,
        }

    }
}

#[pymethods]
impl AzThreadSendMsgEnumWrapper {
    #[classattr]
    fn TerminateThread() -> AzThreadSendMsgEnumWrapper { AzThreadSendMsgEnumWrapper { inner: AzThreadSendMsg::TerminateThread } }
    #[classattr]
    fn Tick() -> AzThreadSendMsgEnumWrapper { AzThreadSendMsgEnumWrapper { inner: AzThreadSendMsg::Tick } }
    #[staticmethod]
    fn Custom(v: AzRefAny) -> AzThreadSendMsgEnumWrapper { AzThreadSendMsgEnumWrapper { inner: AzThreadSendMsg::Custom(v) } }
}

#[pymethods]
impl AzThreadReceiveMsgEnumWrapper {
    #[staticmethod]
    fn WriteBack(v: AzThreadWriteBackMsg) -> AzThreadReceiveMsgEnumWrapper { AzThreadReceiveMsgEnumWrapper { inner: AzThreadReceiveMsg::WriteBack(v) } }
    #[staticmethod]
    fn Update(v: AzUpdateEnumWrapper) -> AzThreadReceiveMsgEnumWrapper { AzThreadReceiveMsgEnumWrapper { inner: AzThreadReceiveMsg::Update(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzFmtValueEnumWrapper {
    #[staticmethod]
    fn Bool(v: bool) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Bool(v) } }
    #[staticmethod]
    fn Uchar(v: u8) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Uchar(v) } }
    #[staticmethod]
    fn Schar(v: i8) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Schar(v) } }
    #[staticmethod]
    fn Ushort(v: u16) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Ushort(v) } }
    #[staticmethod]
    fn Sshort(v: i16) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Sshort(v) } }
    #[staticmethod]
    fn Uint(v: u32) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Uint(v) } }
    #[staticmethod]
    fn Sint(v: i32) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Sint(v) } }
    #[staticmethod]
    fn Ulong(v: u64) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Ulong(v) } }
    #[staticmethod]
    fn Slong(v: i64) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Slong(v) } }
    #[staticmethod]
    fn Isize(v: isize) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Isize(v) } }
    #[staticmethod]
    fn Usize(v: usize) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Usize(v) } }
    #[staticmethod]
    fn Float(v: f32) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Float(v) } }
    #[staticmethod]
    fn Double(v: f64) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Double(v) } }
    #[staticmethod]
    fn Str(v: AzString) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::Str(v) } }
    #[staticmethod]
    fn StrVec(v: AzStringVec) -> AzFmtValueEnumWrapper { AzFmtValueEnumWrapper { inner: AzFmtValue::StrVec(v) } }
}

#[pymethods]
impl AzString {
    #[staticmethod]
    fn format(format: String, args: AzFmtArgVec) -> AzString {
        let format = pystring_to_azstring(&format);
        unsafe { mem::transmute(crate::AzString_format(
            mem::transmute(format),
            mem::transmute(args),
        )) }
    }
    fn trim(&self) -> String {
        az_string_to_py_string(unsafe { mem::transmute(crate::AzString_trim(
            mem::transmute(self),
        )) })
    }
}

#[pymethods]
impl AzTesselatedSvgNodeVec {
}

#[pymethods]
impl AzU8Vec {
}

#[pymethods]
impl AzStyleFontFamilyVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzStyleFontFamilyVecDestructorEnumWrapper { AzStyleFontFamilyVecDestructorEnumWrapper { inner: AzStyleFontFamilyVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzStyleFontFamilyVecDestructorEnumWrapper { AzStyleFontFamilyVecDestructorEnumWrapper { inner: AzStyleFontFamilyVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzTesselatedSvgNodeVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzTesselatedSvgNodeVecDestructorEnumWrapper { AzTesselatedSvgNodeVecDestructorEnumWrapper { inner: AzTesselatedSvgNodeVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzTesselatedSvgNodeVecDestructorEnumWrapper { AzTesselatedSvgNodeVecDestructorEnumWrapper { inner: AzTesselatedSvgNodeVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzXmlNodeVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzXmlNodeVecDestructorEnumWrapper { AzXmlNodeVecDestructorEnumWrapper { inner: AzXmlNodeVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzXmlNodeVecDestructorEnumWrapper { AzXmlNodeVecDestructorEnumWrapper { inner: AzXmlNodeVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzFmtArgVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzFmtArgVecDestructorEnumWrapper { AzFmtArgVecDestructorEnumWrapper { inner: AzFmtArgVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzFmtArgVecDestructorEnumWrapper { AzFmtArgVecDestructorEnumWrapper { inner: AzFmtArgVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzInlineLineVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzInlineLineVecDestructorEnumWrapper { AzInlineLineVecDestructorEnumWrapper { inner: AzInlineLineVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzInlineLineVecDestructorEnumWrapper { AzInlineLineVecDestructorEnumWrapper { inner: AzInlineLineVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzInlineWordVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzInlineWordVecDestructorEnumWrapper { AzInlineWordVecDestructorEnumWrapper { inner: AzInlineWordVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzInlineWordVecDestructorEnumWrapper { AzInlineWordVecDestructorEnumWrapper { inner: AzInlineWordVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzInlineGlyphVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzInlineGlyphVecDestructorEnumWrapper { AzInlineGlyphVecDestructorEnumWrapper { inner: AzInlineGlyphVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzInlineGlyphVecDestructorEnumWrapper { AzInlineGlyphVecDestructorEnumWrapper { inner: AzInlineGlyphVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzInlineTextHitVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzInlineTextHitVecDestructorEnumWrapper { AzInlineTextHitVecDestructorEnumWrapper { inner: AzInlineTextHitVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzInlineTextHitVecDestructorEnumWrapper { AzInlineTextHitVecDestructorEnumWrapper { inner: AzInlineTextHitVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzMonitorVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzMonitorVecDestructorEnumWrapper { AzMonitorVecDestructorEnumWrapper { inner: AzMonitorVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzMonitorVecDestructorEnumWrapper { AzMonitorVecDestructorEnumWrapper { inner: AzMonitorVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzVideoModeVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzVideoModeVecDestructorEnumWrapper { AzVideoModeVecDestructorEnumWrapper { inner: AzVideoModeVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzVideoModeVecDestructorEnumWrapper { AzVideoModeVecDestructorEnumWrapper { inner: AzVideoModeVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzDomVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzDomVecDestructorEnumWrapper { AzDomVecDestructorEnumWrapper { inner: AzDomVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzDomVecDestructorEnumWrapper { AzDomVecDestructorEnumWrapper { inner: AzDomVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzIdOrClassVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzIdOrClassVecDestructorEnumWrapper { AzIdOrClassVecDestructorEnumWrapper { inner: AzIdOrClassVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzIdOrClassVecDestructorEnumWrapper { AzIdOrClassVecDestructorEnumWrapper { inner: AzIdOrClassVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzNodeDataInlineCssPropertyVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzNodeDataInlineCssPropertyVecDestructorEnumWrapper { AzNodeDataInlineCssPropertyVecDestructorEnumWrapper { inner: AzNodeDataInlineCssPropertyVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzNodeDataInlineCssPropertyVecDestructorEnumWrapper { AzNodeDataInlineCssPropertyVecDestructorEnumWrapper { inner: AzNodeDataInlineCssPropertyVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzStyleBackgroundContentVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzStyleBackgroundContentVecDestructorEnumWrapper { AzStyleBackgroundContentVecDestructorEnumWrapper { inner: AzStyleBackgroundContentVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzStyleBackgroundContentVecDestructorEnumWrapper { AzStyleBackgroundContentVecDestructorEnumWrapper { inner: AzStyleBackgroundContentVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzStyleBackgroundPositionVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzStyleBackgroundPositionVecDestructorEnumWrapper { AzStyleBackgroundPositionVecDestructorEnumWrapper { inner: AzStyleBackgroundPositionVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzStyleBackgroundPositionVecDestructorEnumWrapper { AzStyleBackgroundPositionVecDestructorEnumWrapper { inner: AzStyleBackgroundPositionVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzStyleBackgroundRepeatVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzStyleBackgroundRepeatVecDestructorEnumWrapper { AzStyleBackgroundRepeatVecDestructorEnumWrapper { inner: AzStyleBackgroundRepeatVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzStyleBackgroundRepeatVecDestructorEnumWrapper { AzStyleBackgroundRepeatVecDestructorEnumWrapper { inner: AzStyleBackgroundRepeatVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzStyleBackgroundSizeVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzStyleBackgroundSizeVecDestructorEnumWrapper { AzStyleBackgroundSizeVecDestructorEnumWrapper { inner: AzStyleBackgroundSizeVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzStyleBackgroundSizeVecDestructorEnumWrapper { AzStyleBackgroundSizeVecDestructorEnumWrapper { inner: AzStyleBackgroundSizeVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzStyleTransformVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzStyleTransformVecDestructorEnumWrapper { AzStyleTransformVecDestructorEnumWrapper { inner: AzStyleTransformVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzStyleTransformVecDestructorEnumWrapper { AzStyleTransformVecDestructorEnumWrapper { inner: AzStyleTransformVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzCssPropertyVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzCssPropertyVecDestructorEnumWrapper { AzCssPropertyVecDestructorEnumWrapper { inner: AzCssPropertyVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzCssPropertyVecDestructorEnumWrapper { AzCssPropertyVecDestructorEnumWrapper { inner: AzCssPropertyVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzSvgMultiPolygonVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzSvgMultiPolygonVecDestructorEnumWrapper { AzSvgMultiPolygonVecDestructorEnumWrapper { inner: AzSvgMultiPolygonVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzSvgMultiPolygonVecDestructorEnumWrapper { AzSvgMultiPolygonVecDestructorEnumWrapper { inner: AzSvgMultiPolygonVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzSvgPathVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzSvgPathVecDestructorEnumWrapper { AzSvgPathVecDestructorEnumWrapper { inner: AzSvgPathVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzSvgPathVecDestructorEnumWrapper { AzSvgPathVecDestructorEnumWrapper { inner: AzSvgPathVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzVertexAttributeVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzVertexAttributeVecDestructorEnumWrapper { AzVertexAttributeVecDestructorEnumWrapper { inner: AzVertexAttributeVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzVertexAttributeVecDestructorEnumWrapper { AzVertexAttributeVecDestructorEnumWrapper { inner: AzVertexAttributeVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzSvgPathElementVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzSvgPathElementVecDestructorEnumWrapper { AzSvgPathElementVecDestructorEnumWrapper { inner: AzSvgPathElementVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzSvgPathElementVecDestructorEnumWrapper { AzSvgPathElementVecDestructorEnumWrapper { inner: AzSvgPathElementVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzSvgVertexVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzSvgVertexVecDestructorEnumWrapper { AzSvgVertexVecDestructorEnumWrapper { inner: AzSvgVertexVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzSvgVertexVecDestructorEnumWrapper { AzSvgVertexVecDestructorEnumWrapper { inner: AzSvgVertexVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzU32VecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzU32VecDestructorEnumWrapper { AzU32VecDestructorEnumWrapper { inner: AzU32VecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzU32VecDestructorEnumWrapper { AzU32VecDestructorEnumWrapper { inner: AzU32VecDestructor::NoDestructor } }
}

#[pymethods]
impl AzXWindowTypeVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzXWindowTypeVecDestructorEnumWrapper { AzXWindowTypeVecDestructorEnumWrapper { inner: AzXWindowTypeVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzXWindowTypeVecDestructorEnumWrapper { AzXWindowTypeVecDestructorEnumWrapper { inner: AzXWindowTypeVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzVirtualKeyCodeVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzVirtualKeyCodeVecDestructorEnumWrapper { AzVirtualKeyCodeVecDestructorEnumWrapper { inner: AzVirtualKeyCodeVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzVirtualKeyCodeVecDestructorEnumWrapper { AzVirtualKeyCodeVecDestructorEnumWrapper { inner: AzVirtualKeyCodeVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzCascadeInfoVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzCascadeInfoVecDestructorEnumWrapper { AzCascadeInfoVecDestructorEnumWrapper { inner: AzCascadeInfoVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzCascadeInfoVecDestructorEnumWrapper { AzCascadeInfoVecDestructorEnumWrapper { inner: AzCascadeInfoVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzScanCodeVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzScanCodeVecDestructorEnumWrapper { AzScanCodeVecDestructorEnumWrapper { inner: AzScanCodeVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzScanCodeVecDestructorEnumWrapper { AzScanCodeVecDestructorEnumWrapper { inner: AzScanCodeVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzCssDeclarationVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzCssDeclarationVecDestructorEnumWrapper { AzCssDeclarationVecDestructorEnumWrapper { inner: AzCssDeclarationVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzCssDeclarationVecDestructorEnumWrapper { AzCssDeclarationVecDestructorEnumWrapper { inner: AzCssDeclarationVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzCssPathSelectorVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzCssPathSelectorVecDestructorEnumWrapper { AzCssPathSelectorVecDestructorEnumWrapper { inner: AzCssPathSelectorVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzCssPathSelectorVecDestructorEnumWrapper { AzCssPathSelectorVecDestructorEnumWrapper { inner: AzCssPathSelectorVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzStylesheetVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzStylesheetVecDestructorEnumWrapper { AzStylesheetVecDestructorEnumWrapper { inner: AzStylesheetVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzStylesheetVecDestructorEnumWrapper { AzStylesheetVecDestructorEnumWrapper { inner: AzStylesheetVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzCssRuleBlockVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzCssRuleBlockVecDestructorEnumWrapper { AzCssRuleBlockVecDestructorEnumWrapper { inner: AzCssRuleBlockVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzCssRuleBlockVecDestructorEnumWrapper { AzCssRuleBlockVecDestructorEnumWrapper { inner: AzCssRuleBlockVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzF32VecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzF32VecDestructorEnumWrapper { AzF32VecDestructorEnumWrapper { inner: AzF32VecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzF32VecDestructorEnumWrapper { AzF32VecDestructorEnumWrapper { inner: AzF32VecDestructor::NoDestructor } }
}

#[pymethods]
impl AzU16VecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzU16VecDestructorEnumWrapper { AzU16VecDestructorEnumWrapper { inner: AzU16VecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzU16VecDestructorEnumWrapper { AzU16VecDestructorEnumWrapper { inner: AzU16VecDestructor::NoDestructor } }
}

#[pymethods]
impl AzU8VecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzU8VecDestructorEnumWrapper { AzU8VecDestructorEnumWrapper { inner: AzU8VecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzU8VecDestructorEnumWrapper { AzU8VecDestructorEnumWrapper { inner: AzU8VecDestructor::NoDestructor } }
}

#[pymethods]
impl AzCallbackDataVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzCallbackDataVecDestructorEnumWrapper { AzCallbackDataVecDestructorEnumWrapper { inner: AzCallbackDataVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzCallbackDataVecDestructorEnumWrapper { AzCallbackDataVecDestructorEnumWrapper { inner: AzCallbackDataVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzDebugMessageVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzDebugMessageVecDestructorEnumWrapper { AzDebugMessageVecDestructorEnumWrapper { inner: AzDebugMessageVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzDebugMessageVecDestructorEnumWrapper { AzDebugMessageVecDestructorEnumWrapper { inner: AzDebugMessageVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzGLuintVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzGLuintVecDestructorEnumWrapper { AzGLuintVecDestructorEnumWrapper { inner: AzGLuintVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzGLuintVecDestructorEnumWrapper { AzGLuintVecDestructorEnumWrapper { inner: AzGLuintVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzGLintVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzGLintVecDestructorEnumWrapper { AzGLintVecDestructorEnumWrapper { inner: AzGLintVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzGLintVecDestructorEnumWrapper { AzGLintVecDestructorEnumWrapper { inner: AzGLintVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzStringVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzStringVecDestructorEnumWrapper { AzStringVecDestructorEnumWrapper { inner: AzStringVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzStringVecDestructorEnumWrapper { AzStringVecDestructorEnumWrapper { inner: AzStringVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzStringPairVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzStringPairVecDestructorEnumWrapper { AzStringPairVecDestructorEnumWrapper { inner: AzStringPairVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzStringPairVecDestructorEnumWrapper { AzStringPairVecDestructorEnumWrapper { inner: AzStringPairVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzNormalizedLinearColorStopVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzNormalizedLinearColorStopVecDestructorEnumWrapper { AzNormalizedLinearColorStopVecDestructorEnumWrapper { inner: AzNormalizedLinearColorStopVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzNormalizedLinearColorStopVecDestructorEnumWrapper { AzNormalizedLinearColorStopVecDestructorEnumWrapper { inner: AzNormalizedLinearColorStopVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzNormalizedRadialColorStopVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzNormalizedRadialColorStopVecDestructorEnumWrapper { AzNormalizedRadialColorStopVecDestructorEnumWrapper { inner: AzNormalizedRadialColorStopVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzNormalizedRadialColorStopVecDestructorEnumWrapper { AzNormalizedRadialColorStopVecDestructorEnumWrapper { inner: AzNormalizedRadialColorStopVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzNodeIdVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzNodeIdVecDestructorEnumWrapper { AzNodeIdVecDestructorEnumWrapper { inner: AzNodeIdVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzNodeIdVecDestructorEnumWrapper { AzNodeIdVecDestructorEnumWrapper { inner: AzNodeIdVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzNodeVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzNodeVecDestructorEnumWrapper { AzNodeVecDestructorEnumWrapper { inner: AzNodeVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzNodeVecDestructorEnumWrapper { AzNodeVecDestructorEnumWrapper { inner: AzNodeVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzStyledNodeVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzStyledNodeVecDestructorEnumWrapper { AzStyledNodeVecDestructorEnumWrapper { inner: AzStyledNodeVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzStyledNodeVecDestructorEnumWrapper { AzStyledNodeVecDestructorEnumWrapper { inner: AzStyledNodeVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzTagIdsToNodeIdsMappingVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzTagIdsToNodeIdsMappingVecDestructorEnumWrapper { AzTagIdsToNodeIdsMappingVecDestructorEnumWrapper { inner: AzTagIdsToNodeIdsMappingVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzTagIdsToNodeIdsMappingVecDestructorEnumWrapper { AzTagIdsToNodeIdsMappingVecDestructorEnumWrapper { inner: AzTagIdsToNodeIdsMappingVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzParentWithNodeDepthVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzParentWithNodeDepthVecDestructorEnumWrapper { AzParentWithNodeDepthVecDestructorEnumWrapper { inner: AzParentWithNodeDepthVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzParentWithNodeDepthVecDestructorEnumWrapper { AzParentWithNodeDepthVecDestructorEnumWrapper { inner: AzParentWithNodeDepthVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzNodeDataVecDestructorEnumWrapper {
    #[classattr]
    fn DefaultRust() -> AzNodeDataVecDestructorEnumWrapper { AzNodeDataVecDestructorEnumWrapper { inner: AzNodeDataVecDestructor::DefaultRust } }
    #[classattr]
    fn NoDestructor() -> AzNodeDataVecDestructorEnumWrapper { AzNodeDataVecDestructorEnumWrapper { inner: AzNodeDataVecDestructor::NoDestructor } }
}

#[pymethods]
impl AzOptionCssPropertyEnumWrapper {
    #[classattr]
    fn None() -> AzOptionCssPropertyEnumWrapper { AzOptionCssPropertyEnumWrapper { inner: AzOptionCssProperty::None } }
    #[staticmethod]
    fn Some(v: AzCssPropertyEnumWrapper) -> AzOptionCssPropertyEnumWrapper { AzOptionCssPropertyEnumWrapper { inner: AzOptionCssProperty::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionPositionInfoEnumWrapper {
    #[classattr]
    fn None() -> AzOptionPositionInfoEnumWrapper { AzOptionPositionInfoEnumWrapper { inner: AzOptionPositionInfo::None } }
    #[staticmethod]
    fn Some(v: AzPositionInfoEnumWrapper) -> AzOptionPositionInfoEnumWrapper { AzOptionPositionInfoEnumWrapper { inner: AzOptionPositionInfo::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionTimerIdEnumWrapper {
    #[classattr]
    fn None() -> AzOptionTimerIdEnumWrapper { AzOptionTimerIdEnumWrapper { inner: AzOptionTimerId::None } }
    #[staticmethod]
    fn Some(v: AzTimerId) -> AzOptionTimerIdEnumWrapper { AzOptionTimerIdEnumWrapper { inner: AzOptionTimerId::Some(v) } }
}

#[pymethods]
impl AzOptionThreadIdEnumWrapper {
    #[classattr]
    fn None() -> AzOptionThreadIdEnumWrapper { AzOptionThreadIdEnumWrapper { inner: AzOptionThreadId::None } }
    #[staticmethod]
    fn Some(v: AzThreadId) -> AzOptionThreadIdEnumWrapper { AzOptionThreadIdEnumWrapper { inner: AzOptionThreadId::Some(v) } }
}

#[pymethods]
impl AzOptionI16EnumWrapper {
    #[classattr]
    fn None() -> AzOptionI16EnumWrapper { AzOptionI16EnumWrapper { inner: AzOptionI16::None } }
    #[staticmethod]
    fn Some(v: i16) -> AzOptionI16EnumWrapper { AzOptionI16EnumWrapper { inner: AzOptionI16::Some(v) } }
}

#[pymethods]
impl AzOptionU16EnumWrapper {
    #[classattr]
    fn None() -> AzOptionU16EnumWrapper { AzOptionU16EnumWrapper { inner: AzOptionU16::None } }
    #[staticmethod]
    fn Some(v: u16) -> AzOptionU16EnumWrapper { AzOptionU16EnumWrapper { inner: AzOptionU16::Some(v) } }
}

#[pymethods]
impl AzOptionU32EnumWrapper {
    #[classattr]
    fn None() -> AzOptionU32EnumWrapper { AzOptionU32EnumWrapper { inner: AzOptionU32::None } }
    #[staticmethod]
    fn Some(v: u32) -> AzOptionU32EnumWrapper { AzOptionU32EnumWrapper { inner: AzOptionU32::Some(v) } }
}

#[pymethods]
impl AzOptionImageRefEnumWrapper {
    #[classattr]
    fn None() -> AzOptionImageRefEnumWrapper { AzOptionImageRefEnumWrapper { inner: AzOptionImageRef::None } }
    #[staticmethod]
    fn Some(v: AzImageRef) -> AzOptionImageRefEnumWrapper { AzOptionImageRefEnumWrapper { inner: AzOptionImageRef::Some(v) } }
}

#[pymethods]
impl AzOptionFontRefEnumWrapper {
    #[classattr]
    fn None() -> AzOptionFontRefEnumWrapper { AzOptionFontRefEnumWrapper { inner: AzOptionFontRef::None } }
    #[staticmethod]
    fn Some(v: AzFontRef) -> AzOptionFontRefEnumWrapper { AzOptionFontRefEnumWrapper { inner: AzOptionFontRef::Some(v) } }
}

#[pymethods]
impl AzOptionSystemClipboardEnumWrapper {
    #[classattr]
    fn None() -> AzOptionSystemClipboardEnumWrapper { AzOptionSystemClipboardEnumWrapper { inner: AzOptionSystemClipboard::None } }
    #[staticmethod]
    fn Some(v: AzSystemClipboard) -> AzOptionSystemClipboardEnumWrapper { AzOptionSystemClipboardEnumWrapper { inner: AzOptionSystemClipboard::Some(v) } }
}

#[pymethods]
impl AzOptionFileTypeListEnumWrapper {
    #[classattr]
    fn None() -> AzOptionFileTypeListEnumWrapper { AzOptionFileTypeListEnumWrapper { inner: AzOptionFileTypeList::None } }
    #[staticmethod]
    fn Some(v: AzFileTypeList) -> AzOptionFileTypeListEnumWrapper { AzOptionFileTypeListEnumWrapper { inner: AzOptionFileTypeList::Some(v) } }
}

#[pymethods]
impl AzOptionWindowStateEnumWrapper {
    #[classattr]
    fn None() -> AzOptionWindowStateEnumWrapper { AzOptionWindowStateEnumWrapper { inner: AzOptionWindowState::None } }
    #[staticmethod]
    fn Some(v: AzWindowState) -> AzOptionWindowStateEnumWrapper { AzOptionWindowStateEnumWrapper { inner: AzOptionWindowState::Some(v) } }
}

#[pymethods]
impl AzOptionMouseStateEnumWrapper {
    #[classattr]
    fn None() -> AzOptionMouseStateEnumWrapper { AzOptionMouseStateEnumWrapper { inner: AzOptionMouseState::None } }
    #[staticmethod]
    fn Some(v: AzMouseState) -> AzOptionMouseStateEnumWrapper { AzOptionMouseStateEnumWrapper { inner: AzOptionMouseState::Some(v) } }
}

#[pymethods]
impl AzOptionKeyboardStateEnumWrapper {
    #[classattr]
    fn None() -> AzOptionKeyboardStateEnumWrapper { AzOptionKeyboardStateEnumWrapper { inner: AzOptionKeyboardState::None } }
    #[staticmethod]
    fn Some(v: AzKeyboardState) -> AzOptionKeyboardStateEnumWrapper { AzOptionKeyboardStateEnumWrapper { inner: AzOptionKeyboardState::Some(v) } }
}

#[pymethods]
impl AzOptionStringVecEnumWrapper {
    #[classattr]
    fn None() -> AzOptionStringVecEnumWrapper { AzOptionStringVecEnumWrapper { inner: AzOptionStringVec::None } }
    #[staticmethod]
    fn Some(v: AzStringVec) -> AzOptionStringVecEnumWrapper { AzOptionStringVecEnumWrapper { inner: AzOptionStringVec::Some(v) } }
}

#[pymethods]
impl AzOptionFileEnumWrapper {
    #[classattr]
    fn None() -> AzOptionFileEnumWrapper { AzOptionFileEnumWrapper { inner: AzOptionFile::None } }
    #[staticmethod]
    fn Some(v: AzFile) -> AzOptionFileEnumWrapper { AzOptionFileEnumWrapper { inner: AzOptionFile::Some(v) } }
}

#[pymethods]
impl AzOptionGlEnumWrapper {
    #[classattr]
    fn None() -> AzOptionGlEnumWrapper { AzOptionGlEnumWrapper { inner: AzOptionGl::None } }
    #[staticmethod]
    fn Some(v: AzGl) -> AzOptionGlEnumWrapper { AzOptionGlEnumWrapper { inner: AzOptionGl::Some(v) } }
}

#[pymethods]
impl AzOptionThreadReceiveMsgEnumWrapper {
    #[classattr]
    fn None() -> AzOptionThreadReceiveMsgEnumWrapper { AzOptionThreadReceiveMsgEnumWrapper { inner: AzOptionThreadReceiveMsg::None } }
    #[staticmethod]
    fn Some(v: AzThreadReceiveMsgEnumWrapper) -> AzOptionThreadReceiveMsgEnumWrapper { AzOptionThreadReceiveMsgEnumWrapper { inner: AzOptionThreadReceiveMsg::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionPercentageValueEnumWrapper {
    #[classattr]
    fn None() -> AzOptionPercentageValueEnumWrapper { AzOptionPercentageValueEnumWrapper { inner: AzOptionPercentageValue::None } }
    #[staticmethod]
    fn Some(v: AzPercentageValue) -> AzOptionPercentageValueEnumWrapper { AzOptionPercentageValueEnumWrapper { inner: AzOptionPercentageValue::Some(v) } }
}

#[pymethods]
impl AzOptionAngleValueEnumWrapper {
    #[classattr]
    fn None() -> AzOptionAngleValueEnumWrapper { AzOptionAngleValueEnumWrapper { inner: AzOptionAngleValue::None } }
    #[staticmethod]
    fn Some(v: AzAngleValue) -> AzOptionAngleValueEnumWrapper { AzOptionAngleValueEnumWrapper { inner: AzOptionAngleValue::Some(v) } }
}

#[pymethods]
impl AzOptionRendererOptionsEnumWrapper {
    #[classattr]
    fn None() -> AzOptionRendererOptionsEnumWrapper { AzOptionRendererOptionsEnumWrapper { inner: AzOptionRendererOptions::None } }
    #[staticmethod]
    fn Some(v: AzRendererOptions) -> AzOptionRendererOptionsEnumWrapper { AzOptionRendererOptionsEnumWrapper { inner: AzOptionRendererOptions::Some(v) } }
}

#[pymethods]
impl AzOptionCallbackEnumWrapper {
    #[classattr]
    fn None() -> AzOptionCallbackEnumWrapper { AzOptionCallbackEnumWrapper { inner: AzOptionCallback::None } }
    #[staticmethod]
    fn Some(v: AzCallback) -> AzOptionCallbackEnumWrapper { AzOptionCallbackEnumWrapper { inner: AzOptionCallback::Some(v) } }
}

#[pymethods]
impl AzOptionThreadSendMsgEnumWrapper {
    #[classattr]
    fn None() -> AzOptionThreadSendMsgEnumWrapper { AzOptionThreadSendMsgEnumWrapper { inner: AzOptionThreadSendMsg::None } }
    #[staticmethod]
    fn Some(v: AzThreadSendMsgEnumWrapper) -> AzOptionThreadSendMsgEnumWrapper { AzOptionThreadSendMsgEnumWrapper { inner: AzOptionThreadSendMsg::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionLayoutRectEnumWrapper {
    #[classattr]
    fn None() -> AzOptionLayoutRectEnumWrapper { AzOptionLayoutRectEnumWrapper { inner: AzOptionLayoutRect::None } }
    #[staticmethod]
    fn Some(v: AzLayoutRect) -> AzOptionLayoutRectEnumWrapper { AzOptionLayoutRectEnumWrapper { inner: AzOptionLayoutRect::Some(v) } }
}

#[pymethods]
impl AzOptionRefAnyEnumWrapper {
    #[classattr]
    fn None() -> AzOptionRefAnyEnumWrapper { AzOptionRefAnyEnumWrapper { inner: AzOptionRefAny::None } }
    #[staticmethod]
    fn Some(v: AzRefAny) -> AzOptionRefAnyEnumWrapper { AzOptionRefAnyEnumWrapper { inner: AzOptionRefAny::Some(v) } }
}

#[pymethods]
impl AzOptionInlineTextEnumWrapper {
    #[classattr]
    fn None() -> AzOptionInlineTextEnumWrapper { AzOptionInlineTextEnumWrapper { inner: AzOptionInlineText::None } }
    #[staticmethod]
    fn Some(v: AzInlineText) -> AzOptionInlineTextEnumWrapper { AzOptionInlineTextEnumWrapper { inner: AzOptionInlineText::Some(v) } }
}

#[pymethods]
impl AzOptionLayoutPointEnumWrapper {
    #[classattr]
    fn None() -> AzOptionLayoutPointEnumWrapper { AzOptionLayoutPointEnumWrapper { inner: AzOptionLayoutPoint::None } }
    #[staticmethod]
    fn Some(v: AzLayoutPoint) -> AzOptionLayoutPointEnumWrapper { AzOptionLayoutPointEnumWrapper { inner: AzOptionLayoutPoint::Some(v) } }
}

#[pymethods]
impl AzOptionLayoutSizeEnumWrapper {
    #[classattr]
    fn None() -> AzOptionLayoutSizeEnumWrapper { AzOptionLayoutSizeEnumWrapper { inner: AzOptionLayoutSize::None } }
    #[staticmethod]
    fn Some(v: AzLayoutSize) -> AzOptionLayoutSizeEnumWrapper { AzOptionLayoutSizeEnumWrapper { inner: AzOptionLayoutSize::Some(v) } }
}

#[pymethods]
impl AzOptionWindowThemeEnumWrapper {
    #[classattr]
    fn None() -> AzOptionWindowThemeEnumWrapper { AzOptionWindowThemeEnumWrapper { inner: AzOptionWindowTheme::None } }
    #[staticmethod]
    fn Some(v: AzWindowThemeEnumWrapper) -> AzOptionWindowThemeEnumWrapper { AzOptionWindowThemeEnumWrapper { inner: AzOptionWindowTheme::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionNodeIdEnumWrapper {
    #[classattr]
    fn None() -> AzOptionNodeIdEnumWrapper { AzOptionNodeIdEnumWrapper { inner: AzOptionNodeId::None } }
    #[staticmethod]
    fn Some(v: AzNodeId) -> AzOptionNodeIdEnumWrapper { AzOptionNodeIdEnumWrapper { inner: AzOptionNodeId::Some(v) } }
}

#[pymethods]
impl AzOptionDomNodeIdEnumWrapper {
    #[classattr]
    fn None() -> AzOptionDomNodeIdEnumWrapper { AzOptionDomNodeIdEnumWrapper { inner: AzOptionDomNodeId::None } }
    #[staticmethod]
    fn Some(v: AzDomNodeId) -> AzOptionDomNodeIdEnumWrapper { AzOptionDomNodeIdEnumWrapper { inner: AzOptionDomNodeId::Some(v) } }
}

#[pymethods]
impl AzOptionColorUEnumWrapper {
    #[classattr]
    fn None() -> AzOptionColorUEnumWrapper { AzOptionColorUEnumWrapper { inner: AzOptionColorU::None } }
    #[staticmethod]
    fn Some(v: AzColorU) -> AzOptionColorUEnumWrapper { AzOptionColorUEnumWrapper { inner: AzOptionColorU::Some(v) } }
}

#[pymethods]
impl AzOptionRawImageEnumWrapper {
    #[classattr]
    fn None() -> AzOptionRawImageEnumWrapper { AzOptionRawImageEnumWrapper { inner: AzOptionRawImage::None } }
    #[staticmethod]
    fn Some(v: AzRawImage) -> AzOptionRawImageEnumWrapper { AzOptionRawImageEnumWrapper { inner: AzOptionRawImage::Some(v) } }
}

#[pymethods]
impl AzOptionSvgDashPatternEnumWrapper {
    #[classattr]
    fn None() -> AzOptionSvgDashPatternEnumWrapper { AzOptionSvgDashPatternEnumWrapper { inner: AzOptionSvgDashPattern::None } }
    #[staticmethod]
    fn Some(v: AzSvgDashPattern) -> AzOptionSvgDashPatternEnumWrapper { AzOptionSvgDashPatternEnumWrapper { inner: AzOptionSvgDashPattern::Some(v) } }
}

#[pymethods]
impl AzOptionWaylandThemeEnumWrapper {
    #[classattr]
    fn None() -> AzOptionWaylandThemeEnumWrapper { AzOptionWaylandThemeEnumWrapper { inner: AzOptionWaylandTheme::None } }
    #[staticmethod]
    fn Some(v: AzWaylandTheme) -> AzOptionWaylandThemeEnumWrapper { AzOptionWaylandThemeEnumWrapper { inner: AzOptionWaylandTheme::Some(v) } }
}

#[pymethods]
impl AzOptionTaskBarIconEnumWrapper {
    #[classattr]
    fn None() -> AzOptionTaskBarIconEnumWrapper { AzOptionTaskBarIconEnumWrapper { inner: AzOptionTaskBarIcon::None } }
    #[staticmethod]
    fn Some(v: AzTaskBarIcon) -> AzOptionTaskBarIconEnumWrapper { AzOptionTaskBarIconEnumWrapper { inner: AzOptionTaskBarIcon::Some(v) } }
}

#[pymethods]
impl AzOptionHwndHandleEnumWrapper {
    #[classattr]
    fn None() -> AzOptionHwndHandleEnumWrapper { AzOptionHwndHandleEnumWrapper { inner: AzOptionHwndHandle::None } }
}

#[pymethods]
impl AzOptionLogicalPositionEnumWrapper {
    #[classattr]
    fn None() -> AzOptionLogicalPositionEnumWrapper { AzOptionLogicalPositionEnumWrapper { inner: AzOptionLogicalPosition::None } }
    #[staticmethod]
    fn Some(v: AzLogicalPosition) -> AzOptionLogicalPositionEnumWrapper { AzOptionLogicalPositionEnumWrapper { inner: AzOptionLogicalPosition::Some(v) } }
}

#[pymethods]
impl AzOptionPhysicalPositionI32EnumWrapper {
    #[classattr]
    fn None() -> AzOptionPhysicalPositionI32EnumWrapper { AzOptionPhysicalPositionI32EnumWrapper { inner: AzOptionPhysicalPositionI32::None } }
    #[staticmethod]
    fn Some(v: AzPhysicalPositionI32) -> AzOptionPhysicalPositionI32EnumWrapper { AzOptionPhysicalPositionI32EnumWrapper { inner: AzOptionPhysicalPositionI32::Some(v) } }
}

#[pymethods]
impl AzOptionWindowIconEnumWrapper {
    #[classattr]
    fn None() -> AzOptionWindowIconEnumWrapper { AzOptionWindowIconEnumWrapper { inner: AzOptionWindowIcon::None } }
    #[staticmethod]
    fn Some(v: AzWindowIconEnumWrapper) -> AzOptionWindowIconEnumWrapper { AzOptionWindowIconEnumWrapper { inner: AzOptionWindowIcon::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionStringEnumWrapper {
    #[classattr]
    fn None() -> AzOptionStringEnumWrapper { AzOptionStringEnumWrapper { inner: AzOptionString::None } }
    #[staticmethod]
    fn Some(v: AzString) -> AzOptionStringEnumWrapper { AzOptionStringEnumWrapper { inner: AzOptionString::Some(v) } }
}

#[pymethods]
impl AzOptionX11VisualEnumWrapper {
    #[classattr]
    fn None() -> AzOptionX11VisualEnumWrapper { AzOptionX11VisualEnumWrapper { inner: AzOptionX11Visual::None } }
}

#[pymethods]
impl AzOptionI32EnumWrapper {
    #[classattr]
    fn None() -> AzOptionI32EnumWrapper { AzOptionI32EnumWrapper { inner: AzOptionI32::None } }
    #[staticmethod]
    fn Some(v: i32) -> AzOptionI32EnumWrapper { AzOptionI32EnumWrapper { inner: AzOptionI32::Some(v) } }
}

#[pymethods]
impl AzOptionF32EnumWrapper {
    #[classattr]
    fn None() -> AzOptionF32EnumWrapper { AzOptionF32EnumWrapper { inner: AzOptionF32::None } }
    #[staticmethod]
    fn Some(v: f32) -> AzOptionF32EnumWrapper { AzOptionF32EnumWrapper { inner: AzOptionF32::Some(v) } }
}

#[pymethods]
impl AzOptionMouseCursorTypeEnumWrapper {
    #[classattr]
    fn None() -> AzOptionMouseCursorTypeEnumWrapper { AzOptionMouseCursorTypeEnumWrapper { inner: AzOptionMouseCursorType::None } }
    #[staticmethod]
    fn Some(v: AzMouseCursorTypeEnumWrapper) -> AzOptionMouseCursorTypeEnumWrapper { AzOptionMouseCursorTypeEnumWrapper { inner: AzOptionMouseCursorType::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionLogicalSizeEnumWrapper {
    #[classattr]
    fn None() -> AzOptionLogicalSizeEnumWrapper { AzOptionLogicalSizeEnumWrapper { inner: AzOptionLogicalSize::None } }
    #[staticmethod]
    fn Some(v: AzLogicalSize) -> AzOptionLogicalSizeEnumWrapper { AzOptionLogicalSizeEnumWrapper { inner: AzOptionLogicalSize::Some(v) } }
}

#[pymethods]
impl AzOptionCharEnumWrapper {
    #[classattr]
    fn None() -> AzOptionCharEnumWrapper { AzOptionCharEnumWrapper { inner: AzOptionChar::None } }
    #[staticmethod]
    fn Some(v: u32) -> AzOptionCharEnumWrapper { AzOptionCharEnumWrapper { inner: AzOptionChar::Some(v) } }
}

#[pymethods]
impl AzOptionVirtualKeyCodeEnumWrapper {
    #[classattr]
    fn None() -> AzOptionVirtualKeyCodeEnumWrapper { AzOptionVirtualKeyCodeEnumWrapper { inner: AzOptionVirtualKeyCode::None } }
    #[staticmethod]
    fn Some(v: AzVirtualKeyCodeEnumWrapper) -> AzOptionVirtualKeyCodeEnumWrapper { AzOptionVirtualKeyCodeEnumWrapper { inner: AzOptionVirtualKeyCode::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionDomEnumWrapper {
    #[classattr]
    fn None() -> AzOptionDomEnumWrapper { AzOptionDomEnumWrapper { inner: AzOptionDom::None } }
    #[staticmethod]
    fn Some(v: AzDom) -> AzOptionDomEnumWrapper { AzOptionDomEnumWrapper { inner: AzOptionDom::Some(v) } }
}

#[pymethods]
impl AzOptionTextureEnumWrapper {
    #[classattr]
    fn None() -> AzOptionTextureEnumWrapper { AzOptionTextureEnumWrapper { inner: AzOptionTexture::None } }
    #[staticmethod]
    fn Some(v: AzTexture) -> AzOptionTextureEnumWrapper { AzOptionTextureEnumWrapper { inner: AzOptionTexture::Some(v) } }
}

#[pymethods]
impl AzOptionImageMaskEnumWrapper {
    #[classattr]
    fn None() -> AzOptionImageMaskEnumWrapper { AzOptionImageMaskEnumWrapper { inner: AzOptionImageMask::None } }
    #[staticmethod]
    fn Some(v: AzImageMask) -> AzOptionImageMaskEnumWrapper { AzOptionImageMaskEnumWrapper { inner: AzOptionImageMask::Some(v) } }
}

#[pymethods]
impl AzOptionTabIndexEnumWrapper {
    #[classattr]
    fn None() -> AzOptionTabIndexEnumWrapper { AzOptionTabIndexEnumWrapper { inner: AzOptionTabIndex::None } }
    #[staticmethod]
    fn Some(v: AzTabIndexEnumWrapper) -> AzOptionTabIndexEnumWrapper { AzOptionTabIndexEnumWrapper { inner: AzOptionTabIndex::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionTagIdEnumWrapper {
    #[classattr]
    fn None() -> AzOptionTagIdEnumWrapper { AzOptionTagIdEnumWrapper { inner: AzOptionTagId::None } }
    #[staticmethod]
    fn Some(v: AzTagId) -> AzOptionTagIdEnumWrapper { AzOptionTagIdEnumWrapper { inner: AzOptionTagId::Some(v) } }
}

#[pymethods]
impl AzOptionDurationEnumWrapper {
    #[classattr]
    fn None() -> AzOptionDurationEnumWrapper { AzOptionDurationEnumWrapper { inner: AzOptionDuration::None } }
    #[staticmethod]
    fn Some(v: AzDurationEnumWrapper) -> AzOptionDurationEnumWrapper { AzOptionDurationEnumWrapper { inner: AzOptionDuration::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionInstantEnumWrapper {
    #[classattr]
    fn None() -> AzOptionInstantEnumWrapper { AzOptionInstantEnumWrapper { inner: AzOptionInstant::None } }
    #[staticmethod]
    fn Some(v: AzInstantEnumWrapper) -> AzOptionInstantEnumWrapper { AzOptionInstantEnumWrapper { inner: AzOptionInstant::Some(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzOptionUsizeEnumWrapper {
    #[classattr]
    fn None() -> AzOptionUsizeEnumWrapper { AzOptionUsizeEnumWrapper { inner: AzOptionUsize::None } }
    #[staticmethod]
    fn Some(v: usize) -> AzOptionUsizeEnumWrapper { AzOptionUsizeEnumWrapper { inner: AzOptionUsize::Some(v) } }
}

#[pymethods]
impl AzOptionU8VecEnumWrapper {
    #[classattr]
    fn None() -> AzOptionU8VecEnumWrapper { AzOptionU8VecEnumWrapper { inner: AzOptionU8Vec::None } }
    #[staticmethod]
    fn Some(v: AzU8Vec) -> AzOptionU8VecEnumWrapper { AzOptionU8VecEnumWrapper { inner: AzOptionU8Vec::Some(v) } }
}

#[pymethods]
impl AzOptionU8VecRefEnumWrapper {
    #[classattr]
    fn None() -> AzOptionU8VecRefEnumWrapper { AzOptionU8VecRefEnumWrapper { inner: AzOptionU8VecRef::None } }
    #[staticmethod]
    fn Some(v: AzU8VecRef) -> AzOptionU8VecRefEnumWrapper { AzOptionU8VecRefEnumWrapper { inner: AzOptionU8VecRef::Some(v) } }
}

#[pymethods]
impl AzResultXmlXmlErrorEnumWrapper {
    #[staticmethod]
    fn Ok(v: AzXml) -> AzResultXmlXmlErrorEnumWrapper { AzResultXmlXmlErrorEnumWrapper { inner: AzResultXmlXmlError::Ok(v) } }
    #[staticmethod]
    fn Err(v: AzXmlErrorEnumWrapper) -> AzResultXmlXmlErrorEnumWrapper { AzResultXmlXmlErrorEnumWrapper { inner: AzResultXmlXmlError::Err(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzResultRawImageDecodeImageErrorEnumWrapper {
    #[staticmethod]
    fn Ok(v: AzRawImage) -> AzResultRawImageDecodeImageErrorEnumWrapper { AzResultRawImageDecodeImageErrorEnumWrapper { inner: AzResultRawImageDecodeImageError::Ok(v) } }
    #[staticmethod]
    fn Err(v: AzDecodeImageErrorEnumWrapper) -> AzResultRawImageDecodeImageErrorEnumWrapper { AzResultRawImageDecodeImageErrorEnumWrapper { inner: AzResultRawImageDecodeImageError::Err(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzResultU8VecEncodeImageErrorEnumWrapper {
    #[staticmethod]
    fn Ok(v: AzU8Vec) -> AzResultU8VecEncodeImageErrorEnumWrapper { AzResultU8VecEncodeImageErrorEnumWrapper { inner: AzResultU8VecEncodeImageError::Ok(v) } }
    #[staticmethod]
    fn Err(v: AzEncodeImageErrorEnumWrapper) -> AzResultU8VecEncodeImageErrorEnumWrapper { AzResultU8VecEncodeImageErrorEnumWrapper { inner: AzResultU8VecEncodeImageError::Err(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzResultSvgXmlNodeSvgParseErrorEnumWrapper {
    #[staticmethod]
    fn Ok(v: AzSvgXmlNode) -> AzResultSvgXmlNodeSvgParseErrorEnumWrapper { AzResultSvgXmlNodeSvgParseErrorEnumWrapper { inner: AzResultSvgXmlNodeSvgParseError::Ok(v) } }
    #[staticmethod]
    fn Err(v: AzSvgParseErrorEnumWrapper) -> AzResultSvgXmlNodeSvgParseErrorEnumWrapper { AzResultSvgXmlNodeSvgParseErrorEnumWrapper { inner: AzResultSvgXmlNodeSvgParseError::Err(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzResultSvgSvgParseErrorEnumWrapper {
    #[staticmethod]
    fn Ok(v: AzSvg) -> AzResultSvgSvgParseErrorEnumWrapper { AzResultSvgSvgParseErrorEnumWrapper { inner: AzResultSvgSvgParseError::Ok(v) } }
    #[staticmethod]
    fn Err(v: AzSvgParseErrorEnumWrapper) -> AzResultSvgSvgParseErrorEnumWrapper { AzResultSvgSvgParseErrorEnumWrapper { inner: AzResultSvgSvgParseError::Err(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzSvgParseErrorEnumWrapper {
    #[classattr]
    fn InvalidFileSuffix() -> AzSvgParseErrorEnumWrapper { AzSvgParseErrorEnumWrapper { inner: AzSvgParseError::InvalidFileSuffix } }
    #[classattr]
    fn FileOpenFailed() -> AzSvgParseErrorEnumWrapper { AzSvgParseErrorEnumWrapper { inner: AzSvgParseError::FileOpenFailed } }
    #[classattr]
    fn NotAnUtf8Str() -> AzSvgParseErrorEnumWrapper { AzSvgParseErrorEnumWrapper { inner: AzSvgParseError::NotAnUtf8Str } }
    #[classattr]
    fn MalformedGZip() -> AzSvgParseErrorEnumWrapper { AzSvgParseErrorEnumWrapper { inner: AzSvgParseError::MalformedGZip } }
    #[classattr]
    fn InvalidSize() -> AzSvgParseErrorEnumWrapper { AzSvgParseErrorEnumWrapper { inner: AzSvgParseError::InvalidSize } }
    #[staticmethod]
    fn ParsingFailed(v: AzXmlErrorEnumWrapper) -> AzSvgParseErrorEnumWrapper { AzSvgParseErrorEnumWrapper { inner: AzSvgParseError::ParsingFailed(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzXmlErrorEnumWrapper {
    #[staticmethod]
    fn InvalidXmlPrefixUri(v: AzSvgParseErrorPosition) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::InvalidXmlPrefixUri(v) } }
    #[staticmethod]
    fn UnexpectedXmlUri(v: AzSvgParseErrorPosition) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::UnexpectedXmlUri(v) } }
    #[staticmethod]
    fn UnexpectedXmlnsUri(v: AzSvgParseErrorPosition) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::UnexpectedXmlnsUri(v) } }
    #[staticmethod]
    fn InvalidElementNamePrefix(v: AzSvgParseErrorPosition) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::InvalidElementNamePrefix(v) } }
    #[staticmethod]
    fn DuplicatedNamespace(v: AzDuplicatedNamespaceError) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::DuplicatedNamespace(v) } }
    #[staticmethod]
    fn UnknownNamespace(v: AzUnknownNamespaceError) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::UnknownNamespace(v) } }
    #[staticmethod]
    fn UnexpectedCloseTag(v: AzUnexpectedCloseTagError) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::UnexpectedCloseTag(v) } }
    #[staticmethod]
    fn UnexpectedEntityCloseTag(v: AzSvgParseErrorPosition) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::UnexpectedEntityCloseTag(v) } }
    #[staticmethod]
    fn UnknownEntityReference(v: AzUnknownEntityReferenceError) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::UnknownEntityReference(v) } }
    #[staticmethod]
    fn MalformedEntityReference(v: AzSvgParseErrorPosition) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::MalformedEntityReference(v) } }
    #[staticmethod]
    fn EntityReferenceLoop(v: AzSvgParseErrorPosition) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::EntityReferenceLoop(v) } }
    #[staticmethod]
    fn InvalidAttributeValue(v: AzSvgParseErrorPosition) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::InvalidAttributeValue(v) } }
    #[staticmethod]
    fn DuplicatedAttribute(v: AzDuplicatedAttributeError) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::DuplicatedAttribute(v) } }
    #[classattr]
    fn NoRootNode() -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::NoRootNode } }
    #[classattr]
    fn SizeLimit() -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::SizeLimit } }
    #[staticmethod]
    fn ParserError(v: AzXmlParseErrorEnumWrapper) -> AzXmlErrorEnumWrapper { AzXmlErrorEnumWrapper { inner: AzXmlError::ParserError(unsafe { mem::transmute(v) }) } }
}

#[pymethods]
impl AzXmlParseErrorEnumWrapper {
    #[staticmethod]
    fn InvalidDeclaration(v: AzXmlTextError) -> AzXmlParseErrorEnumWrapper { AzXmlParseErrorEnumWrapper { inner: AzXmlParseError::InvalidDeclaration(v) } }
    #[staticmethod]
    fn InvalidComment(v: AzXmlTextError) -> AzXmlParseErrorEnumWrapper { AzXmlParseErrorEnumWrapper { inner: AzXmlParseError::InvalidComment(v) } }
    #[staticmethod]
    fn InvalidPI(v: AzXmlTextError) -> AzXmlParseErrorEnumWrapper { AzXmlParseErrorEnumWrapper { inner: AzXmlParseError::InvalidPI(v) } }
    #[staticmethod]
    fn InvalidDoctype(v: AzXmlTextError) -> AzXmlParseErrorEnumWrapper { AzXmlParseErrorEnumWrapper { inner: AzXmlParseError::InvalidDoctype(v) } }
    #[staticmethod]
    fn InvalidEntity(v: AzXmlTextError) -> AzXmlParseErrorEnumWrapper { AzXmlParseErrorEnumWrapper { inner: AzXmlParseError::InvalidEntity(v) } }
    #[staticmethod]
    fn InvalidElement(v: AzXmlTextError) -> AzXmlParseErrorEnumWrapper { AzXmlParseErrorEnumWrapper { inner: AzXmlParseError::InvalidElement(v) } }
    #[staticmethod]
    fn InvalidAttribute(v: AzXmlTextError) -> AzXmlParseErrorEnumWrapper { AzXmlParseErrorEnumWrapper { inner: AzXmlParseError::InvalidAttribute(v) } }
    #[staticmethod]
    fn InvalidCdata(v: AzXmlTextError) -> AzXmlParseErrorEnumWrapper { AzXmlParseErrorEnumWrapper { inner: AzXmlParseError::InvalidCdata(v) } }
    #[staticmethod]
    fn InvalidCharData(v: AzXmlTextError) -> AzXmlParseErrorEnumWrapper { AzXmlParseErrorEnumWrapper { inner: AzXmlParseError::InvalidCharData(v) } }
    #[staticmethod]
    fn UnknownToken(v: AzSvgParseErrorPosition) -> AzXmlParseErrorEnumWrapper { AzXmlParseErrorEnumWrapper { inner: AzXmlParseError::UnknownToken(v) } }
}

#[pymethods]
impl AzXmlStreamErrorEnumWrapper {
    #[classattr]
    fn UnexpectedEndOfStream() -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::UnexpectedEndOfStream } }
    #[classattr]
    fn InvalidName() -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidName } }
    #[staticmethod]
    fn NonXmlChar(v: AzNonXmlCharError) -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::NonXmlChar(v) } }
    #[staticmethod]
    fn InvalidChar(v: AzInvalidCharError) -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidChar(v) } }
    #[staticmethod]
    fn InvalidCharMultiple(v: AzInvalidCharMultipleError) -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidCharMultiple(v) } }
    #[staticmethod]
    fn InvalidQuote(v: AzInvalidQuoteError) -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidQuote(v) } }
    #[staticmethod]
    fn InvalidSpace(v: AzInvalidSpaceError) -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidSpace(v) } }
    #[staticmethod]
    fn InvalidString(v: AzInvalidStringError) -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidString(v) } }
    #[classattr]
    fn InvalidReference() -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidReference } }
    #[classattr]
    fn InvalidExternalID() -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidExternalID } }
    #[classattr]
    fn InvalidCommentData() -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidCommentData } }
    #[classattr]
    fn InvalidCommentEnd() -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidCommentEnd } }
    #[classattr]
    fn InvalidCharacterData() -> AzXmlStreamErrorEnumWrapper { AzXmlStreamErrorEnumWrapper { inner: AzXmlStreamError::InvalidCharacterData } }
}


impl core::convert::From<AzDecodeImageError> for PyErr {
    fn from(err: AzDecodeImageError) -> PyErr {
        let r: azul_impl::resources::decode::DecodeImageError = unsafe { mem::transmute(err) };
        PyException::new_err(format!("{}", r))
    }
}

impl core::convert::From<AzEncodeImageError> for PyErr {
    fn from(err: AzEncodeImageError) -> PyErr {
        let r: azul_impl::resources::encode::EncodeImageError = unsafe { mem::transmute(err) };
        PyException::new_err(format!("{}", r))
    }
}

impl core::convert::From<AzSvgParseError> for PyErr {
    fn from(err: AzSvgParseError) -> PyErr {
        let r: azul_impl::svg::SvgParseError = unsafe { mem::transmute(err) };
        PyException::new_err(format!("{}", r))
    }
}

impl core::convert::From<AzXmlError> for PyErr {
    fn from(err: AzXmlError) -> PyErr {
        let r: azul_impl::xml::XmlError = unsafe { mem::transmute(err) };
        PyException::new_err(format!("{}", r))
    }
}

#[pymodule]
fn azul(py: Python, m: &PyModule) -> PyResult<()> {

    m.add_class::<AzApp>()?;
    m.add_class::<AzAppConfig>()?;
    m.add_class::<AzAppLogLevelEnumWrapper>()?;
    m.add_class::<AzLayoutSolverEnumWrapper>()?;
    m.add_class::<AzSystemCallbacks>()?;

    m.add_class::<AzWindowCreateOptions>()?;
    m.add_class::<AzRendererOptions>()?;
    m.add_class::<AzVsyncEnumWrapper>()?;
    m.add_class::<AzSrgbEnumWrapper>()?;
    m.add_class::<AzHwAccelerationEnumWrapper>()?;
    m.add_class::<AzLayoutPoint>()?;
    m.add_class::<AzLayoutSize>()?;
    m.add_class::<AzLayoutRect>()?;
    m.add_class::<AzRawWindowHandleEnumWrapper>()?;
    m.add_class::<AzIOSHandle>()?;
    m.add_class::<AzMacOSHandle>()?;
    m.add_class::<AzXlibHandle>()?;
    m.add_class::<AzXcbHandle>()?;
    m.add_class::<AzWaylandHandle>()?;
    m.add_class::<AzWindowsHandle>()?;
    m.add_class::<AzWebHandle>()?;
    m.add_class::<AzAndroidHandle>()?;
    m.add_class::<AzXWindowTypeEnumWrapper>()?;
    m.add_class::<AzPhysicalPositionI32>()?;
    m.add_class::<AzPhysicalSizeU32>()?;
    m.add_class::<AzLogicalRect>()?;
    m.add_class::<AzLogicalPosition>()?;
    m.add_class::<AzLogicalSize>()?;
    m.add_class::<AzIconKey>()?;
    m.add_class::<AzSmallWindowIconBytes>()?;
    m.add_class::<AzLargeWindowIconBytes>()?;
    m.add_class::<AzWindowIconEnumWrapper>()?;
    m.add_class::<AzTaskBarIcon>()?;
    m.add_class::<AzVirtualKeyCodeEnumWrapper>()?;
    m.add_class::<AzAcceleratorKeyEnumWrapper>()?;
    m.add_class::<AzWindowSize>()?;
    m.add_class::<AzWindowFlags>()?;
    m.add_class::<AzDebugState>()?;
    m.add_class::<AzKeyboardState>()?;
    m.add_class::<AzMouseCursorTypeEnumWrapper>()?;
    m.add_class::<AzCursorPositionEnumWrapper>()?;
    m.add_class::<AzMouseState>()?;
    m.add_class::<AzPlatformSpecificOptions>()?;
    m.add_class::<AzWindowsWindowOptions>()?;
    m.add_class::<AzWaylandTheme>()?;
    m.add_class::<AzRendererTypeEnumWrapper>()?;
    m.add_class::<AzStringPair>()?;
    m.add_class::<AzLinuxWindowOptions>()?;
    m.add_class::<AzMacWindowOptions>()?;
    m.add_class::<AzWasmWindowOptions>()?;
    m.add_class::<AzFullScreenModeEnumWrapper>()?;
    m.add_class::<AzWindowThemeEnumWrapper>()?;
    m.add_class::<AzWindowPositionEnumWrapper>()?;
    m.add_class::<AzImePositionEnumWrapper>()?;
    m.add_class::<AzTouchState>()?;
    m.add_class::<AzMonitor>()?;
    m.add_class::<AzVideoMode>()?;
    m.add_class::<AzWindowState>()?;

    m.add_class::<AzLayoutCallbackEnumWrapper>()?;
    m.add_class::<AzMarshaledLayoutCallback>()?;
    m.add_class::<AzMarshaledLayoutCallbackInner>()?;
    m.add_class::<AzLayoutCallbackInner>()?;
    m.add_class::<AzCallback>()?;
    m.add_class::<AzCallbackInfo>()?;
    m.add_class::<AzUpdateEnumWrapper>()?;
    m.add_class::<AzNodeId>()?;
    m.add_class::<AzDomId>()?;
    m.add_class::<AzDomNodeId>()?;
    m.add_class::<AzPositionInfoEnumWrapper>()?;
    m.add_class::<AzPositionInfoInner>()?;
    m.add_class::<AzHidpiAdjustedBounds>()?;
    m.add_class::<AzInlineText>()?;
    m.add_class::<AzInlineLine>()?;
    m.add_class::<AzInlineWordEnumWrapper>()?;
    m.add_class::<AzInlineTextContents>()?;
    m.add_class::<AzInlineGlyph>()?;
    m.add_class::<AzInlineTextHit>()?;
    m.add_class::<AzFocusTargetEnumWrapper>()?;
    m.add_class::<AzFocusTargetPath>()?;
    m.add_class::<AzAnimation>()?;
    m.add_class::<AzAnimationRepeatEnumWrapper>()?;
    m.add_class::<AzAnimationRepeatCountEnumWrapper>()?;
    m.add_class::<AzAnimationEasingEnumWrapper>()?;
    m.add_class::<AzIFrameCallback>()?;
    m.add_class::<AzIFrameCallbackInfo>()?;
    m.add_class::<AzIFrameCallbackReturn>()?;
    m.add_class::<AzRenderImageCallback>()?;
    m.add_class::<AzRenderImageCallbackInfo>()?;
    m.add_class::<AzTimerCallback>()?;
    m.add_class::<AzTimerCallbackInfo>()?;
    m.add_class::<AzTimerCallbackReturn>()?;
    m.add_class::<AzWriteBackCallback>()?;
    m.add_class::<AzThreadCallback>()?;
    m.add_class::<AzRefCount>()?;
    m.add_class::<AzRefAny>()?;
    m.add_class::<AzLayoutCallbackInfo>()?;

    m.add_class::<AzDom>()?;
    m.add_class::<AzIFrameNode>()?;
    m.add_class::<AzCallbackData>()?;
    m.add_class::<AzNodeData>()?;
    m.add_class::<AzNodeTypeEnumWrapper>()?;
    m.add_class::<AzOnEnumWrapper>()?;
    m.add_class::<AzEventFilterEnumWrapper>()?;
    m.add_class::<AzHoverEventFilterEnumWrapper>()?;
    m.add_class::<AzFocusEventFilterEnumWrapper>()?;
    m.add_class::<AzNotEventFilterEnumWrapper>()?;
    m.add_class::<AzWindowEventFilterEnumWrapper>()?;
    m.add_class::<AzComponentEventFilterEnumWrapper>()?;
    m.add_class::<AzApplicationEventFilterEnumWrapper>()?;
    m.add_class::<AzTabIndexEnumWrapper>()?;
    m.add_class::<AzIdOrClassEnumWrapper>()?;
    m.add_class::<AzNodeDataInlineCssPropertyEnumWrapper>()?;

    m.add_class::<AzCssRuleBlock>()?;
    m.add_class::<AzCssDeclarationEnumWrapper>()?;
    m.add_class::<AzDynamicCssProperty>()?;
    m.add_class::<AzCssPath>()?;
    m.add_class::<AzCssPathSelectorEnumWrapper>()?;
    m.add_class::<AzNodeTypeKeyEnumWrapper>()?;
    m.add_class::<AzCssPathPseudoSelectorEnumWrapper>()?;
    m.add_class::<AzCssNthChildSelectorEnumWrapper>()?;
    m.add_class::<AzCssNthChildPattern>()?;
    m.add_class::<AzStylesheet>()?;
    m.add_class::<AzCss>()?;
    m.add_class::<AzCssPropertyTypeEnumWrapper>()?;
    m.add_class::<AzAnimationInterpolationFunctionEnumWrapper>()?;
    m.add_class::<AzInterpolateContext>()?;
    m.add_class::<AzColorU>()?;
    m.add_class::<AzSizeMetricEnumWrapper>()?;
    m.add_class::<AzFloatValue>()?;
    m.add_class::<AzPixelValue>()?;
    m.add_class::<AzPixelValueNoPercent>()?;
    m.add_class::<AzBoxShadowClipModeEnumWrapper>()?;
    m.add_class::<AzStyleBoxShadow>()?;
    m.add_class::<AzLayoutAlignContentEnumWrapper>()?;
    m.add_class::<AzLayoutAlignItemsEnumWrapper>()?;
    m.add_class::<AzLayoutBottom>()?;
    m.add_class::<AzLayoutBoxSizingEnumWrapper>()?;
    m.add_class::<AzLayoutFlexDirectionEnumWrapper>()?;
    m.add_class::<AzLayoutDisplayEnumWrapper>()?;
    m.add_class::<AzLayoutFlexGrow>()?;
    m.add_class::<AzLayoutFlexShrink>()?;
    m.add_class::<AzLayoutFloatEnumWrapper>()?;
    m.add_class::<AzLayoutHeight>()?;
    m.add_class::<AzLayoutJustifyContentEnumWrapper>()?;
    m.add_class::<AzLayoutLeft>()?;
    m.add_class::<AzLayoutMarginBottom>()?;
    m.add_class::<AzLayoutMarginLeft>()?;
    m.add_class::<AzLayoutMarginRight>()?;
    m.add_class::<AzLayoutMarginTop>()?;
    m.add_class::<AzLayoutMaxHeight>()?;
    m.add_class::<AzLayoutMaxWidth>()?;
    m.add_class::<AzLayoutMinHeight>()?;
    m.add_class::<AzLayoutMinWidth>()?;
    m.add_class::<AzLayoutPaddingBottom>()?;
    m.add_class::<AzLayoutPaddingLeft>()?;
    m.add_class::<AzLayoutPaddingRight>()?;
    m.add_class::<AzLayoutPaddingTop>()?;
    m.add_class::<AzLayoutPositionEnumWrapper>()?;
    m.add_class::<AzLayoutRight>()?;
    m.add_class::<AzLayoutTop>()?;
    m.add_class::<AzLayoutWidth>()?;
    m.add_class::<AzLayoutFlexWrapEnumWrapper>()?;
    m.add_class::<AzLayoutOverflowEnumWrapper>()?;
    m.add_class::<AzPercentageValue>()?;
    m.add_class::<AzAngleMetricEnumWrapper>()?;
    m.add_class::<AzAngleValue>()?;
    m.add_class::<AzNormalizedLinearColorStop>()?;
    m.add_class::<AzNormalizedRadialColorStop>()?;
    m.add_class::<AzDirectionCornerEnumWrapper>()?;
    m.add_class::<AzDirectionCorners>()?;
    m.add_class::<AzDirectionEnumWrapper>()?;
    m.add_class::<AzExtendModeEnumWrapper>()?;
    m.add_class::<AzLinearGradient>()?;
    m.add_class::<AzShapeEnumWrapper>()?;
    m.add_class::<AzRadialGradientSizeEnumWrapper>()?;
    m.add_class::<AzRadialGradient>()?;
    m.add_class::<AzConicGradient>()?;
    m.add_class::<AzStyleBackgroundContentEnumWrapper>()?;
    m.add_class::<AzBackgroundPositionHorizontalEnumWrapper>()?;
    m.add_class::<AzBackgroundPositionVerticalEnumWrapper>()?;
    m.add_class::<AzStyleBackgroundPosition>()?;
    m.add_class::<AzStyleBackgroundRepeatEnumWrapper>()?;
    m.add_class::<AzStyleBackgroundSizeEnumWrapper>()?;
    m.add_class::<AzStyleBorderBottomColor>()?;
    m.add_class::<AzStyleBorderBottomLeftRadius>()?;
    m.add_class::<AzStyleBorderBottomRightRadius>()?;
    m.add_class::<AzBorderStyleEnumWrapper>()?;
    m.add_class::<AzStyleBorderBottomStyle>()?;
    m.add_class::<AzLayoutBorderBottomWidth>()?;
    m.add_class::<AzStyleBorderLeftColor>()?;
    m.add_class::<AzStyleBorderLeftStyle>()?;
    m.add_class::<AzLayoutBorderLeftWidth>()?;
    m.add_class::<AzStyleBorderRightColor>()?;
    m.add_class::<AzStyleBorderRightStyle>()?;
    m.add_class::<AzLayoutBorderRightWidth>()?;
    m.add_class::<AzStyleBorderTopColor>()?;
    m.add_class::<AzStyleBorderTopLeftRadius>()?;
    m.add_class::<AzStyleBorderTopRightRadius>()?;
    m.add_class::<AzStyleBorderTopStyle>()?;
    m.add_class::<AzLayoutBorderTopWidth>()?;
    m.add_class::<AzScrollbarInfo>()?;
    m.add_class::<AzScrollbarStyle>()?;
    m.add_class::<AzStyleCursorEnumWrapper>()?;
    m.add_class::<AzStyleFontFamilyEnumWrapper>()?;
    m.add_class::<AzStyleFontSize>()?;
    m.add_class::<AzStyleLetterSpacing>()?;
    m.add_class::<AzStyleLineHeight>()?;
    m.add_class::<AzStyleTabWidth>()?;
    m.add_class::<AzStyleOpacity>()?;
    m.add_class::<AzStyleTransformOrigin>()?;
    m.add_class::<AzStylePerspectiveOrigin>()?;
    m.add_class::<AzStyleBackfaceVisibilityEnumWrapper>()?;
    m.add_class::<AzStyleTransformEnumWrapper>()?;
    m.add_class::<AzStyleTransformMatrix2D>()?;
    m.add_class::<AzStyleTransformMatrix3D>()?;
    m.add_class::<AzStyleTransformTranslate2D>()?;
    m.add_class::<AzStyleTransformTranslate3D>()?;
    m.add_class::<AzStyleTransformRotate3D>()?;
    m.add_class::<AzStyleTransformScale2D>()?;
    m.add_class::<AzStyleTransformScale3D>()?;
    m.add_class::<AzStyleTransformSkew2D>()?;
    m.add_class::<AzStyleTextAlignEnumWrapper>()?;
    m.add_class::<AzStyleTextColor>()?;
    m.add_class::<AzStyleWordSpacing>()?;
    m.add_class::<AzStyleBoxShadowValueEnumWrapper>()?;
    m.add_class::<AzLayoutAlignContentValueEnumWrapper>()?;
    m.add_class::<AzLayoutAlignItemsValueEnumWrapper>()?;
    m.add_class::<AzLayoutBottomValueEnumWrapper>()?;
    m.add_class::<AzLayoutBoxSizingValueEnumWrapper>()?;
    m.add_class::<AzLayoutFlexDirectionValueEnumWrapper>()?;
    m.add_class::<AzLayoutDisplayValueEnumWrapper>()?;
    m.add_class::<AzLayoutFlexGrowValueEnumWrapper>()?;
    m.add_class::<AzLayoutFlexShrinkValueEnumWrapper>()?;
    m.add_class::<AzLayoutFloatValueEnumWrapper>()?;
    m.add_class::<AzLayoutHeightValueEnumWrapper>()?;
    m.add_class::<AzLayoutJustifyContentValueEnumWrapper>()?;
    m.add_class::<AzLayoutLeftValueEnumWrapper>()?;
    m.add_class::<AzLayoutMarginBottomValueEnumWrapper>()?;
    m.add_class::<AzLayoutMarginLeftValueEnumWrapper>()?;
    m.add_class::<AzLayoutMarginRightValueEnumWrapper>()?;
    m.add_class::<AzLayoutMarginTopValueEnumWrapper>()?;
    m.add_class::<AzLayoutMaxHeightValueEnumWrapper>()?;
    m.add_class::<AzLayoutMaxWidthValueEnumWrapper>()?;
    m.add_class::<AzLayoutMinHeightValueEnumWrapper>()?;
    m.add_class::<AzLayoutMinWidthValueEnumWrapper>()?;
    m.add_class::<AzLayoutPaddingBottomValueEnumWrapper>()?;
    m.add_class::<AzLayoutPaddingLeftValueEnumWrapper>()?;
    m.add_class::<AzLayoutPaddingRightValueEnumWrapper>()?;
    m.add_class::<AzLayoutPaddingTopValueEnumWrapper>()?;
    m.add_class::<AzLayoutPositionValueEnumWrapper>()?;
    m.add_class::<AzLayoutRightValueEnumWrapper>()?;
    m.add_class::<AzLayoutTopValueEnumWrapper>()?;
    m.add_class::<AzLayoutWidthValueEnumWrapper>()?;
    m.add_class::<AzLayoutFlexWrapValueEnumWrapper>()?;
    m.add_class::<AzLayoutOverflowValueEnumWrapper>()?;
    m.add_class::<AzScrollbarStyleValueEnumWrapper>()?;
    m.add_class::<AzStyleBackgroundContentVecValueEnumWrapper>()?;
    m.add_class::<AzStyleBackgroundPositionVecValueEnumWrapper>()?;
    m.add_class::<AzStyleBackgroundRepeatVecValueEnumWrapper>()?;
    m.add_class::<AzStyleBackgroundSizeVecValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderBottomColorValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderBottomLeftRadiusValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderBottomRightRadiusValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderBottomStyleValueEnumWrapper>()?;
    m.add_class::<AzLayoutBorderBottomWidthValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderLeftColorValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderLeftStyleValueEnumWrapper>()?;
    m.add_class::<AzLayoutBorderLeftWidthValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderRightColorValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderRightStyleValueEnumWrapper>()?;
    m.add_class::<AzLayoutBorderRightWidthValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderTopColorValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderTopLeftRadiusValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderTopRightRadiusValueEnumWrapper>()?;
    m.add_class::<AzStyleBorderTopStyleValueEnumWrapper>()?;
    m.add_class::<AzLayoutBorderTopWidthValueEnumWrapper>()?;
    m.add_class::<AzStyleCursorValueEnumWrapper>()?;
    m.add_class::<AzStyleFontFamilyVecValueEnumWrapper>()?;
    m.add_class::<AzStyleFontSizeValueEnumWrapper>()?;
    m.add_class::<AzStyleLetterSpacingValueEnumWrapper>()?;
    m.add_class::<AzStyleLineHeightValueEnumWrapper>()?;
    m.add_class::<AzStyleTabWidthValueEnumWrapper>()?;
    m.add_class::<AzStyleTextAlignValueEnumWrapper>()?;
    m.add_class::<AzStyleTextColorValueEnumWrapper>()?;
    m.add_class::<AzStyleWordSpacingValueEnumWrapper>()?;
    m.add_class::<AzStyleOpacityValueEnumWrapper>()?;
    m.add_class::<AzStyleTransformVecValueEnumWrapper>()?;
    m.add_class::<AzStyleTransformOriginValueEnumWrapper>()?;
    m.add_class::<AzStylePerspectiveOriginValueEnumWrapper>()?;
    m.add_class::<AzStyleBackfaceVisibilityValueEnumWrapper>()?;
    m.add_class::<AzCssPropertyEnumWrapper>()?;

    m.add_class::<AzNode>()?;
    m.add_class::<AzCascadeInfo>()?;
    m.add_class::<AzCssPropertySourceEnumWrapper>()?;
    m.add_class::<AzStyledNodeState>()?;
    m.add_class::<AzStyledNode>()?;
    m.add_class::<AzTagId>()?;
    m.add_class::<AzTagIdToNodeIdMapping>()?;
    m.add_class::<AzParentWithNodeDepth>()?;
    m.add_class::<AzCssPropertyCache>()?;
    m.add_class::<AzStyledDom>()?;

    m.add_class::<AzTexture>()?;
    m.add_class::<AzGlVoidPtrConst>()?;
    m.add_class::<AzGlVoidPtrMut>()?;
    m.add_class::<AzGl>()?;
    m.add_class::<AzGlShaderPrecisionFormatReturn>()?;
    m.add_class::<AzVertexAttributeTypeEnumWrapper>()?;
    m.add_class::<AzVertexAttribute>()?;
    m.add_class::<AzVertexLayout>()?;
    m.add_class::<AzVertexArrayObject>()?;
    m.add_class::<AzIndexBufferFormatEnumWrapper>()?;
    m.add_class::<AzVertexBuffer>()?;
    m.add_class::<AzGlTypeEnumWrapper>()?;
    m.add_class::<AzDebugMessage>()?;
    m.add_class::<AzU8VecRef>()?;
    m.add_class::<AzU8VecRefMut>()?;
    m.add_class::<AzF32VecRef>()?;
    m.add_class::<AzI32VecRef>()?;
    m.add_class::<AzGLuintVecRef>()?;
    m.add_class::<AzGLenumVecRef>()?;
    m.add_class::<AzGLintVecRefMut>()?;
    m.add_class::<AzGLint64VecRefMut>()?;
    m.add_class::<AzGLbooleanVecRefMut>()?;
    m.add_class::<AzGLfloatVecRefMut>()?;
    m.add_class::<AzRefstrVecRef>()?;
    m.add_class::<AzRefstr>()?;
    m.add_class::<AzGetProgramBinaryReturn>()?;
    m.add_class::<AzGetActiveAttribReturn>()?;
    m.add_class::<AzGLsyncPtr>()?;
    m.add_class::<AzGetActiveUniformReturn>()?;
    m.add_class::<AzTextureFlags>()?;

    m.add_class::<AzImageRef>()?;
    m.add_class::<AzRawImage>()?;
    m.add_class::<AzImageMask>()?;
    m.add_class::<AzRawImageFormatEnumWrapper>()?;
    m.add_class::<AzEncodeImageErrorEnumWrapper>()?;
    m.add_class::<AzDecodeImageErrorEnumWrapper>()?;
    m.add_class::<AzRawImageDataEnumWrapper>()?;

    m.add_class::<AzFontMetrics>()?;
    m.add_class::<AzFontSource>()?;
    m.add_class::<AzFontRef>()?;

    m.add_class::<AzSvg>()?;
    m.add_class::<AzSvgXmlNode>()?;
    m.add_class::<AzSvgMultiPolygon>()?;
    m.add_class::<AzSvgNodeEnumWrapper>()?;
    m.add_class::<AzSvgStyledNode>()?;
    m.add_class::<AzSvgCircle>()?;
    m.add_class::<AzSvgPath>()?;
    m.add_class::<AzSvgPathElementEnumWrapper>()?;
    m.add_class::<AzSvgLine>()?;
    m.add_class::<AzSvgPoint>()?;
    m.add_class::<AzSvgQuadraticCurve>()?;
    m.add_class::<AzSvgCubicCurve>()?;
    m.add_class::<AzSvgRect>()?;
    m.add_class::<AzSvgVertex>()?;
    m.add_class::<AzTesselatedSvgNode>()?;
    m.add_class::<AzTesselatedSvgNodeVecRef>()?;
    m.add_class::<AzSvgParseOptions>()?;
    m.add_class::<AzShapeRenderingEnumWrapper>()?;
    m.add_class::<AzTextRenderingEnumWrapper>()?;
    m.add_class::<AzImageRenderingEnumWrapper>()?;
    m.add_class::<AzFontDatabaseEnumWrapper>()?;
    m.add_class::<AzSvgRenderOptions>()?;
    m.add_class::<AzSvgStringFormatOptions>()?;
    m.add_class::<AzIndentEnumWrapper>()?;
    m.add_class::<AzSvgFitToEnumWrapper>()?;
    m.add_class::<AzSvgStyleEnumWrapper>()?;
    m.add_class::<AzSvgFillRuleEnumWrapper>()?;
    m.add_class::<AzSvgTransform>()?;
    m.add_class::<AzSvgFillStyle>()?;
    m.add_class::<AzSvgStrokeStyle>()?;
    m.add_class::<AzSvgLineJoinEnumWrapper>()?;
    m.add_class::<AzSvgLineCapEnumWrapper>()?;
    m.add_class::<AzSvgDashPattern>()?;

    m.add_class::<AzXml>()?;
    m.add_class::<AzXmlNode>()?;

    m.add_class::<AzFile>()?;

    m.add_class::<AzMsgBox>()?;
    m.add_class::<AzMsgBoxIconEnumWrapper>()?;
    m.add_class::<AzMsgBoxYesNoEnumWrapper>()?;
    m.add_class::<AzMsgBoxOkCancelEnumWrapper>()?;
    m.add_class::<AzFileDialog>()?;
    m.add_class::<AzFileTypeList>()?;
    m.add_class::<AzColorPickerDialog>()?;

    m.add_class::<AzSystemClipboard>()?;

    m.add_class::<AzInstantEnumWrapper>()?;
    m.add_class::<AzInstantPtr>()?;
    m.add_class::<AzInstantPtrCloneFn>()?;
    m.add_class::<AzInstantPtrDestructorFn>()?;
    m.add_class::<AzSystemTick>()?;
    m.add_class::<AzDurationEnumWrapper>()?;
    m.add_class::<AzSystemTimeDiff>()?;
    m.add_class::<AzSystemTickDiff>()?;

    m.add_class::<AzTimerId>()?;
    m.add_class::<AzTimer>()?;
    m.add_class::<AzTerminateTimerEnumWrapper>()?;
    m.add_class::<AzThreadId>()?;
    m.add_class::<AzThread>()?;
    m.add_class::<AzThreadSender>()?;
    m.add_class::<AzThreadReceiver>()?;
    m.add_class::<AzThreadSendMsgEnumWrapper>()?;
    m.add_class::<AzThreadReceiveMsgEnumWrapper>()?;
    m.add_class::<AzThreadWriteBackMsg>()?;
    m.add_class::<AzCreateThreadFn>()?;
    m.add_class::<AzGetSystemTimeFn>()?;
    m.add_class::<AzCheckThreadFinishedFn>()?;
    m.add_class::<AzLibrarySendThreadMsgFn>()?;
    m.add_class::<AzLibraryReceiveThreadMsgFn>()?;
    m.add_class::<AzThreadRecvFn>()?;
    m.add_class::<AzThreadSendFn>()?;
    m.add_class::<AzThreadDestructorFn>()?;
    m.add_class::<AzThreadReceiverDestructorFn>()?;
    m.add_class::<AzThreadSenderDestructorFn>()?;

    m.add_class::<AzFmtValueEnumWrapper>()?;
    m.add_class::<AzFmtArg>()?;
    m.add_class::<AzString>()?;

    m.add_class::<AzTesselatedSvgNodeVec>()?;
    m.add_class::<AzStyleFontFamilyVec>()?;
    m.add_class::<AzXmlNodeVec>()?;
    m.add_class::<AzFmtArgVec>()?;
    m.add_class::<AzInlineLineVec>()?;
    m.add_class::<AzInlineWordVec>()?;
    m.add_class::<AzInlineGlyphVec>()?;
    m.add_class::<AzInlineTextHitVec>()?;
    m.add_class::<AzMonitorVec>()?;
    m.add_class::<AzVideoModeVec>()?;
    m.add_class::<AzDomVec>()?;
    m.add_class::<AzIdOrClassVec>()?;
    m.add_class::<AzNodeDataInlineCssPropertyVec>()?;
    m.add_class::<AzStyleBackgroundContentVec>()?;
    m.add_class::<AzStyleBackgroundPositionVec>()?;
    m.add_class::<AzStyleBackgroundRepeatVec>()?;
    m.add_class::<AzStyleBackgroundSizeVec>()?;
    m.add_class::<AzStyleTransformVec>()?;
    m.add_class::<AzCssPropertyVec>()?;
    m.add_class::<AzSvgMultiPolygonVec>()?;
    m.add_class::<AzSvgPathVec>()?;
    m.add_class::<AzVertexAttributeVec>()?;
    m.add_class::<AzSvgPathElementVec>()?;
    m.add_class::<AzSvgVertexVec>()?;
    m.add_class::<AzU32Vec>()?;
    m.add_class::<AzXWindowTypeVec>()?;
    m.add_class::<AzVirtualKeyCodeVec>()?;
    m.add_class::<AzCascadeInfoVec>()?;
    m.add_class::<AzScanCodeVec>()?;
    m.add_class::<AzCssDeclarationVec>()?;
    m.add_class::<AzCssPathSelectorVec>()?;
    m.add_class::<AzStylesheetVec>()?;
    m.add_class::<AzCssRuleBlockVec>()?;
    m.add_class::<AzU16Vec>()?;
    m.add_class::<AzF32Vec>()?;
    m.add_class::<AzU8Vec>()?;
    m.add_class::<AzCallbackDataVec>()?;
    m.add_class::<AzDebugMessageVec>()?;
    m.add_class::<AzGLuintVec>()?;
    m.add_class::<AzGLintVec>()?;
    m.add_class::<AzStringVec>()?;
    m.add_class::<AzStringPairVec>()?;
    m.add_class::<AzNormalizedLinearColorStopVec>()?;
    m.add_class::<AzNormalizedRadialColorStopVec>()?;
    m.add_class::<AzNodeIdVec>()?;
    m.add_class::<AzNodeVec>()?;
    m.add_class::<AzStyledNodeVec>()?;
    m.add_class::<AzTagIdsToNodeIdsMappingVec>()?;
    m.add_class::<AzParentWithNodeDepthVec>()?;
    m.add_class::<AzNodeDataVec>()?;
    m.add_class::<AzStyleFontFamilyVecDestructorEnumWrapper>()?;
    m.add_class::<AzTesselatedSvgNodeVecDestructorEnumWrapper>()?;
    m.add_class::<AzXmlNodeVecDestructorEnumWrapper>()?;
    m.add_class::<AzFmtArgVecDestructorEnumWrapper>()?;
    m.add_class::<AzInlineLineVecDestructorEnumWrapper>()?;
    m.add_class::<AzInlineWordVecDestructorEnumWrapper>()?;
    m.add_class::<AzInlineGlyphVecDestructorEnumWrapper>()?;
    m.add_class::<AzInlineTextHitVecDestructorEnumWrapper>()?;
    m.add_class::<AzMonitorVecDestructorEnumWrapper>()?;
    m.add_class::<AzVideoModeVecDestructorEnumWrapper>()?;
    m.add_class::<AzDomVecDestructorEnumWrapper>()?;
    m.add_class::<AzIdOrClassVecDestructorEnumWrapper>()?;
    m.add_class::<AzNodeDataInlineCssPropertyVecDestructorEnumWrapper>()?;
    m.add_class::<AzStyleBackgroundContentVecDestructorEnumWrapper>()?;
    m.add_class::<AzStyleBackgroundPositionVecDestructorEnumWrapper>()?;
    m.add_class::<AzStyleBackgroundRepeatVecDestructorEnumWrapper>()?;
    m.add_class::<AzStyleBackgroundSizeVecDestructorEnumWrapper>()?;
    m.add_class::<AzStyleTransformVecDestructorEnumWrapper>()?;
    m.add_class::<AzCssPropertyVecDestructorEnumWrapper>()?;
    m.add_class::<AzSvgMultiPolygonVecDestructorEnumWrapper>()?;
    m.add_class::<AzSvgPathVecDestructorEnumWrapper>()?;
    m.add_class::<AzVertexAttributeVecDestructorEnumWrapper>()?;
    m.add_class::<AzSvgPathElementVecDestructorEnumWrapper>()?;
    m.add_class::<AzSvgVertexVecDestructorEnumWrapper>()?;
    m.add_class::<AzU32VecDestructorEnumWrapper>()?;
    m.add_class::<AzXWindowTypeVecDestructorEnumWrapper>()?;
    m.add_class::<AzVirtualKeyCodeVecDestructorEnumWrapper>()?;
    m.add_class::<AzCascadeInfoVecDestructorEnumWrapper>()?;
    m.add_class::<AzScanCodeVecDestructorEnumWrapper>()?;
    m.add_class::<AzCssDeclarationVecDestructorEnumWrapper>()?;
    m.add_class::<AzCssPathSelectorVecDestructorEnumWrapper>()?;
    m.add_class::<AzStylesheetVecDestructorEnumWrapper>()?;
    m.add_class::<AzCssRuleBlockVecDestructorEnumWrapper>()?;
    m.add_class::<AzF32VecDestructorEnumWrapper>()?;
    m.add_class::<AzU16VecDestructorEnumWrapper>()?;
    m.add_class::<AzU8VecDestructorEnumWrapper>()?;
    m.add_class::<AzCallbackDataVecDestructorEnumWrapper>()?;
    m.add_class::<AzDebugMessageVecDestructorEnumWrapper>()?;
    m.add_class::<AzGLuintVecDestructorEnumWrapper>()?;
    m.add_class::<AzGLintVecDestructorEnumWrapper>()?;
    m.add_class::<AzStringVecDestructorEnumWrapper>()?;
    m.add_class::<AzStringPairVecDestructorEnumWrapper>()?;
    m.add_class::<AzNormalizedLinearColorStopVecDestructorEnumWrapper>()?;
    m.add_class::<AzNormalizedRadialColorStopVecDestructorEnumWrapper>()?;
    m.add_class::<AzNodeIdVecDestructorEnumWrapper>()?;
    m.add_class::<AzNodeVecDestructorEnumWrapper>()?;
    m.add_class::<AzStyledNodeVecDestructorEnumWrapper>()?;
    m.add_class::<AzTagIdsToNodeIdsMappingVecDestructorEnumWrapper>()?;
    m.add_class::<AzParentWithNodeDepthVecDestructorEnumWrapper>()?;
    m.add_class::<AzNodeDataVecDestructorEnumWrapper>()?;

    m.add_class::<AzOptionCssPropertyEnumWrapper>()?;
    m.add_class::<AzOptionPositionInfoEnumWrapper>()?;
    m.add_class::<AzOptionTimerIdEnumWrapper>()?;
    m.add_class::<AzOptionThreadIdEnumWrapper>()?;
    m.add_class::<AzOptionI16EnumWrapper>()?;
    m.add_class::<AzOptionU16EnumWrapper>()?;
    m.add_class::<AzOptionU32EnumWrapper>()?;
    m.add_class::<AzOptionImageRefEnumWrapper>()?;
    m.add_class::<AzOptionFontRefEnumWrapper>()?;
    m.add_class::<AzOptionSystemClipboardEnumWrapper>()?;
    m.add_class::<AzOptionFileTypeListEnumWrapper>()?;
    m.add_class::<AzOptionWindowStateEnumWrapper>()?;
    m.add_class::<AzOptionMouseStateEnumWrapper>()?;
    m.add_class::<AzOptionKeyboardStateEnumWrapper>()?;
    m.add_class::<AzOptionStringVecEnumWrapper>()?;
    m.add_class::<AzOptionFileEnumWrapper>()?;
    m.add_class::<AzOptionGlEnumWrapper>()?;
    m.add_class::<AzOptionThreadReceiveMsgEnumWrapper>()?;
    m.add_class::<AzOptionPercentageValueEnumWrapper>()?;
    m.add_class::<AzOptionAngleValueEnumWrapper>()?;
    m.add_class::<AzOptionRendererOptionsEnumWrapper>()?;
    m.add_class::<AzOptionCallbackEnumWrapper>()?;
    m.add_class::<AzOptionThreadSendMsgEnumWrapper>()?;
    m.add_class::<AzOptionLayoutRectEnumWrapper>()?;
    m.add_class::<AzOptionRefAnyEnumWrapper>()?;
    m.add_class::<AzOptionInlineTextEnumWrapper>()?;
    m.add_class::<AzOptionLayoutPointEnumWrapper>()?;
    m.add_class::<AzOptionLayoutSizeEnumWrapper>()?;
    m.add_class::<AzOptionWindowThemeEnumWrapper>()?;
    m.add_class::<AzOptionNodeIdEnumWrapper>()?;
    m.add_class::<AzOptionDomNodeIdEnumWrapper>()?;
    m.add_class::<AzOptionColorUEnumWrapper>()?;
    m.add_class::<AzOptionRawImageEnumWrapper>()?;
    m.add_class::<AzOptionSvgDashPatternEnumWrapper>()?;
    m.add_class::<AzOptionWaylandThemeEnumWrapper>()?;
    m.add_class::<AzOptionTaskBarIconEnumWrapper>()?;
    m.add_class::<AzOptionHwndHandleEnumWrapper>()?;
    m.add_class::<AzOptionLogicalPositionEnumWrapper>()?;
    m.add_class::<AzOptionPhysicalPositionI32EnumWrapper>()?;
    m.add_class::<AzOptionWindowIconEnumWrapper>()?;
    m.add_class::<AzOptionStringEnumWrapper>()?;
    m.add_class::<AzOptionX11VisualEnumWrapper>()?;
    m.add_class::<AzOptionI32EnumWrapper>()?;
    m.add_class::<AzOptionF32EnumWrapper>()?;
    m.add_class::<AzOptionMouseCursorTypeEnumWrapper>()?;
    m.add_class::<AzOptionLogicalSizeEnumWrapper>()?;
    m.add_class::<AzOptionCharEnumWrapper>()?;
    m.add_class::<AzOptionVirtualKeyCodeEnumWrapper>()?;
    m.add_class::<AzOptionDomEnumWrapper>()?;
    m.add_class::<AzOptionTextureEnumWrapper>()?;
    m.add_class::<AzOptionImageMaskEnumWrapper>()?;
    m.add_class::<AzOptionTabIndexEnumWrapper>()?;
    m.add_class::<AzOptionTagIdEnumWrapper>()?;
    m.add_class::<AzOptionDurationEnumWrapper>()?;
    m.add_class::<AzOptionInstantEnumWrapper>()?;
    m.add_class::<AzOptionUsizeEnumWrapper>()?;
    m.add_class::<AzOptionU8VecEnumWrapper>()?;
    m.add_class::<AzOptionU8VecRefEnumWrapper>()?;

    m.add_class::<AzResultXmlXmlErrorEnumWrapper>()?;
    m.add_class::<AzResultRawImageDecodeImageErrorEnumWrapper>()?;
    m.add_class::<AzResultU8VecEncodeImageErrorEnumWrapper>()?;
    m.add_class::<AzResultSvgXmlNodeSvgParseErrorEnumWrapper>()?;
    m.add_class::<AzResultSvgSvgParseErrorEnumWrapper>()?;
    m.add_class::<AzSvgParseErrorEnumWrapper>()?;
    m.add_class::<AzXmlErrorEnumWrapper>()?;
    m.add_class::<AzDuplicatedNamespaceError>()?;
    m.add_class::<AzUnknownNamespaceError>()?;
    m.add_class::<AzUnexpectedCloseTagError>()?;
    m.add_class::<AzUnknownEntityReferenceError>()?;
    m.add_class::<AzDuplicatedAttributeError>()?;
    m.add_class::<AzXmlParseErrorEnumWrapper>()?;
    m.add_class::<AzXmlTextError>()?;
    m.add_class::<AzXmlStreamErrorEnumWrapper>()?;
    m.add_class::<AzNonXmlCharError>()?;
    m.add_class::<AzInvalidCharError>()?;
    m.add_class::<AzInvalidCharMultipleError>()?;
    m.add_class::<AzInvalidQuoteError>()?;
    m.add_class::<AzInvalidSpaceError>()?;
    m.add_class::<AzInvalidStringError>()?;
    m.add_class::<AzSvgParseErrorPosition>()?;

    Ok(())
}


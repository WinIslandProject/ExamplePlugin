use std::ffi::c_void;

use winisland_plugin_api::*;

const PLUGIN_ID: &str = "winisland-example-plugin";
const PLUGIN_NAME: &str = "WinIsland Example";
const PLUGIN_AUTHOR: &str = "WinIslandProject";
const PLUGIN_DESCRIPTION: &str = "Reference widget plugin rendered through the native Draw API";

struct Instance {
    token: PluginToken,
    widget_api: WidgetApiV1,
    widget_id: ResourceId,
}

static DESCRIPTOR: PluginDescriptorV1 = PluginDescriptorV1 {
    struct_size: std::mem::size_of::<PluginDescriptorV1>() as u32,
    abi_version: ABI_VERSION_1,
    capabilities: CAPABILITY_WIDGET,
    metadata: PluginMetadataC::new(
        PLUGIN_ID,
        PLUGIN_NAME,
        env!("CARGO_PKG_VERSION"),
        PLUGIN_AUTHOR,
        PLUGIN_DESCRIPTION,
    ),
    create: Some(create),
    shutdown: Some(shutdown),
    destroy: Some(destroy),
};

unsafe extern "C" fn create(
    create_info: *const PluginCreateInfoV1,
    out_handle: *mut PluginHandle,
) -> PluginResultC {
    if create_info.is_null() || out_handle.is_null() {
        return PluginResultC::err("null create argument");
    }
    // SAFETY: WinIsland supplies a readable ABI create-info prefix.
    let info = unsafe { &*create_info };
    if info.struct_size < std::mem::size_of::<PluginCreateInfoV1>() as u32
        || info.abi_version != ABI_VERSION_1
        || info.host_api.is_null()
        || info.plugin_token == INVALID_ID
    {
        return PluginResultC::err("unsupported create info");
    }
    // SAFETY: The validated host API pointer remains valid while WinIsland runs.
    let host = unsafe { &*info.host_api };
    // SAFETY: The host table originated from WinIsland and has a validated ABI header.
    let Some(widget_api) = (unsafe { host.widget_api() }) else {
        return PluginResultC::err("widget API is unavailable");
    };
    let Some(create_widget) = widget_api.create else {
        return PluginResultC::err("widget create is unavailable");
    };
    if widget_api.release.is_none() {
        return PluginResultC::err("widget release is unavailable");
    }

    let widget = WidgetDataV1 {
        span_cols: 2,
        span_rows: 2,
        title: str_to_fixed("WinIsland Example"),
        body: str_to_fixed("Native Draw API"),
        on_draw: Some(draw_widget),
        ..Default::default()
    };
    let mut widget_id = INVALID_ID;
    // SAFETY: The inputs and output pointer remain valid for this synchronous call.
    let result = unsafe { create_widget(info.plugin_token, &widget, &mut widget_id) };
    if result.status != 0 {
        return result;
    }
    let instance = Box::new(Instance {
        token: info.plugin_token,
        widget_api,
        widget_id,
    });
    // SAFETY: WinIsland owns this opaque allocation until it calls destroy once.
    unsafe { out_handle.write(Box::into_raw(instance).cast::<c_void>()) };
    PluginResultC::ok()
}

unsafe extern "C" fn draw_widget(_callback_data: *mut c_void, context: *const WidgetDrawContextV1) {
    if context.is_null() {
        return;
    }
    // SAFETY: The context is host-owned and valid for this callback only.
    let context_ref = unsafe { &*context };
    // SAFETY: The draw table is supplied by the validated host context.
    let Some(draw) = (unsafe { context_ref.draw_api() }) else {
        return;
    };
    let (Some(round_rect), Some(circle), Some(arc), Some(text)) = (
        draw.draw_round_rect,
        draw.draw_circle,
        draw.draw_arc,
        draw.draw_text,
    ) else {
        return;
    };
    let width = context_ref.width.max(1.0);
    let height = context_ref.height.max(1.0);
    let inset = (width.min(height) * 0.09).clamp(10.0, 18.0);
    let ring = (width.min(height) * 0.42).clamp(48.0, 82.0);
    let ring_x = width - inset - ring;
    let ring_y = (height - ring) * 0.5;
    // SAFETY: All draw calls are synchronous and use the current valid context.
    unsafe {
        round_rect(context, 0.0, 0.0, width, height, 18.0, 0xFF18191E);
        text(
            context,
            inset,
            inset,
            Utf8SliceV1::borrowed("WinIsland"),
            17.0,
            1,
            0xFFF8F8FA,
        );
        text(
            context,
            inset,
            inset + 25.0,
            Utf8SliceV1::borrowed("Plugin ready"),
            11.0,
            0,
            0xFF9A9CA7,
        );
        circle(
            context,
            ring_x + ring * 0.5,
            ring_y + ring * 0.5,
            ring * 0.5,
            0xFF292A32,
        );
        arc(
            context,
            ring_x + 5.0,
            ring_y + 5.0,
            ring - 10.0,
            ring - 10.0,
            -90.0,
            280.0,
            5.0,
            0xFF58D7E8,
        );
        text(
            context,
            ring_x + ring * 0.33,
            ring_y + ring * 0.35,
            Utf8SliceV1::borrowed("OK"),
            14.0,
            1,
            0xFFF8F8FA,
        );
    }
}

unsafe extern "C" fn shutdown(handle: PluginHandle) -> PluginResultC {
    if handle.is_null() {
        return PluginResultC::ok();
    }
    // SAFETY: The handle came from Box<Instance> and has not been destroyed.
    let instance = unsafe { &mut *handle.cast::<Instance>() };
    if instance.widget_id != INVALID_ID {
        let Some(release) = instance.widget_api.release else {
            return PluginResultC::err("widget release is unavailable");
        };
        // SAFETY: The widget belongs to this instance and plugin token.
        let result = unsafe { release(instance.token, instance.widget_id) };
        if result.status != 0 {
            return result;
        }
        instance.widget_id = INVALID_ID;
    }
    PluginResultC::ok()
}

unsafe extern "C" fn destroy(handle: PluginHandle) {
    if !handle.is_null() {
        // SAFETY: WinIsland calls destroy once after successful shutdown.
        unsafe { drop(Box::from_raw(handle.cast::<Instance>())) };
    }
}

#[unsafe(no_mangle)]
/// # Safety
/// WinIsland calls this entry point using the documented ABI v1 signature.
pub unsafe extern "C" fn winisland_plugin_entry_v1() -> *const PluginDescriptorV1 {
    &DESCRIPTOR
}

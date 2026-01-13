//! Android soft keyboard support via JNI
//!
//! This module provides direct access to Android's InputMethodManager to
//! show/hide the soft keyboard. While winit 0.30.12's `set_ime_allowed()`
//! does call `show_soft_input()`/`hide_soft_input()`, there's a known issue
//! where winit doesn't forward `TextEvent` from `android-activity` as
//! `WindowEvent::Ime` events (see https://github.com/rust-windowing/winit/issues/4067).
//!
//! This means the keyboard may show, but typed characters won't be received
//! by Bevy as `Ime::Commit` events. Full Android soft keyboard support
//! requires fixes in the winit or android-activity crates.
//!
//! This module provides fallback JNI calls in case the standard winit path
//! doesn't work on certain devices or Android versions.

use jni::objects::{JObject, JValue};

/// Shows the Android soft keyboard using InputMethodManager via JNI.
///
/// This is a fallback for when winit's `set_ime_allowed()` doesn't work.
/// Call this after setting `window.ime_enabled = true` if the keyboard
/// still doesn't appear.
pub fn show_soft_keyboard() {
    if let Err(e) = show_soft_keyboard_inner() {
        bevy::log::warn!("Failed to show soft keyboard via JNI: {:?}", e);
    }
}

/// Hides the Android soft keyboard using InputMethodManager via JNI.
///
/// This is a fallback for when winit's `set_ime_allowed()` doesn't work.
pub fn hide_soft_keyboard() {
    if let Err(e) = hide_soft_keyboard_inner() {
        bevy::log::warn!("Failed to hide soft keyboard via JNI: {:?}", e);
    }
}

fn show_soft_keyboard_inner() -> Result<(), jni::errors::Error> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };

    let mut env = vm.attach_current_thread()?;

    // Get the window from the activity
    let window = env
        .call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?
        .l()?;

    // Get the decor view from the window
    let decor_view = env
        .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
        .l()?;

    // Get InputMethodManager via getSystemService
    let context_class = env.find_class("android/content/Context")?;

    // Get the INPUT_METHOD_SERVICE constant
    let input_method_service = env
        .get_static_field(&context_class, "INPUT_METHOD_SERVICE", "Ljava/lang/String;")?
        .l()?;

    // Get the InputMethodManager
    let imm = env
        .call_method(
            &activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[JValue::Object(&input_method_service)],
        )?
        .l()?;

    // Call showSoftInput with SHOW_IMPLICIT flag (1)
    env.call_method(
        &imm,
        "showSoftInput",
        "(Landroid/view/View;I)Z",
        &[JValue::Object(&decor_view), JValue::Int(1)],
    )?;

    Ok(())
}

fn hide_soft_keyboard_inner() -> Result<(), jni::errors::Error> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };

    let mut env = vm.attach_current_thread()?;

    // Get the window from the activity
    let window = env
        .call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?
        .l()?;

    // Get the decor view from the window
    let decor_view = env
        .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
        .l()?;

    // Get the window token from the view
    let window_token = env
        .call_method(
            &decor_view,
            "getWindowToken",
            "()Landroid/os/IBinder;",
            &[],
        )?
        .l()?;

    // Get InputMethodManager via getSystemService
    let context_class = env.find_class("android/content/Context")?;

    // Get the INPUT_METHOD_SERVICE constant
    let input_method_service = env
        .get_static_field(&context_class, "INPUT_METHOD_SERVICE", "Ljava/lang/String;")?
        .l()?;

    // Get the InputMethodManager
    let imm = env
        .call_method(
            &activity,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[JValue::Object(&input_method_service)],
        )?
        .l()?;

    // Call hideSoftInputFromWindow with 0 flags
    env.call_method(
        &imm,
        "hideSoftInputFromWindow",
        "(Landroid/os/IBinder;I)Z",
        &[JValue::Object(&window_token), JValue::Int(0)],
    )?;

    Ok(())
}

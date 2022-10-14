pub unsafe fn load_gl_pointers() {
    #[cfg(target_os = "macos")]
    let library = libloading::os::unix::Library::new("libepoxy.0.dylib").unwrap();
    #[cfg(all(unix, not(target_os = "macos")))]
    let library = libloading::os::unix::Library::new("libepoxy.so.0").unwrap();
    #[cfg(windows)]
    let library = libloading::os::windows::Library::open_already_loaded("epoxy-0.dll").unwrap();

    epoxy::load_with(|name| {
        unsafe { library.get::<_>(name.as_bytes()) }
            .map(|symbol| *symbol)
            .unwrap_or(std::ptr::null())
    });
}

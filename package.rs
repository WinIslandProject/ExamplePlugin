use winisland_plugin_api::packager::PluginPackager;

fn main() {
    PluginPackager::from_cargo()
        .expect("read plugin metadata")
        .name("WinIsland Example")
        .icon("icon.png")
        .readme("README.md")
        .build()
        .expect("build plugin package");
}

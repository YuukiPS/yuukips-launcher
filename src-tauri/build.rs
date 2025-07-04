fn main() {
    // Create Windows manifest file for admin privileges
    #[cfg(target_os = "windows")]
    {
        use std::fs;
        use std::path::Path;
        
        let manifest_content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}" />
      <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}" />
      <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}" />
      <supportedOS Id="{e2011457-1546-43c5-a5fe-008deee3d3f0}" />
    </application>
  </compatibility>
</assembly>"#;
        
        let manifest_path = Path::new("yuukips-launcher.exe.manifest");
        if let Err(e) = fs::write(manifest_path, manifest_content) {
            println!("cargo:warning=Failed to create manifest file: {}", e);
        } else {
            println!("cargo:warning=Created Windows manifest file");
        }
    }
    
    tauri_build::build();
}

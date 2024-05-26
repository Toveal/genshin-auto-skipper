const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0"> 
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v2">
     <security>
        <requestedPrivileges>
           <requestedExecutionLevel 
                level="requireAdministrator" 
                uiAccess="false"/>
        </requestedPrivileges>
      </security>
  </trustInfo>
</assembly>"#;

fn main() {
    if cfg!(target_os = "windows") && std::env::var("PROFILE").unwrap() == "release" {
        let mut res = winres::WindowsResource::new();
        res.set_manifest(MANIFEST);
        res.compile().unwrap();
    }
}

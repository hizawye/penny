; Inno Setup script for Penny — builds a Windows installer (setup .exe).
;
; Compile locally or in CI with:
;   iscc /DAppVersion=0.1.0 installer\penny.iss
; Expects the release binary and docs staged in dist\ :
;   dist\penny.exe, dist\README.md, dist\LICENSE
; The compiled installer is written to installer\Output\.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

#define AppName "Penny"
#define AppPublisher "hizawye"
#define AppURL "https://github.com/hizawye/penny"
#define AppExeName "penny.exe"

[Setup]
AppId={{7E1D9C2A-4B6F-4E2A-9C3D-PENNY0000001}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}/issues
AppUpdatesURL={#AppURL}/releases
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
; Resolve all relative paths (LicenseFile, [Files] Source, OutputDir) from the
; repo root — one level up from this script in installer/ — not the script dir.
SourceDir=..
LicenseFile=dist\LICENSE
OutputDir=installer\Output
OutputBaseFilename=penny-{#AppVersion}-x86_64-windows-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; Penny is a 64-bit app; only allow install on 64-bit Windows.
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "dist\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "dist\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme
Source: "dist\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
